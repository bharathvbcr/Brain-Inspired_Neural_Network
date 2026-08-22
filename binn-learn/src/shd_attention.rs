//! Time-axis attention read-out for the matched SHD instrument.
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER.** This is a gradient reference in
//! the same class as [`crate::shd_alif`] and [`crate::shd_matched_arms`]: it is
//! trained by the matched BPTT instrument, not by three-factor plasticity.
//!
//! # The measurement this exists to answer
//!
//! The instrument's forward model has one cross-bin state variable — the LIF
//! membrane, at `tau_m = 10.05 ms` — and one read-out, the **unweighted mean of
//! the hidden spike train over time**
//! (`shd_matched_arms::loss_and_gradient_arm_scaled`, the `rates` block). Two
//! consequences follow from the code alone:
//!
//! 1. **The memory horizon is ~6% of an utterance.** At the anchor contract
//!    (`published-2ms`) the leak factor is `alpha = exp(-2/10.05) = 0.8195`, so
//!    a drive decays to 1% of itself after 23 bins = 46 ms. Mean SHD utterance
//!    duration is 716 ms (`results/shd_instrument_v4/data_summary.json`). The
//!    hard reset `alpha * u * (1 - s)` truncates it further.
//! 2. **The read-out is permutation-invariant over bins**, and the hidden error
//!    signal `direct_spike[h] = sum_c w_out[c,h] * p_c / T` is *constant in t*,
//!    so no timestep can be credited differently from any other.
//!
//! The measured consequence is on record: training and testing on bin-shuffled
//! data costs **0.0189** accuracy, while additionally destroying within-bin
//! synchrony costs a further **0.1248** — 6.6x more
//! (`results/RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`). The converged
//! `ff+fixed` ceiling is 0.7378 against a registered 0.80 gate, a shortfall of
//! **0.0622** — 3.3x the entire order effect the architecture can express.
//!
//! The registered architectural fix for this is recurrence plus threshold
//! adaptation, and it is *unmeasured rather than refuted*: `rec+alif` at h512
//! produces zero usable cells, with activation peaks spanning 3.08e10 to
//! 3.93e33 against ~0.15 for a healthy `ff+alif`
//! (`results/TODO_2026-08-07_OPEN_WORK.md` §4). That failure mode is specific to
//! backpropagating through `T` sequential steps.
//!
//! This module adds the other fix that recent small language models use for the
//! same structural problem — a fixed-size recurrent state cannot recall across a
//! long context, so a **small number of full-attention layers** is interleaved
//! among the cheap state-passing ones. Here the spiking layer is the cheap
//! state-passing part and this is the attention part. Its gradient path is
//! constant-depth in `T`, so it cannot exhibit the `rec+alif` explosion.
//!
//! # What is added
//!
//! Given the hidden spike train `S` of shape `[t_steps, hidden]`:
//!
//! ```text
//! z_0(t)   = W_e s(t) + pos(t)                       W_e: [hidden, d_model]
//! for each of `layers` blocks:
//!     q,k,v = W_q z(t), W_k z(t), W_v z(t)           each [d_model, d_model]
//!     A     = softmax_row( q k^T / sqrt(d_model) )   [t_steps, t_steps]
//!     z(t) <- z(t) + W_o (A v)(t)                    residual
//! pooled   = mean_t z_L(t)
//! logits  += W_a pooled                              W_a: [n_classes, d_model]
//! ```
//!
//! Attention is **additive on top of the rate read-out**, never a replacement:
//! at `W_a = 0` the arm reduces exactly to its non-attention counterpart. It
//! reads spikes, not membranes, so it consumes the same signal the rate
//! read-out consumes and the spiking substrate is unchanged.
//!
//! Every timestep pair interacts directly, so the horizon is the whole
//! utterance rather than 46 ms — and, because `dL/ds(t,h)` now varies with `t`,
//! the hidden layer receives a *timestep-specific* error signal for the first
//! time in this instrument.
//!
//! # Position is load bearing
//!
//! Attention without positional information is permutation-equivariant, and
//! mean-pooling on top of it is permutation-**invariant** — so an attention
//! block with no position encoding would add pairwise interaction and still be
//! blind to order, which is the failure being fixed. `pos` is therefore not
//! decoration.
//!
//! It is a fixed (non-learned) sinusoidal code over the **normalised** position
//! `u = t / (t_steps - 1)`, with `d_model / 2` geometrically spaced frequencies
//! from [`ATTENTION_MIN_CYCLES`] to [`ATTENTION_MAX_CYCLES`] cycles per
//! utterance. Normalised rather than absolute because SHD utterances vary in
//! length (0.23 s to 1.37 s in the training split) and the discriminative
//! structure is relative timing within the word. Absolute-time position is the
//! obvious alternative axis and is not tested here.
//!
//! # Initialisation
//!
//! `W_o` is **zero**, so every block starts as an exact identity residual and
//! the arm cannot begin in a worse-conditioned state than the base arm. It does
//! not stay zero: `dL/dW_o` is `sum_t upstream(t) (x) c(t)`, which is non-zero
//! at the first optimiser step. Everything else is Glorot from a
//! [`PortableRng`] stream keyed per matrix, so adding a block does not shift the
//! draws of the matrices that were already there.

// Index-based loops throughout: nearly every one walks several arrays at
// different strides (`[t, d]`, `[t, t]`, `[out, in]`), which is what the
// surrounding modules do for the same reason.
#![allow(clippy::needless_range_loop)]

use crate::shd_matched::PortableRng;

/// Slowest positional frequency: one cycle across the whole utterance.
pub const ATTENTION_MIN_CYCLES: f32 = 1.0;
/// Fastest positional frequency, in cycles per utterance. At the anchor
/// (716 ms mean duration, 2 ms bins, ~358 bins) 64 cycles is a period of
/// ~5.6 bins = 11 ms, which is the same order as the membrane horizon — so the
/// ladder spans everything from "where in the word" down to the resolution the
/// LIF layer already has.
pub const ATTENTION_MAX_CYCLES: f32 = 64.0;
/// Registered default width of the attention stream.
pub const DEFAULT_ATTENTION_DIM: usize = 32;
/// Registered default number of attention blocks. One block against one spiking
/// layer is already a higher attention fraction than the language-model recipe
/// this borrows from; depth is a separate axis and is not tested here.
pub const DEFAULT_ATTENTION_LAYERS: usize = 1;

