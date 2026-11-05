//! Multi-area population topology and fused inter-area projection layers.
//!
//! Manages M connected k-WTA areas (A_0, A_1, ..., A_{M-1}) and fuses presynaptic
//! winner collection, inter-area CSR SpMV projection, dendritic coincidence
//! integration, and destination k-WTA winner selection into a single pass.

use std::ops::Range;

use binn_core::sparse::Csr;
use binn_engine::{CellId, Engine};

use crate::area::{ActivityLog, Area};
use crate::wta::{k_wta_in_place, soft_k_wta};

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
    dst_charge_scratch: Vec<f32>,
    scores_scratch: Vec<(CellId, f32)>,
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
            dst_charge_scratch: Vec::with_capacity(cells_per_area),
            scores_scratch: Vec::with_capacity(cells_per_area),
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
        let src_cells = self.areas[src_idx].cells.clone();
        let dst_cells = self.areas[dst_idx].cells.clone();
        let dst_start = dst_cells.start;
        let dst_len = self.areas[dst_idx].len();
        let effective_k = self.areas[dst_idx].effective_k();

        self.dst_charge_scratch.clear();
        self.dst_charge_scratch.resize(dst_len, 0.0);
        let dst_charge = &mut self.dst_charge_scratch[..dst_len];

        // Find relevant projection
        if let Some(proj) = self
            .projections
            .iter()
            .find(|p| p.src_range == src_cells && p.dst_range == dst_cells)
        {
            // Sparse inter-area projection from src_winners
            let src_start = src_cells.start;
            for &w in src_winners {
                let local_src = (w - src_start) as usize;
                if local_src < proj.conn.nrows() {
                    let start_edge = proj.conn.row_ptr[local_src] as usize;
                    let end_edge = proj.conn.row_ptr[local_src + 1] as usize;
                    let cols = &proj.conn.col[start_edge..end_edge];
                    let ws = &proj.weights[start_edge..end_edge];
                    for (&post_col_u32, &weight) in cols.iter().zip(ws.iter()) {
                        let post_col = post_col_u32 as usize;
                        if post_col < dst_len {
                            dst_charge[post_col] += weight;
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
            for (slot, &bias) in dst_charge.iter_mut().zip(extra.iter()) {
                *slot += bias;
            }
        }

        // Combine charge + dendritic coincidence score in reusable scratch.
        let cells = engine.cells();
        self.scores_scratch.clear();
        self.scores_scratch.reserve(dst_len);
        for (i, &charge) in dst_charge.iter().enumerate() {
            let id = dst_start + i as CellId;
            let dend_score = cells[id as usize].dendritic_coincidence_score();
            self.scores_scratch.push((id, charge + dend_score));
        }

        let winners = if opts.temperature > 0.0 {
            soft_k_wta(
                &self.scores_scratch,
                effective_k,
                opts.temperature,
                opts.seed,
            )
        } else {
            k_wta_in_place(&mut self.scores_scratch, effective_k)
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

    #[test]
    fn fused_inter_area_step_reuses_scratch_and_clears_between_steps() {
        let mut net = MultiAreaNetwork::new(2, 16, 3);
        let mut eng = Engine::with_cells(32);
        let csr = Csr::from_adjacency(&[
            vec![1, 5, 9],
            vec![2, 5, 11],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);
        net.add_projection(0, 1, csr, vec![0.4, 0.9, 0.2, 0.8, 0.7, 0.1]);

        let w1 = net.fused_inter_area_step(&mut eng, 0, 1, &[0, 1], InterAreaStepOpts::new(0.0, 0));
        let charge_ptr_1 = net.dst_charge_scratch.as_ptr();
        let scores_ptr_1 = net.scores_scratch.as_ptr();
        assert_eq!(w1, vec![17, 18, 21]); // 16 + {1, 2, 5}

        // Second step with different winners must not leak prior step's charge
        // and must reuse the existing scratch allocations.
        let bias = [
            0.0, 0.0, 0.0, 1.5, 0.0, 0.0, 0.0, 1.2, 0.0, 0.0, 0.0, 0.0, 1.1, 0.0, 0.0, 0.0,
        ];
        let w2 = net.fused_inter_area_step(
            &mut eng,
            0,
            1,
            &[],
            InterAreaStepOpts::new(0.0, 0).with_bias(&bias),
        );
        assert_eq!(w2, vec![19, 23, 28]); // 16 + {3, 7, 12}
        assert_eq!(net.dst_charge_scratch.as_ptr(), charge_ptr_1);
        assert_eq!(net.scores_scratch.as_ptr(), scores_ptr_1);
    }
}
