//! Causal size-science mac-probe protocol (`c1-mac-probe-*`) and Mac/Micro
//! isolate stress (`c1-micro-*`).
//!
//! Synapse-matched width ladder and density cross with disclosed hold/scale:
//! fixed `k_wta`, plumbed `max_fan_out`, `init_w ∝ 1/√mean_fan_in`, readout
//! effective-gain normalization. **Not Gate G2.** Refuses full multi-condition
//! C1 when `n_hidden ≥ 2000` (isolate-only).
//!
//! Micro isolate (`c1-micro-*`): engineering capacity stress after G2 FAIL —
//! `n_hidden ∈ {1000,10000}`, `max_fan_out=256`, activity-scaled `k_wta` so
//! `k/N ∈ [0.005, 0.03]`, `matched_budget_repeat=false`, isolate
//! `local-assembly` only. **Not** a Foundation R0 unlock.
//!
//! Foundation Micro (`c1-micro-foundation-*`): **fixed** `k_wta` (not αN),
//! target measured nnz ≈ 10⁶ — distinct from overnight syn-matched-1e5 @ N=1e4
//! and from activity-scaled micro isolate.
//!
//! H2 dfa-live size protocol (`c1-mac-probe-*-size`): preregistered N=2k
//! syn-matched scientific seeds for Accept / Reject-floor / Reject-gap —
//! distinct from overnight quick H2 smokes and frozen v20.

use crate::Config;

/// Mac-probe protocol version (hashed).
pub const C1_MAC_PROBE_PROTOCOL_VERSION: u64 = 1;
pub const C1_MAC_PROBE_EXPERIMENT: &str = "c1-mac-probe";
pub const C1_MAC_PROBE_HASH_PREFIX: &str = "c1-mac-probe-";
/// Size-protocol marker mixed into H2 dfa-live width-transfer hashes only.
pub const C1_MAC_PROBE_SIZE_PROTOCOL_VERSION: u64 = 1;

/// Micro isolate protocol (activity-scaled; separate hash family).
pub const C1_MICRO_PROTOCOL_VERSION: u64 = 1;
pub const C1_MICRO_EXPERIMENT: &str = "c1-micro";
pub const C1_MICRO_HASH_PREFIX: &str = "c1-micro-";
/// Foundation Microcircuit protocol (fixed-k ~1e6 syn; `c1-micro-foundation-*`).
pub const C1_MICRO_FOUNDATION_PROTOCOL_VERSION: u64 = 1;
pub const C1_MICRO_FOUNDATION_EXPERIMENT: &str = "c1-micro-foundation";
/// Target activity fraction mid-band of [0.005, 0.03].
pub const MICRO_TARGET_ACTIVITY: f32 = 0.01;
pub const MICRO_ACTIVITY_MIN: f32 = 0.005;
pub const MICRO_ACTIVITY_MAX: f32 = 0.03;
/// WiringPrior default / disclosed micro fan-out cap.
pub const MICRO_MAX_FAN_OUT: usize = 256;

/// Target synapse budget for syn-matched presets (~1e5 nnz ±10%).
pub const MAC_PROBE_TARGET_NNZ: usize = 100_000;
/// Foundation Microcircuit target (~10⁶ nnz ±20% Pass band).
pub const FOUNDATION_MICRO_TARGET_NNZ: usize = 1_000_000;
/// Foundation Micro geometry: N=10k · fan=100 · k=8 (fixed) → ≈1e6 nnz.
pub const FOUNDATION_MICRO_N_HIDDEN: usize = 10_000;
pub const FOUNDATION_MICRO_FAN_OUT: usize = 100;
/// Pass band: measured nnz ∈ [0.8e6, 1.2e6].
pub const FOUNDATION_MICRO_NNZ_LO: usize = 800_000;
pub const FOUNDATION_MICRO_NNZ_HI: usize = 1_200_000;
/// Engineering budgets (Pass/Fail floors; not G2).
pub const FOUNDATION_MICRO_RSS_BUDGET_BYTES: u64 = 48 * 1024 * 1024 * 1024;
pub const FOUNDATION_MICRO_WALL_SECS_PER_SEED: u64 = 1200;

