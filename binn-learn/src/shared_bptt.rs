//! Runtime-sized shared-forward temporal stack with exact reverse mode.
//!
//! This replaces the invalid historical `MatchedDeepGradient` ceiling without
//! changing or deleting that legacy type. The forward graph is smooth and
//! label-independent: membrane leak, sigmoid surrogate spikes, soft reset, and
//! cumulative rate state are all explicit graph nodes differentiated by BPTT.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;

/// Stable label for the new ceiling family.
pub const SHARED_BPTT_LABEL: &str = "SHARED_FORWARD_TRUE_BPTT";
/// Frozen optimizer settings.
pub const ADAM_LR: f32 = 1e-3;
pub const ADAM_BETA1: f32 = 0.9;
pub const ADAM_BETA2: f32 = 0.999;
pub const ADAM_EPS: f32 = 1e-8;
pub const GRADIENT_CLIP_NORM: f32 = 5.0;

/// Dense temporal example accepted by both treatment and ceiling.
#[derive(Clone, Debug, PartialEq)]
pub struct DenseTemporalExample {
    pub frames: Vec<f32>,
    pub timesteps: usize,
    pub n_in: usize,
    pub label: u32,
}

/// One hidden layer's parameter arrays.
#[derive(Clone, Debug, PartialEq)]
struct Layer {
    n_in: usize,
    width: usize,
    weights: Vec<f32>,
    bias: Vec<f32>,
}

/// Runtime-sized model shared by treatment and ceiling.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedTemporalNet {
    n_in: usize,
    timesteps: usize,
    n_classes: usize,
    widths: Vec<usize>,
    alpha: f32,
    threshold: f32,
    surrogate_beta: f32,
    layers: Vec<Layer>,
    readout: Vec<f32>,
    readout_bias: Vec<f32>,
}

/// Label-independent forward products.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedForward {
    pub logits: Vec<f32>,
    pub prediction: u32,
    pub final_rates: Vec<f32>,
    cache: ForwardCache,
}

#[derive(Clone, Debug, PartialEq)]
struct LayerCache {
    /// Each is `[t * width + unit]`.
    q: Vec<f32>,
    spike: Vec<f32>,
    membrane: Vec<f32>,
    rate: Vec<f32>,
    /// `[t * layer_input + input]`.
    input: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
struct ForwardCache {
    layers: Vec<LayerCache>,
}

/// Gradient arrays matching every parameter group.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedGradients {
    layer_weights: Vec<Vec<f32>>,
    layer_bias: Vec<Vec<f32>>,
    readout: Vec<f32>,
    readout_bias: Vec<f32>,
}

impl SharedGradients {
    fn zeros(model: &SharedTemporalNet) -> Self {
        Self {
            layer_weights: model
                .layers
                .iter()
                .map(|layer| vec![0.0; layer.weights.len()])
                .collect(),
            layer_bias: model
                .layers
                .iter()
                .map(|layer| vec![0.0; layer.bias.len()])
                .collect(),
            readout: vec![0.0; model.readout.len()],
            readout_bias: vec![0.0; model.readout_bias.len()],
        }
    }

    pub fn flat(&self) -> Vec<f32> {
        let mut out = Vec::new();
        for (weights, bias) in self.layer_weights.iter().zip(&self.layer_bias) {
            out.extend(weights);
            out.extend(bias);
        }
        out.extend(&self.readout);
        out.extend(&self.readout_bias);
        out
    }

    /// The gradient's global L2 norm, widened to f64 **only on overflow**.
    ///
    /// # Why the fallback exists
    ///
    /// `f32::MAX` is ~3.4e38, so a sum of squares overflows once entries reach
    /// ~1e19 — while the norm itself, ~1e19, is comfortably representable. This
    /// is not a diagnostic: `Adam::update` clips against it, so an `inf` here
    /// took the branch
    ///
    /// ```text
    /// inf > GRADIENT_CLIP_NORM  ->  scale(GRADIENT_CLIP_NORM / inf) == scale(0.0)
    /// ```
    ///
    /// and multiplied **every** gradient entry by zero. Adam then stepped on an
    /// all-zero gradient. That is not what clipping to a norm means, and nothing
    /// recorded that the step did nothing — `StepDiagnostics` stores the norms
    /// but no caller gates on their finiteness.
    ///
    /// `AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md` §2b listed this file's f32
    /// sum-of-squares sites and assessed the RMS helper as diagnostic. It is,
    /// and this one is not; the two were not separated. Swept 2026-08-29.
    ///
    /// # Why conditional rather than always-f64
    ///
    /// `AMENDMENT_2026-08-03_L2_NORM_CONDITIONAL_WIDENING.md`, applied verbatim.
    /// Widening unconditionally changes the summation order for every call and
    /// so moves results in the last ulp; gating on `!is_finite()` leaves every
    /// representable norm **bit-identical** and replaces only the ones that were
    /// already wrong. A norm genuinely above `f32::MAX` still returns infinity.
    pub fn global_norm(&self) -> f32 {
        let flat = self.flat();
        let sum = flat.iter().map(|value| value * value).sum::<f32>();
        if sum.is_finite() {
            return sum.sqrt();
        }
        let wide: f64 = flat
            .iter()
            .map(|value| {
                let value = f64::from(*value);
                value * value
            })
            .sum();
        wide.sqrt() as f32
    }

    fn scale(&mut self, factor: f32) {
        for group in self
            .layer_weights
            .iter_mut()
            .chain(self.layer_bias.iter_mut())
        {
            for value in group {
                *value *= factor;
            }
        }
        for value in self.readout.iter_mut().chain(&mut self.readout_bias) {
            *value *= factor;
        }
    }

    /// Accumulate another gradient, useful for deterministic minibatches.
    pub fn add_assign(&mut self, other: &Self) {
        for (dst, src) in self
            .layer_weights
            .iter_mut()
            .zip(&other.layer_weights)
            .chain(self.layer_bias.iter_mut().zip(&other.layer_bias))
        {
            for (a, b) in dst.iter_mut().zip(src) {
                *a += *b;
            }
        }
        for (a, b) in self.readout.iter_mut().zip(&other.readout) {
            *a += *b;
        }
        for (a, b) in self.readout_bias.iter_mut().zip(&other.readout_bias) {
            *a += *b;
        }
    }
}

