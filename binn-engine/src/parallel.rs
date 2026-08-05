//! Deterministic graph partition plan for parallel delta stepping (U18 / F1).
//!
//! Same-tick cell jobs that do not share membrane state can run in parallel.
//! Spike reset / fan-out still serializes across ticks (F1 barrier). Thin ticks
//! (few distinct target cells) stay sequential to avoid rayon overhead.

use binn_core::Csr;

/// Minimum distinct target cells in a delta bucket before rayon is used.
///
/// Below this threshold, in-place sequential integration is both faster and
/// observationally identical. Empirically chosen against the U18 microbench
/// where always-on rayon was slower than sequential on sparse event streams.
pub const PARALLEL_CELL_THRESHOLD: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionPlan {
    owner: Vec<usize>,
    n_partitions: usize,
    cut_edges: usize,
}

impl PartitionPlan {
    /// Greedy degree-balanced assignment in stable cell order.
    pub fn degree_balanced(conn: &Csr, n_partitions: usize) -> Self {
        assert!(n_partitions > 0);
        let n_partitions = n_partitions.min(conn.nrows().max(1));
        let mut owner = vec![0usize; conn.nrows()];
        let mut loads = vec![0usize; n_partitions];
        for (cell, slot) in owner.iter_mut().enumerate() {
            let partition = loads
                .iter()
                .enumerate()
                .min_by_key(|&(partition, load)| (*load, partition))
                .map(|(partition, _)| partition)
                .unwrap_or(0);
            *slot = partition;
            loads[partition] += conn.row_cols(cell).len().max(1);
        }
        let cut_edges = conn
            .edges()
            .filter(|&(pre, post)| owner[pre as usize] != owner[post as usize])
            .count();
        Self {
            owner,
            n_partitions,
            cut_edges,
        }
    }

    #[inline]
    pub fn owner(&self, cell: u32) -> usize {
        self.owner[cell as usize]
    }

    #[inline]
    pub fn n_partitions(&self) -> usize {
        self.n_partitions
    }

    #[inline]
    pub fn cut_edges(&self) -> usize {
        self.cut_edges
    }

    #[inline]
    pub fn n_cells(&self) -> usize {
        self.owner.len()
    }
}

/// Per-run F1 characterization of same-tick parallelism headroom.
///
/// Resets / cross-tick fan-out remain sequential barriers; this only describes
/// how wide each delta bucket is (distinct target cells at one tick).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParallelismProfile {
    /// Delta buckets (ticks that had at least one event).
    pub ticks_with_events: usize,
    /// Buckets integrated with rayon (`distinct_cells >= PARALLEL_CELL_THRESHOLD`).
    pub parallel_ticks: usize,
    /// Buckets kept sequential (thin width).
    pub sequential_ticks: usize,
    /// Sum of distinct target cells across buckets.
    pub total_cell_jobs: usize,
    /// Max distinct target cells in any single bucket.
    pub max_width: usize,
    /// Sum of event counts across buckets.
    pub total_events: usize,
}

impl ParallelismProfile {
    /// Mean distinct cells per eventful tick (`0` when empty).
    #[inline]
    pub fn mean_width(&self) -> f64 {
        if self.ticks_with_events == 0 {
            0.0
        } else {
            self.total_cell_jobs as f64 / self.ticks_with_events as f64
        }
    }

    /// Fraction of eventful ticks that used the parallel path.
    #[inline]
    pub fn parallel_tick_fraction(&self) -> f64 {
        if self.ticks_with_events == 0 {
            0.0
        } else {
            self.parallel_ticks as f64 / self.ticks_with_events as f64
        }
    }

    /// Rough headroom proxy: mean width relative to threshold (capped at 1).
    ///
    /// Values ≪ 1 mean most buckets are below the parallel threshold (reset /
    /// sparsity limited). Values near 1 mean buckets are wide enough to use
    /// rayon often.
    #[inline]
    pub fn width_headroom(&self) -> f64 {
        (self.mean_width() / PARALLEL_CELL_THRESHOLD as f64).min(1.0)
    }

    pub(crate) fn record_bucket(&mut self, distinct_cells: usize, n_events: usize) {
        self.ticks_with_events = self.ticks_with_events.saturating_add(1);
        self.total_cell_jobs = self.total_cell_jobs.saturating_add(distinct_cells);
        self.total_events = self.total_events.saturating_add(n_events);
        self.max_width = self.max_width.max(distinct_cells);
        if distinct_cells >= PARALLEL_CELL_THRESHOLD {
            self.parallel_ticks = self.parallel_ticks.saturating_add(1);
        } else {
            self.sequential_ticks = self.sequential_ticks.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;

    #[test]
    fn plan_is_stable_balanced_and_counts_cuts() {
        let conn = Csr::from_adjacency(&[vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]]);
        let a = PartitionPlan::degree_balanced(&conn, 2);
        let b = PartitionPlan::degree_balanced(&conn, 2);
        assert_eq!(a, b);
        assert_eq!(a.n_partitions(), 2);
        assert_eq!(a.n_cells(), 4);
        assert!(a.cut_edges() <= conn.nnz());
    }

    #[test]
    fn partitioned_delta_step_matches_sequential_engine() {
        let conn = Csr::from_adjacency(&[vec![2, 3], vec![2, 3], vec![4], vec![4], vec![]]);
        let weights = vec![0.6; conn.nnz()];
        let mut sequential = Engine::with_cells(5);
        sequential.set_connectivity(conn.clone(), weights.clone());
        let mut parallel = Engine::with_cells(5);
        parallel.set_connectivity(conn.clone(), weights);
        for engine in [&mut sequential, &mut parallel] {
            engine.force_spike(0, 2);
            engine.force_spike(1, 2);
            engine.inject_weighted(2, 1, 3, 0.4);
            engine.force_spike(0, 8);
            engine.force_spike(1, 8);
        }
        let expected = sequential.step_until(20);
        let plan = PartitionPlan::degree_balanced(&conn, 3);
        let observed = parallel.step_until_partitioned(20, &plan);
        assert_eq!(observed, expected);
        assert_eq!(parallel.spikes(), sequential.spikes());
        assert_eq!(parallel.work(), sequential.work());
        for cell in 0..5u32 {
            assert_eq!(parallel.cell(cell).v, sequential.cell(cell).v);
            assert_eq!(parallel.cell(cell).theta, sequential.cell(cell).theta);
        }
    }

    #[test]
    fn profile_records_thin_vs_wide_ticks() {
        let mut profile = ParallelismProfile::default();
        profile.record_bucket(2, 4);
        profile.record_bucket(PARALLEL_CELL_THRESHOLD, 16);
        assert_eq!(profile.ticks_with_events, 2);
        assert_eq!(profile.sequential_ticks, 1);
        assert_eq!(profile.parallel_ticks, 1);
        assert_eq!(profile.max_width, PARALLEL_CELL_THRESHOLD);
        assert!((profile.parallel_tick_fraction() - 0.5).abs() < 1e-12);
    }
}
