//! Four-arm extension of the matched SHD BPTT instrument (G4 + G5).
//!
//! Arms: `ff+fixed`, `ff+alif`, `rec+fixed`, `rec+alif`.
//!
//! # Why this is a separate module
//!
//! [`crate::shd_matched::loss_and_gradient`] is the shipped reference that
//! produced the 216 completed rust cells. Gate F requires that those cells stay
//! bit-reproducible, so that function is **not edited**. This module adds the
//! general form alongside it, and [`tests::ff_fixed_matches_shipped_reference`]
//! asserts the general path reproduces it bit-for-bit at `ff+fixed`.
//!
//! The python mirror is `scripts/shd_calibration/arms.py`, whose `selftest()`
//! already reports BIT-IDENTICAL on all seven forward/gradient arrays at
//! `ff+fixed`, and cross-checks the recurrent and adaptive backward paths
//! against an independent scalar-loop implementation at ~6e-8 relative
//! deviation. Keep the two files structurally parallel: when Gate E parity
//! fails, the diff between them is the first place to look.
//!
//! # Weight format (G4)
//!
//! `SHDWGT1` is retained unchanged as a reader and is still what `ff+fixed`
//! writes, so existing `initialization/*.weights` files keep loading
//! bit-identically. `SHDWGT2` adds the arm tag, `w_rec`, and the adaptation
//! block. Do not migrate the existing files.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::shd_attention::{
    attention_forward, attention_gradient, attention_logits, AttentionConfig, AttentionGradient,
    AttentionParams,
};
use crate::shd_matched::{
    surrogate_derivative_scaled, MatchedAdam, MatchedForward, MatchedGradient, MatchedShdSample,
    MatchedWeights, MATCHED_PHYSICAL_TAU_MS, MATCHED_THRESHOLD, MATCHED_WEIGHTS_MAGIC,
};

/// Version 2 weight container: arm tag + recurrent block + adaptation block.
pub const MATCHED_WEIGHTS_MAGIC_V2: &[u8; 8] = b"SHDWGT2\0";
/// Version 3 weight container: everything in `SHDWGT2` plus the attention
/// read-out. **Only attention arms write it**, so `SHDWGT1` and `SHDWGT2` files
/// keep loading and rewriting byte-identically and Gate F is untouched.
pub const MATCHED_WEIGHTS_MAGIC_V3: &[u8; 8] = b"SHDWGT3\0";

/// Mirrors `binn-learn/src/shd_alif.rs` `DEFAULT_TAU_A` / `DEFAULT_BETA_A`.
pub const MATCHED_DEFAULT_TAU_A: f32 = 20.0;
pub const MATCHED_DEFAULT_BETA_A: f32 = 0.18;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedArm {
    pub recurrent: bool,
    pub adaptive: bool,
    /// **Axis 3**: a time-axis attention read-out on top of the rate read-out.
    /// See [`crate::shd_attention`] for what it computes and why.
    pub attention: bool,
}

impl MatchedArm {
    pub const FF_FIXED: Self = Self {
        recurrent: false,
        adaptive: false,
        attention: false,
    };
    pub const FF_ALIF: Self = Self {
        recurrent: false,
        adaptive: true,
        attention: false,
    };
    pub const REC_FIXED: Self = Self {
        recurrent: true,
        adaptive: false,
        attention: false,
    };
    pub const REC_ALIF: Self = Self {
        recurrent: true,
        adaptive: true,
        attention: false,
    };

    pub const FF_FIXED_ATTN: Self = Self {
        recurrent: false,
        adaptive: false,
        attention: true,
    };
    pub const FF_ALIF_ATTN: Self = Self {
        recurrent: false,
        adaptive: true,
        attention: true,
    };
    pub const REC_FIXED_ATTN: Self = Self {
        recurrent: true,
        adaptive: false,
        attention: true,
    };
    pub const REC_ALIF_ATTN: Self = Self {
        recurrent: true,
        adaptive: true,
        attention: true,
    };

    /// The four architecture arms registered by
    /// `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF`. This stays four elements:
    /// every recorded cell and every pinned constant is one of these, and
    /// widening it would silently redefine what "every arm" means in tests that
    /// were written before attention existed.
    pub const ALL: [Self; 4] = [
        Self::FF_FIXED,
        Self::FF_ALIF,
        Self::REC_FIXED,
        Self::REC_ALIF,
    ];

    /// The same four with the attention read-out attached.
    pub const ALL_ATTENTION: [Self; 4] = [
        Self::FF_FIXED_ATTN,
        Self::FF_ALIF_ATTN,
        Self::REC_FIXED_ATTN,
        Self::REC_ALIF_ATTN,
    ];

    pub const fn label(self) -> &'static str {
        match (self.recurrent, self.adaptive, self.attention) {
            (false, false, false) => "ff+fixed",
            (false, true, false) => "ff+alif",
            (true, false, false) => "rec+fixed",
            (true, true, false) => "rec+alif",
            (false, false, true) => "ff+fixed+attn",
            (false, true, true) => "ff+alif+attn",
            (true, false, true) => "rec+fixed+attn",
            (true, true, true) => "rec+alif+attn",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "ff+fixed" => Ok(Self::FF_FIXED),
            "ff+alif" => Ok(Self::FF_ALIF),
            "rec+fixed" => Ok(Self::REC_FIXED),
            "rec+alif" => Ok(Self::REC_ALIF),
            "ff+fixed+attn" => Ok(Self::FF_FIXED_ATTN),
            "ff+alif+attn" => Ok(Self::FF_ALIF_ATTN),
            "rec+fixed+attn" => Ok(Self::REC_FIXED_ATTN),
            "rec+alif+attn" => Ok(Self::REC_ALIF_ATTN),
            other => Err(format!(
                "unknown arm {other:?}; expected one of ff+fixed, ff+alif, rec+fixed, \
                 rec+alif, or any of those with a +attn suffix"
            )),
        }
    }

    /// Bit 2 is the attention axis. Existing `SHDWGT2` files were written with
    /// codes 0..=3, so they load back as non-attention arms unchanged.
    const fn code(self) -> u32 {
        (self.recurrent as u32) | ((self.adaptive as u32) << 1) | ((self.attention as u32) << 2)
    }

    fn from_code(code: u32) -> Result<Self, String> {
        if code > 0b111 {
            return Err(format!("unknown arm code {code} in weight file"));
        }
        Ok(Self {
            recurrent: code & 1 != 0,
            adaptive: code & 2 != 0,
            attention: code & 4 != 0,
        })
    }
}

/// `MatchedWeights` plus the recurrent block and adaptation parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct ArmWeights {
    pub base: MatchedWeights,
    pub arm: MatchedArm,
    /// `[hidden, hidden]`, row-major, empty for feed-forward arms.
    /// Diagonal is held at zero: a self-loop is a threshold change in disguise
    /// and would confound the adaptation axis.
    pub w_rec: Vec<f32>,
    pub tau_a: f32,
    pub beta_a: f32,
    /// Present exactly when `arm.attention`. The invariant is enforced by the
    /// constructors and re-checked on load, so a file cannot claim an attention
    /// arm and carry no attention parameters.
    pub attn: Option<AttentionParams>,
}

impl ArmWeights {
    pub fn new(base: MatchedWeights, arm: MatchedArm, w_rec: Vec<f32>) -> Result<Self, String> {
        if arm.attention {
            return Err(format!(
                "arm {} carries an attention read-out; build it with new_attentive",
                arm.label()
            ));
        }
        Self::assemble(base, arm, w_rec, None)
    }

    /// Constructor for the `+attn` arms.
    pub fn new_attentive(
        base: MatchedWeights,
        arm: MatchedArm,
        w_rec: Vec<f32>,
        attn: AttentionParams,
    ) -> Result<Self, String> {
        if !arm.attention {
            return Err(format!(
                "arm {} has no attention axis; build it with new",
                arm.label()
            ));
        }
        Self::assemble(base, arm, w_rec, Some(attn))
    }

    fn assemble(
        base: MatchedWeights,
        arm: MatchedArm,
        w_rec: Vec<f32>,
        attn: Option<AttentionParams>,
    ) -> Result<Self, String> {
        let expected = if arm.recurrent {
            base.hidden * base.hidden
        } else {
            0
        };
        if w_rec.len() != expected {
            return Err(format!(
                "arm {} expects w_rec of len {expected}, got {}",
                arm.label(),
                w_rec.len()
            ));
        }
        match (arm.attention, &attn) {
            (true, Some(params)) => params.check_shapes(base.hidden, base.n_classes)?,
            (false, None) => {}
            (true, None) => return Err(format!("arm {} needs attention params", arm.label())),
            (false, Some(_)) => {
                return Err(format!(
                    "arm {} must not carry attention params",
                    arm.label()
                ))
            }
        }
        let mut weights = Self {
            base,
            arm,
            w_rec,
            tau_a: MATCHED_DEFAULT_TAU_A,
            beta_a: MATCHED_DEFAULT_BETA_A,
            attn,
        };
        weights.enforce_zero_diagonal();
        Ok(weights)
    }

    /// Feed-forward fixed-threshold view of existing weights - no recurrent block.
    pub fn feedforward(base: MatchedWeights) -> Self {
        Self {
            base,
            arm: MatchedArm::FF_FIXED,
            w_rec: Vec::new(),
            tau_a: MATCHED_DEFAULT_TAU_A,
            beta_a: MATCHED_DEFAULT_BETA_A,
            attn: None,
        }
    }

    /// Width and depth of the attention read-out, or `None` for the four base
    /// arms. Reported into the cell record so a config change cannot be silent.
    pub fn attention_config(&self) -> Option<AttentionConfig> {
        self.attn.as_ref().map(|params| params.config)
    }

    pub fn enforce_zero_diagonal(&mut self) {
        if !self.arm.recurrent {
            return;
        }
        let hidden = self.base.hidden;
        for h in 0..hidden {
            self.w_rec[h * hidden + h] = 0.0;
        }
    }