/// Shape of the attention read-out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttentionConfig {
    pub d_model: usize,
    pub layers: usize,
}

impl AttentionConfig {
    pub const DEFAULT: Self = Self {
        d_model: DEFAULT_ATTENTION_DIM,
        layers: DEFAULT_ATTENTION_LAYERS,
    };

    pub fn new(d_model: usize, layers: usize) -> Result<Self, String> {
        let config = Self { d_model, layers };
        config.validate()?;
        Ok(config)
    }

    /// `d_model` must be even because the positional code is sin/cos pairs, and
    /// both fields must be non-zero because a zero-width or zero-depth
    /// attention arm would report as an attention arm while computing nothing.
    pub fn validate(self) -> Result<(), String> {
        if self.d_model < 2 || !self.d_model.is_multiple_of(2) {
            return Err(format!(
                "attention d_model must be even and >= 2, got {}",
                self.d_model
            ));
        }
        if self.layers == 0 {
            return Err("attention layers must be >= 1".into());
        }
        Ok(())
    }
}

/// One attention block. All four matrices are `[d_model, d_model]`, row-major,
/// indexed `[out * d_model + in]`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttentionBlock {
    pub w_q: Vec<f32>,
    pub w_k: Vec<f32>,
    pub w_v: Vec<f32>,
    pub w_o: Vec<f32>,
}

impl AttentionBlock {
    pub fn zeros(d_model: usize) -> Self {
        Self {
            w_q: vec![0.0; d_model * d_model],
            w_k: vec![0.0; d_model * d_model],
            w_v: vec![0.0; d_model * d_model],
            w_o: vec![0.0; d_model * d_model],
        }
    }

    fn iter_all(&self) -> impl Iterator<Item = &f32> {
        self.w_q
            .iter()
            .chain(&self.w_k)
            .chain(&self.w_v)
            .chain(&self.w_o)
    }

    fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.w_q
            .iter_mut()
            .chain(&mut self.w_k)
            .chain(&mut self.w_v)
            .chain(&mut self.w_o)
    }
}

/// The attention read-out's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct AttentionParams {
    pub hidden: usize,
    pub n_classes: usize,
    pub config: AttentionConfig,
    /// `[hidden, d_model]`, row-major. Stored input-major so a spiking unit's
    /// embedding row is contiguous: the forward gathers only the units that
    /// actually fired, and firing is sparse.
    pub w_e: Vec<f32>,
    pub blocks: Vec<AttentionBlock>,
    /// `[n_classes, d_model]`, row-major.
    pub w_a: Vec<f32>,
}

/// Glorot half-width for a `fan_in -> fan_out` block.
fn glorot_limit(fan_in: usize, fan_out: usize) -> f32 {
    (6.0_f32 / (fan_in + fan_out) as f32).sqrt()
}

fn glorot(rng: &mut PortableRng, count: usize, fan_in: usize, fan_out: usize) -> Vec<f32> {
    let limit = glorot_limit(fan_in, fan_out);
    (0..count).map(|_| rng.uniform(-limit, limit)).collect()
}

impl AttentionParams {
    /// Deterministic initialisation from `seed`.
    ///
    /// Each matrix draws from its own [`PortableRng`] stream, keyed by a
    /// per-matrix constant and the block index, so that changing `layers` or
    /// `d_model` does not silently re-roll the matrices that were already
    /// there. `w_o` is zero by construction — see the module documentation.
    pub fn deterministic(
        hidden: usize,
        n_classes: usize,
        config: AttentionConfig,
        seed: u64,
    ) -> Result<Self, String> {
        config.validate()?;
        if hidden == 0 || n_classes == 0 {
            return Err("attention params need non-zero hidden and n_classes".into());
        }
        let d = config.d_model;
        let mut embed_rng = PortableRng::new(seed ^ 0x4154_544E_0000_0001);
        let w_e = glorot(&mut embed_rng, hidden * d, hidden, d);
        let mut blocks = Vec::with_capacity(config.layers);
        for layer in 0..config.layers {
            let layer_key = (layer as u64 + 1) << 32;
            let mut q_rng = PortableRng::new(seed ^ 0x4154_544E_0000_0002 ^ layer_key);
            let mut k_rng = PortableRng::new(seed ^ 0x4154_544E_0000_0003 ^ layer_key);
            let mut v_rng = PortableRng::new(seed ^ 0x4154_544E_0000_0004 ^ layer_key);
            blocks.push(AttentionBlock {
                w_q: glorot(&mut q_rng, d * d, d, d),
                w_k: glorot(&mut k_rng, d * d, d, d),
                w_v: glorot(&mut v_rng, d * d, d, d),
                w_o: vec![0.0; d * d],
            });
        }
        let mut readout_rng = PortableRng::new(seed ^ 0x4154_544E_0000_0005);
        let w_a = glorot(&mut readout_rng, n_classes * d, d, n_classes);
        Ok(Self {
            hidden,
            n_classes,
            config,
            w_e,
            blocks,
            w_a,
        })
    }

    pub fn parameter_count(&self) -> usize {
        let d = self.config.d_model;
        self.hidden * d + self.blocks.len() * 4 * d * d + self.n_classes * d
    }

    /// Every parameter, in the canonical order used by save/load and by the
    /// optimiser: `w_e`, then each block's `w_q, w_k, w_v, w_o`, then `w_a`.
    pub fn iter_all(&self) -> impl Iterator<Item = &f32> {
        self.w_e
            .iter()
            .chain(self.blocks.iter().flat_map(AttentionBlock::iter_all))
            .chain(&self.w_a)
    }

