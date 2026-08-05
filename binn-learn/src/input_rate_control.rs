//! Linear input-rate shortcut control and paired equivalence statistics.
//!
//! This module is deliberately independent of SHD hidden-network code. It
//! consumes caller-supplied dense frames, computes `sum_t spikes / T`, and
//! trains only a multiclass linear softmax readout.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;

use crate::shd_eprop_baseline::ShdExample;

/// Stable protocol label.
pub const INPUT_RATE_CONTROL_LABEL: &str = "LINEAR_INPUT_RATE_CONTROL";

/// Deterministic SGD configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputRateConfig {
    pub n_in: usize,
    pub n_classes: usize,
    pub lr: f32,
    pub epochs: usize,
}

/// Evaluation products retained for paired bootstrap and degeneracy gates.
#[derive(Clone, Debug, PartialEq)]
pub struct InputRateReport {
    pub label: &'static str,
    pub accuracy: f32,
    pub loss: f32,
    pub predictions: Vec<u32>,
    pub n_distinct_predicted: usize,
    pub majority_pred_frac: f32,
    pub no_test_update: bool,
}

/// Linear softmax readout over raw input rates.
#[derive(Clone, Debug)]
pub struct InputRateClassifier {
    cfg: InputRateConfig,
    weights: Vec<f32>,
    bias: Vec<f32>,
}

impl InputRateClassifier {
    pub fn new(cfg: InputRateConfig, seed: u64) -> Self {
        assert!(cfg.n_in > 0);
        assert!(cfg.n_classes >= 2);
        assert!(cfg.lr > 0.0);
        let mut rng = Rng::new(seed ^ 0x1A90_0A7E_0000_0001);
        let scale = 0.2 / (cfg.n_in as f32).sqrt();
        let weights = (0..cfg.n_classes * cfg.n_in)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * scale)
            .collect();
        Self {
            cfg,
            weights,
            bias: vec![0.0; cfg.n_classes],
        }
    }

    pub fn train(&mut self, train: &[ShdExample]) {
        assert!(!train.is_empty());
        for _ in 0..self.cfg.epochs {
            for ex in train {
                self.step(ex);
            }
        }
    }

    pub fn evaluate(&self, test: &[ShdExample]) -> InputRateReport {
        assert!(!test.is_empty());
        let before = self.parameter_fingerprint();
        let mut predictions = Vec::with_capacity(test.len());
        let mut loss = 0.0;
        let mut correct = 0usize;
        let mut counts = vec![0usize; self.cfg.n_classes];
        for ex in test {
            let features = rate_features(ex, self.cfg.n_in);
            let logits = self.logits(&features);
            let probs = softmax(&logits);
            let pred = argmax(&probs);
            predictions.push(pred as u32);
            counts[pred] += 1;
            correct += usize::from(pred == ex.label as usize);
            loss += -probs[ex.label as usize].max(1e-12).ln();
        }
        let after = self.parameter_fingerprint();
        let n = test.len() as f32;
        InputRateReport {
            label: INPUT_RATE_CONTROL_LABEL,
            accuracy: correct as f32 / n,
            loss: loss / n,
            predictions,
            n_distinct_predicted: counts.iter().filter(|&&count| count > 0).count(),
            majority_pred_frac: counts.iter().copied().max().unwrap_or(0) as f32 / n,
            no_test_update: before == after,
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> InputRateReport {
        self.train(train);
        self.evaluate(test)
    }

    pub fn parameter_fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for value in self.weights.iter().chain(&self.bias) {
            hash ^= value.to_bits() as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        hash
    }

    fn step(&mut self, ex: &ShdExample) {
        let features = rate_features(ex, self.cfg.n_in);
        let logits = self.logits(&features);
        let mut delta = softmax(&logits);
        delta[ex.label as usize] -= 1.0;
        for class in 0..self.cfg.n_classes {
            for input in 0..self.cfg.n_in {
                self.weights[class * self.cfg.n_in + input] -=
                    self.cfg.lr * delta[class] * features[input];
            }
            self.bias[class] -= self.cfg.lr * delta[class];
        }
    }

    fn logits(&self, features: &[f32]) -> Vec<f32> {
        let mut out = self.bias.clone();
        for class in 0..self.cfg.n_classes {
            for input in 0..self.cfg.n_in {
                out[class] += self.weights[class * self.cfg.n_in + input] * features[input];
            }
        }
        out
    }
}

/// Per-seed paired predictions retained by the confirmatory runner.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedPredictions {
    pub labels: Vec<u32>,
    pub input_only: Vec<u32>,
    pub hidden: Vec<u32>,
}

impl PairedPredictions {
    pub fn validate(&self) -> Result<(), String> {
        if self.labels.is_empty() {
            return Err("paired predictions are empty".into());
        }
        if self.labels.len() != self.input_only.len() || self.labels.len() != self.hidden.len() {
            return Err("paired prediction lengths differ".into());
        }
        Ok(())
    }

