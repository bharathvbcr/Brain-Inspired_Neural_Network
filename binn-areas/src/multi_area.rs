//! Multi-area population topology and fused inter-area projection layers.
//!
//! Manages M connected k-WTA areas (A_0, A_1, ..., A_{M-1}) and fuses presynaptic
//! winner collection, inter-area CSR SpMV projection, dendritic coincidence
//! integration, and destination k-WTA winner selection into a single pass.

use std::ops::Range;

use binn_core::sparse::Csr;
use binn_engine::{CellId, Engine};

use crate::area::{ActivityLog, Area};
use crate::wta::{k_wta, soft_k_wta};

/// Directed projection layer connecting source area `src` to destination area `dst`.
#[derive(Clone, Debug)]
pub struct InterAreaProjection {
    pub src_range: Range<CellId>,
    pub dst_range: Range<CellId>,
    pub conn: Csr,
    pub weights: Vec<f32>,
    pub feedback_b: Vec<f32>,
}

impl InterAreaProjection {
    /// Create a new inter-area projection layer.
    pub fn new(
        src_range: Range<CellId>,
        dst_range: Range<CellId>,
        conn: Csr,
        weights: Vec<f32>,
    ) -> Self {
        assert_eq!(weights.len(), conn.nnz());
        let nnz = conn.nnz();
        let feedback_b = vec![1.0f32; nnz];
        Self {
            src_range,
            dst_range,
            conn,
            weights,
            feedback_b,
        }
    }
}

/// Multi-area network container supporting M interconnected k-WTA areas.
#[derive(Clone, Debug)]
pub struct MultiAreaNetwork {
    pub areas: Vec<Area>,
    pub projections: Vec<InterAreaProjection>,
    pub activity_logs: Vec<ActivityLog>,
}

impl MultiAreaNetwork {
    /// Construct a multi-area network with `n_areas` populations of `cells_per_area`.
    pub fn new(n_areas: usize, cells_per_area: usize, k: usize) -> Self {
        assert!(n_areas >= 2, "multi-area network requires at least 2 areas");
        assert!(cells_per_area > 0, "cells_per_area must be positive");
        assert!(k > 0, "k must be positive");

        let mut areas = Vec::with_capacity(n_areas);
        let mut activity_logs = Vec::with_capacity(n_areas);
        for i in 0..n_areas {
            let start = (i * cells_per_area) as CellId;
            let end = start + cells_per_area as CellId;
            areas.push(Area::new(start..end, k));
            activity_logs.push(ActivityLog::new());
        }

        Self {
            areas,
            projections: Vec::new(),
            activity_logs,
        }
    }

    /// Add a feedforward projection from area index `src_idx` to `dst_idx`.
    pub fn add_projection(&mut self, src_idx: usize, dst_idx: usize, conn: Csr, weights: Vec<f32>) {
        assert!(src_idx < self.areas.len());
        assert!(dst_idx < self.areas.len());
        let src_range = self.areas[src_idx].cells.clone();
        let dst_range = self.areas[dst_idx].cells.clone();
        let proj = InterAreaProjection::new(src_range, dst_range, conn, weights);
        self.projections.push(proj);
    }

    /// Fused inter-area step: propagates activity from source winners through
    /// inter-area CSR SpMV to target cells, applies dendritic coincidence integration,
    /// and selects destination winners using Soft-to-Hard k-WTA.
    ///
    /// `extra_charge`, when provided, is added to the destination charge before
    /// k-WTA. Deep stacks of pure winner→SpMV→k-WTA erase the stimulus
    /// (identical final winner sets across samples); a weak per-area stimulus
    /// residual keeps the destination state sample-dependent.
    pub fn fused_inter_area_step(
        &mut self,
        engine: &mut Engine,
        src_idx: usize,
        dst_idx: usize,
        src_winners: &[CellId],
        opts: InterAreaStepOpts<'_>,
    ) -> Vec<CellId> {
        let dst_area = &self.areas[dst_idx];
        let dst_start = dst_area.cells.start as usize;
        let dst_len = dst_area.len();

        let mut dst_charge = vec![0.0f32; dst_len];

        // Find relevant projection
        if let Some(proj) = self.projections.iter().find(|p| {
            p.src_range == self.areas[src_idx].cells && p.dst_range == self.areas[dst_idx].cells
        }) {
            // Sparse inter-area projection from src_winners
            let src_start = self.areas[src_idx].cells.start;
            for &w in src_winners {
                let local_src = (w - src_start) as usize;
                if local_src < proj.conn.nrows() {
                    let start_edge = proj.conn.row_ptr[local_src] as usize;
                    let end_edge = proj.conn.row_ptr[local_src + 1] as usize;
                    for edge in start_edge..end_edge {
                        let post_col = proj.conn.col[edge] as usize;
                        if post_col < dst_len {
                            dst_charge[post_col] += proj.weights[edge];
                        }
                    }
                }
            }
        }

        if let Some(extra) = opts.extra_charge {
            assert_eq!(
                extra.len(),
                dst_len,
                "extra_charge length must match destination area"
            );
            for i in 0..dst_len {
                dst_charge[i] += extra[i];
            }
        }

        // Combine charge + dendritic coincidence score
        let scores: Vec<(CellId, f32)> = (0..dst_len)
            .map(|i| {
                let id = dst_start as CellId + i as CellId;
                let dend_score = engine.cell(id).dendritic_coincidence_score();
                (id, dst_charge[i] + dend_score)
            })
            .collect();

        let winners = if opts.temperature > 0.0 {
            soft_k_wta(&scores, dst_area.effective_k(), opts.temperature, opts.seed)
        } else {
            k_wta(&scores, dst_area.effective_k())
        };

        self.activity_logs[dst_idx].record(dst_len, winners.len());
        winners
    }
}

/// Options for [`MultiAreaNetwork::fused_inter_area_step`].
#[derive(Clone, Copy, Debug)]
pub struct InterAreaStepOpts<'a> {
    pub temperature: f32,
    pub seed: u64,
    pub extra_charge: Option<&'a [f32]>,
}

impl<'a> InterAreaStepOpts<'a> {
    pub fn new(temperature: f32, seed: u64) -> Self {
        Self {
            temperature,
            seed,
            extra_charge: None,
        }
    }

    pub fn with_bias(mut self, extra_charge: &'a [f32]) -> Self {
        self.extra_charge = Some(extra_charge);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_area_network_construction() {
        let net = MultiAreaNetwork::new(3, 32, 4);
        assert_eq!(net.areas.len(), 3);
        assert_eq!(net.areas[0].len(), 32);
        assert_eq!(net.areas[1].len(), 32);
        assert_eq!(net.areas[2].len(), 32);
        assert_eq!(net.areas[0].cells, 0..32);
        assert_eq!(net.areas[1].cells, 32..64);
        assert_eq!(net.areas[2].cells, 64..96);
    }
}