/// Fixed k-WTA on the primary ladder (not α-scaled with N).
pub const MAC_PROBE_K_WTA: usize = 8;
/// Reference fan-in for init rescale disclosure (`init_w_eff = init_w * √(ref / mean)`).
pub const MAC_PROBE_REF_MEAN_FAN_IN: f32 = 45.0;
/// Reference mean readout fan-in (hidden→readout) at N=128 baseline (~p=0.5).
pub const MAC_PROBE_REF_MEAN_READOUT_FAN_IN: f32 = 64.0;
/// Refuse full multi-condition C1 at or above this width.
pub const MAC_PROBE_FULL_C1_REFUSE_N: usize = 2000;

/// H2 dfa-live size protocol: N=2k syn-matched scientific seeds.
pub const DFA_LIVE_SIZE_N_HIDDEN: usize = 2000;
pub const DFA_LIVE_SIZE_N_SEEDS: usize = 8;
/// Accuracy floor for Accept / Reject-floor (preregistered; not G2 0.65).
pub const DFA_LIVE_SIZE_ACC_FLOOR: f32 = 0.60;
/// Gap LCB clear threshold vs pm1 reference (Reject-gap if floor cleared but LCB ≤ this).
pub const DFA_LIVE_SIZE_GAP_LCB_CLEAR: f32 = 0.0;

/// Plasticity / feedback mode under mac-probe geometry (new hashes only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MacProbeMode {
    /// Live local-assembly with broadcast ±1 (negative control).
    Pm1,
    /// Structured frozen B (v15-family mapping @ new N; fresh hash).
    StructuredFb,
    /// Graded DFA live (v20-family @ new N; fresh hash).
    DfaLive,
}

impl MacProbeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pm1 => "pm1",
            Self::StructuredFb => "structured-fb",
            Self::DfaLive => "dfa-live",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pm1" | "pm-1" | "+-1" | "±1" => Some(Self::Pm1),
            "structured-fb" | "sfb" | "structured" => Some(Self::StructuredFb),
            "dfa-live" | "dfa" | "graded-dfa" => Some(Self::DfaLive),
            _ => None,
        }
    }

    pub fn experiment_suffix(self) -> &'static str {
        match self {
            Self::Pm1 => "",
            Self::StructuredFb => "-sfb",
            Self::DfaLive => "-dfa-live",
        }
    }
}

/// Wiring regime under the fan-out cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WiringRegime {
    /// Expected degree ≪ max_fan_out; cap inactive.
    Bernoulli,
    /// Cap binds (expected degree ≳ max_fan_out).
    Capped,
}

impl WiringRegime {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bernoulli => "Bernoulli",
            Self::Capped => "capped",
        }
    }

    /// Tag from expected intra-area degree vs cap.
    pub fn from_expected_degree(expected: f32, max_fan_out: usize) -> Self {
        if expected > max_fan_out as f32 * 0.95 {
            Self::Capped
        } else {
            Self::Bernoulli
        }
    }
}

/// Public config for causal mac-probe size science.
#[derive(Clone, Debug, PartialEq)]
pub struct MacProbeConfig {
    pub base: Config,
    pub protocol_version: u64,
    pub max_fan_out: usize,
    pub mode: MacProbeMode,
    /// When true: `init_w_eff = init_w * sqrt(REF_MEAN_FAN_IN / mean_fan_in)`.
    pub init_w_rescale: bool,
    /// When true: boost so `boost * mean_readout_fan_in ≈` baseline gain.
    pub readout_gain_normalize: bool,
    /// Syn-matched / Foundation Micro target (informational; fan usually set via preset).
    pub target_nnz: usize,
    pub quick: bool,
    /// H2 dfa-live width-transfer size protocol (distinct from overnight quick H2).
    pub size_protocol: bool,
}

impl MacProbeConfig {
    /// Synapse-matched preset: nnz≈1e5, k=8, fan≈round(1e5/N).
    pub fn syn_matched(n_hidden: usize, quick: bool) -> Self {
        let fan = syn_matched_fan_out(n_hidden);
        Self::geometry(n_hidden, fan, MAC_PROBE_K_WTA, MacProbeMode::Pm1, quick)
    }