    /// `ff+fixed` writes `SHDWGT1`, byte-identical to the shipped writer, so
    /// Gate F is preserved. Non-attention arms write `SHDWGT2`, unchanged from
    /// before attention existed. Attention arms write `SHDWGT3`.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if self.arm == MatchedArm::FF_FIXED {
            return self.base.save(path);
        }
        let file = File::create(path).map_err(|error| error.to_string())?;
        let mut writer = BufWriter::new(file);
        let magic = if self.arm.attention {
            MATCHED_WEIGHTS_MAGIC_V3
        } else {
            MATCHED_WEIGHTS_MAGIC_V2
        };
        writer.write_all(magic).map_err(|e| e.to_string())?;
        for value in [
            self.base.n_inputs as u32,
            self.base.hidden as u32,
            self.base.n_classes as u32,
            self.arm.code(),
        ] {
            write_u32(&mut writer, value)?;
        }
        for value in [self.tau_a, self.beta_a] {
            writer
                .write_all(&value.to_bits().to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        for &value in self
            .base
            .w_in
            .iter()
            .chain(self.base.w_out.iter())
            .chain(self.base.b_out.iter())
            .chain(self.w_rec.iter())
        {
            writer
                .write_all(&value.to_bits().to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        if let Some(params) = &self.attn {
            write_u32(&mut writer, params.config.d_model as u32)?;
            write_u32(&mut writer, params.config.layers as u32)?;
            for &value in params.iter_all() {
                writer
                    .write_all(&value.to_bits().to_le_bytes())
                    .map_err(|e| e.to_string())?;
            }
        }
        writer.flush().map_err(|error| error.to_string())
    }

    /// Dispatches on magic. `SHDWGT1` yields `ff+fixed` with an empty `w_rec`.
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut reader = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
        let mut magic = [0_u8; 8];
        reader.read_exact(&mut magic).map_err(|e| e.to_string())?;
        if &magic == MATCHED_WEIGHTS_MAGIC {
            drop(reader);
            return Ok(Self::feedforward(MatchedWeights::load(path)?));
        }
        let attentive = &magic == MATCHED_WEIGHTS_MAGIC_V3;
        if &magic != MATCHED_WEIGHTS_MAGIC_V2 && !attentive {
            return Err(format!("bad matched-weight magic in {}", path.display()));
        }
        let n_inputs = read_u32(&mut reader)? as usize;
        let hidden = read_u32(&mut reader)? as usize;
        let n_classes = read_u32(&mut reader)? as usize;
        let arm = MatchedArm::from_code(read_u32(&mut reader)?)?;
        if arm.attention != attentive {
            return Err(format!(
                "{} declares arm {} but was written in the {} container",
                path.display(),
                arm.label(),
                if attentive { "SHDWGT3" } else { "SHDWGT2" }
            ));
        }
        let tau_a = read_f32(&mut reader)?;
        let beta_a = read_f32(&mut reader)?;
        let w_in = read_f32_vec(&mut reader, n_inputs * hidden)?;
        let w_out = read_f32_vec(&mut reader, hidden * n_classes)?;
        let b_out = read_f32_vec(&mut reader, n_classes)?;
        let w_rec = read_f32_vec(&mut reader, if arm.recurrent { hidden * hidden } else { 0 })?;
        // Sized from a placeholder seed and then overwritten entry by entry in
        // the canonical `iter_all` order: the file is the authority on every
        // value, `deterministic` only allocates the blocks. `check_shapes`
        // re-asserts that afterwards.
        let attn = if attentive {
            let d_model = read_u32(&mut reader)? as usize;
            let layers = read_u32(&mut reader)? as usize;
            let config = AttentionConfig::new(d_model, layers)?;
            let mut params = AttentionParams::deterministic(hidden, n_classes, config, 0)?;
            for value in params.iter_all_mut() {
                *value = read_f32(&mut reader)?;
            }
            params.check_shapes(hidden, n_classes)?;
            Some(params)
        } else {
            None
        };
        let mut weights = Self {
            base: MatchedWeights {
                n_inputs,
                hidden,
                n_classes,
                w_in,
                w_out,
                b_out,
            },
            arm,
            w_rec,
            tau_a,
            beta_a,
            attn,
        };
        weights.enforce_zero_diagonal();
        Ok(weights)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArmGradient {
    pub base: MatchedGradient,
    /// `[hidden, hidden]`, empty for feed-forward arms, zero diagonal.
    pub w_rec: Vec<f32>,
    /// Present exactly when the arm carries an attention read-out.
    pub attn: Option<AttentionGradient>,
}

/// Read-optimised views of the input and recurrent weights.
///
/// Construct this once for a group of samples evaluated against the same
/// weights. Training must rebuild it after every optimiser update. The values
/// are copied without arithmetic, so using the prepared path changes neither
/// the forward pass nor the gradient bit pattern.
#[derive(Clone, Debug)]
pub struct ArmWeightLayout {
    n_inputs: usize,
    hidden: usize,
    recurrent: bool,
    w_in_t: Vec<f32>,
    w_rec_t: Vec<f32>,
}

impl ArmWeightLayout {
    pub fn prepare(weights: &ArmWeights) -> Self {
        let hidden = weights.base.hidden;
        Self {
            n_inputs: weights.base.n_inputs,
            hidden,
            recurrent: weights.arm.recurrent,
            w_in_t: transpose_rows_to_columns(&weights.base.w_in, hidden, weights.base.n_inputs),
            w_rec_t: if weights.arm.recurrent {
                transpose_rows_to_columns(&weights.w_rec, hidden, hidden)
            } else {
                Vec::new()
            },
        }
    }

    fn check_compatible(&self, weights: &ArmWeights) -> Result<(), String> {
        if self.n_inputs != weights.base.n_inputs
            || self.hidden != weights.base.hidden
            || self.recurrent != weights.arm.recurrent
            || self.w_in_t.len() != weights.base.w_in.len()
            || self.w_rec_t.len() != weights.w_rec.len()
        {
            return Err("prepared weight layout/model mismatch".into());
        }
        Ok(())
    }
}

impl ArmGradient {
    pub fn zeros_like(weights: &ArmWeights) -> Self {
        Self {
            base: MatchedGradient::zeros_like(&weights.base),
            w_rec: vec![0.0; weights.w_rec.len()],
            attn: weights.attn.as_ref().map(AttentionGradient::zeros_like),
        }
    }

    pub fn add_assign(&mut self, other: &Self) {
        self.base.add_assign(&other.base);
        for (value, delta) in self.w_rec.iter_mut().zip(other.w_rec.iter()) {
            *value += *delta;
        }
        if let (Some(target), Some(source)) = (self.attn.as_mut(), other.attn.as_ref()) {
            target.add_assign(source);
        }
    }

    pub fn scale(&mut self, factor: f32) {
        self.base.scale(factor);
        for value in self.w_rec.iter_mut() {
            *value *= factor;
        }
        if let Some(attn) = self.attn.as_mut() {
            attn.scale(factor);
        }
    }

    pub fn l2_norm(&self) -> f32 {
        // Feed-forward arms return the shipped value untouched rather than
        // round-tripping through sqrt(x*x), which is not exact in f32. An
        // attention arm has parameters outside `base`, so it must not take this
        // path even when it is feed-forward — a norm that silently ignored a
        // whole parameter block would under-report exactly the block whose
        // stability is in question.
        if self.w_rec.is_empty() && self.attn.is_none() {
            return self.base.l2_norm();
        }
        let base = self.base.l2_norm();
        let rec: f32 = self.w_rec.iter().map(|v| v * v).sum();
        let attn: f32 = self
            .attn
            .as_ref()
            .map_or(0.0, AttentionGradient::sum_squares);
        let combined = base * base + rec + attn;
        if combined.is_finite() {
            return combined.sqrt();
        }
        // Same conditional-widening rationale as `MatchedWeights::l2_norm`, and
        // the recurrent arm is where it actually fires. Note `base * base` can
        // overflow even when `base` itself is finite, so the fallback has to
        // redo both halves in f64 rather than reusing `base`.
        let wide_base: f64 = self
            .base
            .w_in
            .iter()
            .chain(self.base.w_out.iter())
            .chain(self.base.b_out.iter())
            .map(|v| {
                let v = f64::from(*v);
                v * v
            })
            .sum();
        let wide_rec: f64 = self
            .w_rec
            .iter()
            .map(|v| {
                let v = f64::from(*v);
                v * v
            })
            .sum();
        let wide_attn: f64 = self
            .attn
            .as_ref()
            .map_or(0.0, AttentionGradient::sum_squares_wide);
        (wide_base + wide_rec + wide_attn).sqrt() as f32
    }

    pub fn all_finite(&self) -> bool {
        self.base.all_finite()
            && self.w_rec.iter().all(|v| v.is_finite())
            && self.attn.as_ref().is_none_or(AttentionGradient::all_finite)
    }
}

/// General four-arm forward + BPTT backward.
///
/// At `ff+fixed` every guarded branch is skipped and the arithmetic collapses
/// term-for-term, in the same order, onto [`crate::shd_matched::loss_and_gradient`].
/// That equality is asserted bit-exactly in the tests below and is Gate F.
///
/// Reset stays **detached**, matching the shipped reference: the path
/// `du(t+1)/ds(t) = -alpha * u(t)` is deliberately not differentiated.
pub fn loss_and_gradient_arm(
    weights: &ArmWeights,
    sample: &MatchedShdSample,
) -> Result<(MatchedForward, ArmGradient), String> {
    loss_and_gradient_arm_scaled(weights, sample, 1.0)
}

/// [`loss_and_gradient_arm`] with the surrogate gain scaled.
///
/// `scale == 1.0` reproduces [`loss_and_gradient_arm`] bit-for-bit, and that is
/// the path every existing caller takes. See
/// `results/AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md`.
pub fn loss_and_gradient_arm_scaled(
    weights: &ArmWeights,
    sample: &MatchedShdSample,
    surrogate_scale: f32,
) -> Result<(MatchedForward, ArmGradient), String> {
    if sample.n_inputs != weights.base.n_inputs {
        return Err("sample/model input mismatch".into());
    }
    if sample.label as usize >= weights.base.n_classes || sample.frames.is_empty() {
        return Err("invalid label or empty framed sample".into());
    }
    let layout = ArmWeightLayout::prepare(weights);
    loss_and_gradient_arm_scaled_prepared(weights, &layout, sample, surrogate_scale)
}

/// [`loss_and_gradient_arm_scaled`] using a layout shared by many samples.
///
/// This is the batch hot path. The caller must rebuild `layout` after mutating
/// `weights`; shape compatibility is checked here, while value freshness is a
/// batch-lifetime invariant owned by the caller.
pub fn loss_and_gradient_arm_scaled_prepared(
    weights: &ArmWeights,
    layout: &ArmWeightLayout,
    sample: &MatchedShdSample,
    surrogate_scale: f32,
) -> Result<(MatchedForward, ArmGradient), String> {
    let base = &weights.base;
    let arm = weights.arm;
    if sample.n_inputs != base.n_inputs {
        return Err("sample/model input mismatch".into());
    }
    if sample.label as usize >= base.n_classes || sample.frames.is_empty() {
        return Err("invalid label or empty framed sample".into());
    }
    layout.check_compatible(weights)?;
    let t_steps = sample.frames.len();
    let hidden = base.hidden;
    let alpha = (-sample.dt_ms / MATCHED_PHYSICAL_TAU_MS).exp();
    let rho = (-1.0_f32 / weights.tau_a).exp();
    let beta_a = weights.beta_a;

    let mut membrane = vec![0.0_f32; t_steps * hidden];
    let mut spikes = vec![0.0_f32; t_steps * hidden];
    // Both of these are `t_steps * hidden` — 1 MB each at the anchor — and both
    // are dead for `ff+fixed`: `thresholds` is the constant `MATCHED_THRESHOLD`
    // unless the arm adapts, and `previous_spike_log` is only ever read by the
    // recurrent backward. Allocating and filling them per sample was pure
    // memory traffic. Values consumed downstream are unchanged.
    let mut thresholds = if arm.adaptive {
        vec![MATCHED_THRESHOLD; t_steps * hidden]
    } else {
        Vec::new()
    };
    let mut previous_spike_log = if arm.recurrent {
        vec![0.0_f32; t_steps * hidden]
    } else {
        Vec::new()
    };
    let mut previous_u = vec![0.0_f32; hidden];
    let mut previous_s = vec![0.0_f32; hidden];
    let mut adaptation = vec![0.0_f32; hidden];

    // Input-drive fast path, non-recurrent arms only.
    //
    // `w_in` is `[hidden, n_inputs]`, so the drive `w_in[h * n_inputs + channel]`
    // gathers scattered elements out of a different row for every hidden unit —
    // one strided sweep of the whole matrix per timestep, which is what makes
    // this loop memory-bound rather than compute-bound. Transposing to
    // `[n_inputs, hidden]` turns each event into a contiguous `hidden`-length
    // AXPY that vectorises.
    //
    // This is a layout change and nothing else: every `current[h]` receives the
    // same addends in the same order (decay first, then events in frame order),
    // and the lanes are independent accumulator chains, so the result is
    // bit-identical. rustc does not enable fast-math, so no reassociation is
    // introduced. Gate F against the recorded cells is the binding check —
    // per the 2026-08-02 amendment, fixture-level parity alone is not evidence
    // of bit-identity at training density.
    //
    // Recurrent arms now take the same split-and-transpose path. This was
    // forbidden while the recurrent drive read the live `previous_s` — splitting
    // the loop would have changed which timestep unit `j` contributed — and is
    // safe now that it reads the `previous_spike_log` snapshot. Measured at
    // h128, the recurrent arms were 21x slower than feed-forward before this.
    //
    // Per hidden unit the addends still arrive in exactly the original order:
    // decay, then the frame's events in frame order, then the recurrent term in
    // ascending `j`. No reassociation, so this is bit-identical, and
    // `every_arm_forward_and_backward_is_bit_pinned` is the binding check.
    let w_in_t = &layout.w_in_t;
    // `w_rec` is `[hidden, hidden]` indexed `[h * hidden + j]`, so the drive for
    // a fixed `j` gathers a column. Transposing once per sample turns the inner
    // loop into a contiguous AXPY over `h`; at h512 that is one transpose
    // against every timestep of strided access — a few hundred at a 2 ms frame
    // (366 for test sample 0).
    let w_rec_t = &layout.w_rec_t;
    let mut current = vec![0.0_f32; hidden];
    // Indices of units that spiked at `t-1`. Spikes are exactly 0.0 or 1.0, and
    // adding `w * 0.0` is an exact no-op here, so skipping silent units is
    // bit-identical rather than merely close. Firing is sparse, so this is where
    // most of the recurrent saving comes from.
    //
    // The exactness rests on the accumulator never holding `-0.0`, the one value
    // for which `x + 0.0 != x`. It cannot:
    //   * it starts at `alpha * previous_u[h] * (1 - previous_s[h])`, and
    //     `previous_u` is zero-initialised to `+0.0`;
    //   * when `previous_s[h] == 1.0` the trailing factor is `0.0`, so the
    //     product is `-0.0` only if `previous_u[h]` were negative — but spiking
    //     requires `u >= MATCHED_THRESHOLD`, and that constant is `1.0`, strictly
    //     positive (adaptation only raises it). So `previous_u[h] > 0` there;
    //   * a sum reaches `-0.0` only from `-0.0 + -0.0`; exact cancellation
    //     `a + (-a)` gives `+0.0` under round-to-nearest.
    // `sparse_recurrent_skip_requires_a_positive_threshold` pins the one premise
    // of that argument which lives outside this file.
    let mut active_previous: Vec<usize> = Vec::with_capacity(hidden);

    for (t, frame) in sample.frames.iter().enumerate() {
        if arm.recurrent {
            previous_spike_log[t * hidden..(t + 1) * hidden].copy_from_slice(&previous_s);
        }
        if arm.adaptive {
            for h in 0..hidden {
                adaptation[h] = rho * adaptation[h] + previous_s[h];
            }
        }
        for h in 0..hidden {
            current[h] = alpha * previous_u[h] * (1.0 - previous_s[h]);
        }
        for &(channel, count) in frame {
            let column = &w_in_t[channel * hidden..(channel + 1) * hidden];
            for h in 0..hidden {
                current[h] += column[h] * count;
            }
        }
        if arm.recurrent {
            // The drive must read the previous timestep's spikes, which is
            // what the `previous_spike_log` snapshot taken above holds.
            //
            // Before the fix, this read the live `previous_s` from inside a
            // single fused `h` loop, where `previous_s[h]` was overwritten at
            // the end of each iteration — so units `j < h` contributed the
            // *current* timestep's spike while `j > h` correctly contributed
            // the previous one. That aliasing made the forward compute
            // something other than `sum_j w_rec[h,j] * s_j(t-1)`, putting it
            // at odds with both its own backward and `arms.py:139`. See
            // `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`.
            // Splitting the loop is what makes the aliasing structurally
            // impossible rather than merely absent.
            active_previous.clear();
            for j in 0..hidden {
                if previous_spike_log[t * hidden + j] != 0.0 {
                    active_previous.push(j);
                }
            }
            for &j in &active_previous {
                let spike = previous_spike_log[t * hidden + j];
                let column = &w_rec_t[j * hidden..(j + 1) * hidden];
                for h in 0..hidden {
                    current[h] += column[h] * spike;
                }
            }
        }
        for h in 0..hidden {
            let threshold = if arm.adaptive {
                MATCHED_THRESHOLD + beta_a * adaptation[h]
            } else {
                MATCHED_THRESHOLD
            };
            let spike = f32::from(current[h] >= threshold);
            membrane[t * hidden + h] = current[h];
            spikes[t * hidden + h] = spike;
            if arm.adaptive {
                thresholds[t * hidden + h] = threshold;
            }
            previous_u[h] = current[h];
            previous_s[h] = spike;
        }
    }

    let inv_t = 1.0 / t_steps as f32;
    let mut rates = vec![0.0_f32; hidden];
    for t in 0..t_steps {
        for h in 0..hidden {
            rates[h] += spikes[t * hidden + h] * inv_t;
        }
    }
    // Time-axis attention read-out. Runs on the completed spike train, adds to
    // the logits, and leaves the spiking forward untouched — at `w_a = 0` the
    // arm is numerically its own non-attention counterpart.
    let attention_cache = match &weights.attn {
        Some(params) => Some(attention_forward(params, &spikes, t_steps)?),
        None => None,
    };

    let mut logits = base.b_out.clone();
    for (class, logit) in logits.iter_mut().enumerate() {
        let row = class * hidden;
        for (h, rate) in rates.iter().enumerate() {
            *logit += base.w_out[row + h] * rate;
        }
    }
    if let (Some(params), Some(cache)) = (&weights.attn, &attention_cache) {
        for (logit, delta) in logits
            .iter_mut()
            .zip(attention_logits(params, cache.pooled()))
        {
            *logit += delta;
        }
    }
    let prediction = argmax(&logits);
    let mut probabilities = softmax(&logits);
    let loss = -probabilities[sample.label as usize].max(1e-30).ln();
    probabilities[sample.label as usize] -= 1.0;

    let mut gradient = ArmGradient::zeros_like(weights);
    gradient.base.b_out.copy_from_slice(&probabilities);
    // `ds_attn[t, h]` is the term that makes credit timestep-specific. Without
    // it every timestep receives the identical `direct_spike[h]`, which is why
    // the rate-only arms cannot learn *when* a unit should fire.
    let ds_attn = match (&weights.attn, &attention_cache) {
        (Some(params), Some(cache)) => {
            let (attention, ds_attn) = attention_gradient(params, cache, &spikes, &probabilities)?;
            gradient.attn = Some(attention);
            ds_attn
        }
        _ => Vec::new(),
    };
    let mut direct_spike = vec![0.0_f32; hidden];
    for (class, probability) in probabilities.iter().copied().enumerate() {
        let row = class * hidden;
        for h in 0..hidden {
            gradient.base.w_out[row + h] = probability * rates[h];
            direct_spike[h] += base.w_out[row + h] * probability * inv_t;
        }
    }

    let mut du_next = vec![0.0_f32; hidden];
    let mut da_next = vec![0.0_f32; hidden];
    let mut du = vec![0.0_f32; hidden];
    let mut da = vec![0.0_f32; hidden];
    // Same transposition on the backward side. The original scatter
    // `gradient.base.w_in[h * n_inputs + channel] += du[h] * count` is a
    // read-modify-write into a different row per hidden unit; accumulating into
    // `[n_inputs, hidden]` makes it a contiguous AXPY. For any fixed
    // `(h, channel)` the addends still arrive in the same order — reverse `t`,
    // then frame order within a timestep — so the fold back to canonical layout
    // below reproduces the original array exactly.
    let mut grad_w_in_t = vec![0.0_f32; base.n_inputs * hidden];
    let mut ds_all = vec![0.0_f32; hidden];
    let mut ds_combined = if arm.attention {
        vec![0.0_f32; hidden]
    } else {
        Vec::new()
    };

    for t in (0..t_steps).rev() {
        let frame = &sample.frames[t];
        if arm.recurrent {
            ds_all.copy_from_slice(&direct_spike);
            // `ds[h] += sum_j du_next[j] * w_rec[j * hidden + h]` gathers column
            // `h` across rows when written with `h` outermost — the same strided
            // pattern the input drive had. Hoisting `j` outside makes each step a
            // contiguous AXPY along a single row of `w_rec`, and for any fixed
            // `h` the addends still arrive in ascending `j`, so the sum is
            // unchanged term for term.
            //
            // No sparsity is available here: `du_next` is dense.
            #[allow(clippy::needless_range_loop)]
            for j in 0..hidden {
                let backward_drive = du_next[j];
                let row = &weights.w_rec[j * hidden..(j + 1) * hidden];
                for h in 0..hidden {
                    ds_all[h] += backward_drive * row[h];
                }
            }
        }
        // Feed-forward arms read `direct_spike` straight through; only the
        // recurrent arms need the staged `ds_all`. Select the slice once per
        // timestep rather than testing `arm.recurrent` inside the `h` loop —
        // the value is constant for the whole call, and evaluating it per
        // `(timestep, hidden unit)` stopped this loop vectorising.
        //
        // Both mistakes here cost the feed-forward arms measurably, on the path
        // that carries all 216 recorded cells: first staging `direct_spike`
        // through `ds_all` for every arm (+9.5% over the 13-cell Gate F suite,
        // 291.4 s -> 319.0 s), then leaving the branch in the inner loop
        // (+7%, against a 1.4% same-binary spread).
        let base_ds: &[f32] = if arm.recurrent {
            &ds_all
        } else {
            &direct_spike
        };
        // Staged into a scratch buffer rather than added inside the `h` loop.
        // Two reasons, both load bearing: the inner loop stays branch-free for
        // the four base arms, and no `+ 0.0` is executed on the recorded path —
        // `x + 0.0` turns `-0.0` into `+0.0`, which
        // `every_arm_forward_and_backward_is_bit_pinned` hashes and would have
        // reported as a kernel change.
        let ds_source: &[f32] = if arm.attention {
            for h in 0..hidden {
                ds_combined[h] = base_ds[h] + ds_attn[t * hidden + h];
            }
            &ds_combined
        } else {
            base_ds
        };
        for h in 0..hidden {
            let index = t * hidden + h;
            let mut ds = ds_source[h];
            if arm.adaptive {
                ds += da_next[h];
            }
            let threshold_at = if arm.adaptive {
                thresholds[index]
            } else {
                MATCHED_THRESHOLD
            };
            let gated =
                ds * surrogate_derivative_scaled(membrane[index] - threshold_at, surrogate_scale);
            du[h] = gated + alpha * (1.0 - spikes[index]) * du_next[h];
            if arm.adaptive {
                da[h] = -beta_a * gated + rho * da_next[h];
            }
        }
        for &(channel, count) in frame {
            let column = &mut grad_w_in_t[channel * hidden..(channel + 1) * hidden];
            for h in 0..hidden {
                column[h] += du[h] * count;
            }
        }
        if arm.recurrent {
            // Sparse for the same reason as the forward drive: this accumulates
            // `du[h] * s_j(t-1)`, and a silent `j` contributes an exact zero to
            // an accumulator that starts at `+0.0` and never reaches `-0.0`.
            active_previous.clear();
            for j in 0..hidden {
                if previous_spike_log[t * hidden + j] != 0.0 {
                    active_previous.push(j);
                }
            }
            // Indexed on purpose: `h` selects a row of `w_rec` as well as an
            // element of `du`, and the addend order fixes the f32 result.
            #[allow(clippy::needless_range_loop)]
            for h in 0..hidden {
                let rec_row = h * hidden;
                let unit_drive = du[h];
                for &j in &active_previous {
                    gradient.w_rec[rec_row + j] += unit_drive * previous_spike_log[t * hidden + j];
                }
            }
        }
        du_next.copy_from_slice(&du);
        if arm.adaptive {
            da_next.copy_from_slice(&da);
        }
    }

    // Fold back to `[hidden, n_inputs]` so `add_assign`, `l2_norm`, and Adam
    // see byte-for-byte what they saw before this optimisation.
    for channel in 0..base.n_inputs {
        for h in 0..hidden {
            gradient.base.w_in[h * base.n_inputs + channel] = grad_w_in_t[channel * hidden + h];
        }
    }

    if arm.recurrent {
        for h in 0..hidden {
            gradient.w_rec[h * hidden + h] = 0.0;
        }
    }

    Ok((
        MatchedForward {
            membrane,
            spikes,
            rates,
            logits,
            loss,
            prediction,
        },
        gradient,
    ))
}

/// `[rows, cols]` row-major to `[cols, rows]` row-major.
///
/// Used to pivot `w_in` for the input-drive loop. Pure data movement — no
/// arithmetic, so it cannot perturb any float result.
fn transpose_rows_to_columns(source: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut transposed = vec![0.0_f32; rows * cols];
    for row in 0..rows {
        let values = &source[row * cols..(row + 1) * cols];
        for (col, &value) in values.iter().enumerate() {
            transposed[col * rows + row] = value;
        }
    }
    transposed
}

/// Adam over the base parameters plus the recurrent block.
///
/// At `ff+fixed` this delegates entirely to [`MatchedAdam`] and returns its
/// value unchanged, so the shipped optimiser path is bit-preserved (Gate F).
#[derive(Clone, Debug)]
pub struct ArmAdam {
    base: MatchedAdam,
    m_rec: Vec<f32>,
    v_rec: Vec<f32>,
    m_attn: Vec<f32>,
    v_attn: Vec<f32>,
    step: usize,
    base_parameter_count: usize,
}

impl ArmAdam {
    pub fn new(weights: &ArmWeights) -> Self {
        let attention_parameters = weights
            .attn
            .as_ref()
            .map_or(0, AttentionParams::parameter_count);
        Self {
            base: MatchedAdam::new(&weights.base),
            m_rec: vec![0.0; weights.w_rec.len()],
            v_rec: vec![0.0; weights.w_rec.len()],
            m_attn: vec![0.0; attention_parameters],
            v_attn: vec![0.0; attention_parameters],
            step: 0,
            base_parameter_count: weights.base.w_in.len()
                + weights.base.w_out.len()
                + weights.base.b_out.len(),
        }
    }

    /// Returns the RMS update magnitude across every parameter, matching the
    /// shipped definition. For recurrent arms the base contribution is
    /// recovered as `rms^2 * n_base`; that recovery is exact to f64 rounding
    /// and only feeds the `mean_update_rms` diagnostic, never a parity gate.
    pub fn update(
        &mut self,
        weights: &mut ArmWeights,
        gradient: &ArmGradient,
        lr: f32,
        weight_decay: f32,
    ) -> f32 {
        self.step += 1;
        let base_rms = self
            .base
            .update(&mut weights.base, &gradient.base, lr, weight_decay);
        if !weights.arm.recurrent && !weights.arm.attention {
            return base_rms;
        }
        let correction1 = 1.0 - crate::shd_matched::MATCHED_ADAM_BETA1.powi(self.step as i32);
        let correction2 = 1.0 - crate::shd_matched::MATCHED_ADAM_BETA2.powi(self.step as i32);
        let mut squared_step =
            f64::from(base_rms) * f64::from(base_rms) * self.base_parameter_count as f64;
        let mut n_step = self.base_parameter_count;
        for index in 0..weights.w_rec.len() {
            let gradient_value = gradient.w_rec[index] + weight_decay * weights.w_rec[index];
            self.m_rec[index] = crate::shd_matched::MATCHED_ADAM_BETA1 * self.m_rec[index]
                + (1.0 - crate::shd_matched::MATCHED_ADAM_BETA1) * gradient_value;
            self.v_rec[index] = crate::shd_matched::MATCHED_ADAM_BETA2 * self.v_rec[index]
                + (1.0 - crate::shd_matched::MATCHED_ADAM_BETA2) * gradient_value * gradient_value;
            let update = lr * (self.m_rec[index] / correction1)
                / ((self.v_rec[index] / correction2).sqrt() + crate::shd_matched::MATCHED_ADAM_EPS);
            weights.w_rec[index] -= update;
            squared_step += f64::from(update) * f64::from(update);
            n_step += 1;
        }
        // Attention parameters take the same Adam rule and the same decoupled
        // weight decay, walked in the canonical `iter_all` order so the moment
        // buffers stay aligned with the parameters across save and load.
        if let (Some(params), Some(attention)) = (weights.attn.as_mut(), gradient.attn.as_ref()) {
            for (index, (parameter, entry)) in
                params.iter_all_mut().zip(attention.iter_all()).enumerate()
            {
                let gradient_value = *entry + weight_decay * *parameter;
                self.m_attn[index] = crate::shd_matched::MATCHED_ADAM_BETA1 * self.m_attn[index]
                    + (1.0 - crate::shd_matched::MATCHED_ADAM_BETA1) * gradient_value;
                self.v_attn[index] = crate::shd_matched::MATCHED_ADAM_BETA2 * self.v_attn[index]
                    + (1.0 - crate::shd_matched::MATCHED_ADAM_BETA2)
                        * gradient_value
                        * gradient_value;
                let update = lr * (self.m_attn[index] / correction1)
                    / ((self.v_attn[index] / correction2).sqrt()
                        + crate::shd_matched::MATCHED_ADAM_EPS);
                *parameter -= update;
                squared_step += f64::from(update) * f64::from(update);
                n_step += 1;
            }
        }
        weights.enforce_zero_diagonal();
        (squared_step / n_step.max(1) as f64).sqrt() as f32
    }
}

// ---------------------------------------------------------------------------
// local helpers (kept private; the shipped module's copies are not public)
// ---------------------------------------------------------------------------

/// Byte-for-byte the shipped [`crate::shd_matched`] rule, and it must stay that
/// way.
///
/// This previously used a strict `>` scan, which keeps the **first** maximum,
/// while the shipped path uses `max_by(total_cmp)`, which keeps the **last**.
/// The two disagree wherever the maximum is not unique:
///
/// | logits | shipped | old scan |
/// |---|---|---|
/// | `[0, 0, 0, 0]` — a silent network | 3 | 0 |
/// | `[1, 3, 3, 2]` — tie at the maximum | 2 | 1 |
/// | `[-0.0, 0.0]` | 1 | 0 |
/// | `[1, NaN, 3]` | 1 | 2 |
///
/// `prediction` feeds `accuracy`, `majority_prediction` and
/// `classes_predicted` — three fields Gate F compares against the 216 recorded
/// cells, which were produced by the shipped path. The instrument now runs this
/// path, so a single tied sample would have silently changed a recorded result.
/// Gate F passed only because no tie happened to occur in the 13 regressed
/// cells; that is luck, not a guarantee, and all-equal logits are exactly what
/// a collapsed network produces — a failure mode `silent_fraction` exists to
/// track.
///
/// `total_cmp` is also a total order, so NaN is ranked deterministically rather
/// than being skipped by comparisons that are false by definition.
fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map_or(0, |(index, _)| index)
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let maximum = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values: Vec<f32> = logits.iter().map(|v| (v - maximum).exp()).collect();
    let total: f32 = values.iter().sum();
    for value in values.iter_mut() {
        *value /= total;
    }
    values
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, String> {
    let mut buffer = [0_u8; 4];
    reader
        .read_exact(&mut buffer)
        .map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_f32<R: Read>(reader: &mut R) -> Result<f32, String> {
    Ok(f32::from_bits(read_u32(reader)?))
}

fn read_f32_vec<R: Read>(reader: &mut R, count: usize) -> Result<Vec<f32>, String> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_f32(reader)?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shd_matched::loss_and_gradient as shipped_loss_and_gradient;

    fn sample() -> MatchedShdSample {
        let mut frames = Vec::new();
        for t in 0..30 {
            let mut frame = Vec::new();
            for k in 0..6 {
                frame.push((((t * 7 + k * 11) % 40) as usize, 1.0_f32));
            }
            frames.push(frame);
        }
        MatchedShdSample {
            label: 7,
            frames,
            n_inputs: 40,
            dt_ms: 10.0,
        }
    }

    fn arm_weights(arm: MatchedArm, seed: u64) -> ArmWeights {
        let base = MatchedWeights::deterministic(40, 24, 20, seed);
        let hidden = base.hidden;
        let w_rec = if arm.recurrent {
            (0..hidden * hidden)
                .map(|i| (((i % 17) as f32) - 8.0) * 2e-2)
                .collect()
        } else {
            Vec::new()
        };
        ArmWeights::new(base, arm, w_rec).unwrap()
    }

    /// Central-difference derivative of the loss w.r.t. one parameter.
    fn numerical_gradient(
        weights: &ArmWeights,
        sample: &MatchedShdSample,
        select: impl Fn(&mut ArmWeights) -> &mut f32,
        epsilon: f32,
    ) -> f32 {
        let mut plus = weights.clone();
        *select(&mut plus) += epsilon;
        let mut minus = weights.clone();
        *select(&mut minus) -= epsilon;
        let plus_loss = loss_and_gradient_arm(&plus, sample).unwrap().0.loss;
        let minus_loss = loss_and_gradient_arm(&minus, sample).unwrap().0.loss;
        (plus_loss - minus_loss) / (2.0 * epsilon)
    }

    /// Finite-difference check of the readout gradient, for all four arms.
    ///
    /// **Only `w_out` and `b_out` can be checked this way, and that is a fact
    /// about the model, not a shortcut.** The spike function is a hard
    /// threshold, so the loss is piecewise constant in `w_in` and `w_rec`: the
    /// true derivative is 0 almost everywhere and undefined on the flip set.
    /// The analytic values this module produces for those blocks are
    /// *surrogate* gradients, deliberately not the true gradient. Finite
    /// differencing them returns exactly 0.0 and proves nothing — an early
    /// version of this test asserted otherwise and failed for that reason.
    ///
    /// The readout is different: given the spike train, the loss is smooth in
    /// `w_out`/`b_out`, so the gradient there is exact and checkable. This
    /// generalises `shd_matched::tests::readout_gradient_matches_finite_difference`
    /// from the shipped `ff+fixed` path to all four arms.
    ///
    /// The consequence for the recurrent defect is worth stating plainly: no
    /// gradient check can catch a wrong-timestep recurrent forward, because the
    /// gradient that would expose it is unavailable in principle. That is why
    /// the forward has to be pinned directly, which is the next test.
    #[test]
    fn readout_gradient_matches_finite_difference_for_every_arm() {
        let sample = sample();
        for arm in MatchedArm::ALL {
            let weights = arm_weights(arm, 91);
            let (_, gradient) = loss_and_gradient_arm(&weights, &sample).unwrap();

            let mut checks: Vec<(&str, usize, f32, f32)> = Vec::new();
            for &index in &[0_usize, 41, 233] {
                checks.push((
                    "w_out",
                    index,
                    gradient.base.w_out[index],
                    numerical_gradient(&weights, &sample, |w| &mut w.base.w_out[index], 1e-3),
                ));
            }
            for &index in &[0_usize, 7] {
                checks.push((
                    "b_out",
                    index,
                    gradient.base.b_out[index],
                    numerical_gradient(&weights, &sample, |w| &mut w.base.b_out[index], 1e-3),
                ));
            }

            // Tolerance is set by f32 central differencing, not by how exact the
            // gradient is. With `eps = 1e-3` the roundoff term is about
            // `f32::EPSILON * |loss| / eps` ~= 1e-4 * |loss|, and the loss here
            // is O(1), so absolute deviations of ~1e-4 are arithmetic noise. The
            // shipped `readout_gradient_matches_finite_difference` uses a bare
            // `< 1e-4` absolute bound for the same reason; this keeps that as
            // the floor and adds a relative bound so the check does not go
            // vacuous on large gradients.
            for (block, index, analytic, numerical) in checks {
                let absolute = (analytic - numerical).abs();
                let scale = analytic.abs().max(numerical.abs()).max(1e-5);
                let relative = absolute / scale;
                assert!(
                    absolute < 2e-4 || relative < 0.05,
                    "{} {block}[{index}]: analytic {analytic:e} vs numerical {numerical:e} \
                     (absolute {absolute:e}, relative {relative:.4})",
                    arm.label(),
                );
            }
        }
    }

    /// The recurrent drive must read the previous timestep's spikes, not a
    /// mixture of previous and current ones.
    ///
    /// Reproduces the forward independently with an explicit `s(t-1)` snapshot
    /// and demands bit-equality of the membrane trace. Under the aliasing bug
    /// this differed by ~4e-1 against a unit threshold and flipped spikes.
    #[test]
    fn recurrent_drive_uses_previous_timestep_spikes() {
        let sample = sample();
        for arm in [MatchedArm::REC_FIXED, MatchedArm::REC_ALIF] {
            let weights = arm_weights(arm, 91);
            let (forward, _) = loss_and_gradient_arm(&weights, &sample).unwrap();

            let hidden = weights.base.hidden;
            let n_inputs = weights.base.n_inputs;
            let alpha = (-sample.dt_ms / MATCHED_PHYSICAL_TAU_MS).exp();
            let rho = (-1.0_f32 / weights.tau_a).exp();
            let mut previous_u = vec![0.0_f32; hidden];
            let mut previous_s = vec![0.0_f32; hidden];
            let mut adaptation = vec![0.0_f32; hidden];

            for (t, frame) in sample.frames.iter().enumerate() {
                // The whole point: s(t-1) is frozen before any unit updates.
                let snapshot = previous_s.clone();
                if arm.adaptive {
                    for h in 0..hidden {
                        adaptation[h] = rho * adaptation[h] + snapshot[h];
                    }
                }
                let mut next_u = vec![0.0_f32; hidden];
                let mut next_s = vec![0.0_f32; hidden];
                for h in 0..hidden {
                    let mut accumulator = alpha * previous_u[h] * (1.0 - snapshot[h]);
                    for &(channel, count) in frame {
                        accumulator += weights.base.w_in[h * n_inputs + channel] * count;
                    }
                    // `j` indexes both `w_rec` and `snapshot`; the ascending
                    // order is what makes this sum reproducible bit for bit.
                    #[allow(clippy::needless_range_loop)]
                    for j in 0..hidden {
                        accumulator += weights.w_rec[h * hidden + j] * snapshot[j];
                    }
                    let threshold = if arm.adaptive {
                        MATCHED_THRESHOLD + weights.beta_a * adaptation[h]
                    } else {
                        MATCHED_THRESHOLD
                    };
                    next_u[h] = accumulator;
                    next_s[h] = f32::from(accumulator >= threshold);
                    assert_eq!(
                        forward.membrane[t * hidden + h],
                        accumulator,
                        "{} membrane mismatch at t={t} h={h}",
                        arm.label(),
                    );
                }
                previous_u = next_u;
                previous_s = next_s;
            }
        }
    }

    /// GATE F. The general path must reproduce the shipped reference exactly.
    #[test]
    fn ff_fixed_matches_shipped_reference() {
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let sample = sample();
        let (expected_forward, expected_gradient) =
            shipped_loss_and_gradient(&base, &sample).expect("shipped reference");
        let weights = ArmWeights::feedforward(base);
        let (forward, gradient) = loss_and_gradient_arm(&weights, &sample).expect("arm path");
        assert_eq!(expected_forward.membrane, forward.membrane, "membrane");
        assert_eq!(expected_forward.spikes, forward.spikes, "spikes");
        assert_eq!(expected_forward.rates, forward.rates, "rates");
        assert_eq!(expected_forward.logits, forward.logits, "logits");
        assert_eq!(
            expected_forward.loss.to_bits(),
            forward.loss.to_bits(),
            "loss"
        );
        // `prediction` was omitted here originally, which left the two paths'
        // `argmax` implementations free to diverge on tied or non-finite logits
        // — and they did. It feeds `accuracy`, `majority_prediction` and
        // `classes_predicted`, all Gate F compared fields.
        assert_eq!(
            expected_forward.prediction, forward.prediction,
            "prediction"
        );
        assert_eq!(expected_gradient.w_in, gradient.base.w_in, "grad_w_in");
        assert_eq!(expected_gradient.w_out, gradient.base.w_out, "grad_w_out");
        assert_eq!(expected_gradient.b_out, gradient.base.b_out, "grad_b_out");
        assert!(
            gradient.w_rec.is_empty(),
            "ff+fixed must carry no recurrent block"
        );
    }

    #[test]
    fn prepared_weight_layout_is_bit_identical_to_the_public_path() {
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let config = AttentionConfig::new(32, 4).expect("attention config");
        let attention =
            AttentionParams::deterministic(24, 20, config, 92).expect("attention parameters");
        let weights =
            ArmWeights::new_attentive(base, MatchedArm::FF_FIXED_ATTN, Vec::new(), attention)
                .expect("attention arm");
        let sample = sample();
        let expected =
            loss_and_gradient_arm_scaled(&weights, &sample, 1.0).expect("ordinary layout");
        let layout = ArmWeightLayout::prepare(&weights);
        let actual = loss_and_gradient_arm_scaled_prepared(&weights, &layout, &sample, 1.0)
            .expect("prepared layout");
        assert_eq!(expected, actual);
    }

    /// GATE F, storage half. Existing SHDWGT1 files must round-trip untouched.
    #[test]
    fn v1_round_trip_is_byte_identical() {
        let directory = std::env::temp_dir().join("shd-arm-v1-roundtrip");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("weights.bin");
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        base.save(&path).unwrap();
        let shipped_bytes = std::fs::read(&path).unwrap();

        let loaded = ArmWeights::load(&path).unwrap();
        assert_eq!(loaded.arm, MatchedArm::FF_FIXED);
        assert!(loaded.w_rec.is_empty());
        assert_eq!(loaded.base, base);

        let rewritten = directory.join("weights-rewritten.bin");
        loaded.save(&rewritten).unwrap();
        assert_eq!(
            shipped_bytes,
            std::fs::read(&rewritten).unwrap(),
            "SHDWGT1 bytes changed"
        );
    }

    #[test]
    fn v2_round_trip_preserves_recurrent_block() {
        let directory = std::env::temp_dir().join("shd-arm-v2-roundtrip");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("weights.bin");
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let hidden = base.hidden;
        let w_rec: Vec<f32> = (0..hidden * hidden).map(|i| (i as f32) * 1e-3).collect();
        let weights = ArmWeights::new(base, MatchedArm::REC_ALIF, w_rec).unwrap();
        weights.save(&path).unwrap();
        let loaded = ArmWeights::load(&path).unwrap();
        assert_eq!(loaded, weights);
        for h in 0..hidden {
            assert_eq!(loaded.w_rec[h * hidden + h], 0.0, "diagonal must stay zero");
        }
    }

    #[test]
    fn every_arm_changes_the_spike_train() {
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let sample = sample();
        let hidden = base.hidden;
        let baseline = loss_and_gradient_arm(&ArmWeights::feedforward(base.clone()), &sample)
            .unwrap()
            .0
            .spikes;
        for arm in [
            MatchedArm::FF_ALIF,
            MatchedArm::REC_FIXED,
            MatchedArm::REC_ALIF,
        ] {
            let w_rec = if arm.recurrent {
                (0..hidden * hidden)
                    .map(|i| ((i % 17) as f32 - 8.0) * 2e-2)
                    .collect()
            } else {
                Vec::new()
            };
            let weights = ArmWeights::new(base.clone(), arm, w_rec).unwrap();
            let spikes = loss_and_gradient_arm(&weights, &sample).unwrap().0.spikes;
            assert_ne!(
                baseline,
                spikes,
                "arm {} did not change spiking",
                arm.label()
            );
        }
    }

    /// FNV-1a over the raw bit patterns, so the pin is bit-exact rather than
    /// tolerance-based. Printing a hash keeps the pinned constants readable;
    /// printing 48*48 floats would not.
    fn fnv1a_f32(values: &[f32]) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for value in values {
            for byte in value.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100_0000_01b3);
            }
        }
        hash
    }

    /// Wider and denser than [`sample`], so the pin below actually exercises
    /// the `hidden * hidden` recurrent loops rather than a near-degenerate
    /// corner of them.
    fn dense_fixture(arm: MatchedArm) -> (MatchedShdSample, ArmWeights) {
        let hidden = 48;
        let n_inputs = 60;
        let mut frames = Vec::new();
        for t in 0..40 {
            let mut frame = Vec::new();
            for k in 0..14 {
                frame.push(((t * 13 + k * 7) % n_inputs, 1.0 + (k % 3) as f32));
            }
            frames.push(frame);
        }
        let sample = MatchedShdSample {
            label: 5,
            frames,
            n_inputs,
            dt_ms: 4.0,
        };
        let base = MatchedWeights::deterministic(n_inputs, hidden, 20, 4242);
        let w_rec = if arm.recurrent {
            (0..hidden * hidden)
                .map(|i| (((i % 23) as f32) - 11.0) * 9e-3)
                .collect()
        } else {
            Vec::new()
        };
        (sample, ArmWeights::new(base, arm, w_rec).unwrap())
    }

    /// Bit-pin of every arm's forward and backward, at a non-degenerate density.
    ///
    /// # Why this exists
    ///
    /// Gate F regresses recorded cells, and **every recorded cell is
    /// `ff+fixed`** — 216 of them. The other three arms have no recorded output
    /// at all, so a change to the shared kernel could silently alter
    /// `rec+fixed`, `ff+alif` or `rec+alif` and every gate in the repository
    /// would still pass. That is the same shape of hole that let the recurrent
    /// aliasing defect survive (see
    /// `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`), and it
    /// is the reason the recurrent kernel could not be optimised until now.
    ///
    /// These constants were captured from the kernel *after* that defect was
    /// fixed and after `recurrent_drive_uses_previous_timestep_spikes` had
    /// independently confirmed the forward. They pin behaviour, not
    /// correctness: if a deliberate model change lands, re-pin them in the same
    /// commit and say so. If they move for any other reason, the kernel changed
    /// when it should not have.
    ///
    /// The spike-density assertion is load bearing. A pin taken over a fixture
    /// where nothing spikes — or everything does — would hash a constant and
    /// catch nothing, and in particular would not exercise a sparse recurrent
    /// path at all.
    #[test]
    fn every_arm_forward_and_backward_is_bit_pinned() {
        // (arm, membrane, spikes, logits, grad_w_in, grad_w_out, grad_w_rec)
        let pinned: [(MatchedArm, [u64; 6]); 4] = [
            (MatchedArm::FF_FIXED, PIN_FF_FIXED),
            (MatchedArm::FF_ALIF, PIN_FF_ALIF),
            (MatchedArm::REC_FIXED, PIN_REC_FIXED),
            (MatchedArm::REC_ALIF, PIN_REC_ALIF),
        ];
        let mut failures = Vec::new();
        for (arm, expected) in pinned {
            let (sample, weights) = dense_fixture(arm);
            let (forward, gradient) = loss_and_gradient_arm(&weights, &sample).unwrap();

            let density = forward.spikes.iter().sum::<f32>() / forward.spikes.len() as f32;
            assert!(
                (0.05..=0.95).contains(&density),
                "{} fixture is degenerate: spike density {density:.4}. A pin over an \
                 all-on or all-off spike train exercises nothing.",
                arm.label(),
            );

            let observed = [
                fnv1a_f32(&forward.membrane),
                fnv1a_f32(&forward.spikes),
                fnv1a_f32(&forward.logits),
                fnv1a_f32(&gradient.base.w_in),
                fnv1a_f32(&gradient.base.w_out),
                fnv1a_f32(&gradient.w_rec),
            ];
            if observed != expected {
                // Print rather than assert, so one run reports every arm that
                // moved and the re-pin is a single copy-paste.
                println!(
                    "    const PIN_{}: [u64; 6] = {:#018x?};",
                    arm.label().to_uppercase().replace(['+', '-'], "_"),
                    observed,
                );
                failures.push(arm.label());
            }
        }
        assert!(
            failures.is_empty(),
            "kernel output moved for {failures:?}; re-pin from the log above"
        );
    }

    // Captured 2026-08-03 from the post-defect-fix kernel. The trailing entry is
    // `grad_w_rec`; for the feed-forward arms that block is empty, so both hash
    // to the bare FNV offset basis — that is expected, not a copy-paste slip.
    const PIN_FF_FIXED: [u64; 6] = [
        0x59bad35bf85e82b6,
        0xb09c4cee717f9978,
        0x0e2e65b8ce75b73d,
        0x87bfb0ba05863c41,
        0x360a16ae918846c2,
        0xcbf29ce484222325,
    ];
    const PIN_FF_ALIF: [u64; 6] = [
        0xc52ad841b5a668b8,
        0x5a6e0d090d97e165,
        0xe9bbdf274c440735,
        0x20f6ebf507e3fa84,
        0x3e6678e3a6285023,
        0xcbf29ce484222325,
    ];
    const PIN_REC_FIXED: [u64; 6] = [
        0x3655a2f23174aa02,
        0x796b1eeee0df6ab5,
        0x6a4c8fd824ae0122,
        0xb99751dc9e12ff9e,
        0x8d6f14d7bafb159d,
        0x1be22a5c0af9ce93,
    ];
    /// Bit-pin of the four attention arms, for the same reason the base arms
    /// are pinned: they have no recorded cells anywhere, so without this a
    /// change to the attention kernel would pass every gate in the repository.
    ///
    /// The seventh entry is the attention read-out's own gradient, hashed over
    /// `AttentionGradient::iter_all` — `w_e`, then each block's `w_q, w_k, w_v,
    /// w_o`, then `w_a`. Pinning only the base blocks would leave the entire
    /// attention gradient unpinned while the test read as complete.
    #[test]
    fn every_attention_arm_forward_and_backward_is_bit_pinned() {
        let pinned: [(MatchedArm, [u64; 7]); 4] = [
            (MatchedArm::FF_FIXED_ATTN, PIN_FF_FIXED_ATTN),
            (MatchedArm::FF_ALIF_ATTN, PIN_FF_ALIF_ATTN),
            (MatchedArm::REC_FIXED_ATTN, PIN_REC_FIXED_ATTN),
            (MatchedArm::REC_ALIF_ATTN, PIN_REC_ALIF_ATTN),
        ];
        let mut failures = Vec::new();
        for (arm, expected) in pinned {
            let (sample, base_weights) = dense_fixture(MatchedArm {
                attention: false,
                ..arm
            });
            let weights = ArmWeights::new_attentive(
                base_weights.base.clone(),
                arm,
                base_weights.w_rec.clone(),
                attention_params(base_weights.base.hidden, base_weights.base.n_classes),
            )
            .unwrap();
            let (forward, gradient) = loss_and_gradient_arm(&weights, &sample).unwrap();

            let density = forward.spikes.iter().sum::<f32>() / forward.spikes.len() as f32;
            assert!(
                (0.05..=0.95).contains(&density),
                "{} fixture is degenerate: spike density {density:.4}",
                arm.label(),
            );
            let attention: Vec<f32> = gradient
                .attn
                .as_ref()
                .unwrap()
                .iter_all()
                .copied()
                .collect();
            assert!(
                attention.iter().any(|value| *value != 0.0),
                "{} produced an all-zero attention gradient; the pin would hash nothing",
                arm.label(),
            );

            let observed = [
                fnv1a_f32(&forward.membrane),
                fnv1a_f32(&forward.spikes),
                fnv1a_f32(&forward.logits),
                fnv1a_f32(&gradient.base.w_in),
                fnv1a_f32(&gradient.base.w_out),
                fnv1a_f32(&gradient.w_rec),
                fnv1a_f32(&attention),
            ];
            if observed != expected {
                println!(
                    "    const PIN_{}: [u64; 7] = {:#018x?};",
                    arm.label().to_uppercase().replace(['+', '-'], "_"),
                    observed,
                );
                failures.push(arm.label());
            }
        }
        assert!(
            failures.is_empty(),
            "attention kernel output moved for {failures:?}; re-pin from the log above"
        );
    }

    // Captured 2026-08-19 from the kernel these tests accompany, alongside the
    // finite-difference checks that establish the attention gradient is right
    // rather than merely stable. Re-pin in the same commit as any deliberate
    // model change, and say so.
    //
    // Entries 0 and 1 — membrane and spikes — are **identical** to the
    // corresponding non-attention pin above, for all four arms. That is the
    // additive property under test, taken here on a denser fixture than
    // `a_zero_read_out_reduces_every_attention_arm_to_its_base_arm` uses: the
    // read-out cannot perturb the spiking forward.
    const PIN_FF_FIXED_ATTN: [u64; 7] = [
        0x59bad35bf85e82b6,
        0xb09c4cee717f9978,
        0x487757e346dce48c,
        0x83541a2b6792da5e,
        0xb05540e69e9d8df8,
        0xcbf29ce484222325,
        0xa6d935b559f92964,
    ];
    const PIN_FF_ALIF_ATTN: [u64; 7] = [
        0xc52ad841b5a668b8,
        0x5a6e0d090d97e165,
        0x7a636a5eaf66b249,
        0xe8a344f7dd9d7873,
        0x0e63b260391d974d,
        0xcbf29ce484222325,
        0x8e5307da30382e06,
    ];
    const PIN_REC_FIXED_ATTN: [u64; 7] = [
        0x3655a2f23174aa02,
        0x796b1eeee0df6ab5,
        0xed3be5b13d7c9964,
        0xfe9dcc250c5302c9,
        0xc8ada25d7fe6513b,
        0xe0bdef08e6b7d2f4,
        0x91be1d618fe8525b,
    ];
    const PIN_REC_ALIF_ATTN: [u64; 7] = [
        0x17e9947421f4b648,
        0x69f2330042a9be55,
        0x75d956b694943eae,
        0x0aca7e135a8536dc,
        0x792176abaa648960,
        0x624e796a8ff00dc6,
        0x4df1538156afb8ec,
    ];

    const PIN_REC_ALIF: [u64; 6] = [
        0x17e9947421f4b648,
        0x69f2330042a9be55,
        0x60ac78982c23de22,
        0x0978488ade260646,
        0x8cfca1e0d90b5f30,
        0x700c85153e47b8db,
    ];

    /// The recurrent drive skips silent units, which is exact only because a
    /// spiking unit's membrane is strictly positive — and that follows from the
    /// firing threshold being strictly positive. The threshold lives in
    /// `shd_matched`, so a change there could silently invalidate an argument
    /// made in this file. Fail loudly instead.
    ///
    /// If the threshold ever legitimately goes to zero or below, the fix is to
    /// drop the skip (add every `j` back), not to relax this test.
    #[test]
    // Asserting on constants is the whole purpose here: this test exists to trip
    // if either constant is edited in `shd_matched`.
    #[allow(clippy::assertions_on_constants)]
    fn sparse_recurrent_skip_requires_a_positive_threshold() {
        assert!(
            MATCHED_THRESHOLD > 0.0,
            "MATCHED_THRESHOLD is {MATCHED_THRESHOLD}; the sparse recurrent drive assumes a \
             spiking unit has u >= threshold > 0, which is what rules out a -0.0 accumulator",
        );
        assert!(
            MATCHED_DEFAULT_BETA_A >= 0.0,
            "adaptation must only ever raise the threshold, never lower it below zero",
        );
    }

    /// `argmax` must agree with the shipped rule on every degenerate input, not
    /// just on well-separated logits.
    ///
    /// These cases are the ones that actually reach the instrument: a collapsed
    /// network produces all-equal logits, `-0.0` falls out of the kernel, and
    /// non-finite logits are reachable now that h512 gradients hit 1e29. The
    /// shipped expectations below are the recorded behaviour of
    /// `max_by(total_cmp)`, which produced the 216 cells.
    #[test]
    fn argmax_matches_the_shipped_rule_on_degenerate_logits() {
        let shipped = |values: &[f32]| -> usize {
            values
                .iter()
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(right.1))
                .map_or(0, |(index, _)| index)
        };
        let cases: [(&str, Vec<f32>); 7] = [
            ("all tied — a collapsed network", vec![0.0, 0.0, 0.0, 0.0]),
            ("tie at the maximum", vec![1.0, 3.0, 3.0, 2.0]),
            ("well separated", vec![1.0, 5.0, 3.0]),
            ("negative zero against zero", vec![-0.0, 0.0]),
            ("NaN present", vec![1.0, f32::NAN, 3.0]),
            ("all NaN", vec![f32::NAN, f32::NAN]),
            ("infinity present", vec![1.0, f32::INFINITY, 3.0]),
        ];
        for (name, logits) in cases {
            assert_eq!(
                argmax(&logits),
                shipped(&logits),
                "{name}: the arm path must predict what the recorded cells did",
            );
        }
    }

    /// The f64 fallback must recover overflowed norms *without* perturbing any
    /// norm f32 could already represent.
    ///
    /// The second half is the load-bearing one: `mean_gradient_norm` is a Gate F
    /// compared field across 216 recorded cells, so a change in the last ulp
    /// here is a change to a registered result. Widening the accumulation
    /// unconditionally would do exactly that, which is why the fallback is
    /// gated on `!is_finite()`.
    #[test]
    fn l2_norm_widens_only_when_f32_overflows() {
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let mut weights = loss_and_gradient_arm(&ArmWeights::feedforward(base), &sample())
            .unwrap()
            .1
            .base;

        // Ordinary magnitudes: must be bit-identical to the naive f32 fold.
        let naive: f32 = weights
            .w_in
            .iter()
            .chain(weights.w_out.iter())
            .chain(weights.b_out.iter())
            .map(|v| v * v)
            .sum::<f32>()
            .sqrt();
        assert_eq!(
            weights.l2_norm().to_bits(),
            naive.to_bits(),
            "finite norms must not move: this field is compared by Gate F",
        );

        // A norm f32 can hold (~1e20) whose sum of squares (~1e40) it cannot.
        for value in weights.w_in.iter_mut() {
            *value = 0.0;
        }
        weights.w_in[0] = 1e20;
        weights.w_in[1] = 1e20;
        for value in weights.w_out.iter_mut().chain(weights.b_out.iter_mut()) {
            *value = 0.0;
        }
        let overflowing: f32 = weights.w_in.iter().map(|v| v * v).sum();
        assert!(
            overflowing.is_infinite(),
            "fixture must actually overflow f32"
        );

        let norm = weights.l2_norm();
        assert!(
            norm.is_finite(),
            "fallback must recover a representable norm"
        );
        let expected = (2.0_f64 * 1e40).sqrt() as f32;
        assert_eq!(
            norm.to_bits(),
            expected.to_bits(),
            "recovered norm is wrong"
        );

        // Genuinely beyond f32: still infinity, and correctly so.
        weights.w_in[0] = f32::MAX;
        weights.w_in[1] = f32::MAX;
        assert!(
            weights.l2_norm().is_infinite(),
            "a norm above f32::MAX must stay infinite rather than wrap",
        );
    }

    /// Attention weights for the fixtures above, with a non-degenerate `w_o`.
    ///
    /// `AttentionParams::deterministic` zeroes `w_o` on purpose (identity
    /// residual at initialisation), which would make every gradient inside a
    /// block exactly zero and the checks below vacuous.
    fn attention_params(hidden: usize, n_classes: usize) -> AttentionParams {
        let config = AttentionConfig::new(6, 1).unwrap();
        let mut params = AttentionParams::deterministic(hidden, n_classes, config, 771).unwrap();
        for block in params.blocks.iter_mut() {
            for (position, value) in block.w_o.iter_mut().enumerate() {
                *value = (((position % 13) as f32) - 6.0) * 2e-2;
            }
        }
        params
    }

    /// An attention arm with `w_a = 0` must be its own base arm, numerically.
    ///
    /// This is the structural guarantee the axis rests on: attention is
    /// **additive on top of** the rate read-out, never a replacement for it, so
    /// a difference between `ff+fixed` and `ff+fixed+attn` can only come from
    /// the attention read-out having learned something — never from the
    /// spiking forward having been perturbed by attaching it.
    #[test]
    fn a_zero_read_out_reduces_every_attention_arm_to_its_base_arm() {
        let sample = sample();
        for (base_arm, attention_arm) in MatchedArm::ALL.into_iter().zip(MatchedArm::ALL_ATTENTION)
        {
            let plain = arm_weights(base_arm, 91);
            let mut params = attention_params(plain.base.hidden, plain.base.n_classes);
            params.w_a.iter_mut().for_each(|value| *value = 0.0);
            let attentive = ArmWeights::new_attentive(
                plain.base.clone(),
                attention_arm,
                plain.w_rec.clone(),
                params,
            )
            .unwrap();

            let (expected, expected_gradient) = loss_and_gradient_arm(&plain, &sample).unwrap();
            let (observed, observed_gradient) = loss_and_gradient_arm(&attentive, &sample).unwrap();
            let label = attention_arm.label();
            assert_eq!(expected.membrane, observed.membrane, "{label} membrane");
            assert_eq!(expected.spikes, observed.spikes, "{label} spikes");
            assert_eq!(expected.rates, observed.rates, "{label} rates");
            assert_eq!(expected.logits, observed.logits, "{label} logits");
            assert_eq!(expected.loss, observed.loss, "{label} loss");
            assert_eq!(
                expected.prediction, observed.prediction,
                "{label} prediction"
            );
            assert_eq!(
                expected_gradient.base.w_in, observed_gradient.base.w_in,
                "{label} grad_w_in"
            );
            assert_eq!(
                expected_gradient.base.w_out, observed_gradient.base.w_out,
                "{label} grad_w_out"
            );
            assert_eq!(
                expected_gradient.w_rec, observed_gradient.w_rec,
                "{label} grad_w_rec"
            );
            // The read-out gradient is non-zero even at `w_a = 0`, which is what
            // lifts the arm off the reduction on the first optimiser step.
            let attention_gradient = observed_gradient.attn.as_ref().expect("attention gradient");
            assert!(
                attention_gradient.w_a.iter().any(|value| *value != 0.0),
                "{label} would never leave the zero read-out"
            );
        }
    }

    /// Attention parameters sit downstream of the spike threshold, so the loss
    /// is genuinely smooth in them and the finite-difference check is exact —
    /// the reason `w_in` and `w_rec` cannot be checked this way does not apply.
    /// Checked here at the arm level, on top of the module's own per-parameter
    /// check, so that the wiring into the logits and into `ds` is covered too.
    #[test]
    fn attention_parameters_match_finite_difference_through_the_arm() {
        let sample = sample();
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let params = attention_params(base.hidden, base.n_classes);
        let weights =
            ArmWeights::new_attentive(base, MatchedArm::FF_FIXED_ATTN, Vec::new(), params).unwrap();
        let (_, gradient) = loss_and_gradient_arm(&weights, &sample).unwrap();
        let attention = gradient.attn.as_ref().unwrap();

        let mut checks: Vec<(String, f32, f32)> = Vec::new();
        for index in [0_usize, 37, 91] {
            checks.push((
                format!("w_e[{index}]"),
                attention.w_e[index],
                numerical_gradient(
                    &weights,
                    &sample,
                    |w| &mut w.attn.as_mut().unwrap().w_e[index],
                    1e-3,
                ),
            ));
        }
        for index in [0_usize, 17, 41] {
            checks.push((
                format!("w_a[{index}]"),
                attention.w_a[index],
                numerical_gradient(
                    &weights,
                    &sample,
                    |w| &mut w.attn.as_mut().unwrap().w_a[index],
                    1e-3,
                ),
            ));
        }
        for index in [0_usize, 11, 29] {
            checks.push((
                format!("w_v[{index}]"),
                attention.blocks[0].w_v[index],
                numerical_gradient(
                    &weights,
                    &sample,
                    |w| &mut w.attn.as_mut().unwrap().blocks[0].w_v[index],
                    1e-3,
                ),
            ));
            checks.push((
                format!("w_o[{index}]"),
                attention.blocks[0].w_o[index],
                numerical_gradient(
                    &weights,
                    &sample,
                    |w| &mut w.attn.as_mut().unwrap().blocks[0].w_o[index],
                    1e-3,
                ),
            ));
        }
        for (name, analytic, numerical) in checks {
            let absolute = (analytic - numerical).abs();
            let scale = analytic.abs().max(numerical.abs()).max(1e-5);
            assert!(
                absolute < 2e-4 || absolute / scale < 0.05,
                "{name}: analytic {analytic:e} vs numerical {numerical:e} (absolute {absolute:e})"
            );
        }
    }

    /// The `+attn` arms are the reason `ds` is no longer constant in `t`. If the
    /// gradient reaching `w_in` were still the same signal at every timestep,
    /// the arm would be adding parameters and nothing else.
    #[test]
    fn attention_makes_the_input_gradient_depend_on_more_than_the_rate() {
        let sample = sample();
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let plain = ArmWeights::feedforward(base.clone());
        let attentive = ArmWeights::new_attentive(
            base,
            MatchedArm::FF_FIXED_ATTN,
            Vec::new(),
            attention_params(24, 20),
        )
        .unwrap();
        let (plain_forward, plain_gradient) = loss_and_gradient_arm(&plain, &sample).unwrap();
        let (attentive_forward, attentive_gradient) =
            loss_and_gradient_arm(&attentive, &sample).unwrap();
        // Same spiking forward: only the read-out and the credit signal moved.
        assert_eq!(plain_forward.spikes, attentive_forward.spikes);
        assert_ne!(
            plain_gradient.base.w_in, attentive_gradient.base.w_in,
            "attention did not change the credit the hidden layer receives"
        );
    }

    /// GATE F, storage. `SHDWGT3` must round-trip, and the two older containers
    /// must be untouched by its existence.
    #[test]
    fn v3_round_trip_preserves_the_attention_block() {
        let directory = std::env::temp_dir().join("shd-arm-v3-roundtrip");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("weights.bin");
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let hidden = base.hidden;
        let w_rec: Vec<f32> = (0..hidden * hidden).map(|i| (i as f32) * 1e-3).collect();
        let weights = ArmWeights::new_attentive(
            base,
            MatchedArm::REC_ALIF_ATTN,
            w_rec,
            attention_params(hidden, 20),
        )
        .unwrap();
        weights.save(&path).unwrap();
        assert_eq!(
            &std::fs::read(&path).unwrap()[..8],
            MATCHED_WEIGHTS_MAGIC_V3,
            "attention arms must write SHDWGT3"
        );
        let loaded = ArmWeights::load(&path).unwrap();
        assert_eq!(loaded, weights);
        assert_eq!(
            loaded.attention_config(),
            Some(AttentionConfig::new(6, 1).unwrap())
        );
    }

    /// The two constructors must not be able to produce an arm whose tag and
    /// parameters disagree — that is how an arm reports as attentive while
    /// computing the base model.
    #[test]
    fn the_attention_tag_and_the_attention_parameters_cannot_disagree() {
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        assert!(ArmWeights::new(base.clone(), MatchedArm::FF_FIXED_ATTN, Vec::new()).is_err());
        assert!(ArmWeights::new_attentive(
            base.clone(),
            MatchedArm::FF_FIXED,
            Vec::new(),
            attention_params(24, 20)
        )
        .is_err());
        // Wrong width for the network it is attached to.
        assert!(ArmWeights::new_attentive(
            base,
            MatchedArm::FF_FIXED_ATTN,
            Vec::new(),
            attention_params(8, 20)
        )
        .is_err());
    }

    #[test]
    fn arm_labels_round_trip_through_parse() {
        for arm in MatchedArm::ALL.into_iter().chain(MatchedArm::ALL_ATTENTION) {
            assert_eq!(MatchedArm::parse(arm.label()).unwrap(), arm);
        }
        assert!(MatchedArm::parse("ff+fixed+attention").is_err());
    }

    #[test]
    fn recurrent_gradient_has_zero_diagonal() {
        let base = MatchedWeights::deterministic(40, 24, 20, 91);
        let hidden = base.hidden;
        let w_rec: Vec<f32> = (0..hidden * hidden)
            .map(|i| ((i % 13) as f32 - 6.0) * 1e-2)
            .collect();
        let weights = ArmWeights::new(base, MatchedArm::REC_ALIF, w_rec).unwrap();
        let (_, gradient) = loss_and_gradient_arm(&weights, &sample()).unwrap();
        for h in 0..hidden {
            assert_eq!(gradient.w_rec[h * hidden + h], 0.0);
        }
    }
}
