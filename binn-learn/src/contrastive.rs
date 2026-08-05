//! Spiking contrastive wake-sleep learning rule.

use crate::eligibility::decay;
use binn_core::time::Tick;
use binn_engine::Synapse;

/// Contrastive wake-sleep learner:
/// - Wake phase: Hebbian update (+eta * e * M)
/// - Sleep phase: Anti-Hebbian update (-eta * e * M)
#[derive(Clone, Debug)]
pub struct ContrastiveWakeSleepLearner {
    pub eta: f32,
    pub lambda: f32,
    pub tau_e: f32,
}

impl ContrastiveWakeSleepLearner {
    pub fn new(eta: f32, lambda: f32, tau_e: f32) -> Self {
        assert!(eta > 0.0);
        assert!(tau_e > 0.0);
        Self { eta, lambda, tau_e }
    }

    pub fn update_wake(&self, syn: &mut Synapse, modulator: f32, t: Tick) {
        let dt = (t - syn.last_elig_update) as f32;
        syn.eligibility = decay(syn.eligibility, dt, self.tau_e);
        syn.last_elig_update = t;
        let dw = self.eta * syn.eligibility * modulator - self.lambda * syn.weight;
        syn.weight += dw;
    }

    pub fn update_sleep(&self, syn: &mut Synapse, modulator: f32, t: Tick) {
        let dt = (t - syn.last_elig_update) as f32;
        syn.eligibility = decay(syn.eligibility, dt, self.tau_e);
        syn.last_elig_update = t;
        // Anti-Hebbian update during sleep phase
        let dw = -self.eta * syn.eligibility * modulator - self.lambda * syn.weight;
        syn.weight += dw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrastive_wake_sleep() {
        let learner = ContrastiveWakeSleepLearner::new(0.1, 0.0, 20.0);
        let mut syn = Synapse::new(1.0, 1);
        syn.eligibility = 0.5;
        syn.last_elig_update = 0;
        learner.update_wake(&mut syn, 1.0, 0);
        assert!((syn.weight - 1.05).abs() < 1e-5);
        learner.update_sleep(&mut syn, 1.0, 0);
        assert!((syn.weight - 1.00).abs() < 1e-5);
    }
}
