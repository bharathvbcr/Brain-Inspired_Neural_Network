//! Metrics (U12): work-per-accuracy, forgetting, sparsity, overlap.
//!
//! Efficiency accounting follows v7 F5 / v8 U12: **synaptic events × fan-out
//! including per-event overhead**, never a linear-in-activity estimate.

use crate::encoder::CellId;

/// Disjoint operation counters for auditable efficiency accounting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkCounters {
    pub source_spikes: u64,
    pub synaptic_deliveries: u64,
    pub cell_updates: u64,
    pub plasticity_updates: u64,
}

/// Measured or calibrated cost per operation in a disclosed common unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkCosts {
    pub routing: f64,
    pub delivery: f64,
    pub cell_update: f64,
    pub plasticity_update: f64,
}

impl WorkCosts {
    /// Unit-cost reference; real reports should additionally disclose wall time.
    pub const fn unit() -> Self {
        Self {
            routing: 1.0,
            delivery: 1.0,
            cell_update: 1.0,
            plasticity_update: 1.0,
        }
    }
}

/// F5 activity≠compute disclosure: event work vs naive activity-scaled proxy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityComputeAccount {
    /// Modeled event work from disjoint counters.
    pub event_work: f64,
    /// Naive proxy `n_cells × activity_sparsity` (active-cell count as work).
    pub naive_activity_work: f64,
    /// `event_work / max(naive_activity_work, ε)` — ≫1 means activity understates compute.
    pub work_vs_activity_ratio: f64,
    /// Activity sparsity used for the naive proxy.
    pub activity_sparsity: f32,
    /// Population size used for the naive proxy.
    pub n_cells: usize,
}

/// Stateless metric helpers used by the lab harness and unit tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct Metrics;

impl Metrics {
    /// Auditable modeled work from disjoint counters.
    pub fn total_work(counts: WorkCounters, costs: WorkCosts) -> f64 {
        for cost in [
            costs.routing,
            costs.delivery,
            costs.cell_update,
            costs.plasticity_update,
        ] {
            assert!(
                cost.is_finite() && cost >= 0.0,
                "costs must be finite and non-negative"
            );
        }
        counts.source_spikes as f64 * costs.routing
            + counts.synaptic_deliveries as f64 * costs.delivery
            + counts.cell_updates as f64 * costs.cell_update
            + counts.plasticity_updates as f64 * costs.plasticity_update
    }

    /// Modeled work per unit accuracy. Reports must accompany this proxy with
    /// measured wall-clock time and peak memory rather than calling it energy.
    pub fn work_per_accuracy(counts: WorkCounters, costs: WorkCosts, accuracy: f64) -> f64 {
        assert!(
            accuracy.is_finite() && accuracy > 0.0,
            "accuracy must be > 0"
        );
        Self::total_work(counts, costs) / accuracy
    }

    /// **Incorrect** linear-in-activity estimate — kept only so tests / reports
    /// can contrast F5-honest work against the naive figure.
    #[inline]
    pub fn naive_linear_activity_work(dense_ops: f64, activity_fraction: f64) -> f64 {
        assert!((0.0..=1.0).contains(&activity_fraction));
        dense_ops * activity_fraction
    }

    /// F5: contrast event-counter work against a naive `n_cells × activity` proxy.
    ///
    /// Engineering accounting only — not a biology claim and not a G2 reopen.
    pub fn activity_compute_account(
        counts: WorkCounters,
        costs: WorkCosts,
        n_cells: usize,
        activity_sparsity: f32,
    ) -> ActivityComputeAccount {
        assert!(
            (0.0..=1.0).contains(&activity_sparsity),
            "activity_sparsity must be in [0, 1]"
        );
        let event_work = Self::total_work(counts, costs);
        let naive_activity_work = n_cells as f64 * f64::from(activity_sparsity);
        let work_vs_activity_ratio = event_work / naive_activity_work.max(1e-12);
        ActivityComputeAccount {
            event_work,
            naive_activity_work,
            work_vs_activity_ratio,
            activity_sparsity,
            n_cells,
        }
    }