/// Per-step scale diagnostics. No gradient rescaling other than the frozen
/// global clip is performed.
#[derive(Clone, Debug, PartialEq)]
pub struct StepDiagnostics {
    pub unclipped_global_norm: f32,
    pub clipped_global_norm: f32,
    pub layer_gradient_rms: Vec<f32>,
    pub layer_step_rms: Vec<f32>,
    pub readout_gradient_rms: f32,
    pub readout_step_rms: f32,
}

impl SharedTemporalNet {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        n_in: usize,
        timesteps: usize,
        n_classes: usize,
        widths: &[usize],
        alpha: f32,
        threshold: f32,
        surrogate_beta: f32,
        seed: u64,
    ) -> Self {
        assert!(n_in > 0 && timesteps > 0 && n_classes >= 2);
        assert!(!widths.is_empty() && widths.iter().all(|&width| width > 0));
        assert!((0.0..1.0).contains(&alpha));
        assert!(threshold > 0.0 && surrogate_beta > 0.0);
        let mut rng = Rng::new(seed ^ 0x5A4E_D870_0000_0001);
        let mut previous = n_in;
        let mut layers = Vec::with_capacity(widths.len());
        for &width in widths {
            let scale = (2.0 / (previous + width) as f32).sqrt();
            let weights = (0..width * previous)
                .map(|_| (rng.next_f32() * 2.0 - 1.0) * scale)
                .collect();
            layers.push(Layer {
                n_in: previous,
                width,
                weights,
                bias: vec![0.0; width],
            });
            previous = width;
        }
        let out_scale = (2.0 / (previous + n_classes) as f32).sqrt();
        let readout = (0..n_classes * previous)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();
        Self {
            n_in,
            timesteps,
            n_classes,
            widths: widths.to_vec(),
            alpha,
            threshold,
            surrogate_beta,
            layers,
            readout,
            readout_bias: vec![0.0; n_classes],
        }
    }

    /// Forward execution never reads `example.label`.
    pub fn forward(&self, example: &DenseTemporalExample) -> SharedForward {
        self.validate_example(example);
        let mut caches: Vec<LayerCache> = Vec::with_capacity(self.layers.len());
        for (layer_index, layer) in self.layers.iter().enumerate() {
            let mut cache = LayerCache {
                q: vec![0.0; self.timesteps * layer.width],
                spike: vec![0.0; self.timesteps * layer.width],
                membrane: vec![0.0; self.timesteps * layer.width],
                rate: vec![0.0; self.timesteps * layer.width],
                input: vec![0.0; self.timesteps * layer.n_in],
            };
            for t in 0..self.timesteps {
                for input in 0..layer.n_in {
                    cache.input[t * layer.n_in + input] = if layer_index == 0 {
                        example.frames[t * self.n_in + input]
                    } else {
                        caches[layer_index - 1].rate[t * layer.n_in + input]
                    };
                }
                for unit in 0..layer.width {
                    let previous_membrane = if t == 0 {
                        0.0
                    } else {
                        cache.membrane[(t - 1) * layer.width + unit]
                    };
                    let mut q = self.alpha * previous_membrane + layer.bias[unit];
                    for input in 0..layer.n_in {
                        q += layer.weights[unit * layer.n_in + input]
                            * cache.input[t * layer.n_in + input];
                    }
                    let spike = sigmoid(self.surrogate_beta * (q - self.threshold));
                    let membrane = q - self.threshold * spike;
                    let previous_rate = if t == 0 {
                        0.0
                    } else {
                        cache.rate[(t - 1) * layer.width + unit]
                    };
                    let rate = previous_rate + spike / self.timesteps as f32;
                    let index = t * layer.width + unit;
                    cache.q[index] = q;
                    cache.spike[index] = spike;
                    cache.membrane[index] = membrane;
                    cache.rate[index] = rate;
                }
            }
            caches.push(cache);
        }
        let last_width = *self.widths.last().unwrap();
        let last_rate = &caches.last().unwrap().rate;
        let rate_offset = (self.timesteps - 1) * last_width;
        let final_rates = last_rate[rate_offset..rate_offset + last_width].to_vec();
        let mut logits = self.readout_bias.clone();
        for class in 0..self.n_classes {
            for unit in 0..last_width {
                logits[class] += self.readout[class * last_width + unit] * final_rates[unit];
            }
        }
        SharedForward {
            prediction: argmax(&logits) as u32,
            logits,
            final_rates,
            cache: ForwardCache { layers: caches },
        }
    }

    pub fn loss(&self, example: &DenseTemporalExample) -> f32 {
        let forward = self.forward(example);
        cross_entropy(&forward.logits, example.label as usize)
    }

    /// Exact reverse mode through the complete shared forward graph.
    pub fn loss_and_gradients(&self, example: &DenseTemporalExample) -> (f32, SharedGradients) {
        let forward = self.forward(example);
        let loss = cross_entropy(&forward.logits, example.label as usize);
        let mut gradients = SharedGradients::zeros(self);
        let mut delta = softmax(&forward.logits);
        delta[example.label as usize] -= 1.0;

        let depth = self.layers.len();
        let last_width = *self.widths.last().unwrap();
        for class in 0..self.n_classes {
            gradients.readout_bias[class] = delta[class];
            for unit in 0..last_width {
                gradients.readout[class * last_width + unit] =
                    delta[class] * forward.final_rates[unit];
            }
        }

        let mut d_membrane: Vec<Vec<f32>> = self
            .layers
            .iter()
            .map(|layer| vec![0.0; self.timesteps * layer.width])
            .collect();
        let mut d_rate: Vec<Vec<f32>> = self
            .layers
            .iter()
            .map(|layer| vec![0.0; self.timesteps * layer.width])
            .collect();
        for unit in 0..last_width {
            let mut value = 0.0;
            for class in 0..self.n_classes {
                value += self.readout[class * last_width + unit] * delta[class];
            }
            d_rate[depth - 1][(self.timesteps - 1) * last_width + unit] = value;
        }

        // Reverse topological order: timestep descending, and within a
        // timestep layer descending because layer L consumes rate[L-1, t].
        for t in (0..self.timesteps).rev() {
            for layer_index in (0..depth).rev() {
                let layer = &self.layers[layer_index];
                let cache = &forward.cache.layers[layer_index];
                for unit in 0..layer.width {
                    let index = t * layer.width + unit;
                    let dr = d_rate[layer_index][index];
                    if t > 0 {
                        d_rate[layer_index][(t - 1) * layer.width + unit] += dr;
                    }
                    let mut ds = dr / self.timesteps as f32;

                    let du = d_membrane[layer_index][index];
                    let mut dq = du;
                    ds -= self.threshold * du;
                    let spike = cache.spike[index];
                    dq += ds * self.surrogate_beta * spike * (1.0 - spike);

                    gradients.layer_bias[layer_index][unit] += dq;
                    for input in 0..layer.n_in {
                        let input_value = cache.input[t * layer.n_in + input];
                        gradients.layer_weights[layer_index][unit * layer.n_in + input] +=
                            dq * input_value;
                        if layer_index > 0 {
                            d_rate[layer_index - 1][t * layer.n_in + input] +=
                                layer.weights[unit * layer.n_in + input] * dq;
                        }
                    }
                    if t > 0 {
                        d_membrane[layer_index][(t - 1) * layer.width + unit] += self.alpha * dq;
                    }
                }
            }
        }
        (loss, gradients)
    }

    /// Local random-feedback gradients on the exact same cached forward.
    ///
    /// Readout gradients are exact. Hidden layers receive class error through
    /// immutable feedback matrices and update local weights using a temporal
    /// eligibility trace. The trace differentiates the layer's final
    /// cumulative rate through membrane leak and soft reset while treating the
    /// layer's input trajectory as fixed. This is a treatment, not a ceiling:
    /// it does not transport derivatives between hidden layers.
    pub fn feedback_gradients(
        &self,
        example: &DenseTemporalExample,
        feedback: &[Vec<f32>],
    ) -> SharedGradients {
        assert_eq!(feedback.len(), self.layers.len());
        let forward = self.forward(example);
        let mut gradients = SharedGradients::zeros(self);
        let mut delta = softmax(&forward.logits);
        delta[example.label as usize] -= 1.0;
        let last_width = *self.widths.last().unwrap();
        for class in 0..self.n_classes {
            gradients.readout_bias[class] = delta[class];
            for unit in 0..last_width {
                gradients.readout[class * last_width + unit] =
                    delta[class] * forward.final_rates[unit];
            }
        }
        for (layer_index, layer) in self.layers.iter().enumerate() {
            assert_eq!(feedback[layer_index].len(), layer.width * self.n_classes);
            let cache = &forward.cache.layers[layer_index];
            for unit in 0..layer.width {
                let mut modulator = 0.0;
                for class in 0..self.n_classes {
                    modulator +=
                        feedback[layer_index][unit * self.n_classes + class] * delta[class];
                }
                let (weight_eligibility, bias_eligibility) =
                    self.local_rate_eligibility(layer, cache, unit);
                gradients.layer_bias[layer_index][unit] = modulator * bias_eligibility;
                for input in 0..layer.n_in {
                    gradients.layer_weights[layer_index][unit * layer.n_in + input] =
                        modulator * weight_eligibility[input];
                }
            }
        }
        gradients
    }

    /// Derivative of one unit's final cumulative rate with respect to its
    /// incoming weights and bias, holding the cached input trajectory fixed.
    fn local_rate_eligibility(
        &self,
        layer: &Layer,
        cache: &LayerCache,
        unit: usize,
    ) -> (Vec<f32>, f32) {
        let mut membrane_weight = vec![0.0f32; layer.n_in];
        let mut rate_weight = vec![0.0f32; layer.n_in];
        let mut membrane_bias = 0.0f32;
        let mut rate_bias = 0.0f32;
        for t in 0..self.timesteps {
            let spike = cache.spike[t * layer.width + unit];
            let surrogate = self.surrogate_beta * spike * (1.0 - spike);
            for input in 0..layer.n_in {
                let direct = cache.input[t * layer.n_in + input];
                let q_eligibility = self.alpha * membrane_weight[input] + direct;
                let spike_eligibility = surrogate * q_eligibility;
                rate_weight[input] += spike_eligibility / self.timesteps as f32;
                membrane_weight[input] = q_eligibility - self.threshold * spike_eligibility;
            }
            let q_bias_eligibility = self.alpha * membrane_bias + 1.0;
            let spike_bias_eligibility = surrogate * q_bias_eligibility;
            rate_bias += spike_bias_eligibility / self.timesteps as f32;
            membrane_bias = q_bias_eligibility - self.threshold * spike_bias_eligibility;
        }
        (rate_weight, rate_bias)
    }

    pub fn accuracy(&self, examples: &[DenseTemporalExample]) -> f32 {
        assert!(!examples.is_empty());
        let before = self.parameter_fingerprint();
        let correct = examples
            .iter()
            .filter(|example| self.forward(example).prediction == example.label)
            .count();
        assert_eq!(
            before,
            self.parameter_fingerprint(),
            "evaluation mutated shared-forward parameters"
        );
        correct as f32 / examples.len() as f32
    }

    /// RMS of the immutable-feedback modulators at the model's current
    /// parameters, evaluated without applying an update.
    ///
    /// One value is returned per hidden layer. This is intentionally separate
    /// from gradient RMS: a live modulator can still be masked by a broken
    /// eligibility path.
    pub fn feedback_modulator_rms(
        &self,
        examples: &[DenseTemporalExample],
        feedback: &[Vec<f32>],
    ) -> Vec<f32> {
        assert!(!examples.is_empty());
        assert_eq!(feedback.len(), self.layers.len());
        let before = self.parameter_fingerprint();
        let mut squared = vec![0.0f64; self.layers.len()];
        let mut counts = vec![0usize; self.layers.len()];
        for example in examples {
            self.validate_example(example);
            let forward = self.forward(example);
            let mut delta = softmax(&forward.logits);
            delta[example.label as usize] -= 1.0;
            for (layer_index, layer) in self.layers.iter().enumerate() {
                assert_eq!(feedback[layer_index].len(), layer.width * self.n_classes);
                for unit in 0..layer.width {
                    let mut modulator = 0.0f32;
                    for class in 0..self.n_classes {
                        modulator +=
                            feedback[layer_index][unit * self.n_classes + class] * delta[class];
                    }
                    squared[layer_index] += f64::from(modulator * modulator);
                    counts[layer_index] += 1;
                }
            }
        }
        assert_eq!(
            before,
            self.parameter_fingerprint(),
            "modulator diagnostics mutated shared-forward parameters"
        );
        squared
            .into_iter()
            .zip(counts)
            .map(|(sum, count)| (sum / count as f64).sqrt() as f32)
            .collect()
    }

    pub fn parameter_count(&self) -> usize {
        self.parameter_values().len()
    }

    pub fn parameter_values(&self) -> Vec<f32> {
        let mut values = Vec::new();
        for layer in &self.layers {
            values.extend(&layer.weights);
            values.extend(&layer.bias);
        }
        values.extend(&self.readout);
        values.extend(&self.readout_bias);
        values
    }

    pub fn set_parameter(&mut self, index: usize, value: f32) {
        let mut remaining = index;
        for layer in &mut self.layers {
            if remaining < layer.weights.len() {
                layer.weights[remaining] = value;
                return;
            }
            remaining -= layer.weights.len();
            if remaining < layer.bias.len() {
                layer.bias[remaining] = value;
                return;
            }
            remaining -= layer.bias.len();
        }
        if remaining < self.readout.len() {
            self.readout[remaining] = value;
            return;
        }
        remaining -= self.readout.len();
        if remaining < self.readout_bias.len() {
            self.readout_bias[remaining] = value;
            return;
        }
        panic!("parameter index {index} out of range");
    }

    pub fn parameter_fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for value in self.parameter_values() {
            hash ^= value.to_bits() as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        hash
    }

    fn apply_flat_step(&mut self, step: &[f32]) {
        assert_eq!(step.len(), self.parameter_count());
        let values = self.parameter_values();
        for (index, (&value, &delta)) in values.iter().zip(step).enumerate() {
            self.set_parameter(index, value + delta);
        }
    }

    fn validate_example(&self, example: &DenseTemporalExample) {
        assert_eq!(example.n_in, self.n_in);
        assert_eq!(example.timesteps, self.timesteps);
        assert_eq!(example.frames.len(), self.n_in * self.timesteps);
        assert!((example.label as usize) < self.n_classes);
    }
}

