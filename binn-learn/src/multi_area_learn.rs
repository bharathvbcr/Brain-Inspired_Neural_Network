//! Multi-area credit routing and feedback alignment learning across deep hidden layers.
//!
//! # 2026-07-25 fixes
//!
//! * [`MultiAreaLearner::update_inter_area_feedback`] previously took
//!   `_weights: &mut [f32]` and never touched it. The feedback matrix `B` was
//!   learned but nothing ever consumed it, so the forward pass was frozen and
//!   the network could not learn regardless of the input. Forward-weight
//!   plasticity now lives in [`MultiAreaLearner::update_inter_area_weights`],
//!   and the unused `weights` parameter is gone from the feedback update.
//! * `find_row_for_edge` scanned all rows per edge, making a full update
//!   `O(nnz · nrows)`. Both updates now walk rows directly, `O(nnz)`.

#![allow(clippy::needless_range_loop)]

use binn_core::sparse::Csr;
use binn_core::Rng;

/// Multi-area credit alignment learner.
#[derive(Clone, Debug)]
pub struct MultiAreaLearner {
    pub eta: f32,
    pub eta_b: f32,
    pub lambda: f32,
    pub tau_e: f32,
    pub rng: Rng,
}

impl MultiAreaLearner {
    /// Construct a new multi-area learner.
    pub fn new(eta: f32, eta_b: f32, lambda: f32, tau_e: f32, seed: u64) -> Self {
        Self {
            eta,
            eta_b,
            lambda,
            tau_e,
            rng: Rng::new(seed ^ 0x4D75_6C74_6941_7265),
        }
    }

    /// Online feedback-alignment update of the per-edge feedback matrix:
    ///
    /// `B_ij <- clamp(B_ij + eta_b · rpe · x_pre_i · a_post_j, -2, 2)`
    ///
    /// # Panics
    ///
    /// Panics if `feedback_b.len() != conn.nnz()`, or if the activity vectors
    /// are shorter than the connectivity requires. The length assertions are
    /// deliberate: passing constant `vec![1.0; n]` "dummy" activity used to be
    /// silently accepted and produced sample-independent updates.
    pub fn update_inter_area_feedback(
        &mut self,
        conn: &Csr,
        feedback_b: &mut [f32],
        rpe: f32,
        pre_activity: &[f32],
        post_activity: &[f32],
    ) {
        assert_eq!(
            feedback_b.len(),
            conn.nnz(),
            "feedback_b must align with CSR nnz"
        );
        assert!(
            pre_activity.len() >= conn.nrows(),
            "pre_activity ({}) shorter than presynaptic population ({})",
            pre_activity.len(),
            conn.nrows()
        );
        assert!(rpe.is_finite(), "rpe must be finite");

        for row in 0..conn.nrows() {
            let start = conn.row_ptr[row] as usize;
            let end = conn.row_ptr[row + 1] as usize;
            let x_pre = pre_activity[row];
            if x_pre == 0.0 {
                continue;
            }
            for edge in start..end {
                let col = conn.col[edge] as usize;
                if col >= post_activity.len() {
                    continue;
                }
                let delta_b = self.eta_b * rpe * post_activity[col] * x_pre;
                feedback_b[edge] = (feedback_b[edge] + delta_b).clamp(-2.0, 2.0);
            }
        }
    }

    /// Feedback-aligned forward-weight update:
    ///
    /// `w_ij <- w_ij + eta · rpe · B_ij · x_pre_i · a_post_j − lambda · w_ij`
    ///
    /// This is the step that makes the feedback matrix matter. Without it, `B`
    /// is learned into a void and the forward pass never changes.
    ///
    /// # Panics
    ///
    /// Panics on length mismatch between `weights`, `feedback_b` and `conn`.
    pub fn update_inter_area_weights(
        &mut self,
        conn: &Csr,
        weights: &mut [f32],
        feedback_b: &[f32],
        rpe: f32,
        pre_activity: &[f32],
        post_activity: &[f32],
    ) {
        assert_eq!(weights.len(), conn.nnz(), "weights must align with CSR nnz");
        assert_eq!(
            feedback_b.len(),
            conn.nnz(),
            "feedback_b must align with CSR nnz"
        );
        assert!(
            pre_activity.len() >= conn.nrows(),
            "pre_activity ({}) shorter than presynaptic population ({})",
            pre_activity.len(),
            conn.nrows()
        );
        assert!(rpe.is_finite(), "rpe must be finite");

        for row in 0..conn.nrows() {
            let start = conn.row_ptr[row] as usize;
            let end = conn.row_ptr[row + 1] as usize;
            let x_pre = pre_activity[row];
            for edge in start..end {
                let col = conn.col[edge] as usize;
                let a_post = if col < post_activity.len() {
                    post_activity[col]
                } else {
                    0.0
                };
                let credit = self.eta * rpe * feedback_b[edge] * x_pre * a_post;
                weights[edge] += credit - self.lambda * weights[edge];
            }
        }
    }
}

