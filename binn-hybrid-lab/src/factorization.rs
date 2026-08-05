use binn_hybrid_learn::CreditFeatures;

use crate::teacher::TeacherTargets;

#[derive(Clone, Debug, PartialEq)]
pub struct FactorizationAudit {
    pub existing_post_deltas: Vec<f32>,
    pub oracle_post_deltas: Vec<f32>,
    pub direct_edge_deltas: Vec<f32>,
    pub existing_cosine: f32,
    pub oracle_cosine: f32,
    pub existing_mse: f32,
    pub oracle_mse: f32,
    pub existing_sign_agreement: f32,
    pub oracle_sign_agreement: f32,
}

pub fn factorization_audit(targets: &TeacherTargets, edge_posts: &[usize]) -> FactorizationAudit {
    assert_eq!(targets.edge_deltas.len(), targets.features.len());
    assert_eq!(targets.edge_deltas.len(), edge_posts.len());
    let n_posts = targets.post_credits.len();
    let existing_post_deltas = targets
        .features
        .iter()
        .zip(edge_posts)
        .map(|(features, &post)| features.eligibility * targets.post_credits[post])
        .collect::<Vec<_>>();

    let mut numerators = vec![0.0f32; n_posts];
    let mut denominators = vec![0.0f32; n_posts];
    for ((features, &target), &post) in targets
        .features
        .iter()
        .zip(&targets.edge_deltas)
        .zip(edge_posts)
    {
        numerators[post] += features.eligibility * target;
        denominators[post] += features.eligibility * features.eligibility;
    }
    let oracle_credit = numerators
        .into_iter()
        .zip(denominators)
        .map(|(numerator, denominator)| {
            if denominator > 1e-12 {
                numerator / denominator
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();
    let oracle_post_deltas = targets
        .features
        .iter()
        .zip(edge_posts)
        .map(|(features, &post)| features.eligibility * oracle_credit[post])
        .collect::<Vec<_>>();

    FactorizationAudit {
        existing_cosine: cosine(&existing_post_deltas, &targets.edge_deltas),
        oracle_cosine: cosine(&oracle_post_deltas, &targets.edge_deltas),
        existing_mse: mse(&existing_post_deltas, &targets.edge_deltas),
        oracle_mse: mse(&oracle_post_deltas, &targets.edge_deltas),
        existing_sign_agreement: sign_agreement(&existing_post_deltas, &targets.edge_deltas),
        oracle_sign_agreement: sign_agreement(&oracle_post_deltas, &targets.edge_deltas),
        existing_post_deltas,
        oracle_post_deltas,
        direct_edge_deltas: targets.edge_deltas.clone(),
    }
}

pub fn attach_reward(features: &mut [CreditFeatures], reward: f32) {
    for feature in features {
        feature.broadcast_reward = Some(reward);
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    let aa = a.iter().map(|value| value * value).sum::<f32>().sqrt();
    let bb = b.iter().map(|value| value * value).sum::<f32>().sqrt();
    if aa <= 1e-12 || bb <= 1e-12 {
        0.0
    } else {
        dot / (aa * bb)
    }
}

fn mse(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() {
        return 0.0;
    }
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            let error = x - y;
            error * error
        })
        .sum::<f32>()
        / a.len() as f32
}

fn sign_agreement(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() {
        return 0.0;
    }
    a.iter()
        .zip(b)
        .filter(|(x, y)| x.signum() == y.signum())
        .count() as f32
        / a.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn least_squares_oracle_is_not_worse_than_existing_credit() {
        let targets = TeacherTargets {
            loss: 1.0,
            edge_deltas: vec![1.0, -0.5, 0.25],
            post_credits: vec![0.0, 0.1],
            features: vec![
                CreditFeatures {
                    eligibility: 1.0,
                    ..CreditFeatures::default()
                },
                CreditFeatures {
                    eligibility: 0.5,
                    ..CreditFeatures::default()
                },
                CreditFeatures {
                    eligibility: -0.25,
                    ..CreditFeatures::default()
                },
            ],
        };
        let audit = factorization_audit(&targets, &[1, 1, 1]);
        assert!(audit.oracle_mse <= audit.existing_mse);
        let residual = targets
            .features
            .iter()
            .zip(&targets.edge_deltas)
            .zip(&audit.oracle_post_deltas)
            .map(|((features, target), reconstructed)| {
                features.eligibility * (target - reconstructed)
            })
            .sum::<f32>();
        assert!(
            residual.abs() < 1e-6,
            "least-squares normal-equation residual={residual}"
        );
    }
}