    pub fn hidden_minus_input(&self) -> f32 {
        let n = self.labels.len() as f32;
        let hidden = self
            .labels
            .iter()
            .zip(&self.hidden)
            .filter(|(label, pred)| label == pred)
            .count() as f32
            / n;
        let input = self
            .labels
            .iter()
            .zip(&self.input_only)
            .filter(|(label, pred)| label == pred)
            .count() as f32
            / n;
        hidden - input
    }
}

/// Fixed-seed hierarchical bootstrap summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EquivalenceSummary {
    pub mean_hidden_minus_input: f32,
    pub lower_95: f32,
    pub upper_95: f32,
    pub equivalent: bool,
    pub hidden_clearly_better: bool,
}

/// Resample seeds, then examples within each sampled seed.
pub fn hierarchical_bootstrap(
    pairs: &[PairedPredictions],
    bootstrap_seed: u64,
    draws: usize,
) -> Result<EquivalenceSummary, String> {
    if pairs.is_empty() || draws < 100 {
        return Err("hierarchical bootstrap needs seeds and at least 100 draws".into());
    }
    for pair in pairs {
        pair.validate()?;
    }
    let mean_hidden_minus_input = pairs
        .iter()
        .map(PairedPredictions::hidden_minus_input)
        .sum::<f32>()
        / pairs.len() as f32;
    let mut rng = Rng::new(bootstrap_seed ^ 0xB007_57A9_0000_0001);
    let mut samples = Vec::with_capacity(draws);
    for _ in 0..draws {
        let mut difference = 0.0f32;
        for _ in 0..pairs.len() {
            let pair = &pairs[rng.gen_index(pairs.len())];
            let mut hidden_correct = 0usize;
            let mut input_correct = 0usize;
            for _ in 0..pair.labels.len() {
                let index = rng.gen_index(pair.labels.len());
                hidden_correct += usize::from(pair.hidden[index] == pair.labels[index]);
                input_correct += usize::from(pair.input_only[index] == pair.labels[index]);
            }
            difference += (hidden_correct as f32 - input_correct as f32) / pair.labels.len() as f32;
        }
        samples.push(difference / pairs.len() as f32);
    }
    samples.sort_by(f32::total_cmp);
    let lower_95 = percentile(&samples, 0.025);
    let upper_95 = percentile(&samples, 0.975);
    Ok(EquivalenceSummary {
        mean_hidden_minus_input,
        lower_95,
        upper_95,
        equivalent: mean_hidden_minus_input < 0.02 && upper_95 < 0.05,
        hidden_clearly_better: lower_95 > 0.05,
    })
}

fn rate_features(ex: &ShdExample, n_in: usize) -> Vec<f32> {
    assert_eq!(ex.n_in, n_in);
    let mut features = vec![0.0f32; n_in];
    for frame in ex.frames.chunks_exact(n_in) {
        for (feature, &spike) in features.iter_mut().zip(frame) {
            *feature += spike / ex.t as f32;
        }
    }
    features
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut out: Vec<f32> = logits.iter().map(|&value| (value - max).exp()).collect();
    let sum = out.iter().sum::<f32>().max(1e-12);
    for value in &mut out {
        *value /= sum;
    }
    out
}

fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn percentile(sorted: &[f32], p: f32) -> f32 {
    let index = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn examples(n: usize) -> Vec<ShdExample> {
        (0..n)
            .map(|index| {
                let label = (index % 3) as u32;
                let mut frames = vec![0.0f32; 6 * 9];
                for t in 0..6 {
                    frames[t * 9 + label as usize] = 1.0;
                }
                ShdExample {
                    frames,
                    t: 6,
                    n_in: 9,
                    label,
                }
            })
            .collect()
    }

    #[test]
    fn input_rate_control_learns_and_does_not_mutate_on_test() {
        let train = examples(60);
        let test = examples(30);
        let mut model = InputRateClassifier::new(
            InputRateConfig {
                n_in: 9,
                n_classes: 3,
                lr: 0.05,
                epochs: 10,
            },
            7,
        );
        let report = model.train_and_evaluate(&train, &test);
        assert!(report.accuracy > 0.95);
        assert!(report.no_test_update);
        assert_eq!(report.n_distinct_predicted, 3);
    }

    #[test]
    fn hierarchical_equivalence_gate_uses_paired_predictions() {
        let pair = PairedPredictions {
            labels: (0..200).map(|i| (i % 4) as u32).collect(),
            input_only: (0..200).map(|i| (i % 4) as u32).collect(),
            hidden: (0..200).map(|i| (i % 4) as u32).collect(),
        };
        let result = hierarchical_bootstrap(&vec![pair; 10], 99, 2_000).unwrap();
        assert_eq!(result.mean_hidden_minus_input, 0.0);
        assert!(result.equivalent);
        assert!(!result.hidden_clearly_better);
    }
}