    pub fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.w_e
            .iter_mut()
            .chain(
                self.blocks
                    .iter_mut()
                    .flat_map(AttentionBlock::iter_all_mut),
            )
            .chain(&mut self.w_a)
    }

    /// Shape check against the network the arm is attached to.
    pub fn check_shapes(&self, hidden: usize, n_classes: usize) -> Result<(), String> {
        self.config.validate()?;
        let d = self.config.d_model;
        if self.hidden != hidden || self.n_classes != n_classes {
            return Err(format!(
                "attention params are for hidden {} / classes {}, network is hidden {hidden} / classes {n_classes}",
                self.hidden, self.n_classes
            ));
        }
        if self.w_e.len() != hidden * d {
            return Err(format!(
                "attention w_e must be {} long, got {}",
                hidden * d,
                self.w_e.len()
            ));
        }
        if self.blocks.len() != self.config.layers {
            return Err(format!(
                "attention config declares {} layers, params carry {}",
                self.config.layers,
                self.blocks.len()
            ));
        }
        for (index, block) in self.blocks.iter().enumerate() {
            for (name, matrix) in [
                ("w_q", &block.w_q),
                ("w_k", &block.w_k),
                ("w_v", &block.w_v),
                ("w_o", &block.w_o),
            ] {
                if matrix.len() != d * d {
                    return Err(format!(
                        "attention block {index} {name} must be {} long, got {}",
                        d * d,
                        matrix.len()
                    ));
                }
            }
        }
        if self.w_a.len() != n_classes * d {
            return Err(format!(
                "attention w_a must be {} long, got {}",
                n_classes * d,
                self.w_a.len()
            ));
        }
        Ok(())
    }
}

/// Gradient of the attention read-out. Same shapes as [`AttentionParams`].
#[derive(Clone, Debug, PartialEq)]
pub struct AttentionGradient {
    pub w_e: Vec<f32>,
    pub blocks: Vec<AttentionBlock>,
    pub w_a: Vec<f32>,
}

impl AttentionGradient {
    pub fn zeros_like(params: &AttentionParams) -> Self {
        Self {
            w_e: vec![0.0; params.w_e.len()],
            blocks: params
                .blocks
                .iter()
                .map(|_| AttentionBlock::zeros(params.config.d_model))
                .collect(),
            w_a: vec![0.0; params.w_a.len()],
        }
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &f32> {
        self.w_e
            .iter()
            .chain(self.blocks.iter().flat_map(AttentionBlock::iter_all))
            .chain(&self.w_a)
    }

    pub fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.w_e
            .iter_mut()
            .chain(
                self.blocks
                    .iter_mut()
                    .flat_map(AttentionBlock::iter_all_mut),
            )
            .chain(&mut self.w_a)
    }

    pub fn add_assign(&mut self, other: &Self) {
        for (value, delta) in self.iter_all_mut().zip(other.iter_all()) {
            *value += *delta;
        }
    }

    pub fn scale(&mut self, factor: f32) {
        for value in self.iter_all_mut() {
            *value *= factor;
        }
    }

    /// Sum of squares, in f32, for folding into the arm's global norm.
    pub fn sum_squares(&self) -> f32 {
        self.iter_all().map(|v| v * v).sum()
    }

    /// Sum of squares widened to f64, for the overflow fallback the arm's
    /// `l2_norm` already has for the recurrent block.
    pub fn sum_squares_wide(&self) -> f64 {
        self.iter_all()
            .map(|v| {
                let v = f64::from(*v);
                v * v
            })
            .sum()
    }

    pub fn all_finite(&self) -> bool {
        self.iter_all().all(|v| v.is_finite())
    }
}

/// Fixed sinusoidal position code, written into `out` (`d_model` long).
///
/// `u = t / (t_steps - 1)` in `[0, 1]`; a one-frame sample has `u = 0`.
pub fn positional_code(t: usize, t_steps: usize, out: &mut [f32]) {
    let d_model = out.len();
    let pairs = d_model / 2;
    let u = if t_steps > 1 {
        t as f32 / (t_steps - 1) as f32
    } else {
        0.0
    };
    let ratio = ATTENTION_MAX_CYCLES / ATTENTION_MIN_CYCLES;
    for pair in 0..pairs {
        let exponent = if pairs > 1 {
            pair as f32 / (pairs - 1) as f32
        } else {
            0.0
        };
        let cycles = ATTENTION_MIN_CYCLES * ratio.powf(exponent);
        let phase = std::f32::consts::TAU * cycles * u;
        out[2 * pair] = phase.sin();
        out[2 * pair + 1] = phase.cos();
    }
}

/// Everything the gradient pass needs from the forward pass.
#[derive(Clone, Debug)]
pub struct AttentionCache {
    t_steps: usize,
    d_model: usize,
    /// `layers + 1` residual-stream snapshots, each `[t_steps, d_model]`.
    /// `z[0]` is the embedded input; `z[layers]` is what gets pooled.
    z: Vec<Vec<f32>>,
    q: Vec<Vec<f32>>,
    k: Vec<Vec<f32>>,
    v: Vec<Vec<f32>>,
    /// Row-softmax attention weights, `[t_steps, t_steps]` per layer.
    a: Vec<Vec<f32>>,
    /// `A v` per layer, `[t_steps, d_model]`.
    c: Vec<Vec<f32>>,
    pooled: Vec<f32>,
}

impl AttentionCache {
    pub fn pooled(&self) -> &[f32] {
        &self.pooled
    }
    pub fn t_steps(&self) -> usize {
        self.t_steps
    }
    /// Attention weight row for `t` in the final block. Diagnostic only.
    pub fn final_attention_row(&self, t: usize) -> &[f32] {
        let last = self.a.len() - 1;
        &self.a[last][t * self.t_steps..(t + 1) * self.t_steps]
    }
}

/// `W_a pooled` — the attention read-out's contribution to the logits.
pub fn attention_logits(params: &AttentionParams, pooled: &[f32]) -> Vec<f32> {
    let d = params.config.d_model;
    (0..params.n_classes)
        .map(|class| {
            let row = &params.w_a[class * d..(class + 1) * d];
            row.iter().zip(pooled).map(|(w, p)| w * p).sum()
        })
        .collect()
}

/// Row-major `[rows, cols] * vector[cols]` into `out[rows]`, overwriting.
fn apply_matrix(matrix: &[f32], input: &[f32], rows: usize, cols: usize, out: &mut [f32]) {
    for row in 0..rows {
        let weights = &matrix[row * cols..(row + 1) * cols];
        out[row] = weights.iter().zip(input).map(|(w, x)| w * x).sum();
    }
}

