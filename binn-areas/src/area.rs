//! Area population + GC7 activity-sparsity log hooks (U06).

use std::ops::Range;

use binn_engine::CellId;

/// One recorded k-WTA cycle for an area (GC7 activity-sparsity field).
#[derive(Clone, Debug, PartialEq)]
pub struct ActivitySample {
    /// Monotone cycle index within this area's log.
    pub cycle: u64,
    /// Population size `N`.
    pub n: usize,
    /// Number of winners that fired (`≤ k`).
    pub winners: usize,
    /// Activity sparsity `winners / N` (GC7 field name).
    pub activity_sparsity: f32,
}

/// Append-only activity log for GC7 (harness refuse lands in U13).
#[derive(Clone, Debug, Default)]
pub struct ActivityLog {
    samples: Vec<ActivitySample>,
}

impl ActivityLog {
    /// Empty log.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of recorded cycles.
    #[inline]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// True when no cycles have been logged.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Borrow recorded samples.
    #[inline]
    pub fn samples(&self) -> &[ActivitySample] {
        &self.samples
    }

    /// Mean activity sparsity over recorded cycles, or `0.0` if empty.
    pub fn mean_activity_sparsity(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|s| s.activity_sparsity).sum();
        sum / self.samples.len() as f32
    }

    /// Record one cycle. `winners` must be `≤ n`.
    pub fn record(&mut self, n: usize, winners: usize) {
        assert!(winners <= n, "winners ({winners}) exceed population ({n})");
        let cycle = self.samples.len() as u64;
        let activity_sparsity = if n == 0 {
            0.0
        } else {
            winners as f32 / n as f32
        };
        self.samples.push(ActivitySample {
            cycle,
            n,
            winners,
            activity_sparsity,
        });
    }
}

/// Neural area: a contiguous population with a shared k-WTA cap.
#[derive(Clone, Debug)]
pub struct Area {
    /// Inclusive-exclusive cell-id range owned by this area.
    pub cells: Range<CellId>,
    /// Maximum winners per cycle.
    pub k: usize,
    /// Per-cycle activity log (GC7 hook).
    pub activity: ActivityLog,
}

impl Area {
    /// Area over `cells` with WTA cap `k`.
    ///
    /// # Panics
    ///
    /// Panics if `k == 0` or `cells` is empty.
    pub fn new(cells: Range<CellId>, k: usize) -> Self {
        assert!(!cells.is_empty(), "area requires a non-empty cell range");
        assert!(k > 0, "area requires k > 0");
        Self {
            cells,
            k,
            activity: ActivityLog::new(),
        }
    }

    /// Population size `N`.
    #[inline]
    pub fn len(&self) -> usize {
        (self.cells.end - self.cells.start) as usize
    }

    /// Always false for a valid area (constructor rejects empty ranges).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// True when `id` lies in this area's cell range.
    #[inline]
    pub fn contains(&self, id: CellId) -> bool {
        self.cells.contains(&id)
    }

    /// Effective WTA cap for this population (`min(k, N)`).
    #[inline]
    pub fn effective_k(&self) -> usize {
        self.k.min(self.len())
    }

    /// Log a firing cycle with `winners` active cells (GC7).
    pub fn log_activity(&mut self, winners: usize) {
        let n = self.len();
        let capped = winners.min(n);
        self.activity.record(n, capped);
    }
}