    /// Density-cross / custom geometry with disclosed rescale hooks.
    pub fn geometry(
        n_hidden: usize,
        max_fan_out: usize,
        k_wta: usize,
        mode: MacProbeMode,
        quick: bool,
    ) -> Self {
        let mut base = if quick {
            Config::c1_quick()
        } else {
            Config::c1_default()
        };
        // Fresh experiment name ⇒ new hash lineage (does not remassage v13–v20).
        base.experiment = format!(
            "{C1_MAC_PROBE_EXPERIMENT}{}-n{n_hidden}-f{max_fan_out}",
            mode.experiment_suffix()
        );
        base.n_hidden = n_hidden;
        base.k_wta = k_wta;
        base.matched_budget_repeat = false;
        base.max_fan_out = max_fan_out;
        base.init_w_rescale = true;
        base.readout_gain_normalize = true;
        // Activity band: k/N shrinks at large N with fixed k=8.
        let kn = k_wta as f32 / n_hidden.max(1) as f32;
        base.activity_sparsity_min = (kn * 0.25).max(1e-5);
        base.activity_sparsity_max = (kn * 4.0).min(0.25).max(base.activity_sparsity_min);
        if quick {
            base.n_seeds = 2;
            base.n_train = 16;
            base.n_test = 12;
            base.bptt_epochs = 8;
        }
        // Mode plasticity: map onto existing protocol experiment prefixes so
        // runner hooks (SFB / DFA-live) fire, while mac-probe hash stays primary.
        match mode {
            MacProbeMode::Pm1 => {}
            MacProbeMode::StructuredFb => {
                // Runner keys off experiment prefix for structured B.
                base.experiment = format!(
                    "{}{}-n{n_hidden}-f{max_fan_out}",
                    crate::config::C1_STRUCTURED_FB_EXPERIMENT_PREFIX,
                    "-mac",
                );
            }
            MacProbeMode::DfaLive => {
                base.experiment = format!(
                    "{}{}-n{n_hidden}-f{max_fan_out}",
                    crate::config::C1_DFA_LIVE_EXPERIMENT_PREFIX,
                    "-mac",
                );
            }
        }
        Self {
            protocol_version: C1_MAC_PROBE_PROTOCOL_VERSION,
            max_fan_out,
            mode,
            init_w_rescale: true,
            readout_gain_normalize: true,
            target_nnz: MAC_PROBE_TARGET_NNZ,
            quick,
            size_protocol: false,
            base,
        }
    }

    /// Micro isolate: N∈{1000,10000}, fan=256, activity-scaled k, isolate-only.
    pub fn micro_isolate(n_hidden: usize, quick: bool) -> Self {
        assert!(
            n_hidden == 1_000 || n_hidden == 10_000 || n_hidden == 100_000,
            "micro isolate n_hidden must be 1e3, 1e4, or optional 1e5"
        );
        let k = scaled_k_wta(n_hidden);
        let mut base = if quick {
            Config::c1_quick()
        } else {
            Config::c1_default()
        };
        base.experiment = format!("{C1_MICRO_EXPERIMENT}-n{n_hidden}-f{MICRO_MAX_FAN_OUT}");
        base.n_hidden = n_hidden;
        base.k_wta = k;
        base.matched_budget_repeat = false;
        base.max_fan_out = MICRO_MAX_FAN_OUT;
        base.init_w_rescale = true;
        base.readout_gain_normalize = true;
        // Enforce disclosed activity band [0.005, 0.03].
        base.activity_sparsity_min = MICRO_ACTIVITY_MIN;
        base.activity_sparsity_max = MICRO_ACTIVITY_MAX;
        // Disable dense SurrogateLif / matched-budget G2 arms for isolate stress.
        base.use_surrogate_lif_reference = false;
        if quick {
            base.n_seeds = 2;
            base.n_train = 16;
            base.n_test = 12;
            base.bptt_epochs = 8;
        }
        Self {
            protocol_version: C1_MICRO_PROTOCOL_VERSION,
            max_fan_out: MICRO_MAX_FAN_OUT,
            mode: MacProbeMode::Pm1,
            init_w_rescale: true,
            readout_gain_normalize: true,
            target_nnz: n_hidden.saturating_mul(MICRO_MAX_FAN_OUT),
            quick,
            size_protocol: false,
            base,
        }
    }