/// Deterministic Adam state for the BPTT ceiling.
#[derive(Clone, Debug)]
pub struct Adam {
    moment1: Vec<f32>,
    moment2: Vec<f32>,
    step: u64,
}

impl Adam {
    pub fn new(model: &SharedTemporalNet) -> Self {
        Self {
            moment1: vec![0.0; model.parameter_count()],
            moment2: vec![0.0; model.parameter_count()],
            step: 0,
        }
    }

    pub fn update(
        &mut self,
        model: &mut SharedTemporalNet,
        gradients: &SharedGradients,
    ) -> StepDiagnostics {
        let mut clipped = gradients.clone();
        let unclipped_global_norm = clipped.global_norm();
        if unclipped_global_norm > GRADIENT_CLIP_NORM {
            clipped.scale(GRADIENT_CLIP_NORM / unclipped_global_norm);
        }
        let clipped_global_norm = clipped.global_norm();
        let flat = clipped.flat();
        self.step += 1;
        let mut applied = vec![0.0f32; flat.len()];
        let bias1 = 1.0 - ADAM_BETA1.powi(self.step as i32);
        let bias2 = 1.0 - ADAM_BETA2.powi(self.step as i32);
        for index in 0..flat.len() {
            self.moment1[index] =
                ADAM_BETA1 * self.moment1[index] + (1.0 - ADAM_BETA1) * flat[index];
            self.moment2[index] =
                ADAM_BETA2 * self.moment2[index] + (1.0 - ADAM_BETA2) * flat[index] * flat[index];
            let m_hat = self.moment1[index] / bias1;
            let v_hat = self.moment2[index] / bias2;
            applied[index] = -ADAM_LR * m_hat / (v_hat.sqrt() + ADAM_EPS);
        }

        let mut layer_gradient_rms = Vec::with_capacity(model.layers.len());
        let mut layer_step_rms = Vec::with_capacity(model.layers.len());
        let mut cursor = 0usize;
        for layer in &model.layers {
            let length = layer.weights.len() + layer.bias.len();
            layer_gradient_rms.push(rms(&flat[cursor..cursor + length]));
            layer_step_rms.push(rms(&applied[cursor..cursor + length]));
            cursor += length;
        }
        let readout_length = model.readout.len() + model.readout_bias.len();
        let readout_gradient_rms = rms(&flat[cursor..cursor + readout_length]);
        let readout_step_rms = rms(&applied[cursor..cursor + readout_length]);
        model.apply_flat_step(&applied);
        StepDiagnostics {
            unclipped_global_norm,
            clipped_global_norm,
            layer_gradient_rms,
            layer_step_rms,
            readout_gradient_rms,
            readout_step_rms,
        }
    }
}