/// Forward pass of the attention read-out.
///
/// `spikes` is `[t_steps, hidden]` and is read, never written. Values are
/// exactly 0.0 or 1.0, which is why the embedding gathers only firing units.
pub fn attention_forward(
    params: &AttentionParams,
    spikes: &[f32],
    t_steps: usize,
) -> Result<AttentionCache, String> {
    let hidden = params.hidden;
    if spikes.len() != t_steps * hidden {
        return Err(format!(
            "attention forward expects {} spike entries, got {}",
            t_steps * hidden,
            spikes.len()
        ));
    }
    if t_steps == 0 {
        return Err("attention forward needs at least one timestep".into());
    }
    let d = params.config.d_model;
    let layers = params.blocks.len();
    let inv_sqrt_d = 1.0 / (d as f32).sqrt();

    // --- embedding + position ------------------------------------------------
    let mut z0 = vec![0.0_f32; t_steps * d];
    let mut position = vec![0.0_f32; d];
    // `(unit, value)`. Spikes are exactly 0.0 or 1.0 on every real trace, so
    // skipping silent units is exact; the value is carried rather than assumed
    // so that the embedding is genuinely linear in `spikes` — which is what
    // `attention_gradient` claims and what the finite-difference test checks.
    let mut active: Vec<(usize, f32)> = Vec::with_capacity(hidden);
    for t in 0..t_steps {
        positional_code(t, t_steps, &mut position);
        let stream = &mut z0[t * d..(t + 1) * d];
        stream.copy_from_slice(&position);
        active.clear();
        for h in 0..hidden {
            let spike = spikes[t * hidden + h];
            if spike != 0.0 {
                active.push((h, spike));
            }
        }
        for &(h, spike) in &active {
            let embedding = &params.w_e[h * d..(h + 1) * d];
            for (value, weight) in stream.iter_mut().zip(embedding) {
                *value += *weight * spike;
            }
        }
    }

    let mut z = Vec::with_capacity(layers + 1);
    z.push(z0);
    let mut q_all = Vec::with_capacity(layers);
    let mut k_all = Vec::with_capacity(layers);
    let mut v_all = Vec::with_capacity(layers);
    let mut a_all = Vec::with_capacity(layers);
    let mut c_all = Vec::with_capacity(layers);

    for (layer, block) in params.blocks.iter().enumerate() {
        let input = &z[layer];
        let mut q = vec![0.0_f32; t_steps * d];
        let mut k = vec![0.0_f32; t_steps * d];
        let mut v = vec![0.0_f32; t_steps * d];
        for t in 0..t_steps {
            let stream = &input[t * d..(t + 1) * d];
            apply_matrix(&block.w_q, stream, d, d, &mut q[t * d..(t + 1) * d]);
            apply_matrix(&block.w_k, stream, d, d, &mut k[t * d..(t + 1) * d]);
            apply_matrix(&block.w_v, stream, d, d, &mut v[t * d..(t + 1) * d]);
        }

        let mut a = vec![0.0_f32; t_steps * t_steps];
        let mut c = vec![0.0_f32; t_steps * d];
        for t in 0..t_steps {
            let query = &q[t * d..(t + 1) * d];
            let scores = &mut a[t * t_steps..(t + 1) * t_steps];
            let mut maximum = f32::NEG_INFINITY;
            for (other, score) in scores.iter_mut().enumerate() {
                let key = &k[other * d..(other + 1) * d];
                let dot: f32 = query.iter().zip(key).map(|(a, b)| a * b).sum();
                *score = dot * inv_sqrt_d;
                if *score > maximum {
                    maximum = *score;
                }
            }
            let mut total = 0.0_f32;
            for score in scores.iter_mut() {
                *score = (*score - maximum).exp();
                total += *score;
            }
            // `total >= 1` always: the maximal entry contributes exp(0) = 1.
            for score in scores.iter_mut() {
                *score /= total;
            }
            let context = &mut c[t * d..(t + 1) * d];
            for (other, &weight) in scores.iter().enumerate() {
                let value = &v[other * d..(other + 1) * d];
                for (accumulator, item) in context.iter_mut().zip(value) {
                    *accumulator += weight * item;
                }
            }
        }

        let mut next = input.clone();
        let mut projected = vec![0.0_f32; d];
        for t in 0..t_steps {
            apply_matrix(&block.w_o, &c[t * d..(t + 1) * d], d, d, &mut projected);
            for (value, delta) in next[t * d..(t + 1) * d].iter_mut().zip(&projected) {
                *value += *delta;
            }
        }

        q_all.push(q);
        k_all.push(k);
        v_all.push(v);
        a_all.push(a);
        c_all.push(c);
        z.push(next);
        let _ = layer;
    }

    let inverse_t = 1.0 / t_steps as f32;
    let final_stream = &z[layers];
    let mut pooled = vec![0.0_f32; d];
    for t in 0..t_steps {
        for (accumulator, value) in pooled.iter_mut().zip(&final_stream[t * d..(t + 1) * d]) {
            *accumulator += *value * inverse_t;
        }
    }

    Ok(AttentionCache {
        t_steps,
        d_model: d,
        z,
        q: q_all,
        k: k_all,
        v: v_all,
        a: a_all,
        c: c_all,
        pooled,
    })
}