    /// Foundation Microcircuit ≈10⁶ synapses: N=10k, fan=100, **fixed** k=8.
    ///
    /// Distinct from overnight syn-matched-1e5 @ N=1e4 (fan=10) and from
    /// activity-scaled `micro_isolate` (αN k, fan=256). Isolate-only; refuses
    /// full dense+SurrogateLif C1.
    pub fn foundation_micro(quick: bool) -> Self {
        let n = FOUNDATION_MICRO_N_HIDDEN;
        let fan = FOUNDATION_MICRO_FAN_OUT;
        let k = MAC_PROBE_K_WTA;
        let mut base = if quick {
            Config::c1_quick()
        } else {
            Config::c1_default()
        };
        base.experiment = format!("{C1_MICRO_FOUNDATION_EXPERIMENT}-n{n}-f{fan}");
        base.n_hidden = n;
        base.k_wta = k;
        base.matched_budget_repeat = false;
        base.max_fan_out = fan;
        base.init_w_rescale = true;
        base.readout_gain_normalize = true;
        base.use_surrogate_lif_reference = false;
        // Fixed-k activity band (same formula as mac-probe ladder; not MICRO αN band).
        let kn = k as f32 / n.max(1) as f32;
        base.activity_sparsity_min = (kn * 0.25).max(1e-5);
        base.activity_sparsity_max = (kn * 4.0).min(0.25).max(base.activity_sparsity_min);
        if quick {
            base.n_seeds = 2;
            base.n_train = 16;
            base.n_test = 12;
            base.bptt_epochs = 8;
        }
        Self {
            protocol_version: C1_MICRO_FOUNDATION_PROTOCOL_VERSION,
            max_fan_out: fan,
            mode: MacProbeMode::Pm1,
            init_w_rescale: true,
            readout_gain_normalize: true,
            target_nnz: FOUNDATION_MICRO_TARGET_NNZ,
            quick,
            size_protocol: false,
            base,
        }
    }

    /// H2 dfa-live / SFB / pm1 width-transfer under preregistered size protocol.
    ///
    /// Geometry: N=2000 syn-matched (fan=50), k=8. Scientific: n_seeds=8
    /// (not overnight quick n_test=12 smoke; not frozen v20).
    pub fn dfa_live_size(mode: MacProbeMode, quick: bool) -> Self {
        let n = DFA_LIVE_SIZE_N_HIDDEN;
        let fan = syn_matched_fan_out(n);
        let mut c = Self::geometry(n, fan, MAC_PROBE_K_WTA, mode, quick);
        c.size_protocol = true;
        c.protocol_version = C1_MAC_PROBE_SIZE_PROTOCOL_VERSION;
        // Tag experiment so hashes diverge from overnight mac H2 smokes.
        match mode {
            MacProbeMode::Pm1 => {
                c.base.experiment = format!("{C1_MAC_PROBE_EXPERIMENT}-size-n{n}-f{fan}");
            }
            MacProbeMode::StructuredFb => {
                c.base.experiment = format!(
                    "{}{}-size-n{n}-f{fan}",
                    crate::config::C1_STRUCTURED_FB_EXPERIMENT_PREFIX,
                    "-mac",
                );
            }
            MacProbeMode::DfaLive => {
                c.base.experiment = format!(
                    "{}{}-size-n{n}-f{fan}",
                    crate::config::C1_DFA_LIVE_EXPERIMENT_PREFIX,
                    "-mac",
                );
            }
        }
        if !quick {
            c.base.n_seeds = DFA_LIVE_SIZE_N_SEEDS;
            c.base.scientific_n_seeds = DFA_LIVE_SIZE_N_SEEDS;
            // Keep scientific train/test from c1_default (80/40).
        }
        c
    }