fn apply_sgd_step(
    model: &mut SharedTemporalNet,
    gradients: &SharedGradients,
    lr: f32,
) -> StepDiagnostics {
    let mut clipped = gradients.clone();
    let unclipped_global_norm = clipped.global_norm();
    if unclipped_global_norm > GRADIENT_CLIP_NORM {
        clipped.scale(GRADIENT_CLIP_NORM / unclipped_global_norm);
    }
    let clipped_global_norm = clipped.global_norm();
    let flat = clipped.flat();
    let applied: Vec<f32> = flat.iter().map(|gradient| -lr * gradient).collect();
    let mut layer_gradient_rms = Vec::with_capacity(model.layers.len());
    let mut layer_step_rms = Vec::with_capacity(model.layers.len());
    let mut cursor = 0usize;
    for layer in &model.layers {
        let length = layer.weights.len() + layer.bias.len();
        layer_gradient_rms.push(rms(&flat[cursor..cursor + length]));
        layer_step_rms.push(rms(&applied[cursor..cursor + length]));
        cursor += length;
    }
    let readout_length = model.readout.len() + model.readout_bias.len();
    let readout_gradient_rms = rms(&flat[cursor..cursor + readout_length]);
    let readout_step_rms = rms(&applied[cursor..cursor + readout_length]);
    model.apply_flat_step(&applied);
    StepDiagnostics {
        unclipped_global_norm,
        clipped_global_norm,
        layer_gradient_rms,
        layer_step_rms,
        readout_gradient_rms,
        readout_step_rms,
    }
}

