//! Multiclass SHD calibration baselines (GC1-exempt).
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER.**
//!
//! Runtime-sized feed-forward dense-LIF (`N_IN`, `T`, `n_classes`) for the
//! `c1-shd-cal-*` / `c1-shd-full-*` protocols.
//!
//! Ceilings:
//! - **True surrogate e-prop** (local eligibility × transported δ) — overnight
//!   capped p27 ceiling.
//! - **True SuperSpike reverse-mode BPTT** on the same hard-reset feed-forward
//!   LIF — protocol-29 full-corpus ceiling (`ShdSuperSpikeCeiling`). Feed-forward
//!   (no `W_rec`) keeps BPTT memory O(`H·T`) and compute O(`T·H·N_IN`) per
//!   example; disclose wall time. Not a drop-in match to Zenke SuperSpike on a
//!   recurrent net, and not neuromorphic SOTA.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::{DEFAULT_TAU_M, THETA_REST, V_RESET};

use crate::credit::{CreditSignal, LearnedReinforceFeedback, ReinforceFeedback};
use crate::matched_deep_gradient::{rescale_rms_to, ModulatorScale};

/// Init scale for the readout matrix, shared by every arm.
///
/// The DFA feedback matrix is initialised at the **same** scale so that the DFA
/// arm and the e-prop ceiling apply hidden-layer updates of comparable
/// magnitude at a common learning rate. See [`shd_out_scale`].
pub fn shd_out_scale(hidden: usize) -> f32 {
    0.2f32 / (hidden as f32).sqrt()
}

/// Tolerance for cross-arm modulator-scale parity (ratio of RMS values).
///
/// Anything above this means the arms are running at materially different
/// effective learning rates and the comparison is not interpretable.
pub const MODULATOR_PARITY_TOLERANCE: f32 = 3.5;

/// Stable labels.
pub const SHD_BROADCAST_PM1_LABEL: &str = "SHD_BROADCAST_PM1";
pub const SHD_DFA_LABEL: &str = "SHD_DFA";
pub const SHD_RL_REINFORCE_FB_LABEL: &str = "SHD_RL_REINFORCE_FB";
pub const SHD_EPROP_CEILING_LABEL: &str = "SHD_EPROP_CEILING";
pub const SHD_SUPERSPIKE_CEILING_LABEL: &str = "SHD_SUPERSPIKE_CEILING";

/// One passthrough spike example: flat `[T × N_IN]` + class id.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdExample {
    pub frames: Vec<f32>,
    pub t: usize,
    pub n_in: usize,
    pub label: u32,
}

/// Shared train knobs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShdTrainConfig {
    pub hidden: usize,
    pub n_classes: usize,
    pub lr: f32,
    pub beta: f32,
    pub epochs: usize,
}

/// Per-arm accuracy / loss.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdArmReport {
    pub label: &'static str,
    pub accuracy: f32,
    pub loss: f32,
}

/// Shared feed-forward LIF + linear readout (multiclass logits).
#[derive(Clone, Debug)]
struct ShdArch {
    hidden: usize,
    n_in: usize,
    t: usize,
    n_classes: usize,
    beta: f32,
    alpha: f32,
    win: Vec<f32>,  // hidden × n_in
    wout: Vec<f32>, // n_classes × hidden
    bout: Vec<f32>, // n_classes
}

/// Input-weight scale that leaves the hidden layer spiking at initialisation.
///
/// `0.35` left it silent; see the note in [`ShdArch::new`] and
/// `PREREG_2026-08-22_SILENT_INITIALISATION_REPAIR.md`.
const SILENCE_FREE_INPUT_SCALE: f32 = 2.8;

