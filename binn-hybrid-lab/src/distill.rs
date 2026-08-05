use binn_hybrid_learn::{
    CreditFeatures, CreditGranularity, CreditHeadArtifact, CREDIT_FEATURE_COUNT,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistillationConfig {
    pub epochs: usize,
    pub learning_rate: f32,
    pub l2: f32,
    pub output_scale: f32,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            epochs: 40,
            learning_rate: 0.02,
            l2: 1e-5,
            output_scale: 0.1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistillationExample {
    pub features: CreditFeatures,
    pub target_delta: f32,
}

pub fn distill_linear_head(
    examples: &[DistillationExample],
    config: DistillationConfig,
    topology_signature: u64,
    granularity: CreditGranularity,
    teacher_protocol_hash: u64,
    training_seed_hash: u64,
) -> Result<CreditHeadArtifact, binn_hybrid_learn::ArtifactError> {
    let mut coefficients = [0.0f32; CREDIT_FEATURE_COUNT];
    let mut bias = 0.0f32;
    if !examples.is_empty() {
        for _ in 0..config.epochs {
            for example in examples {
                let values = example.features.values();
                let raw = coefficients
                    .iter()
                    .zip(values)
                    .fold(bias, |sum, (coefficient, value)| sum + coefficient * value);
                let tanh = raw.tanh();
                let normalized_target =
                    (example.target_delta / config.output_scale).clamp(-0.999, 0.999);
                let error = (tanh - normalized_target).clamp(-1.0, 1.0);
                let derivative = 1.0 - tanh * tanh;
                for (coefficient, value) in coefficients.iter_mut().zip(values) {
                    let gradient = error * derivative * value + config.l2 * *coefficient;
                    *coefficient -= config.learning_rate * gradient;
                }
                bias -= config.learning_rate * error * derivative;
            }
        }
    }
    CreditHeadArtifact::new(
        topology_signature,
        granularity,
        teacher_protocol_hash,
        training_seed_hash,
        coefficients,
        bias,
        config.output_scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_hybrid_learn::HybridLearner;

    #[test]
    fn distilled_head_reduces_simple_imitation_error() {
        let examples = (-10..=10)
            .map(|value| {
                let x = value as f32 / 10.0;
                DistillationExample {
                    features: CreditFeatures {
                        eligibility: x,
                        ..CreditFeatures::default()
                    },
                    target_delta: x * 0.05,
                }
            })
            .collect::<Vec<_>>();
        let artifact = distill_linear_head(
            &examples,
            DistillationConfig {
                epochs: 200,
                ..DistillationConfig::default()
            },
            9,
            CreditGranularity::PerSynapse,
            1,
            2,
        )
        .expect("distill");
        let learner = HybridLearner::load(artifact, 9).expect("load");
        let predicted = learner
            .predict_delta(CreditFeatures {
                eligibility: 0.8,
                ..CreditFeatures::default()
            })
            .expect("predict");
        assert!((predicted - 0.04).abs() < 0.02);
    }
}