/// Build a `{0,1}` activity vector for a population from its winner set.
///
/// `winners` are global cell ids; `range_start` is the population's first id.
pub fn winners_to_activity(winners: &[u32], range_start: u32, len: usize) -> Vec<f32> {
    let mut act = vec![0.0f32; len];
    for &w in winners {
        if w >= range_start {
            let local = (w - range_start) as usize;
            if local < len {
                act[local] = 1.0;
            }
        }
    }
    act
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_area_learner_construction() {
        let learner = MultiAreaLearner::new(0.05, 0.01, 0.0, 20.0, 42);
        assert_eq!(learner.eta, 0.05);
        assert_eq!(learner.eta_b, 0.01);
    }

    #[test]
    fn feedback_update_moves_b() {
        let adj = vec![vec![0, 1], vec![1]];
        let csr = Csr::from_adjacency(&adj);
        let mut feedback_b = vec![1.0, 1.0, 1.0];
        let mut lnr = MultiAreaLearner::new(0.05, 0.01, 0.0, 20.0, 42);
        lnr.update_inter_area_feedback(&csr, &mut feedback_b, 0.5, &[1.0, 1.0], &[1.0, 1.0]);
        assert!(feedback_b[0] > 1.0);
    }

    /// Regression: the forward weights must actually move, or no amount of
    /// stimulus can change the network's behaviour.
    #[test]
    fn forward_weights_actually_change() {
        let adj = vec![vec![0, 1], vec![1]];
        let csr = Csr::from_adjacency(&adj);
        let mut weights = vec![0.5, 0.5, 0.5];
        let feedback_b = vec![1.0, 1.0, 1.0];
        let before = weights.clone();
        let mut lnr = MultiAreaLearner::new(0.05, 0.01, 0.0, 20.0, 42);
        lnr.update_inter_area_weights(
            &csr,
            &mut weights,
            &feedback_b,
            1.0,
            &[1.0, 1.0],
            &[1.0, 1.0],
        );
        assert_ne!(before, weights, "forward weights must be plastic");
    }

    /// Zero presynaptic activity must produce zero forward credit: this is what
    /// makes the update stimulus-dependent rather than constant.
    #[test]
    fn silent_presynaptic_cells_get_no_credit() {
        let adj = vec![vec![0, 1], vec![1]];
        let csr = Csr::from_adjacency(&adj);
        let mut weights = vec![0.5, 0.5, 0.5];
        let feedback_b = vec![1.0, 1.0, 1.0];
        let mut lnr = MultiAreaLearner::new(0.05, 0.01, 0.0, 20.0, 42);
        // Row 0 silent, row 1 active.
        lnr.update_inter_area_weights(
            &csr,
            &mut weights,
            &feedback_b,
            1.0,
            &[0.0, 1.0],
            &[1.0, 1.0],
        );
        assert_eq!(weights[0], 0.5, "edge from silent row 0 must not move");
        assert_eq!(weights[1], 0.5, "edge from silent row 0 must not move");
        assert!(weights[2] > 0.5, "edge from active row 1 must move");
    }

    #[test]
    fn winners_map_to_local_activity() {
        let act = winners_to_activity(&[32, 35], 32, 4);
        assert_eq!(act, vec![1.0, 0.0, 0.0, 1.0]);
        // Out-of-range winners are ignored rather than panicking.
        let act2 = winners_to_activity(&[10, 99], 32, 4);
        assert_eq!(act2, vec![0.0; 4]);
    }
}