/// Gradient of `sum_c d_logits[c] * (W_a pooled)[c]` with respect to every
/// attention parameter and with respect to the hidden spike train.
///
/// Returns the parameter gradient and `ds_attn`, shaped `[t_steps, hidden]`,
/// which the caller adds to its own per-timestep spike gradient before applying
/// the surrogate derivative.
///
/// The whole path is smooth in every attention parameter and in `spikes`
/// treated as a continuous input, so all of it is finite-difference checkable —
/// unlike `w_in` / `w_rec`, whose analytic values are surrogates. The tests do
/// exactly that.
pub fn attention_gradient(
    params: &AttentionParams,
    cache: &AttentionCache,
    spikes: &[f32],
    d_logits: &[f32],
) -> Result<(AttentionGradient, Vec<f32>), String> {
    let hidden = params.hidden;
    let d = params.config.d_model;
    let t_steps = cache.t_steps;
    if cache.d_model != d {
        return Err("attention cache width does not match the parameters".into());
    }
    if d_logits.len() != params.n_classes {
        return Err(format!(
            "attention gradient expects {} logit gradients, got {}",
            params.n_classes,
            d_logits.len()
        ));
    }
    if spikes.len() != t_steps * hidden {
        return Err("attention gradient spike shape does not match the cache".into());
    }
    let inv_sqrt_d = 1.0 / (d as f32).sqrt();
    let mut gradient = AttentionGradient::zeros_like(params);

    // --- read-out ------------------------------------------------------------
    let mut d_pooled = vec![0.0_f32; d];
    for class in 0..params.n_classes {
        let error = d_logits[class];
        let row = class * d;
        for index in 0..d {
            gradient.w_a[row + index] = error * cache.pooled[index];
            d_pooled[index] += params.w_a[row + index] * error;
        }
    }

    // Mean pooling spreads the read-out gradient evenly over timesteps. Every
    // *later* term is what makes the total signal timestep-dependent.
    let inverse_t = 1.0 / t_steps as f32;
    let mut d_stream = vec![0.0_f32; t_steps * d];
    for t in 0..t_steps {
        for index in 0..d {
            d_stream[t * d + index] = d_pooled[index] * inverse_t;
        }
    }

    let mut d_context = vec![0.0_f32; t_steps * d];
    let mut d_attention = vec![0.0_f32; t_steps * t_steps];
    let mut d_q = vec![0.0_f32; t_steps * d];
    let mut d_k = vec![0.0_f32; t_steps * d];
    let mut d_v = vec![0.0_f32; t_steps * d];

    for layer in (0..params.blocks.len()).rev() {
        let block = &params.blocks[layer];
        let block_gradient = &mut gradient.blocks[layer];
        let input = &cache.z[layer];
        let q = &cache.q[layer];
        let k = &cache.k[layer];
        let v = &cache.v[layer];
        let a = &cache.a[layer];
        let c = &cache.c[layer];

        // The residual carries `d_stream` through untouched; the block adds to it.
        let mut d_input = d_stream.clone();

        d_context.iter_mut().for_each(|value| *value = 0.0);
        for t in 0..t_steps {
            let upstream = &d_stream[t * d..(t + 1) * d];
            let context = &c[t * d..(t + 1) * d];
            let target = &mut d_context[t * d..(t + 1) * d];
            for out_index in 0..d {
                let error = upstream[out_index];
                if error == 0.0 {
                    continue;
                }
                let row = out_index * d;
                for in_index in 0..d {
                    block_gradient.w_o[row + in_index] += error * context[in_index];
                    target[in_index] += block.w_o[row + in_index] * error;
                }
            }
        }

        d_v.iter_mut().for_each(|value| *value = 0.0);
        for t in 0..t_steps {
            let upstream = &d_context[t * d..(t + 1) * d];
            let weights = &a[t * t_steps..(t + 1) * t_steps];
            let scores = &mut d_attention[t * t_steps..(t + 1) * t_steps];
            for other in 0..t_steps {
                let value = &v[other * d..(other + 1) * d];
                scores[other] = upstream.iter().zip(value).map(|(g, x)| g * x).sum();
                let weight = weights[other];
                if weight == 0.0 {
                    continue;
                }
                for (accumulator, g) in d_v[other * d..(other + 1) * d].iter_mut().zip(upstream) {
                    *accumulator += weight * *g;
                }
            }
            // Row softmax: `de = a * (da - sum_j a_j da_j)`.
            let shift: f32 = weights.iter().zip(scores.iter()).map(|(w, g)| w * g).sum();
            for (score, weight) in scores.iter_mut().zip(weights) {
                *score = weight * (*score - shift);
            }
        }

        d_q.iter_mut().for_each(|value| *value = 0.0);
        d_k.iter_mut().for_each(|value| *value = 0.0);
        for t in 0..t_steps {
            let scores = &d_attention[t * t_steps..(t + 1) * t_steps];
            let query = &q[t * d..(t + 1) * d];
            for other in 0..t_steps {
                let error = scores[other] * inv_sqrt_d;
                if error == 0.0 {
                    continue;
                }
                let key = &k[other * d..(other + 1) * d];
                for index in 0..d {
                    d_q[t * d + index] += error * key[index];
                    d_k[other * d + index] += error * query[index];
                }
            }
        }

        for t in 0..t_steps {
            let stream = &input[t * d..(t + 1) * d];
            let target = &mut d_input[t * d..(t + 1) * d];
            for (source, (matrix, matrix_gradient)) in [
                (&d_q, (&block.w_q, &mut block_gradient.w_q)),
                (&d_k, (&block.w_k, &mut block_gradient.w_k)),
                (&d_v, (&block.w_v, &mut block_gradient.w_v)),
            ] {
                for out_index in 0..d {
                    let error = source[t * d + out_index];
                    if error == 0.0 {
                        continue;
                    }
                    let row = out_index * d;
                    for in_index in 0..d {
                        matrix_gradient[row + in_index] += error * stream[in_index];
                        target[in_index] += matrix[row + in_index] * error;
                    }
                }
            }
        }

        d_stream = d_input;
    }

    // --- embedding -----------------------------------------------------------
    // `pos` is a constant, so it takes no gradient.
    let mut ds_attn = vec![0.0_f32; t_steps * hidden];
    for t in 0..t_steps {
        let upstream = &d_stream[t * d..(t + 1) * d];
        for h in 0..hidden {
            let embedding = &params.w_e[h * d..(h + 1) * d];
            ds_attn[t * hidden + h] = embedding.iter().zip(upstream).map(|(w, g)| w * g).sum();
            let spike = spikes[t * hidden + h];
            if spike != 0.0 {
                for (accumulator, g) in gradient.w_e[h * d..(h + 1) * d].iter_mut().zip(upstream) {
                    *accumulator += *g * spike;
                }
            }
        }
    }

    Ok((gradient, ds_attn))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(hidden: usize, n_classes: usize, d_model: usize, layers: usize) -> AttentionParams {
        // `deterministic` zeroes `w_o`, which would make every gradient above
        // the first block exactly zero and the finite-difference checks below
        // vacuous. Fill it with a non-degenerate deterministic pattern instead.
        let config = AttentionConfig::new(d_model, layers).unwrap();
        let mut params = AttentionParams::deterministic(hidden, n_classes, config, 4242).unwrap();
        for (index, block) in params.blocks.iter_mut().enumerate() {
            for (position, value) in block.w_o.iter_mut().enumerate() {
                *value = (((position + 3 * index) % 11) as f32 - 5.0) * 3e-2;
            }
        }
        params
    }

    fn spike_train(t_steps: usize, hidden: usize) -> Vec<f32> {
        (0..t_steps * hidden)
            .map(|index| {
                let t = index / hidden;
                let h = index % hidden;
                f32::from((t * 5 + h * 3) % 7 < 3)
            })
            .collect()
    }

    /// Scalar the gradient pass differentiates: `sum_c d_logits[c] * logits[c]`.
    fn objective(
        params: &AttentionParams,
        spikes: &[f32],
        t_steps: usize,
        d_logits: &[f32],
    ) -> f32 {
        let cache = attention_forward(params, spikes, t_steps).unwrap();
        attention_logits(params, &cache.pooled)
            .iter()
            .zip(d_logits)
            .map(|(logit, weight)| logit * weight)
            .sum()
    }

    fn central_difference(
        params: &AttentionParams,
        spikes: &[f32],
        t_steps: usize,
        d_logits: &[f32],
        select: impl Fn(&mut AttentionParams) -> &mut f32,
        epsilon: f32,
    ) -> f32 {
        let mut plus = params.clone();
        *select(&mut plus) += epsilon;
        let mut minus = params.clone();
        *select(&mut minus) -= epsilon;
        (objective(&plus, spikes, t_steps, d_logits) - objective(&minus, spikes, t_steps, d_logits))
            / (2.0 * epsilon)
    }

    fn assert_close(name: &str, analytic: f32, numerical: f32) {
        let absolute = (analytic - numerical).abs();
        let scale = analytic.abs().max(numerical.abs()).max(1e-5);
        assert!(
            absolute < 2e-4 || absolute / scale < 0.02,
            "{name}: analytic {analytic:e} vs numerical {numerical:e} (absolute {absolute:e})"
        );
    }

    /// Every attention parameter is finite-difference checkable, and is checked.
    ///
    /// This is the load-bearing correctness test for the whole module. Nothing
    /// downstream of the spike threshold is a surrogate, so a wrong gradient
    /// here is a bug, not a modelling choice — the excuse that makes `w_in`
    /// uncheckable does not apply.
    #[test]
    fn every_attention_parameter_matches_finite_difference() {
        for layers in [1_usize, 2] {
            let (t_steps, hidden, n_classes, d_model) = (9_usize, 6_usize, 4_usize, 4_usize);
            let params = params(hidden, n_classes, d_model, layers);
            let spikes = spike_train(t_steps, hidden);
            let d_logits = vec![0.31_f32, -0.72, 0.14, 0.27];
            let cache = attention_forward(&params, &spikes, t_steps).unwrap();
            let (gradient, _) = attention_gradient(&params, &cache, &spikes, &d_logits).unwrap();

            for index in [0_usize, 7, 13, 23] {
                assert_close(
                    &format!("layers {layers} w_e[{index}]"),
                    gradient.w_e[index],
                    central_difference(
                        &params,
                        &spikes,
                        t_steps,
                        &d_logits,
                        |p| &mut p.w_e[index],
                        1e-3,
                    ),
                );
            }
            for index in [0_usize, 5, 11] {
                assert_close(
                    &format!("layers {layers} w_a[{index}]"),
                    gradient.w_a[index],
                    central_difference(
                        &params,
                        &spikes,
                        t_steps,
                        &d_logits,
                        |p| &mut p.w_a[index],
                        1e-3,
                    ),
                );
            }
            for layer in 0..layers {
                for index in [0_usize, 5, 11, 15] {
                    for name in ["w_q", "w_k", "w_v", "w_o"] {
                        let analytic = match name {
                            "w_q" => gradient.blocks[layer].w_q[index],
                            "w_k" => gradient.blocks[layer].w_k[index],
                            "w_v" => gradient.blocks[layer].w_v[index],
                            _ => gradient.blocks[layer].w_o[index],
                        };
                        let numerical = central_difference(
                            &params,
                            &spikes,
                            t_steps,
                            &d_logits,
                            |p| match name {
                                "w_q" => &mut p.blocks[layer].w_q[index],
                                "w_k" => &mut p.blocks[layer].w_k[index],
                                "w_v" => &mut p.blocks[layer].w_v[index],
                                _ => &mut p.blocks[layer].w_o[index],
                            },
                            1e-3,
                        );
                        assert_close(
                            &format!("layers {layers} block {layer} {name}[{index}]"),
                            analytic,
                            numerical,
                        );
                    }
                }
            }
        }
    }

    /// `ds_attn` is the gradient the spiking layer receives, and it is the term
    /// that makes credit timestep-specific. It is checkable for the same reason
    /// the parameters are: the attention head is smooth in its input.
    #[test]
    fn spike_gradient_matches_finite_difference() {
        let (t_steps, hidden, n_classes, d_model) = (9_usize, 6_usize, 4_usize, 4_usize);
        let params = params(hidden, n_classes, d_model, 2);
        let spikes = spike_train(t_steps, hidden);
        let d_logits = vec![0.31_f32, -0.72, 0.14, 0.27];
        let cache = attention_forward(&params, &spikes, t_steps).unwrap();
        let (_, ds_attn) = attention_gradient(&params, &cache, &spikes, &d_logits).unwrap();

        for &index in &[0_usize, 17, 31, 44] {
            let mut plus = spikes.clone();
            plus[index] += 1e-3;
            let mut minus = spikes.clone();
            minus[index] -= 1e-3;
            let numerical = (objective(&params, &plus, t_steps, &d_logits)
                - objective(&params, &minus, t_steps, &d_logits))
                / 2e-3;
            assert_close(&format!("ds_attn[{index}]"), ds_attn[index], numerical);
        }
    }

    /// Without position, mean-pooled attention is permutation-invariant, and the
    /// arm would add pairwise interaction while staying blind to order — the
    /// exact failure it exists to fix. This pins that the position code is what
    /// breaks the symmetry, so deleting it cannot pass silently.
    #[test]
    fn position_is_what_makes_the_read_out_order_sensitive() {
        let (t_steps, hidden, n_classes, d_model) = (12_usize, 6_usize, 4_usize, 4_usize);
        let params = params(hidden, n_classes, d_model, 1);
        let spikes = spike_train(t_steps, hidden);
        // Reverse the timesteps: identical per-unit rates, different order.
        let mut reversed = vec![0.0_f32; spikes.len()];
        for t in 0..t_steps {
            let source = (t_steps - 1 - t) * hidden;
            reversed[t * hidden..(t + 1) * hidden]
                .copy_from_slice(&spikes[source..source + hidden]);
        }
        let intact = attention_forward(&params, &spikes, t_steps).unwrap();
        let flipped = attention_forward(&params, &reversed, t_steps).unwrap();
        let moved: f32 = intact
            .pooled
            .iter()
            .zip(&flipped.pooled)
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            moved > 1e-3,
            "reversing time moved the pooled read-out by only {moved:e}: the arm is \
             order-blind and the position code is not doing its job"
        );

        // Control: the rate read-out this arm sits beside cannot tell them apart.
        let rate = |train: &[f32]| -> Vec<f32> {
            (0..hidden)
                .map(|h| (0..t_steps).map(|t| train[t * hidden + h]).sum::<f32>() / t_steps as f32)
                .collect()
        };
        assert_eq!(
            rate(&spikes),
            rate(&reversed),
            "fixture must hold rates fixed"
        );
    }

    /// Attention rows are a probability distribution over the whole utterance —
    /// that is the property the 46 ms membrane horizon does not have.
    #[test]
    fn attention_rows_are_normalised_over_every_timestep() {
        let (t_steps, hidden, n_classes, d_model) = (11_usize, 6_usize, 4_usize, 6_usize);
        let params = params(hidden, n_classes, d_model, 1);
        let spikes = spike_train(t_steps, hidden);
        let cache = attention_forward(&params, &spikes, t_steps).unwrap();
        for t in 0..t_steps {
            let row = cache.final_attention_row(t);
            assert_eq!(row.len(), t_steps);
            let total: f32 = row.iter().sum();
            assert!((total - 1.0).abs() < 1e-5, "row {t} sums to {total}");
            assert!(
                row.iter().all(|w| *w >= 0.0),
                "row {t} has a negative weight"
            );
        }
    }

    /// The registered initialisation must start as an exact identity residual.
    #[test]
    fn default_initialisation_zeroes_the_output_projection() {
        let params = AttentionParams::deterministic(8, 4, AttentionConfig::DEFAULT, 91).unwrap();
        assert_eq!(params.blocks.len(), DEFAULT_ATTENTION_LAYERS);
        for block in &params.blocks {
            assert!(
                block.w_o.iter().all(|v| *v == 0.0),
                "w_o must start at zero"
            );
            assert!(
                block.w_q.iter().any(|v| *v != 0.0),
                "w_q must not start at zero"
            );
        }
        params.check_shapes(8, 4).unwrap();
    }

    /// Adding a block must not re-roll the matrices that were already there.
    #[test]
    fn initialisation_streams_are_stable_across_depth() {
        let one =
            AttentionParams::deterministic(8, 4, AttentionConfig::new(4, 1).unwrap(), 91).unwrap();
        let two =
            AttentionParams::deterministic(8, 4, AttentionConfig::new(4, 2).unwrap(), 91).unwrap();
        assert_eq!(one.w_e, two.w_e);
        assert_eq!(one.w_a, two.w_a);
        assert_eq!(one.blocks[0], two.blocks[0]);
        assert_ne!(two.blocks[0].w_q, two.blocks[1].w_q);
    }

    #[test]
    fn odd_or_empty_configurations_are_refused() {
        assert!(AttentionConfig::new(5, 1).is_err());
        assert!(AttentionConfig::new(0, 1).is_err());
        assert!(AttentionConfig::new(4, 0).is_err());
        assert!(AttentionConfig::new(4, 1).is_ok());
    }

    /// A one-frame sample must not divide by zero.
    #[test]
    fn single_timestep_is_well_defined() {
        let params = params(6, 4, 4, 1);
        let spikes = spike_train(1, 6);
        let cache = attention_forward(&params, &spikes, 1).unwrap();
        assert!(cache.pooled.iter().all(|v| v.is_finite()));
        assert_eq!(cache.final_attention_row(0), &[1.0]);
    }

    // ---- adversarial / stress coverage, added 2026-08-21 -------------------
    //
    // The read-out is the paper's core artifact and every campaign cell runs it
    // 8156 times per epoch. These pin the inputs a real corpus can actually
    // produce (silent and saturated traces are both inside the instrument's
    // validity gates) and the failure modes that would be worst if silent.

    /// `silent_fraction <= 0.95` is a *passing* validity gate, so an all-silent
    /// window reaches this code on real data. It must produce a finite,
    /// position-only read-out rather than a NaN.
    #[test]
    fn an_all_silent_trace_is_finite_and_carries_only_position() {
        let params = params(8, 4, 4, 2);
        let t = 12;
        let silent = vec![0.0_f32; t * 8];
        let cache = attention_forward(&params, &silent, t).unwrap();
        assert!(cache.pooled.iter().all(|v| v.is_finite()));
        // Every attention row must still be a distribution.
        for step in 0..t {
            let row = cache.final_attention_row(step);
            let total: f32 = row.iter().sum();
            assert!((total - 1.0).abs() < 1e-5, "row {step} sums to {total}");
            assert!(row.iter().all(|w| *w >= 0.0));
        }
        // Silent input is not a constant read-out: position still varies with t.
        let logits = attention_logits(&params, cache.pooled());
        assert!(logits.iter().all(|v| v.is_finite()));
    }

    /// The opposite extreme: every unit firing at every step.
    #[test]
    fn a_fully_saturated_trace_stays_finite_through_both_passes() {
        let params = params(8, 4, 4, 2);
        let t = 12;
        let saturated = vec![1.0_f32; t * 8];
        let cache = attention_forward(&params, &saturated, t).unwrap();
        assert!(cache.pooled.iter().all(|v| v.is_finite()));
        let d_logits = vec![0.25_f32; 4];
        let (grad, ds) = attention_gradient(&params, &cache, &saturated, &d_logits).unwrap();
        assert!(grad.iter_all().all(|v| v.is_finite()));
        assert!(ds.iter().all(|v| v.is_finite()));

        // Finiteness alone is satisfied by a block that returned zeros, which
        // `scripts/find_weak_checks.py` flagged as a check a degenerate result
        // would pass. The invariants below are what make it a check.
        //
        // Every unit firing at every timestep means the only thing separating
        // one timestep from another is the positional code — so this is also
        // the case where a lost `pos` would show up as an exactly uniform
        // attention row and an identically zero spike gradient.
        for step in 0..t {
            let row = cache.final_attention_row(step);
            let total: f32 = row.iter().sum();
            assert!(
                (total - 1.0).abs() < 1e-4,
                "row {step} of a saturated trace sums to {total}, not 1"
            );
        }
        assert!(
            grad.iter_all().any(|v| *v != 0.0),
            "a saturated trace produced an all-zero parameter gradient"
        );
        assert!(
            ds.iter().any(|v| *v != 0.0),
            "a saturated trace produced an all-zero spike gradient"
        );
        // Position is what makes the read-out order sensitive, so a saturated
        // trace must still credit timesteps differently. If every entry were
        // equal, the block would be reporting rate and calling it timing.
        let first = ds[0];
        assert!(
            ds.iter().any(|v| (v - first).abs() > 1e-9),
            "every timestep received identical credit on a saturated trace;              the read-out is not distinguishing position"
        );
    }

    /// A non-finite parameter must **propagate**, not be swallowed.
    ///
    /// The row softmax subtracts its row maximum, and `maximum` is seeded at
    /// `NEG_INFINITY` with a `>` comparison — under which `NaN > x` is false. It
    /// would be easy for such a scheme to quietly emit a plausible finite
    /// distribution from a corrupt score. The instrument's `non_finite_events`
    /// gate can only catch what reaches it, so this pins that corruption is
    /// visible rather than absorbed.
    #[test]
    fn a_non_finite_parameter_reaches_the_output_instead_of_being_absorbed() {
        for poison in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut params = params(6, 4, 4, 1);
            params.blocks[0].w_q[0] = poison;
            let t = 5;
            let spikes = spike_train(t, 6);
            let cache = attention_forward(&params, &spikes, t).unwrap();
            assert!(
                cache.pooled.iter().any(|v| !v.is_finite()),
                "poison {poison} was absorbed: pooled is entirely finite"
            );
        }
    }

    /// Bit-identity is the property the whole `--config-hash` replay story rests
    /// on. Repeated evaluation must be bitwise equal, not merely close.
    #[test]
    fn repeated_evaluation_is_bitwise_identical() {
        let params = params(10, 5, 6, 3);
        let t = 9;
        let spikes = spike_train(t, 10);
        let first = attention_forward(&params, &spikes, t).unwrap();
        let second = attention_forward(&params, &spikes, t).unwrap();
        for (a, b) in first.pooled().iter().zip(second.pooled()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        let d_logits = vec![0.1_f32, -0.2, 0.3, -0.4, 0.5];
        let (ga, dsa) = attention_gradient(&params, &first, &spikes, &d_logits).unwrap();
        let (gb, dsb) = attention_gradient(&params, &second, &spikes, &d_logits).unwrap();
        for (a, b) in dsa.iter().zip(dsb.iter()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        for (a, b) in ga.iter_all().zip(gb.iter_all()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    /// A malformed call must be refused, never truncated or over-read. The
    /// `t x t` attention matrix means a wrong `t_steps` would quietly reshape
    /// the whole computation rather than crash.
    #[test]
    fn a_length_that_disagrees_with_t_steps_is_refused_in_both_directions() {
        let params = params(6, 4, 4, 1);
        let spikes = spike_train(5, 6);
        assert!(
            attention_forward(&params, &spikes, 4).is_err(),
            "too few steps"
        );
        assert!(
            attention_forward(&params, &spikes, 6).is_err(),
            "too many steps"
        );
        assert!(
            attention_forward(&params, &spikes, 0).is_err(),
            "zero steps"
        );
        assert!(
            attention_forward(&params, &[], 0).is_err(),
            "empty and zero"
        );
        assert!(
            attention_forward(&params, &spikes, 5).is_ok(),
            "the honest call"
        );
    }

    /// `published-2ms` frames an utterance into 358 steps and the attention
    /// matrix is `t x t`, so the real corpus already runs 128k-entry rows. Check
    /// a longer one stays finite and normalised.
    #[test]
    fn a_long_sequence_stays_normalised() {
        let params = params(4, 3, 4, 1);
        let t = 400;
        let spikes = spike_train(t, 4);
        let cache = attention_forward(&params, &spikes, t).unwrap();
        assert!(cache.pooled.iter().all(|v| v.is_finite()));
        for step in [0, t / 2, t - 1] {
            let total: f32 = cache.final_attention_row(step).iter().sum();
            assert!((total - 1.0).abs() < 1e-4, "row {step} sums to {total}");
        }
    }

    /// The gradient must be defined on a silent trace too - an arm that panics
    /// or NaNs on a quiet window would fail mid-campaign rather than at review.
    #[test]
    fn the_gradient_is_defined_on_a_silent_trace() {
        let params = params(8, 4, 4, 2);
        let t = 7;
        let silent = vec![0.0_f32; t * 8];
        let cache = attention_forward(&params, &silent, t).unwrap();
        let d_logits = vec![0.5_f32, -0.5, 0.25, -0.25];
        let (grad, ds) = attention_gradient(&params, &cache, &silent, &d_logits).unwrap();
        assert!(grad.iter_all().all(|v| v.is_finite()));
        assert!(ds.iter().all(|v| v.is_finite()));
        // The embedding is linear in the spikes, so a silent trace gives a zero
        // `dL/dW_e` - but the position path still moves everything downstream.
        assert!(
            grad.sum_squares() > 0.0,
            "silent input froze every parameter"
        );
    }
}
