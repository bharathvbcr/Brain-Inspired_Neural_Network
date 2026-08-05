//! Local, budgeted synaptic pruning (U22).
//!
//! Strategies use only synapse-local state: weight magnitude, last eligibility
//! touch, eligibility magnitude, or a seeded random control. Pruning preserves
//! topology/index stability by zeroing weights in both engine views; this keeps
//! CSR/CSC edge identities reproducible for matched-sparsity ablations.

use binn_core::Rng;
use binn_engine::Engine;

/// Preregistered local pruning rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PruningStrategy {
    /// Remove the smallest absolute weights first.
    Magnitude,
    /// Remove the least recently eligibility-touched synapses first.
    Age,
    /// Remove the smallest absolute eligibility traces first.
    Eligibility,
    /// Seeded matched-sparsity negative control.
    Random,
}

impl PruningStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Magnitude => "magnitude",
            Self::Age => "age",
            Self::Eligibility => "eligibility",
            Self::Random => "random",
        }
    }
}

/// Result of one exact-budget pruning operation.
#[derive(Clone, Debug, PartialEq)]
pub struct PruneReport {
    pub strategy: PruningStrategy,
    pub requested_sparsity: f32,
    pub active_before: usize,
    pub pruned: usize,
    pub active_after: usize,
    pub realized_sparsity: f32,
}

/// Zero exactly the requested fraction of currently active synapses.
pub fn prune(
    engine: &mut Engine,
    strategy: PruningStrategy,
    target_sparsity: f32,
    seed: u64,
) -> PruneReport {
    assert!(
        target_sparsity.is_finite() && (0.0..=1.0).contains(&target_sparsity),
        "target_sparsity must be in [0, 1]"
    );
    assert_eq!(engine.edge_w.len(), engine.syn.len());
    let mut candidates: Vec<usize> = engine
        .edge_w
        .iter()
        .enumerate()
        .filter_map(|(i, &weight)| (weight != 0.0).then_some(i))
        .collect();
    let active_before = candidates.len();
    let requested = ((active_before as f64) * target_sparsity as f64).round() as usize;
    let requested = requested.min(active_before);

    match strategy {
        PruningStrategy::Magnitude => candidates.sort_by(|&a, &b| {
            engine.edge_w[a]
                .abs()
                .partial_cmp(&engine.edge_w[b].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cmp(&b))
        }),
        PruningStrategy::Age => candidates.sort_by_key(|&i| {
            (
                engine.syn.get(i).expect("aligned synapse").last_elig_update,
                i,
            )
        }),
        PruningStrategy::Eligibility => candidates.sort_by(|&a, &b| {
            let ea = engine
                .syn
                .get(a)
                .expect("aligned synapse")
                .eligibility
                .abs();
            let eb = engine
                .syn
                .get(b)
                .expect("aligned synapse")
                .eligibility
                .abs();
            ea.partial_cmp(&eb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cmp(&b))
        }),
        PruningStrategy::Random => {
            let mut rng = Rng::new(seed ^ 0xA11C_E5E0_0000_0001);
            for i in 0..candidates.len() {
                let j = i + rng.gen_index(candidates.len() - i);
                candidates.swap(i, j);
            }
        }
    }

    for &index in candidates.iter().take(requested) {
        engine.edge_w[index] = 0.0;
        let synapse = engine.syn.get_mut(index).expect("aligned synapse");
        synapse.weight = 0.0;
        synapse.eligibility = 0.0;
    }
    let active_after = active_before - requested;
    PruneReport {
        strategy,
        requested_sparsity: target_sparsity,
        active_before,
        pruned: requested,
        active_after,
        realized_sparsity: if active_before == 0 {
            0.0
        } else {
            requested as f32 / active_before as f32
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_core::Csr;

    fn engine() -> Engine {
        let conn = Csr::from_adjacency(&[vec![1, 2, 3], vec![0, 2], vec![0, 1], vec![0]]);
        let weights = vec![0.8, 0.1, 0.4, 0.7, 0.2, 0.6, 0.3, 0.9];
        let mut engine = Engine::with_cells(4);
        engine.set_connectivity(conn, weights);
        for (i, synapse) in engine.syn.as_mut_slice().iter_mut().enumerate() {
            synapse.eligibility = i as f32 / 10.0;
            synapse.last_elig_update = i as u64;
        }
        engine
    }

    #[test]
    fn every_strategy_hits_the_same_exact_budget() {
        for strategy in [
            PruningStrategy::Magnitude,
            PruningStrategy::Age,
            PruningStrategy::Eligibility,
            PruningStrategy::Random,
        ] {
            let mut engine = engine();
            let report = prune(&mut engine, strategy, 0.5, 7);
            assert_eq!(report.active_before, 8);
            assert_eq!(report.pruned, 4);
            assert_eq!(report.active_after, 4);
            assert_eq!(
                engine
                    .edge_w
                    .iter()
                    .filter(|&&weight| weight != 0.0)
                    .count(),
                4
            );
            assert!(engine
                .edge_w
                .iter()
                .zip(engine.syn.as_slice())
                .all(|(edge, synapse)| *edge == synapse.weight));
        }
    }

    #[test]
    fn magnitude_removes_smallest_weights() {
        let mut engine = engine();
        prune(&mut engine, PruningStrategy::Magnitude, 0.25, 0);
        assert_eq!(engine.edge_w[1], 0.0);
        assert_eq!(engine.edge_w[4], 0.0);
    }

    #[test]
    fn random_control_is_seeded() {
        let mut a = engine();
        let mut b = engine();
        prune(&mut a, PruningStrategy::Random, 0.5, 99);
        prune(&mut b, PruningStrategy::Random, 0.5, 99);
        assert_eq!(a.edge_w, b.edge_w);
    }
}