/// Train a BPTT ceiling for a fixed number of epochs.
pub fn train_bptt(
    model: &mut SharedTemporalNet,
    train: &[DenseTemporalExample],
    epochs: usize,
) -> Vec<StepDiagnostics> {
    assert!(!train.is_empty());
    let mut optimizer = Adam::new(model);
    let mut diagnostics = Vec::new();
    for _ in 0..epochs {
        for example in train {
            let (_, gradients) = model.loss_and_gradients(example);
            diagnostics.push(optimizer.update(model, &gradients));
        }
    }
    diagnostics
}

/// Immutable random feedback matrices for the matched treatment.
/// True BPTT trained with **plain SGD at an explicit learning rate**.
///
/// # Why this exists
///
/// [`train_bptt`] uses [`Adam`]; [`train_feedback`] uses plain SGD at a caller-
/// supplied `lr`. Comparing them therefore varies **two** things at once — the
/// credit pathway *and* the optimiser — so a gap between them cannot be
/// attributed to credit assignment. Adam is far more forgiving on badly-scaled
/// problems, which is exactly the regime this codebase keeps landing in.
///
/// This function is the optimiser-matched ceiling: identical update rule and
/// step size as the feedback treatment, differing **only** in whether the
/// gradients are true or feedback-projected. Report it alongside the Adam
/// ceiling, which remains the "best achievable" reference.
pub fn train_bptt_sgd(
    model: &mut SharedTemporalNet,
    train: &[DenseTemporalExample],
    epochs: usize,
    lr: f32,
) -> Vec<StepDiagnostics> {
    assert!(!train.is_empty(), "train_bptt_sgd needs training data");
    assert!(lr > 0.0, "learning rate must be positive");
    let mut diagnostics = Vec::with_capacity(epochs.saturating_mul(train.len()));
    for _ in 0..epochs {
        for example in train {
            let (_, gradients) = model.loss_and_gradients(example);
            diagnostics.push(apply_sgd_step(model, &gradients, lr));
        }
    }
    diagnostics
}

/// Mean of `layer_step_rms` and `readout_step_rms` over a training run.
///
/// The realised step size actually taken, as opposed to the nominal `lr`. Two
/// arms whose realised step sizes differ by an order of magnitude are not
/// comparable regardless of what their learning rates say.
pub fn mean_step_rms(diagnostics: &[StepDiagnostics]) -> f32 {
    if diagnostics.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f64;
    let mut count = 0usize;
    for d in diagnostics {
        for &rms in &d.layer_step_rms {
            if rms.is_finite() {
                total += rms as f64;
                count += 1;
            }
        }
        if d.readout_step_rms.is_finite() {
            total += d.readout_step_rms as f64;
            count += 1;
        }
    }
    if count == 0 {
        return 0.0;
    }
    (total / count as f64) as f32
}

pub fn random_feedback(model: &SharedTemporalNet, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = Rng::new(seed ^ 0xFEE0_BAC0_0000_0001);
    model
        .layers
        .iter()
        .map(|layer| {
            let scale = 0.2 / (layer.width as f32).sqrt();
            (0..layer.width * model.n_classes)
                .map(|_| (rng.next_f32() * 2.0 - 1.0) * scale)
                .collect()
        })
        .collect()
}

/// Train the matched random-feedback treatment with deterministic SGD.
pub fn train_feedback(
    model: &mut SharedTemporalNet,
    feedback: &[Vec<f32>],
    train: &[DenseTemporalExample],
    epochs: usize,
    lr: f32,
) -> Vec<StepDiagnostics> {
    assert!(!train.is_empty() && lr > 0.0);
    let mut diagnostics = Vec::with_capacity(epochs * train.len());
    for _ in 0..epochs {
        for example in train {
            let gradients = model.feedback_gradients(example, feedback);
            diagnostics.push(apply_sgd_step(model, &gradients, lr));
        }
    }
    diagnostics
}

/// Train a feedback-alignment treatment while adapting each feedback matrix
/// toward the current downstream transpose product. Forward execution remains
/// exactly the shared model; feedback affects updates only.
pub fn train_learned_feedback(
    model: &mut SharedTemporalNet,
    feedback: &mut [Vec<f32>],
    train: &[DenseTemporalExample],
    epochs: usize,
    lr: f32,
    feedback_lr: f32,
) {
    assert!(!train.is_empty() && lr > 0.0);
    assert!((0.0..=1.0).contains(&feedback_lr));
    for _ in 0..epochs {
        for example in train {
            let gradients = model.feedback_gradients(example, feedback);
            let _ = apply_sgd_step(model, &gradients, lr);
            align_feedback(model, feedback, feedback_lr);
        }
    }
}

