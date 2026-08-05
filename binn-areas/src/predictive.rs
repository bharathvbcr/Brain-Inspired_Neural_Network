//! Predictive coding inter-area projections & prediction errors.

use binn_core::Csr;

/// Inter-area top-down feedback projection predicting lower-area activity.
#[derive(Clone, Debug)]
pub struct PredictiveAreaProjection {
    pub feedback_conn: Csr,
    pub weights: Vec<f32>,
    pub n_lower: usize,
    pub n_upper: usize,
}

impl PredictiveAreaProjection {
    pub fn new(n_lower: usize, n_upper: usize, feedback_conn: Csr, initial_weight: f32) -> Self {
        let nnz = feedback_conn.nnz();
        Self {
            feedback_conn,
            weights: vec![initial_weight; nnz],
            n_lower,
            n_upper,
        }
    }

    /// Predict lower-area activities from upper-area activities.
    pub fn predict(&self, upper_activities: &[f32]) -> Vec<f32> {
        assert_eq!(upper_activities.len(), self.n_upper);
        let mut pred = vec![0.0f32; self.n_lower];
        for (row, p) in pred.iter_mut().enumerate() {
            let cols = self.feedback_conn.row_cols(row);
            let row_start = self.feedback_conn.row_ptr[row] as usize;
            for (idx, &col) in cols.iter().enumerate() {
                *p += self.weights[row_start + idx] * upper_activities[col as usize];
            }
        }
        pred
    }

    /// Compute elementwise prediction error: e = a_lower - a_pred
    pub fn prediction_error(&self, lower_activities: &[f32], upper_activities: &[f32]) -> Vec<f32> {
        let pred = self.predict(upper_activities);
        assert_eq!(lower_activities.len(), pred.len());
        lower_activities
            .iter()
            .zip(pred.iter())
            .map(|(&a, &p)| a - p)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predictive_projection() {
        let conn = Csr::from_adjacency(&[vec![0, 1], vec![1]]);
        let proj = PredictiveAreaProjection::new(2, 2, conn, 0.5);
        let upper = vec![1.0, 0.8];
        let pred = proj.predict(&upper);
        assert_eq!(pred.len(), 2);
        assert!((pred[0] - 0.9).abs() < 1e-5);
    }
}
