//! Multi-area hub (U16 / R1 minimal surface — not a full P5 engine).
//!
//! A hub is a designated area among `n` contiguous populations. Wiring uses
//! [`WiringPrior`] with [`AreaRole::Hub`] on the hub index so inter-area edges
//! concentrate through the hub while preserving event locality.

use std::ops::Range;

use binn_core::Csr;
use binn_engine::CellId;

use crate::area::Area;
use crate::wiring::{intra_area_event_fraction, wire, AreaRole, Pos, WiringPrior};

/// Hub of interconnected areas sufficient for R1 composition experiments.
#[derive(Clone, Debug)]
pub struct Hub {
    /// Contiguous populations (uniform size for R1).
    pub areas: Vec<Area>,
    /// Index of the designated hub area.
    pub hub_index: usize,
    /// Cells per area (uniform).
    pub cells_per_area: usize,
}

impl Hub {
    /// Build `n_areas` populations of `cells_per_area` with k-WTA cap `k`.
    ///
    /// # Panics
    ///
    /// Panics if `n_areas < 2`, `cells_per_area == 0`, or `hub_index >= n_areas`.
    pub fn new(n_areas: usize, cells_per_area: usize, k: usize, hub_index: usize) -> Self {
        assert!(n_areas >= 2, "hub requires at least two areas");
        assert!(cells_per_area > 0, "cells_per_area must be positive");
        assert!(
            hub_index < n_areas,
            "hub_index {hub_index} out of range ({n_areas} areas)"
        );
        let mut areas = Vec::with_capacity(n_areas);
        for i in 0..n_areas {
            let start = (i * cells_per_area) as CellId;
            let end = start + cells_per_area as CellId;
            areas.push(Area::new(start..end, k));
        }
        Self {
            areas,
            hub_index,
            cells_per_area,
        }
    }

    /// Convenience: hub at the middle index.
    pub fn with_central_hub(n_areas: usize, cells_per_area: usize, k: usize) -> Self {
        Self::new(n_areas, cells_per_area, k, n_areas / 2)
    }

    /// Number of areas.
    #[inline]
    pub fn n_areas(&self) -> usize {
        self.areas.len()
    }

    /// Total cell count.
    #[inline]
    pub fn num_cells(&self) -> usize {
        self.n_areas() * self.cells_per_area
    }

    /// Contiguous cell ranges for wiring.
    pub fn area_ranges(&self) -> Vec<Range<CellId>> {
        self.areas.iter().map(|a| a.cells.clone()).collect()
    }

    /// Borrow the hub area.
    #[inline]
    pub fn hub_area(&self) -> &Area {
        &self.areas[self.hub_index]
    }

    /// Wiring prior with elevated inter-area density through the hub role.
    pub fn wiring_prior(&self, seed: u64, p_intra: f32, p_inter: f32) -> WiringPrior {
        WiringPrior::new(seed, self.area_ranges(), p_intra, p_inter)
            .with_max_fan_out(self.cells_per_area.saturating_mul(2).max(8))
    }

    /// Compose CSR with hub-role modulation at [`Self::hub_index`].
    ///
    /// One `wire(Hub, hub_pos, prior)` call covers the full multi-area graph;
    /// the hub role elevates inter-area fan-out that touches the hub while
    /// keeping event locality dominated by intra-area edges.
    pub fn compose_csr(&self, seed: u64, p_intra: f32, p_inter: f32) -> Csr {
        let prior = self.wiring_prior(seed, p_intra, p_inter);
        wire(AreaRole::Hub, Pos::new(self.hub_index), &prior)
    }

    /// Fraction of edges that are intra-area under a uniform spike workload.
    pub fn event_locality(&self, csr: &Csr, seed: u64, p_intra: f32, p_inter: f32) -> f32 {
        let prior = self.wiring_prior(seed, p_intra, p_inter);
        let spikes = vec![1u64; csr.nrows()];
        intra_area_event_fraction(csr, &prior, &spikes) as f32
    }

    /// Nominal budget disclosure for harness logs.
    pub fn budget_cells(&self) -> usize {
        self.num_cells()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_builds_contiguous_areas() {
        let hub = Hub::with_central_hub(5, 8, 2);
        assert_eq!(hub.n_areas(), 5);
        assert_eq!(hub.hub_index, 2);
        assert_eq!(hub.num_cells(), 40);
        assert_eq!(hub.areas[0].cells, 0..8);
        assert_eq!(hub.areas[4].cells, 32..40);
    }

    #[test]
    fn compose_csr_is_deterministic_and_local() {
        let hub = Hub::new(4, 16, 2, 1);
        let a = hub.compose_csr(0xA0B0_0001, 0.35, 0.05);
        let b = hub.compose_csr(0xA0B0_0001, 0.35, 0.05);
        assert_eq!(a.nnz(), b.nnz());
        assert_eq!(a.row_ptr, b.row_ptr);
        assert_eq!(a.col, b.col);
        let loc = hub.event_locality(&a, 0xA0B0_0001, 0.35, 0.05);
        assert!(
            loc > 0.5,
            "hub wiring should remain locality-dominated (got {loc})"
        );
    }

    #[test]
    #[should_panic(expected = "hub_index")]
    fn hub_index_must_be_in_range() {
        let _ = Hub::new(3, 8, 1, 3);
    }
}