    /// Known presets: syn-matched N∈{512,2000,10000} quick+scientific; density fans;
    /// micro isolate N∈{1000,10000} (+ optional 1e5); Foundation Micro; H2 size protocol.
    pub fn known_presets() -> Vec<Self> {
        let mut out = Vec::new();
        for &n in &[512usize, 2000, 10000] {
            out.push(Self::syn_matched(n, true));
            out.push(Self::syn_matched(n, false));
        }
        for &fan in &[10usize, 32, 64, 256] {
            out.push(Self::geometry(
                2000,
                fan,
                MAC_PROBE_K_WTA,
                MacProbeMode::Pm1,
                true,
            ));
            out.push(Self::geometry(
                2000,
                fan,
                MAC_PROBE_K_WTA,
                MacProbeMode::Pm1,
                false,
            ));
        }
        // H2 mode trio at syn-matched N=2000 (overnight lineage; unchanged).
        for mode in [
            MacProbeMode::Pm1,
            MacProbeMode::StructuredFb,
            MacProbeMode::DfaLive,
        ] {
            let fan = syn_matched_fan_out(2000);
            out.push(Self::geometry(2000, fan, MAC_PROBE_K_WTA, mode, true));
            out.push(Self::geometry(2000, fan, MAC_PROBE_K_WTA, mode, false));
        }
        // H2 size protocol (new hashes; scientific n_seeds=8).
        for mode in [
            MacProbeMode::Pm1,
            MacProbeMode::StructuredFb,
            MacProbeMode::DfaLive,
        ] {
            out.push(Self::dfa_live_size(mode, true));
            out.push(Self::dfa_live_size(mode, false));
        }
        // Micro isolate ladder (activity-scaled k; fan=256).
        for &n in &[1_000usize, 10_000] {
            out.push(Self::micro_isolate(n, true));
            out.push(Self::micro_isolate(n, false));
        }
        out.push(Self::micro_isolate(100_000, true));
        // Foundation Micro ~1e6 (fixed k).
        out.push(Self::foundation_micro(true));
        out.push(Self::foundation_micro(false));
        out
    }

    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x0100_0000_01b3);
        }
        mix(&mut h, self.protocol_version);
        for b in self.base.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.base.master_seed);
        mix(&mut h, self.base.n_seeds as u64);
        mix(&mut h, self.base.n_hidden as u64);
        mix(&mut h, self.base.k_wta as u64);
        mix(&mut h, self.max_fan_out as u64);
        mix(&mut h, self.base.p_sparse.to_bits() as u64);
        mix(&mut h, self.base.init_w.to_bits() as u64);
        mix(&mut h, u64::from(self.base.matched_budget_repeat));
        mix(&mut h, u64::from(self.init_w_rescale));
        mix(&mut h, u64::from(self.readout_gain_normalize));
        mix(&mut h, self.mode as u64);
        mix(&mut h, u64::from(self.quick));
        mix(&mut h, self.base.n_train as u64);
        mix(&mut h, self.base.n_test as u64);
        mix(&mut h, self.target_nnz as u64);
        // Formula markers (disclose in overnight note).
        mix(&mut h, MAC_PROBE_REF_MEAN_FAN_IN.to_bits() as u64);
        mix(&mut h, MAC_PROBE_REF_MEAN_READOUT_FAN_IN.to_bits() as u64);
        mix(&mut h, u64::from(self.size_protocol));
        if self.size_protocol {
            mix(&mut h, C1_MAC_PROBE_SIZE_PROTOCOL_VERSION);
            mix(&mut h, DFA_LIVE_SIZE_N_SEEDS as u64);
            mix(&mut h, DFA_LIVE_SIZE_ACC_FLOOR.to_bits() as u64);
        }
        if self.is_foundation_micro() {
            mix(&mut h, C1_MICRO_FOUNDATION_PROTOCOL_VERSION);
            mix(&mut h, FOUNDATION_MICRO_TARGET_NNZ as u64);
        }
        h
    }

    pub fn hash_string(&self) -> String {
        if self.is_foundation_micro() || self.base.experiment.starts_with(C1_MICRO_EXPERIMENT) {
            format!("{C1_MICRO_HASH_PREFIX}{:016x}", self.hash())
        } else {
            format!("{C1_MAC_PROBE_HASH_PREFIX}{:016x}", self.hash())
        }
    }

    pub fn from_hash(hash: &str) -> Option<Self> {
        let trimmed = hash.trim();
        Self::known_presets()
            .into_iter()
            .find(|p| trimmed.eq_ignore_ascii_case(&p.hash_string()))
    }

    /// True for activity-scaled micro isolate family (not Foundation Micro).
    pub fn is_micro_isolate(&self) -> bool {
        self.base.experiment.starts_with(C1_MICRO_EXPERIMENT) && !self.is_foundation_micro()
    }

    /// True for Foundation Microcircuit ~1e6 fixed-k protocol.
    pub fn is_foundation_micro(&self) -> bool {
        self.base
            .experiment
            .starts_with(C1_MICRO_FOUNDATION_EXPERIMENT)
    }

    /// Predicted nnz under cap-dominated recurrent prior: ≈ N · min(p·N, fan).
    pub fn predicted_nnz(&self) -> usize {
        let n = self.base.n_hidden;
        let expected_deg = self.base.p_sparse * n as f32;
        let deg = expected_deg.min(self.max_fan_out as f32).round() as usize;
        // Recurrent + light input/readout overhead (~2k + N).
        n.saturating_mul(deg)
            .saturating_add(n)
            .saturating_add(self.base.k_wta.saturating_mul(4))
    }

    pub fn expected_degree(&self) -> f32 {
        self.base.p_sparse * self.base.n_hidden as f32
    }

    pub fn regime(&self) -> WiringRegime {
        WiringRegime::from_expected_degree(self.expected_degree(), self.max_fan_out)
    }

    /// Full multi-condition C1 is refused at large N (isolate-only).
    /// Micro isolate + Foundation Micro always refuse full C1 (capacity stress, not G2).
    pub fn refuses_full_c1(&self) -> bool {
        self.is_micro_isolate()
            || self.is_foundation_micro()
            || self.base.n_hidden >= MAC_PROBE_FULL_C1_REFUSE_N
    }

    /// Engineering Pass/Fail for Foundation Micro nnz band (not G2).
    pub fn foundation_nnz_in_band(measured_nnz: usize) -> bool {
        (FOUNDATION_MICRO_NNZ_LO..=FOUNDATION_MICRO_NNZ_HI).contains(&measured_nnz)
    }

    /// Materialize Config for the runner (copies fan / rescale flags onto base).
    pub fn to_config(&self) -> Config {
        let mut c = self.base.clone();
        c.max_fan_out = self.max_fan_out;
        c.init_w_rescale = self.init_w_rescale;
        c.readout_gain_normalize = self.readout_gain_normalize;
        c.matched_budget_repeat = false;
        c
    }

    #[inline]
    pub fn seeds(&self) -> Vec<u64> {
        self.base.seeds()
    }
}