    /// Relative forgetting: `(acc_initial − acc_after) / acc_initial`.
    ///
    /// Returns `0` when `acc_initial ≤ 0`. Clamped to `[0, 1]`.
    pub fn forgetting(acc_initial: f64, acc_after: f64) -> f64 {
        if acc_initial <= 0.0 {
            return 0.0;
        }
        ((acc_initial - acc_after) / acc_initial).clamp(0.0, 1.0)
    }

    /// Activity sparsity `active / population` (GC7 field name: activity_sparsity).
    ///
    /// Returns `0` when `population == 0`.
    pub fn sparsity(active: usize, population: usize) -> f32 {
        if population == 0 {
            return 0.0;
        }
        assert!(active <= population, "active exceeds population");
        active as f32 / population as f32
    }

    /// Jaccard overlap of two assemblies (cell-id sets): `|A ∩ B| / |A ∪ B|`.
    ///
    /// Empty∪empty → `0`. Duplicate ids within a side are ignored.
    pub fn overlap(a: &[CellId], b: &[CellId]) -> f32 {
        let mut aa: Vec<CellId> = a.to_vec();
        let mut bb: Vec<CellId> = b.to_vec();
        aa.sort_unstable();
        aa.dedup();
        bb.sort_unstable();
        bb.dedup();
        if aa.is_empty() && bb.is_empty() {
            return 0.0;
        }
        let mut i = 0usize;
        let mut j = 0usize;
        let mut inter = 0usize;
        while i < aa.len() && j < bb.len() {
            match aa[i].cmp(&bb[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    inter += 1;
                    i += 1;
                    j += 1;
                }
            }
        }
        let union = aa.len() + bb.len() - inter;
        inter as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_per_accuracy_includes_per_event_overhead() {
        let counts = WorkCounters {
            source_spikes: 1_000,
            synaptic_deliveries: 10_000,
            cell_updates: 10_000,
            plasticity_updates: 500,
        };
        let costs = WorkCosts {
            routing: 4.0,
            delivery: 1.0,
            cell_update: 0.5,
            plasticity_update: 2.0,
        };
        let acc = 0.5;
        let honest = Metrics::work_per_accuracy(counts, costs, acc);
        let expected = (4_000.0 + 10_000.0 + 5_000.0 + 1_000.0) / acc;
        assert!((honest - expected).abs() < 1e-9);

        let activity = 0.02;
        let dense_ops = counts.synaptic_deliveries as f64 / activity;
        let naive = Metrics::naive_linear_activity_work(dense_ops, activity);
        let honest_work = Metrics::total_work(counts, costs);
        assert!((naive - counts.synaptic_deliveries as f64).abs() < 1e-9);
        assert!(honest_work > naive);
    }

    #[test]
    fn f5_activity_compute_account_shows_event_overhead() {
        let counts = WorkCounters {
            source_spikes: 100,
            synaptic_deliveries: 800,
            cell_updates: 800,
            plasticity_updates: 0,
        };
        let acct = Metrics::activity_compute_account(counts, WorkCosts::unit(), 10_000, 0.02);
        // naive ≈ 10000 * 0.02 (f32→f64 may not be bit-exact 200)
        assert!((acct.naive_activity_work - 200.0).abs() < 1e-3);
        // event = 100 + 800 + 800 = 1700
        assert!((acct.event_work - 1700.0).abs() < 1e-9);
        assert!(acct.work_vs_activity_ratio > 8.0);
    }

    #[test]
    fn forgetting_and_sparsity_and_overlap() {
        assert!((Metrics::forgetting(0.9, 0.6) - (0.3 / 0.9)).abs() < 1e-12);
        assert_eq!(Metrics::forgetting(0.0, 0.5), 0.0);
        assert!((Metrics::sparsity(2, 100) - 0.02).abs() < 1e-6);

        let a = [1u32, 2, 3, 3];
        let b = [2u32, 3, 4];
        // ∩={2,3} ∪={1,2,3,4} → 0.5
        assert!((Metrics::overlap(&a, &b) - 0.5).abs() < 1e-6);
        assert_eq!(Metrics::overlap(&[], &[]), 0.0);
    }
}