/// Feedback-alignment treatment trained with **Adam**, matching [`train_bptt`].
///
/// # Why this exists
///
/// [`train_learned_feedback`] and [`train_bptt_sgd`] are an optimiser-matched
/// pair under plain SGD. That pairing is only useful where SGD can train the
/// architecture at all. On the matched `CoincidenceTask` stack it cannot: at
/// depth >= 2 the SGD ceiling sits at chance for every step size on the
/// registered ladder while the Adam ceiling reaches 0.90+, so an SGD-matched
/// comparison measures the optimiser's failure and nothing else.
///
/// This is the same pair at the optimiser that works: identical update rule,
/// identical hyper-parameters, differing **only** in whether the gradients are
/// true or feedback-projected. Forward execution is unchanged — feedback affects
/// updates only, exactly as in [`train_learned_feedback`].
pub fn train_learned_feedback_adam(
    model: &mut SharedTemporalNet,
    feedback: &mut [Vec<f32>],
    train: &[DenseTemporalExample],
    epochs: usize,
    feedback_lr: f32,
) -> Vec<StepDiagnostics> {
    assert!(!train.is_empty(), "train_learned_feedback_adam needs data");
    assert!((0.0..=1.0).contains(&feedback_lr));
    let mut optimizer = Adam::new(model);
    let mut diagnostics = Vec::with_capacity(epochs.saturating_mul(train.len()));
    for _ in 0..epochs {
        for example in train {
            let gradients = model.feedback_gradients(example, feedback);
            diagnostics.push(optimizer.update(model, &gradients));
            align_feedback(model, feedback, feedback_lr);
        }
    }
    diagnostics
}

fn align_feedback(model: &SharedTemporalNet, feedback: &mut [Vec<f32>], lr: f32) {
    assert_eq!(feedback.len(), model.layers.len());
    for layer_index in (0..model.layers.len()).rev() {
        let width = model.layers[layer_index].width;
        let mut target = vec![0.0f32; width * model.n_classes];
        if layer_index + 1 == model.layers.len() {
            for unit in 0..width {
                for class in 0..model.n_classes {
                    target[unit * model.n_classes + class] = model.readout[class * width + unit];
                }
            }
        } else {
            let downstream = &model.layers[layer_index + 1];
            let scale = 1.0 / (downstream.width as f32).sqrt();
            for unit in 0..width {
                for class in 0..model.n_classes {
                    let mut value = 0.0;
                    for next in 0..downstream.width {
                        value += downstream.weights[next * width + unit]
                            * feedback[layer_index + 1][next * model.n_classes + class];
                    }
                    target[unit * model.n_classes + class] = value * scale;
                }
            }
        }
        for (actual, desired) in feedback[layer_index].iter_mut().zip(target) {
            *actual += lr * (desired - *actual);
        }
    }
}

fn sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values: Vec<f32> = logits.iter().map(|&value| (value - max).exp()).collect();
    let sum = values.iter().sum::<f32>().max(1e-12);
    for value in &mut values {
        *value /= sum;
    }
    values
}

fn cross_entropy(logits: &[f32], label: usize) -> f32 {
    -softmax(logits)[label].max(1e-12).ln()
}

fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn rms(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        (values.iter().map(|value| value * value).sum::<f32>() / values.len() as f32).sqrt()
    }
}

#[cfg(test)]
mod tests {

    /// A gradient whose norm overflows f32 must be CLIPPED, not zeroed.
    ///
    /// `AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md` §2b listed `shared_bptt.rs`
    /// among the unswept f32 sum-of-squares sites and assessed the one at the
    /// RMS helper as diagnostic. **`global_norm` is not diagnostic** — it drives
    /// gradient clipping in `Adam::update`, so this is the load-bearing member
    /// of that list and it was not separated out.
    ///
    /// `f32::MAX` is ~3.4e38, so the sum of squares overflows once entries reach
    /// ~1e19 while the norm itself stays representable. The old code then did:
    ///
    ///     unclipped = inf;  inf > 5.0;  scale(5.0 / inf) == scale(0.0)
    ///
    /// which multiplies **every** gradient entry by zero. Adam then takes a step
    /// on an all-zero gradient. That is not what "clip to norm 5" means, and
    /// nothing anywhere reports that the step did nothing.
    #[test]
    fn a_gradient_whose_norm_overflows_is_clipped_not_zeroed() {
        let mut gradients = SharedGradients {
            layer_weights: vec![vec![3.0e19_f32, -3.0e19]],
            layer_bias: vec![vec![1.0e19]],
            readout: vec![2.0e19, -1.0e19],
            readout_bias: vec![0.0],
        };
        assert!(
            gradients.flat().iter().all(|v| v.is_finite()),
            "every entry must be finite; only the SUM may overflow"
        );
        let norm = gradients.global_norm();
        assert!(
            norm.is_finite(),
            "global_norm returned {norm} for a gradient whose true norm is ~5e19, \
             which f32 can represent"
        );
        gradients.scale(GRADIENT_CLIP_NORM / norm);
        let after = gradients.global_norm();
        assert!(
            after > GRADIENT_CLIP_NORM * 0.5 && after < GRADIENT_CLIP_NORM * 1.5,
            "clipped norm {after} is not near {GRADIENT_CLIP_NORM}; the gradient was destroyed"
        );
    }

    /// Widening must not move a norm f32 could already represent.
    #[test]
    fn global_norm_is_bit_identical_below_the_overflow_threshold() {
        let gradients = SharedGradients {
            layer_weights: vec![vec![0.5_f32, -0.25], vec![1e-7, 3.0]],
            layer_bias: vec![vec![0.125], vec![-2.0]],
            readout: vec![1.5, -0.75, 1e18],
            readout_bias: vec![0.0625],
        };
        let flat = gradients.flat();
        let naive = flat.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(naive.is_finite(), "the case must not overflow");
        assert_eq!(
            gradients.global_norm().to_bits(),
            naive.to_bits(),
            "widening moved a representable norm"
        );
    }
    use super::*;

    fn example(label: u32) -> DenseTemporalExample {
        let mut frames = vec![0.0f32; 4 * 3];
        frames[label as usize] = 1.0;
        frames[3 + label as usize] = 0.5;
        frames[6 + (label as usize + 1) % 3] = 0.25;
        DenseTemporalExample {
            frames,
            timesteps: 4,
            n_in: 3,
            label,
        }
    }