/// `max_fan_out ≈ round(target_nnz / N)` for syn-matched ladder.
pub fn syn_matched_fan_out(n_hidden: usize) -> usize {
    let n = n_hidden.max(1);
    ((MAC_PROBE_TARGET_NNZ + n / 2) / n).max(1)
}

/// Activity-scaled k-WTA: target ≈1% winners, clamped so `k/N ∈ [0.005, 0.03]`.
pub fn scaled_k_wta(n_hidden: usize) -> usize {
    let n = n_hidden.max(1);
    let mut k = ((MICRO_TARGET_ACTIVITY * n as f32).round() as usize).max(1);
    let kn = k as f32 / n as f32;
    if kn < MICRO_ACTIVITY_MIN {
        k = ((MICRO_ACTIVITY_MIN * n as f32).ceil() as usize).max(1);
    } else if kn > MICRO_ACTIVITY_MAX {
        k = ((MICRO_ACTIVITY_MAX * n as f32).floor() as usize).max(1);
    }
    k.min(n)
}

/// Disclosed init rescale: `init_w * sqrt(REF / mean_fan_in)`.
pub fn effective_init_w(init_w: f32, mean_fan_in: f32, rescale: bool) -> f32 {
    if !rescale {
        return init_w;
    }
    let mean = mean_fan_in.max(1.0);
    init_w * (MAC_PROBE_REF_MEAN_FAN_IN / mean).sqrt()
}