impl ShdArch {
    fn new(n_in: usize, t: usize, n_classes: usize, hidden: usize, beta: f32, seed: u64) -> Self {
        assert!(n_in >= 1 && t >= 1 && n_classes >= 2 && hidden >= 1);
        let mut rng = Rng::new(seed ^ 0x54D0_A4C4_0000_00F1);
        // 2026-08-22 repair, registered in
        // `PREREG_2026-08-22_SILENT_INITIALISATION_REPAIR.md`.
        //
        // This was `0.35 / sqrt(n_in)`, which leaves the hidden layer **below
        // threshold**: mean firing rate 0.00000000 at initialisation and still
        // 0.00000000 after 60 epochs. Every local arm in this module was
        // therefore a constant predictor - `wout` never moves because the rate
        // it multiplies is zero, and the logits are pure bias, bit-identical
        // across classes.
        //
        // Only `ShdSuperSpikeCeiling` escaped, by growing `sum|win|` 2.2x
        // (52.6 -> 115.5) until it crossed threshold. The local rules grew it
        // 1.08x and never did: no spikes gives vanishing eligibility, which
        // gives no weight growth, which gives no spikes.
        //
        // The multiplier puts the initial firing rate mid-band in the activity
        // window this workspace already uses (`[0.001, 0.500]`): measured
        // 0.087 on the reference fixture, against 0.020 at 6x and 0.454 at 16x,
        // with 24x saturating at 0.81. It changes only the operating point -
        // no rule, threshold, surrogate, readout or optimiser is touched, and
        // every arm shares it, so none is advantaged.
        let in_scale = SILENCE_FREE_INPUT_SCALE / (n_in as f32).sqrt();
        let out_scale = shd_out_scale(hidden);
        let win: Vec<f32> = (0..hidden * n_in)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let wout: Vec<f32> = (0..n_classes * hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();
        Self {
            hidden,
            n_in,
            t,
            n_classes,
            beta,
            alpha: (-1.0f32 / DEFAULT_TAU_M).exp(),
            win,
            wout,
            bout: vec![0.0; n_classes],
        }
    }

    fn forward(&self, frames: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        // returns (rates[h], logits[c], u_last_trace unused — pack u/s via rates only for e-prop path)
        let h = self.hidden;
        let n_in = self.n_in;
        let t_len = self.t;
        let theta = THETA_REST;
        let mut u = vec![0.0f32; h];
        let mut rates = vec![0.0f32; h];
        // Keep last membrane for optional diagnostics; eligibility rebuilt in train.
        for t in 0..t_len {
            for i in 0..h {
                let mut cur = 0.0f32;
                let base = t * n_in;
                for j in 0..n_in {
                    cur += self.win[i * n_in + j] * frames[base + j];
                }
                let ui = self.alpha * u[i] + cur;
                let spike = if ui >= theta { 1.0f32 } else { 0.0 };
                u[i] = if spike > 0.5 { V_RESET } else { ui };
                rates[i] += spike;
            }
        }
        let mut logits = self.bout.clone();
        for c in 0..self.n_classes {
            let mut z = logits[c];
            for i in 0..h {
                z += self.wout[c * h + i] * rates[i];
            }
            logits[c] = z;
        }
        (rates, logits, u)
    }

    fn evaluate(&self, test: &[ShdExample]) -> (f32, f32) {
        let mut correct = 0usize;
        let mut loss_sum = 0.0f32;
        for ex in test {
            let (_r, logits, _) = self.forward(&ex.frames);
            let (pred, loss) = softmax_pred_nll(&logits, ex.label as usize);
            loss_sum += loss;
            if pred == ex.label as usize {
                correct += 1;
            }
        }
        (
            correct as f32 / test.len().max(1) as f32,
            loss_sum / test.len().max(1) as f32,
        )
    }

    fn predictions(&self, test: &[ShdExample]) -> Vec<u32> {
        test.iter()
            .map(|example| {
                let (_, logits, _) = self.forward(&example.frames);
                softmax_pred_nll(&logits, example.label as usize).0 as u32
            })
            .collect()
    }
}

/// Broadcast ±1 three-factor on SHD (sampled action → ±1 reward × eligibility).
#[derive(Clone, Debug)]
pub struct ShdBroadcastPm1 {
    arch: ShdArch,
    lr: f32,
    rng: Rng,
}

impl ShdBroadcastPm1 {
    pub fn new(ex: &ShdExample, cfg: ShdTrainConfig, seed: u64) -> Self {
        Self {
            arch: ShdArch::new(ex.n_in, ex.t, cfg.n_classes, cfg.hidden, cfg.beta, seed),
            lr: cfg.lr,
            rng: Rng::new(seed ^ 0xB10A_DC45_0000_00F1),
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        for _ in 0..epochs {
            for ex in train {
                self.step(ex);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        ShdArmReport {
            label: SHD_BROADCAST_PM1_LABEL,
            accuracy,
            loss,
        }
    }

    fn step(&mut self, ex: &ShdExample) {
        let (rates, e_in, logits) = forward_with_elig(&self.arch, &ex.frames);
        let probs = softmax(&logits);
        let a = sample_cat(&probs, &mut self.rng);
        let y = ex.label as usize;
        let reward = if a == y { 1.0f32 } else { -1.0 };
        // Readout: REINFORCE-style (1_a − p) × reward; hidden: broadcast reward × E.
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let c = self.arch.n_classes;
        for k in 0..c {
            let adv = if k == a { 1.0 } else { 0.0 } - probs[k];
            for i in 0..h {
                self.arch.wout[k * h + i] += self.lr * reward * adv * rates[i];
            }
            self.arch.bout[k] += self.lr * reward * adv;
        }
        for i in 0..h {
            for j in 0..n_in {
                self.arch.win[i * n_in + j] += self.lr * reward * e_in[i * n_in + j];
            }
        }
    }
}

/// Graded DFA: CE gradient × fixed-random feedback on hidden.
///
/// # Scale parity with the e-prop ceiling (2026-07-25 fix)
///
/// This arm previously drew `B ~ U[−1, 1]` while [`ShdEpropCeiling`] used the
/// transported readout matrix `wout ~ U[−1, 1] · 0.2/√h` as its hidden
/// modulator. At `h = 128` that is `sd(B)/sd(wout) ≈ 56.6`: at the shared
/// `lr = 0.02` the "ceiling" ran its hidden layer at roughly 2% of the DFA arm's
/// effective step size. That, not credit-assignment quality, is why the ceiling
/// (0.092–0.126) scored *below* the treatment (0.209–0.239) at every width — and
/// a ceiling below the arm it bounds invalidates the comparison.
///
/// `B` is now initialised at [`shd_out_scale`], identical to `wout`, and both
/// arms record their realised modulator RMS via [`ShdDfa::modulator_scale`] /
/// [`ShdEpropCeiling::modulator_scale`] so any residual asymmetry is visible
/// rather than silent.
#[derive(Clone, Debug)]
pub struct ShdDfa {
    arch: ShdArch,
    lr: f32,
    feedback: Vec<f32>, // hidden × n_classes
    modulator: ModulatorScale,
}

impl ShdDfa {
    pub fn new(ex: &ShdExample, cfg: ShdTrainConfig, seed: u64) -> Self {
        let arch = ShdArch::new(ex.n_in, ex.t, cfg.n_classes, cfg.hidden, cfg.beta, seed);
        let mut rng = Rng::new(seed ^ 0xDFA5_4D00_0000_00F1);
        // Scale-matched to `wout` so the DFA and e-prop hidden updates are
        // comparable at a common learning rate.
        let fb_scale = shd_out_scale(cfg.hidden);
        let feedback: Vec<f32> = (0..cfg.hidden * cfg.n_classes)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * fb_scale)
            .collect();
        Self {
            arch,
            lr: cfg.lr,
            feedback,
            modulator: ModulatorScale::new(),
        }
    }

    /// Realised RMS of the hidden-layer modulator. Compare against
    /// [`ShdEpropCeiling::modulator_scale`]; a ratio above
    /// [`MODULATOR_PARITY_TOLERANCE`] means the arms are not comparable.
    pub fn modulator_scale(&self) -> ModulatorScale {
        self.modulator
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        for _ in 0..epochs {
            for ex in train {
                self.step(ex);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        ShdArmReport {
            label: SHD_DFA_LABEL,
            accuracy,
            loss,
        }
    }

    fn step(&mut self, ex: &ShdExample) {
        let (rates, e_in, logits) = forward_with_elig(&self.arch, &ex.frames);
        let y = ex.label as usize;
        let probs = softmax(&logits);
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let c = self.arch.n_classes;
        // dL/dz = p − onehot
        let mut delta = probs;
        delta[y] -= 1.0;
        for k in 0..c {
            for i in 0..h {
                self.arch.wout[k * h + i] -= self.lr * delta[k] * rates[i];
            }
            self.arch.bout[k] -= self.lr * delta[k];
        }
        let mut mods = vec![0.0f32; h];
        for i in 0..h {
            let mut m = 0.0f32;
            for k in 0..c {
                m += self.feedback[i * c + k] * (-delta[k]);
            }
            mods[i] = m;
            for j in 0..n_in {
                self.arch.win[i * n_in + j] += self.lr * m * e_in[i * n_in + j];
            }
        }
        self.modulator.observe(&mods);
    }
}

/// REINFORCE × frozen per-hidden feedback `B_i` (matched-mech RL×B parity).
///
/// Readout uses the same categorical REINFORCE term as [`ShdBroadcastPm1`].
/// Hidden plasticity is `B_i · r · (1 − p_a)` × eligibility — directional
/// REINFORCE projected through frozen Uniform[-1,1] `B` (production
/// [`ReinforceFeedback`] lineage), not flat reward broadcast.
#[derive(Clone, Debug)]
pub struct ShdRlReinforceFb {
    arch: ShdArch,
    lr: f32,
    /// Frozen per-hidden `B_i ∈ [-1, 1]`.
    feedback: Vec<f32>,
    rng: Rng,
}

impl ShdRlReinforceFb {
    pub fn new(ex: &ShdExample, cfg: ShdTrainConfig, seed: u64) -> Self {
        let feedback = ReinforceFeedback::new(cfg.hidden, seed).weights().to_vec();
        Self {
            arch: ShdArch::new(ex.n_in, ex.t, cfg.n_classes, cfg.hidden, cfg.beta, seed),
            lr: cfg.lr,
            feedback,
            rng: Rng::new(seed ^ 0xFB54_D0A1_0000_00F1),
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        for _ in 0..epochs {
            for ex in train {
                self.step(ex);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        ShdArmReport {
            label: SHD_RL_REINFORCE_FB_LABEL,
            accuracy,
            loss,
        }
    }

    pub fn feedback_weights(&self) -> &[f32] {
        &self.feedback
    }

    fn step(&mut self, ex: &ShdExample) {
        let (rates, e_in, logits) = forward_with_elig(&self.arch, &ex.frames);
        let probs = softmax(&logits);
        let a = sample_cat(&probs, &mut self.rng);
        let y = ex.label as usize;
        let reward = if a == y { 1.0f32 } else { -1.0 };
        // Categorical analogue of binary `r·(a−p)`: taken-action REINFORCE scale.
        let directional = reward * (1.0 - probs[a]);
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let c = self.arch.n_classes;
        for k in 0..c {
            let adv = if k == a { 1.0 } else { 0.0 } - probs[k];
            for i in 0..h {
                self.arch.wout[k * h + i] += self.lr * reward * adv * rates[i];
            }
            self.arch.bout[k] += self.lr * reward * adv;
        }
        for i in 0..h {
            let m = self.feedback[i] * directional;
            for j in 0..n_in {
                self.arch.win[i * n_in + j] += self.lr * m * e_in[i * n_in + j];
            }
        }
    }
}

/// Online Learned Feedback Alignment on SHD audio benchmark.
#[derive(Clone, Debug)]
pub struct ShdRlLearnedFb {
    arch: ShdArch,
    lr: f32,
    feedback: LearnedReinforceFeedback,
    rng: Rng,
}

impl ShdRlLearnedFb {
    pub fn new(ex: &ShdExample, cfg: ShdTrainConfig, seed: u64) -> Self {
        Self {
            arch: ShdArch::new(ex.n_in, ex.t, cfg.n_classes, cfg.hidden, cfg.beta, seed),
            lr: cfg.lr,
            feedback: LearnedReinforceFeedback::new(cfg.hidden, seed, 0.01),
            rng: Rng::new(seed ^ 0xFB54_D0A1_0000_00F2),
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        for _ in 0..epochs {
            for ex in train {
                self.step(ex);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        ShdArmReport {
            label: "SHD_RL_LEARNED_FB",
            accuracy,
            loss,
        }
    }

    fn step(&mut self, ex: &ShdExample) {
        let (rates, e_in, logits) = forward_with_elig(&self.arch, &ex.frames);
        let probs = softmax(&logits);
        let a = sample_cat(&probs, &mut self.rng);
        let y = ex.label as usize;
        let reward = if a == y { 1.0f32 } else { -1.0 };
        let directional = reward * (1.0 - probs[a]);
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let c = self.arch.n_classes;

        let fb_sig = self.feedback.credit(directional);
        self.feedback.update(directional, &rates);

        for k in 0..c {
            let adv = if k == a { 1.0 } else { 0.0 } - probs[k];
            for i in 0..h {
                self.arch.wout[k * h + i] += self.lr * reward * adv * rates[i];
            }
            self.arch.bout[k] += self.lr * reward * adv;
        }
        for i in 0..h {
            let m = fb_sig.for_post(i as u32);
            for j in 0..n_in {
                self.arch.win[i * n_in + j] += self.lr * m * e_in[i * n_in + j];
            }
        }
    }
}

/// True surrogate e-prop ceiling (eligibility × transported δ via `wout`).
///
/// # 2026-07-25 fix
///
/// The transported modulator is now computed from a **snapshot of `wout` taken
/// before the readout update in the same step**. Previously `wout` was updated
/// first and the hidden update then transported through the already-modified
/// matrix, which is neither e-prop nor BPTT.
///
/// See [`ShdDfa`] for the scale-parity fix that makes this arm a valid ceiling
/// rather than an under-stepped control.
#[derive(Clone, Debug)]
pub struct ShdEpropCeiling {
    arch: ShdArch,
    lr: f32,
    /// Whether the transport matrix is rescaled to [`shd_out_scale`] before use.
    normalise_transport: bool,
    modulator: ModulatorScale,
}

impl ShdEpropCeiling {
    pub fn new(ex: &ShdExample, cfg: ShdTrainConfig, seed: u64) -> Self {
        Self {
            arch: ShdArch::new(ex.n_in, ex.t, cfg.n_classes, cfg.hidden, cfg.beta, seed),
            lr: cfg.lr,
            normalise_transport: true,
            modulator: ModulatorScale::new(),
        }
    }

    /// Transport through the raw, un-rescaled `wout`.
    ///
    /// This is the pre-2026-08-22 rule. It is kept so the scale pathology stays
    /// demonstrable — see
    /// `defect_raw_transport_outgrows_dfa_feedback_once_the_network_spikes` — and
    /// mirrors [`crate::MatchedDeepGradient::with_raw_transport`]. Do not report a
    /// ceiling built this way against [`ShdDfa`] without also reporting
    /// [`Self::modulator_scale`].
    pub fn with_raw_transport(mut self) -> Self {
        self.normalise_transport = false;
        self
    }

    /// Realised RMS of the hidden-layer modulator; compare with
    /// [`ShdDfa::modulator_scale`].
    pub fn modulator_scale(&self) -> ModulatorScale {
        self.modulator
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        for _ in 0..epochs {
            for ex in train {
                self.step(ex);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        ShdArmReport {
            label: SHD_EPROP_CEILING_LABEL,
            accuracy,
            loss,
        }
    }

    fn step(&mut self, ex: &ShdExample) {
        let (rates, e_in, logits) = forward_with_elig(&self.arch, &ex.frames);
        let y = ex.label as usize;
        let probs = softmax(&logits);
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let c = self.arch.n_classes;
        let mut delta = probs;
        delta[y] -= 1.0;

        // Snapshot the transport matrix BEFORE the readout update: the hidden
        // update must use the `wout` that produced these logits, not the
        // post-update one.
        let mut wout_snapshot = self.arch.wout.clone();

        // Scale parity, registered in
        // `PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md`.
        //
        // `ShdDfa` transports through a **frozen** matrix at `shd_out_scale`,
        // this arm through `wout`, which is **trained**. Parity was established
        // at initialisation and then drifted as `wout` grew: measured at 5.08
        // against a 3.5 tolerance. That is a difference in effective learning
        // rate, not in credit-assignment quality, and it is exactly the
        // pathology this module's header warns about.
        //
        // Rescaling to the DFA feedback's own scale removes that drift and
        // nothing else. The modulator stays `Sum_k wout[k,i]*delta_k` in
        // direction and stays data-dependent; only the matrix norm is pinned.
        if self.normalise_transport {
            rescale_rms_to(&mut wout_snapshot, shd_out_scale(h));
        }

        for k in 0..c {
            for i in 0..h {
                self.arch.wout[k * h + i] -= self.lr * delta[k] * rates[i];
            }
            self.arch.bout[k] -= self.lr * delta[k];
        }
        // True e-prop: δ_i = Σ_k wout[k,i] · (p_k − y_k)
        let mut mods = vec![0.0f32; h];
        for i in 0..h {
            let mut di = 0.0f32;
            for k in 0..c {
                di += wout_snapshot[k * h + i] * delta[k];
            }
            mods[i] = -di;
            for j in 0..n_in {
                self.arch.win[i * n_in + j] -= self.lr * di * e_in[i * n_in + j];
            }
        }
        self.modulator.observe(&mods);
    }
}

/// True SuperSpike reverse-mode BPTT ceiling on the SHD feed-forward hard-reset LIF.
///
/// Same forward as [`ShdArch`] / e-prop arms (no recurrent `W_rec`). Stores
/// per-timestep membrane + spikes, then backpropagates SuperSpike surrogates.
/// Hard-reset cuts the membrane chain: `du[t] = ds·σ' + α·du[t+1]·(1−s[t])`.
#[derive(Clone, Debug)]
pub struct ShdSuperSpikeCeiling {
    arch: ShdArch,
    lr: f32,
}

impl ShdSuperSpikeCeiling {
    pub fn new(ex: &ShdExample, cfg: ShdTrainConfig, seed: u64) -> Self {
        Self {
            arch: ShdArch::new(ex.n_in, ex.t, cfg.n_classes, cfg.hidden, cfg.beta, seed),
            lr: cfg.lr,
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        for _ in 0..epochs {
            for ex in train {
                self.step(ex);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        ShdArmReport {
            label: SHD_SUPERSPIKE_CEILING_LABEL,
            accuracy,
            loss,
        }
    }

    /// Per-example predictions retained by the paired 0c-1 protocol.
    pub fn predictions(&self, test: &[ShdExample]) -> Vec<u32> {
        self.arch.predictions(test)
    }

    fn step(&mut self, ex: &ShdExample) {
        let cache = forward_bptt_cache(&self.arch, &ex.frames);
        let y = ex.label as usize;
        let probs = softmax(&cache.logits);
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let t_len = self.arch.t;
        let c = self.arch.n_classes;
        let theta = THETA_REST;
        let alpha = self.arch.alpha;
        let beta = self.arch.beta;

        let mut delta = probs;
        delta[y] -= 1.0;

        let mut dwin = vec![0.0f32; h * n_in];
        let mut dwout = vec![0.0f32; c * h];
        let mut dbout = vec![0.0f32; c];
        let mut g_r = vec![0.0f32; h];
        for k in 0..c {
            dbout[k] = delta[k];
            for i in 0..h {
                dwout[k * h + i] = delta[k] * cache.rates[i];
                g_r[i] += self.arch.wout[k * h + i] * delta[k];
            }
        }

        let mut du_next = vec![0.0f32; h];
        for t in (0..t_len).rev() {
            let mut du = vec![0.0f32; h];
            let base = t * n_in;
            for i in 0..h {
                let s_t = cache.s[i * t_len + t];
                let u_t = cache.u[i * t_len + t];
                let ds = g_r[i];
                let surr = surrogate(u_t - theta, beta);
                // Hard reset: spiked steps do not propagate membrane adjoint.
                du[i] = ds * surr + alpha * du_next[i] * (1.0 - s_t);
                for j in 0..n_in {
                    dwin[i * n_in + j] += du[i] * ex.frames[base + j];
                }
            }
            du_next = du;
        }

        for (w, dw) in self.arch.win.iter_mut().zip(dwin.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.arch.wout.iter_mut().zip(dwout.iter()) {
            *w -= self.lr * *dw;
        }
        for (b, db) in self.arch.bout.iter_mut().zip(dbout.iter()) {
            *b -= self.lr * *db;
        }
    }
}

struct BpttCache {
    /// Pre-reset membrane, flat `[h * T]`.
    u: Vec<f32>,
    /// Spikes, flat `[h * T]`.
    s: Vec<f32>,
    rates: Vec<f32>,
    logits: Vec<f32>,
}

fn forward_bptt_cache(arch: &ShdArch, frames: &[f32]) -> BpttCache {
    let h = arch.hidden;
    let n_in = arch.n_in;
    let t_len = arch.t;
    let theta = THETA_REST;
    let mut u_mem = vec![0.0f32; h];
    let mut u = vec![0.0f32; h * t_len];
    let mut s = vec![0.0f32; h * t_len];
    let mut rates = vec![0.0f32; h];
    for t in 0..t_len {
        let base = t * n_in;
        for i in 0..h {
            let mut cur = 0.0f32;
            for j in 0..n_in {
                cur += arch.win[i * n_in + j] * frames[base + j];
            }
            let ui = arch.alpha * u_mem[i] + cur;
            let spike = if ui >= theta { 1.0f32 } else { 0.0 };
            u[i * t_len + t] = ui;
            s[i * t_len + t] = spike;
            u_mem[i] = if spike > 0.5 { V_RESET } else { ui };
            rates[i] += spike;
        }
    }
    let mut logits = arch.bout.clone();
    for c in 0..arch.n_classes {
        let mut z = logits[c];
        for i in 0..h {
            z += arch.wout[c * h + i] * rates[i];
        }
        logits[c] = z;
    }
    BpttCache {
        u,
        s,
        rates,
        logits,
    }
}

/// Forward with SuperSpike-style input eligibility traces.
fn forward_with_elig(arch: &ShdArch, frames: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let h = arch.hidden;
    let n_in = arch.n_in;
    let t_len = arch.t;
    let theta = THETA_REST;
    let mut u = vec![0.0f32; h];
    let mut rates = vec![0.0f32; h];
    let mut e_in = vec![0.0f32; h * n_in];
    for t in 0..t_len {
        for i in 0..h {
            let mut cur = 0.0f32;
            let base = t * n_in;
            for j in 0..n_in {
                cur += arch.win[i * n_in + j] * frames[base + j];
            }
            let ui = arch.alpha * u[i] + cur;
            let surr = surrogate(ui - theta, arch.beta);
            let spike = if ui >= theta { 1.0f32 } else { 0.0 };
            u[i] = if spike > 0.5 { V_RESET } else { ui };
            rates[i] += spike;
            for j in 0..n_in {
                let idx = i * n_in + j;
                e_in[idx] = arch.alpha * e_in[idx] + surr * frames[base + j];
            }
        }
    }
    let mut logits = arch.bout.clone();
    for c in 0..arch.n_classes {
        let mut z = logits[c];
        for i in 0..h {
            z += arch.wout[c * h + i] * rates[i];
        }
        logits[c] = z;
    }
    (rates, e_in, logits)
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let m = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let ex: Vec<f32> = logits.iter().map(|z| (z - m).exp()).collect();
    let s: f32 = ex.iter().sum::<f32>().max(1e-12);
    ex.iter().map(|e| e / s).collect()
}

fn softmax_pred_nll(logits: &[f32], y: usize) -> (usize, f32) {
    let p = softmax(logits);
    let mut pred = 0usize;
    let mut best = f32::NEG_INFINITY;
    for (i, &pi) in p.iter().enumerate() {
        if pi > best {
            best = pi;
            pred = i;
        }
    }
    let nll = -p[y].max(1e-12).ln();
    (pred, nll)
}

fn sample_cat(probs: &[f32], rng: &mut Rng) -> usize {
    let mut u = rng.next_f32();
    for (i, &p) in probs.iter().enumerate() {
        if u <= p {
            return i;
        }
        u -= p;
    }
    probs.len().saturating_sub(1)
}

#[inline]
fn surrogate(u_minus_theta: f32, beta: f32) -> f32 {
    let d = 1.0 + beta * u_minus_theta.abs();
    1.0 / (d * d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy_data(n: usize, n_in: usize, t: usize, n_classes: usize, seed: u64) -> Vec<ShdExample> {
        let mut rng = Rng::new(seed);
        (0..n)
            .map(|_| {
                let label = rng.gen_index(n_classes) as u32;
                let mut frames = vec![0.0f32; t * n_in];
                for _ in 0..(t / 3).max(1) {
                    let tt = rng.gen_index(t);
                    let c = (label as usize * 2 + rng.gen_index(3)) % n_in;
                    frames[tt * n_in + c] = 1.0;
                }
                ShdExample {
                    frames,
                    t,
                    n_in,
                    label,
                }
            })
            .collect()
    }

    #[test]
    fn header_forbids_production() {
        let src = include_str!("shd_eprop_baseline.rs");
        assert!(src.contains("MUST NEVER BE THE PRODUCTION LEARNER"));
        assert!(src.contains("GC1"));
    }

    /// Regression guard for the 56× effective-learning-rate asymmetry that made
    /// the e-prop "ceiling" score below the DFA treatment at every SHD width.
    ///
    /// The DFA feedback matrix and the readout matrix must be initialised at the
    /// same scale, so the two arms apply hidden updates of comparable magnitude
    /// at a shared learning rate.
    #[test]
    fn dfa_feedback_and_readout_share_init_scale() {
        let n_in = 24;
        let t = 12;
        let n_classes = 5;
        for hidden in [32usize, 128, 512] {
            let train = toy_data(4, n_in, t, n_classes, 1);
            let cfg = ShdTrainConfig {
                hidden,
                n_classes,
                lr: 0.02,
                beta: 5.0,
                epochs: 1,
            };
            let dfa = ShdDfa::new(&train[0], cfg, 0xD1A0_0001);
            let ceil = ShdEpropCeiling::new(&train[0], cfg, 0xD1A0_0001);

            let rms = |v: &[f32]| (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt();
            let b_rms = rms(&dfa.feedback);
            let w_rms = rms(&ceil.arch.wout);
            let ratio = if b_rms >= w_rms {
                b_rms / w_rms
            } else {
                w_rms / b_rms
            };
            assert!(
                ratio < 1.5,
                "hidden={hidden}: DFA feedback RMS {b_rms:.6} vs readout RMS {w_rms:.6} \
                 (ratio {ratio:.2}); the arms would run at different effective learning rates"
            );
            // And the scale must track `hidden`, not be a fixed U[-1,1].
            assert!(
                b_rms < 0.5,
                "hidden={hidden}: feedback RMS {b_rms:.6} looks like unscaled U[-1,1]"
            );
        }
    }

    /// After training, the two arms' realised hidden modulators stay within
    /// [`MODULATOR_PARITY_TOLERANCE`] of each other.
    ///
    /// **This guard was vacuous until 2026-08-22, then failed, then was repaired.**
    ///
    /// While the hidden layer was silent both modulators were driven by the same
    /// sub-threshold surrogate values and the ratio was **1.03** — the check could
    /// not fail whatever the rules did. Repairing the initialisation
    /// (`PREREG_2026-08-22_SILENT_INITIALISATION_REPAIR.md`) made it capable of
    /// failing for the first time, and it failed immediately at **5.08**.
    ///
    /// The cause was real and always there: e-prop transports `wout`, which is
    /// trained, while DFA's feedback is frozen, so the two step at different rates
    /// once the arms actually learn. `ShdEpropCeiling` now rescales the transport
    /// matrix to [`shd_out_scale`] — the frozen feedback's own scale — under
    /// `PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md`, criterion E-1.
    ///
    /// **The tolerance was not moved.** It is the same 3.5 the guard failed against.
    #[test]
    fn repaired_transport_scale_keeps_the_two_arms_comparable() {
        let (dfa, ceil) = parity_fixture(false);
        let ratio = ModulatorScale::ratio(&dfa, &ceil);
        assert!(
            ratio <= MODULATOR_PARITY_TOLERANCE,
            "hidden-modulator RMS ratio is {ratio:.2}, outside the \
             {MODULATOR_PARITY_TOLERANCE} tolerance: DFA={:.3e}, e-prop={:.3e}",
            dfa.rms(),
            ceil.rms(),
        );
    }

    /// The parity guard above is **not** vacuous: the same measurement on the
    /// un-rescaled rule still violates the same tolerance.
    ///
    /// Registered as criterion E-2. Without this, rescaling could have satisfied
    /// E-1 by making the ratio identically 1 by construction, which would be a
    /// check that cannot fail — the failure mode this workspace keeps finding.
    #[test]
    fn defect_raw_transport_outgrows_dfa_feedback_once_the_network_spikes() {
        let (dfa, raw) = parity_fixture(true);
        let ratio = ModulatorScale::ratio(&dfa, &raw);
        assert!(
            ratio > MODULATOR_PARITY_TOLERANCE,
            "raw transport now scores {ratio:.2}, inside the \
             {MODULATOR_PARITY_TOLERANCE} tolerance. The parity guard can no \
             longer fail and is worthless; find out why before trusting it.",
        );
    }

    /// The rescale reaches exactly one arm.
    ///
    /// Registered as criterion E-3. `ShdBroadcastPm1`, `ShdRlReinforceFb` and
    /// `ShdSuperSpikeCeiling` do not transport through `wout` and must be
    /// numerically untouched by this repair. Structural argument alone is not
    /// enough — this asserts it against the source, so a later edit that reuses
    /// the primitive in another arm fails here instead of silently moving that
    /// arm's numbers.
    #[test]
    fn the_transport_rescale_reaches_only_the_eprop_ceiling() {
        let whole = include_str!("shd_eprop_baseline.rs");
        // Scan production code only. The test module quotes the symbol itself,
        // and counting those would make this pass for the wrong reason.
        let src = whole
            .split_once("\n#[cfg(test)]\n")
            .expect("this module has a test section")
            .0;
        let call_sites: Vec<&str> = src
            .lines()
            .filter(|l| l.contains("rescale_rms_to(") && !l.trim_start().starts_with("//"))
            .collect();
        assert_eq!(
            call_sites.len(),
            1,
            "the transport rescale must have exactly one call site, found {call_sites:#?}"
        );

        // That call must sit inside `impl ShdEpropCeiling`.
        let call_at = src.find("rescale_rms_to(").expect("call site");
        let impl_at = src.find("impl ShdEpropCeiling {").expect("impl block");
        let next_impl = src[impl_at + 1..]
            .find("\nimpl ")
            .map(|o| impl_at + 1 + o)
            .unwrap_or(src.len());
        assert!(
            impl_at < call_at && call_at < next_impl,
            "the rescale escaped `impl ShdEpropCeiling`"
        );
    }

    /// Shared fixture for the two parity tests, so they differ in exactly one bit.
    fn parity_fixture(raw: bool) -> (ModulatorScale, ModulatorScale) {
        let n_in = 24;
        let t = 12;
        let n_classes = 5;
        let train = toy_data(60, n_in, t, n_classes, 1);
        let test = toy_data(20, n_in, t, n_classes, 2);
        let cfg = ShdTrainConfig {
            hidden: 64,
            n_classes,
            lr: 0.02,
            beta: 5.0,
            epochs: 5,
        };
        let mut dfa = ShdDfa::new(&train[0], cfg, 0xD1A0_0002);
        dfa.train_and_evaluate(cfg.epochs, &train, &test);
        let mut ceil = ShdEpropCeiling::new(&train[0], cfg, 0xD1A0_0002);
        if raw {
            ceil = ceil.with_raw_transport();
        }
        ceil.train_and_evaluate(cfg.epochs, &train, &test);
        (dfa.modulator_scale(), ceil.modulator_scale())
    }

    /// The e-prop hidden update must transport through the pre-update readout.
    #[test]
    fn eprop_transport_uses_pre_update_readout() {
        let src = include_str!("shd_eprop_baseline.rs");
        assert!(
            src.contains("let wout_snapshot = self.arch.wout.clone();"),
            "e-prop must snapshot wout before the readout update"
        );
        assert!(
            src.contains("di += wout_snapshot[k * h + i] * delta[k];"),
            "e-prop must transport through the snapshot, not the updated wout"
        );
    }

    #[test]
    fn eprop_ceiling_learns_toy_above_chance() {
        let n_in = 24;
        let t = 12;
        let n_classes = 5;
        let train = toy_data(80, n_in, t, n_classes, 1);
        let test = toy_data(40, n_in, t, n_classes, 2);
        let cfg = ShdTrainConfig {
            hidden: 48,
            n_classes,
            lr: 0.05,
            beta: 5.0,
            epochs: 25,
        };
        let mut ceil = ShdEpropCeiling::new(&train[0], cfg, 0xCE17);
        let r = ceil.train_and_evaluate(cfg.epochs, &train, &test);
        assert_eq!(r.label, SHD_EPROP_CEILING_LABEL);
        assert!(
            r.accuracy > 1.0 / n_classes as f32 + 0.05,
            "e-prop should beat chance on toy; got {}",
            r.accuracy
        );
    }

    #[test]
    fn superspike_ceiling_learns_toy_above_chance() {
        let n_in = 24;
        let t = 12;
        let n_classes = 5;
        let train = toy_data(80, n_in, t, n_classes, 1);
        let test = toy_data(40, n_in, t, n_classes, 2);
        let cfg = ShdTrainConfig {
            hidden: 48,
            n_classes,
            lr: 0.05,
            beta: 5.0,
            epochs: 25,
        };
        let mut ceil = ShdSuperSpikeCeiling::new(&train[0], cfg, 0x5055);
        let r = ceil.train_and_evaluate(cfg.epochs, &train, &test);
        assert_eq!(r.label, SHD_SUPERSPIKE_CEILING_LABEL);
        assert!(
            r.accuracy > 1.0 / n_classes as f32 + 0.05,
            "SuperSpike BPTT should beat chance on toy; got {}",
            r.accuracy
        );
    }

    #[test]
    fn rl_fb_shares_reinforce_feedback_lineage() {
        let ex = &toy_data(1, 16, 8, 4, 9)[0];
        let cfg = ShdTrainConfig {
            hidden: 24,
            n_classes: 4,
            lr: 0.05,
            beta: 5.0,
            epochs: 1,
        };
        let seed = 0x54D0_F1B0_0001;
        let arm = ShdRlReinforceFb::new(ex, cfg, seed);
        let product = ReinforceFeedback::new(cfg.hidden, seed);
        assert_eq!(arm.feedback_weights(), product.weights());
    }

    #[test]
    fn rl_fb_runs_on_toy() {
        let train = toy_data(40, 16, 8, 4, 3);
        let test = toy_data(20, 16, 8, 4, 4);
        let cfg = ShdTrainConfig {
            hidden: 32,
            n_classes: 4,
            lr: 0.05,
            beta: 5.0,
            epochs: 8,
        };
        let mut arm = ShdRlReinforceFb::new(&train[0], cfg, 0xF1B0_54D0);
        let r = arm.train_and_evaluate(cfg.epochs, &train, &test);
        assert_eq!(r.label, SHD_RL_REINFORCE_FB_LABEL);
        assert!(r.accuracy.is_finite());
        assert!(r.loss.is_finite());
    }

    // ---- characterization of a known defect, 2026-08-22 --------------------
    //
    // `shd-scientific-sweep` reported the e-prop ceiling at 0.2140 against a
    // chance of 0.2000. These localise that and **pin the broken behaviour**.
    // They assert the defect, not a fix: there is no fix, and a green suite must
    // not imply one. A repair makes them fail, which is the intended signal.
    //
    // See `FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md`.

    fn disjoint_fixture(
        n: usize,
        n_in: usize,
        t: usize,
        classes: usize,
        seed: u64,
    ) -> Vec<ShdExample> {
        let mut rng = Rng::new(seed);
        (0..n)
            .map(|_| {
                let label = rng.gen_index(classes) as u32;
                let mut frames = vec![0.0f32; t * n_in];
                for _ in 0..(t / 3).max(1) {
                    let tt = rng.gen_index(t);
                    let c = (label as usize * 3 + rng.gen_index(3)) % n_in;
                    frames[tt * n_in + c] = 1.0;
                }
                ShdExample {
                    frames,
                    t,
                    n_in,
                    label,
                }
            })
            .collect()
    }

    fn diag_cfg(classes: usize) -> ShdTrainConfig {
        ShdTrainConfig {
            hidden: 24,
            n_classes: classes,
            lr: 0.05,
            beta: 5.0,
            epochs: 60,
        }
    }

    /// **Repaired 2026-08-22.** Was a constant predictor at every configuration;
    /// now uses its whole output layer.
    #[test]
    fn repaired_the_eprop_ceiling_uses_more_than_one_class() {
        for (classes, seed) in [(5usize, 11u64), (5, 99), (4, 11)] {
            let train = toy_data(120, 24, 16, classes, seed);
            let test = toy_data(60, 24, 16, classes, seed + 1);
            let mut eprop = ShdEpropCeiling::new(&train[0], diag_cfg(classes), 7);
            eprop.train_and_evaluate(60, &train, &test);
            let preds = eprop.arch.predictions(&test);
            let mut distinct = preds.clone();
            distinct.sort_unstable();
            distinct.dedup();
            assert!(
                distinct.len() > 1,
                "classes {classes} seed {seed}: the ceiling collapsed back to \
                 {} predicted class - the silent-initialisation repair has regressed",
                distinct.len()
            );
        }
    }

    /// A working reference solves the same fixtures, so the forward pass and the
    /// data are fine and the defect is in the local arm.
    #[test]
    fn defect_is_localised_superspike_solves_the_same_fixtures() {
        for build in [
            toy_data as fn(usize, usize, usize, usize, u64) -> Vec<ShdExample>,
            disjoint_fixture,
        ] {
            let train = build(120, 24, 16, 5, 11);
            let test = build(60, 24, 16, 5, 12);
            let acc = ShdSuperSpikeCeiling::new(&train[0], diag_cfg(5), 7)
                .train_and_evaluate(60, &train, &test)
                .accuracy;
            assert!(acc > 0.99, "SuperSpike scored {acc}, expected ~1.0");
        }
    }

    /// The modulator must stay non-zero and data-dependent after the repair.
    ///
    /// An earlier revision read this as evidence that the mechanism **differed**
    /// from `MatchedDeepGradient`'s silence. That was wrong: the modulator is
    /// non-zero below threshold because the surrogate derivative is, not because
    /// anything spiked. Both defects were the same silent initialisation.
    #[test]
    fn repaired_the_modulator_stays_live_and_data_dependent() {
        let mut scales = Vec::new();
        for seed in [11u64, 99] {
            let train = toy_data(120, 24, 16, 5, seed);
            let test = toy_data(60, 24, 16, 5, seed + 1);
            let mut eprop = ShdEpropCeiling::new(&train[0], diag_cfg(5), 7);
            eprop.train_and_evaluate(60, &train, &test);
            let rms = eprop.modulator_scale().rms();
            assert!(
                rms > 1e-3,
                "modulator collapsed to {rms} - this is now the \
                                 silence mechanism, not the readout one"
            );
            scales.push(rms);
        }
        assert_ne!(
            scales[0].to_bits(),
            scales[1].to_bits(),
            "the modulator is identical on different data - it has stopped \
             depending on the input and the diagnosis has changed"
        );
    }

    /// **Repaired 2026-08-22.** Neither local arm used to escape the constant
    /// predictor at any width; both now do.
    ///
    /// This closes the question left open in
    /// `FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md` §3,
    /// which declined to call `ShdDfa`'s collapse a defect because the sweep had
    /// reported it at 1.0000 under a different configuration. Width (16..128,
    /// including the sweep's 64) and budget (1..60, spanning the sweep's 30) are
    /// both ruled out.
    ///
    /// Asserts the defect, not a fix. A repair makes this fail, which is the
    /// intended signal.
    #[test]
    fn repaired_both_local_arms_escape_the_constant_predictor() {
        let (n_in, t, classes) = (24usize, 16usize, 5usize);
        let train = disjoint_fixture(120, n_in, t, classes, 11);
        let test = disjoint_fixture(60, n_in, t, classes, 12);

        for hidden in [24usize, 64, 128] {
            let cfg = ShdTrainConfig {
                hidden,
                n_classes: classes,
                lr: 0.05,
                beta: 5.0,
                epochs: 60,
            };
            for label in ["dfa", "eprop"] {
                let preds = if label == "dfa" {
                    let mut arm = ShdDfa::new(&train[0], cfg, 7);
                    arm.train_and_evaluate(60, &train, &test);
                    arm.arch.predictions(&test)
                } else {
                    let mut arm = ShdEpropCeiling::new(&train[0], cfg, 7);
                    arm.train_and_evaluate(60, &train, &test);
                    arm.arch.predictions(&test)
                };
                let mut distinct = preds.clone();
                distinct.sort_unstable();
                distinct.dedup();
                assert!(
                    distinct.len() > 1,
                    "{label} at hidden {hidden} collapsed to {} predicted class - \
                     the silent-initialisation repair has regressed",
                    distinct.len()
                );
            }
        }
    }

    /// The same fixture is solved by a working reference, so the collapse above
    /// is about the local arms and not about the task or the budget.
    #[test]
    fn defect_superspike_escapes_the_same_fixture_by_epoch_twenty() {
        let train = disjoint_fixture(120, 24, 16, 5, 11);
        let test = disjoint_fixture(60, 24, 16, 5, 12);
        let cfg = ShdTrainConfig {
            hidden: 64,
            n_classes: 5,
            lr: 0.05,
            beta: 5.0,
            epochs: 20,
        };
        let acc = ShdSuperSpikeCeiling::new(&train[0], cfg, 7)
            .train_and_evaluate(20, &train, &test)
            .accuracy;
        assert!(
            acc > 0.99,
            "SuperSpike scored {acc} at 20 epochs, expected ~1.0"
        );
    }

    /// Was the modulator-parity violation caused by the repair, or merely
    /// revealed by it? Compare the ratio at the old scale and the new one.
    #[test]
    fn diagnostic_was_modulator_parity_masked_by_silence() {
        let (n_in, t, classes) = (24usize, 12usize, 5usize);
        let train = toy_data(60, n_in, t, classes, 1);
        let test = toy_data(20, n_in, t, classes, 2);
        let cfg = ShdTrainConfig {
            hidden: 64,
            n_classes: classes,
            lr: 0.02,
            beta: 5.0,
            epochs: 5,
        };
        // `SILENCE_FREE_INPUT_SCALE / 2.8` recovers the original 0.35 exactly.
        for (label, rescale) in [
            ("old scale 0.35 (silent)", 0.35 / SILENCE_FREE_INPUT_SCALE),
            ("new scale 2.8 (spiking)", 1.0f32),
        ] {
            let mut dfa = ShdDfa::new(&train[0], cfg, 0xD1A0_0002);
            for w in dfa.arch.win.iter_mut() {
                *w *= rescale;
            }
            dfa.train_and_evaluate(cfg.epochs, &train, &test);
            let mut ceil = ShdEpropCeiling::new(&train[0], cfg, 0xD1A0_0002);
            for w in ceil.arch.win.iter_mut() {
                *w *= rescale;
            }
            ceil.train_and_evaluate(cfg.epochs, &train, &test);
            let (a, b) = (dfa.modulator_scale(), ceil.modulator_scale());
            let (r, _, _) = ceil.arch.forward(&test[0].frames);
            println!(
                "{label}: hidden rate {:.6}  DFA {:.3e}  e-prop {:.3e}  ratio {:.2}",
                r.iter().sum::<f32>() / r.len() as f32,
                a.rms(),
                b.rms(),
                ModulatorScale::ratio(&a, &b)
            );
        }
    }
}