    /// The Adam feedback treatment differs from the Adam ceiling in the credit
    /// pathway and in nothing else.
    ///
    /// Registered in `PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` section
    /// 7a. Both must move the parameters, and they must move them **differently**
    /// — if the two agreed bit-for-bit, `deep-snn-scaling` v136 would be comparing
    /// an arm against itself and reporting a gap of exactly zero as a result.
    #[test]
    fn the_adam_feedback_treatment_is_not_secretly_the_adam_ceiling() {
        let train: Vec<DenseTemporalExample> = (0..3).map(example).collect();
        let widths = [5usize, 4];

        let mut treatment = SharedTemporalNet::new(3, 4, 3, &widths, 0.8, 0.5, 2.0, 11);
        let start = treatment.parameter_fingerprint();
        let mut feedback = random_feedback(&treatment, 11);
        train_learned_feedback_adam(&mut treatment, &mut feedback, &train, 3, 0.01);

        let mut ceiling = SharedTemporalNet::new(3, 4, 3, &widths, 0.8, 0.5, 2.0, 11);
        assert_eq!(
            start,
            ceiling.parameter_fingerprint(),
            "the two arms must start from the same parameters"
        );
        train_bptt(&mut ceiling, &train, 3);

        assert_ne!(
            start,
            treatment.parameter_fingerprint(),
            "the feedback treatment did not move the parameters at all"
        );
        assert_ne!(
            start,
            ceiling.parameter_fingerprint(),
            "the ceiling did not move the parameters at all"
        );
        assert_ne!(
            treatment.parameter_fingerprint(),
            ceiling.parameter_fingerprint(),
            "feedback-projected and true gradients produced identical parameters; \
             the treatment is not projecting through the feedback matrix"
        );
    }

    /// Adam is what makes the deep arms trainable at all, so the treatment must
    /// really be using it rather than falling back to plain SGD.
    ///
    /// The tell is Adam's normalisation: the first step's magnitude is set by the
    /// optimiser, not by the gradient's scale. Plain SGD at the same nominal rate
    /// would leave a step proportional to the gradient.
    #[test]
    fn the_adam_feedback_treatment_takes_adam_sized_steps() {
        let train: Vec<DenseTemporalExample> = (0..2).map(example).collect();
        let widths = [4usize, 4];
        let mut model = SharedTemporalNet::new(3, 4, 3, &widths, 0.8, 0.5, 2.0, 5);
        let mut feedback = random_feedback(&model, 5);
        let diagnostics = train_learned_feedback_adam(&mut model, &mut feedback, &train, 1, 0.01);

        assert_eq!(
            diagnostics.len(),
            train.len(),
            "one diagnostic per example, so a caller can see the realised steps"
        );
        // Adam's first update is +/- lr per parameter, up to the epsilon and the
        // bias correction. A step far from that is not Adam.
        let first = diagnostics[0].readout_step_rms;
        assert!(
            first.is_finite() && first > ADAM_LR * 0.5 && first < ADAM_LR * 2.0,
            "first readout step RMS {first:.3e} is not Adam-sized (lr = {ADAM_LR:.0e})"
        );
    }

    /// Forward execution must not read the feedback matrices. If it did, the
    /// treatment and the ceiling would not share a forward graph and the whole
    /// comparison would be confounded.
    #[test]
    fn feedback_training_leaves_the_forward_graph_shared() {
        let train: Vec<DenseTemporalExample> = (0..2).map(example).collect();
        let widths = [4usize];
        let mut model = SharedTemporalNet::new(3, 4, 3, &widths, 0.8, 0.5, 2.0, 3);
        let mut feedback = random_feedback(&model, 3);
        train_learned_feedback_adam(&mut model, &mut feedback, &train, 2, 0.01);

        // Same parameters, evaluated twice, with the feedback matrices scrambled
        // in between. The forward must not notice.
        let before = model.forward(&train[0]).logits;
        for matrix in feedback.iter_mut() {
            for value in matrix.iter_mut() {
                *value = -*value * 7.0;
            }
        }
        let after = model.forward(&train[0]).logits;
        assert_eq!(
            before.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            after.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            "the forward changed when only the feedback matrices did"
        );
    }

    fn finite_difference_check(widths: &[usize]) {
        let model = SharedTemporalNet::new(3, 4, 3, widths, 0.8, 0.5, 2.0, 7);
        let ex = example(1);
        let (_, gradients) = model.loss_and_gradients(&ex);
        let analytic = gradients.flat();
        let values = model.parameter_values();
        // Check every group by sampling the first, middle, and last parameter.
        let mut indices = Vec::new();
        let mut cursor = 0usize;
        for layer in &model.layers {
            for length in [layer.weights.len(), layer.bias.len()] {
                indices.push(cursor);
                indices.push(cursor + length / 2);
                indices.push(cursor + length - 1);
                cursor += length;
            }
        }
        for length in [model.readout.len(), model.readout_bias.len()] {
            indices.push(cursor);
            indices.push(cursor + length / 2);
            indices.push(cursor + length - 1);
            cursor += length;
        }
        indices.sort_unstable();
        indices.dedup();
        let epsilon = 1e-3f32;
        for index in indices {
            let mut plus = model.clone();
            plus.set_parameter(index, values[index] + epsilon);
            let mut minus = model.clone();
            minus.set_parameter(index, values[index] - epsilon);
            let numerical = (plus.loss(&ex) - minus.loss(&ex)) / (2.0 * epsilon);
            let error = (numerical - analytic[index]).abs();
            let scale = numerical.abs().max(analytic[index].abs()).max(1e-3);
            assert!(
                error / scale < 0.04,
                "index={index}, numerical={numerical}, analytic={}, rel={}",
                analytic[index],
                error / scale
            );
        }
    }

    #[test]
    fn finite_differences_agree_at_depths_one_and_two() {
        finite_difference_check(&[4]);
        finite_difference_check(&[4, 3]);
    }