/// Disclosed readout boost under optional gain normalization.
///
/// Legacy: `boost = 1.15 / init_w` (clamped).
/// Normalized: choose boost so `boost * mean_readout_fan_in ≈`
/// `(1.15 / init_w_ref) * REF_MEAN_READOUT_FAN_IN` with `init_w_ref = 0.15`.
pub fn readout_boost_and_gain(
    init_w: f32,
    mean_readout_fan_in: f32,
    gain_normalize: bool,
) -> (f32, f32) {
    let legacy = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
    if !gain_normalize {
        let gain = legacy * mean_readout_fan_in.max(1.0);
        return (legacy, gain);
    }
    let target_gain = (1.15 / 0.15) * MAC_PROBE_REF_MEAN_READOUT_FAN_IN;
    let boost = (target_gain / mean_readout_fan_in.max(1.0)).clamp(1.0, 64.0);
    let gain = boost * mean_readout_fan_in.max(1.0);
    (boost, gain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syn_matched_fans_match_plan() {
        assert_eq!(syn_matched_fan_out(512), 195);
        assert_eq!(syn_matched_fan_out(2000), 50);
        assert_eq!(syn_matched_fan_out(10000), 10);
    }

    #[test]
    fn syn_matched_presets_assert_invariants() {
        for &n in &[512usize, 2000, 10000] {
            let c = MacProbeConfig::syn_matched(n, true);
            assert_eq!(c.base.k_wta, MAC_PROBE_K_WTA);
            assert!(!c.base.matched_budget_repeat);
            assert!(c.init_w_rescale);
            assert!(c.readout_gain_normalize);
            assert_eq!(c.max_fan_out, syn_matched_fan_out(n));
            assert!(c.hash_string().starts_with(C1_MAC_PROBE_HASH_PREFIX));
        }
    }

    #[test]
    fn distinct_geometry_distinct_hash() {
        let a = MacProbeConfig::geometry(2000, 10, 8, MacProbeMode::Pm1, true);
        let b = MacProbeConfig::geometry(2000, 32, 8, MacProbeMode::Pm1, true);
        assert_ne!(a.hash_string(), b.hash_string());
    }

    #[test]
    fn mode_trio_distinct_hashes() {
        let fan = syn_matched_fan_out(2000);
        let pm1 = MacProbeConfig::geometry(2000, fan, 8, MacProbeMode::Pm1, true);
        let sfb = MacProbeConfig::geometry(2000, fan, 8, MacProbeMode::StructuredFb, true);
        let dfa = MacProbeConfig::geometry(2000, fan, 8, MacProbeMode::DfaLive, true);
        assert_ne!(pm1.hash_string(), sfb.hash_string());
        assert_ne!(pm1.hash_string(), dfa.hash_string());
        assert_ne!(sfb.hash_string(), dfa.hash_string());
    }

    #[test]
    fn refuse_full_c1_at_2k() {
        assert!(MacProbeConfig::syn_matched(2000, true).refuses_full_c1());
        assert!(!MacProbeConfig::syn_matched(512, true).refuses_full_c1());
    }

    #[test]
    fn regime_tags() {
        // N=512, p=0.35 → expected ≈179; fan=195 → Bernoulli (just under 0.95*195)
        let s512 = MacProbeConfig::syn_matched(512, true);
        assert_eq!(s512.regime(), WiringRegime::Bernoulli);
        // N=2000, expected ≈700 ≫ fan=50 → capped
        let s2k = MacProbeConfig::syn_matched(2000, true);
        assert_eq!(s2k.regime(), WiringRegime::Capped);
    }

    #[test]
    fn init_rescale_formula() {
        let w = effective_init_w(0.15, MAC_PROBE_REF_MEAN_FAN_IN, true);
        assert!((w - 0.15).abs() < 1e-5);
        let w2 = effective_init_w(0.15, 180.0, true);
        assert!(w2 < 0.15);
    }

    #[test]
    fn from_hash_roundtrip() {
        let c = MacProbeConfig::syn_matched(512, true);
        let again = MacProbeConfig::from_hash(&c.hash_string()).expect("preset");
        assert_eq!(again.hash_string(), c.hash_string());
    }

    #[test]
    fn micro_scaled_k_in_activity_band() {
        for &n in &[1_000usize, 10_000, 100_000] {
            let k = scaled_k_wta(n);
            let kn = k as f32 / n as f32;
            assert!(
                (MICRO_ACTIVITY_MIN..=MICRO_ACTIVITY_MAX).contains(&kn),
                "n={n} k={k} kn={kn}"
            );
        }
        assert_eq!(scaled_k_wta(1_000), 10);
        assert_eq!(scaled_k_wta(10_000), 100);
    }

    #[test]
    fn micro_isolate_presets() {
        let m1 = MacProbeConfig::micro_isolate(1_000, true);
        let m2 = MacProbeConfig::micro_isolate(10_000, true);
        assert!(m1.hash_string().starts_with(C1_MICRO_HASH_PREFIX));
        assert!(m2.hash_string().starts_with(C1_MICRO_HASH_PREFIX));
        assert_ne!(m1.hash_string(), m2.hash_string());
        assert_eq!(m1.max_fan_out, MICRO_MAX_FAN_OUT);
        assert_eq!(m1.base.k_wta, 10);
        assert_eq!(m2.base.k_wta, 100);
        assert!(!m1.base.matched_budget_repeat);
        assert!(!m1.base.use_surrogate_lif_reference);
        assert!(m1.refuses_full_c1());
        assert!(m2.refuses_full_c1());
        let again = MacProbeConfig::from_hash(&m1.hash_string()).expect("micro preset");
        assert_eq!(again.hash_string(), m1.hash_string());
    }

    #[test]
    fn foundation_micro_fixed_k_and_nnz_target() {
        let q = MacProbeConfig::foundation_micro(true);
        let s = MacProbeConfig::foundation_micro(false);
        assert!(q.is_foundation_micro());
        assert!(!q.is_micro_isolate());
        assert_eq!(q.base.n_hidden, FOUNDATION_MICRO_N_HIDDEN);
        assert_eq!(q.max_fan_out, FOUNDATION_MICRO_FAN_OUT);
        assert_eq!(q.base.k_wta, MAC_PROBE_K_WTA);
        assert_eq!(q.target_nnz, FOUNDATION_MICRO_TARGET_NNZ);
        assert!(q.refuses_full_c1());
        assert!(!q.base.use_surrogate_lif_reference);
        assert!(q.hash_string().starts_with(C1_MICRO_HASH_PREFIX));
        assert_ne!(q.hash_string(), s.hash_string());
        // Predicted nnz ≈ N·fan ≈ 1e6 (cap binds: p·N ≫ fan).
        let pred = q.predicted_nnz();
        assert!(
            MacProbeConfig::foundation_nnz_in_band(pred),
            "predicted_nnz={pred} outside Foundation Micro band"
        );
        // Distinct from activity-scaled micro isolate @ N=10k.
        let iso = MacProbeConfig::micro_isolate(10_000, true);
        assert_ne!(q.hash_string(), iso.hash_string());
        assert_ne!(q.base.k_wta, iso.base.k_wta);
        let again = MacProbeConfig::from_hash(&s.hash_string()).expect("foundation preset");
        assert_eq!(again.hash_string(), s.hash_string());
    }

    #[test]
    fn dfa_live_size_protocol_distinct_from_overnight() {
        let overnight = MacProbeConfig::geometry(
            2000,
            syn_matched_fan_out(2000),
            8,
            MacProbeMode::DfaLive,
            true,
        );
        let size_q = MacProbeConfig::dfa_live_size(MacProbeMode::DfaLive, true);
        let size_s = MacProbeConfig::dfa_live_size(MacProbeMode::DfaLive, false);
        assert!(size_q.size_protocol);
        assert!(!overnight.size_protocol);
        assert_ne!(overnight.hash_string(), size_q.hash_string());
        assert_ne!(size_q.hash_string(), size_s.hash_string());
        assert_eq!(size_s.base.n_seeds, DFA_LIVE_SIZE_N_SEEDS);
        assert_eq!(size_s.base.n_hidden, DFA_LIVE_SIZE_N_HIDDEN);
        let pm1 = MacProbeConfig::dfa_live_size(MacProbeMode::Pm1, false);
        let sfb = MacProbeConfig::dfa_live_size(MacProbeMode::StructuredFb, false);
        assert_ne!(pm1.hash_string(), size_s.hash_string());
        assert_ne!(sfb.hash_string(), size_s.hash_string());
        let again = MacProbeConfig::from_hash(&size_s.hash_string()).expect("size preset");
        assert_eq!(again.hash_string(), size_s.hash_string());
    }
}