    #[test]
    fn local_temporal_eligibility_matches_fixed_input_finite_difference() {
        let model = SharedTemporalNet::new(3, 4, 3, &[4, 3], 0.8, 0.5, 2.0, 9);
        let forward = model.forward(&example(1));
        let layer = &model.layers[1];
        let cache = &forward.cache.layers[1];
        let unit = 0;
        let (weight_eligibility, bias_eligibility) =
            model.local_rate_eligibility(layer, cache, unit);
        let epsilon = 1e-3f32;
        for input in 0..layer.n_in {
            let mut plus_weights =
                layer.weights[unit * layer.n_in..(unit + 1) * layer.n_in].to_vec();
            let mut minus_weights = plus_weights.clone();
            plus_weights[input] += epsilon;
            minus_weights[input] -= epsilon;
            let plus = fixed_input_final_rate(
                &cache.input,
                &plus_weights,
                layer.bias[unit],
                model.timesteps,
                layer.n_in,
                model.alpha,
                model.threshold,
                model.surrogate_beta,
            );
            let minus = fixed_input_final_rate(
                &cache.input,
                &minus_weights,
                layer.bias[unit],
                model.timesteps,
                layer.n_in,
                model.alpha,
                model.threshold,
                model.surrogate_beta,
            );
            let numerical = (plus - minus) / (2.0 * epsilon);
            assert!((numerical - weight_eligibility[input]).abs() < 2e-4);
        }
        let weights = &layer.weights[unit * layer.n_in..(unit + 1) * layer.n_in];
        let plus = fixed_input_final_rate(
            &cache.input,
            weights,
            layer.bias[unit] + epsilon,
            model.timesteps,
            layer.n_in,
            model.alpha,
            model.threshold,
            model.surrogate_beta,
        );
        let minus = fixed_input_final_rate(
            &cache.input,
            weights,
            layer.bias[unit] - epsilon,
            model.timesteps,
            layer.n_in,
            model.alpha,
            model.threshold,
            model.surrogate_beta,
        );
        let numerical = (plus - minus) / (2.0 * epsilon);
        assert!((numerical - bias_eligibility).abs() < 2e-4);
    }

    #[test]
    fn labels_change_gradients_but_never_forward() {
        let model = SharedTemporalNet::new(3, 4, 3, &[5, 4], 0.8, 0.5, 2.0, 11);
        let a = example(0);
        let mut b = a.clone();
        b.label = 2;
        assert_eq!(model.forward(&a), model.forward(&b));
        let (_, ga) = model.loss_and_gradients(&a);
        let (_, gb) = model.loss_and_gradients(&b);
        assert_ne!(ga.flat(), gb.flat());
        assert!(ga
            .flat()
            .iter()
            .zip(gb.flat())
            .any(|(left, right)| left.signum() != right.signum()));
    }

    #[test]
    fn cloned_treatment_and_ceiling_have_identical_pretraining_forward() {
        let initial = SharedTemporalNet::new(3, 4, 3, &[5, 5], 0.8, 0.5, 2.0, 13);
        let treatment = initial.clone();
        let ceiling = initial.clone();
        assert_eq!(treatment.forward(&example(0)), ceiling.forward(&example(0)));
        assert_eq!(
            treatment.parameter_fingerprint(),
            ceiling.parameter_fingerprint()
        );
    }

    #[test]
    fn adam_clips_global_norm_and_records_steps() {
        let mut model = SharedTemporalNet::new(3, 4, 3, &[4], 0.8, 0.5, 2.0, 17);
        let (_, mut gradients) = model.loss_and_gradients(&example(1));
        gradients.scale(1_000.0);
        let diagnostics = Adam::new(&model).update(&mut model, &gradients);
        assert!(diagnostics.unclipped_global_norm > GRADIENT_CLIP_NORM);
        assert!(diagnostics.clipped_global_norm <= GRADIENT_CLIP_NORM + 1e-4);
        assert!(diagnostics.layer_step_rms.iter().all(|value| *value > 0.0));
    }

    #[test]
    fn depth_one_bptt_overfits_easy_fixture() {
        let train: Vec<_> = (0..3)
            .flat_map(|label| std::iter::repeat_with(move || example(label)).take(8))
            .collect();
        let mut model = SharedTemporalNet::new(3, 4, 3, &[8], 0.7, 0.4, 3.0, 23);
        train_bptt(&mut model, &train, 30);
        assert!(model.accuracy(&train) > 0.95);
    }

    #[test]
    fn gradient_addition_covers_all_groups() {
        let model = SharedTemporalNet::new(3, 4, 3, &[4, 3], 0.8, 0.5, 2.0, 29);
        let (_, mut a) = model.loss_and_gradients(&example(0));
        let (_, b) = model.loss_and_gradients(&example(1));
        let before = a.flat();
        let other = b.flat();
        a.add_assign(&b);
        for ((sum, left), right) in a.flat().iter().zip(before).zip(other) {
            assert!((*sum - (left + right)).abs() < 1e-6);
        }
    }

    #[test]
    fn learned_feedback_moves_toward_downstream_transport() {
        let model = SharedTemporalNet::new(3, 4, 3, &[4, 3], 0.8, 0.5, 2.0, 31);
        let mut feedback = random_feedback(&model, 31);
        let before = feedback.clone();
        align_feedback(&model, &mut feedback, 0.1);
        assert_ne!(feedback, before);
    }

    #[test]
    fn feedback_modulator_rms_is_live_and_read_only() {
        let model = SharedTemporalNet::new(3, 4, 3, &[4, 3], 0.8, 0.5, 2.0, 37);
        let feedback = random_feedback(&model, 37);
        let examples = vec![example(0), example(1), example(2)];
        let before = model.parameter_fingerprint();
        let values = model.feedback_modulator_rms(&examples, &feedback);
        assert_eq!(values.len(), 2);
        assert!(values.iter().all(|value| value.is_finite() && *value > 0.0));
        assert_eq!(before, model.parameter_fingerprint());
    }

    #[allow(clippy::too_many_arguments)]
    fn fixed_input_final_rate(
        input_trajectory: &[f32],
        weights: &[f32],
        bias: f32,
        timesteps: usize,
        n_in: usize,
        alpha: f32,
        threshold: f32,
        surrogate_beta: f32,
    ) -> f32 {
        let mut membrane = 0.0f32;
        let mut rate = 0.0f32;
        for t in 0..timesteps {
            let mut q = alpha * membrane + bias;
            for input in 0..n_in {
                q += weights[input] * input_trajectory[t * n_in + input];
            }
            let spike = sigmoid(surrogate_beta * (q - threshold));
            membrane = q - threshold * spike;
            rate += spike / timesteps as f32;
        }
        rate
    }
}
