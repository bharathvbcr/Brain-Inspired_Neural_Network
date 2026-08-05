//! Continuous inhibitory interneuron competition for soft-WTA dynamics (Tier 2).
//!
//! Provides a deterministic population of PV+ inhibitory interneurons with
//! heterogeneous E→I and I→E projections. The projections are normalized at
//! the population level, so changing the number of interneurons changes the
//! overall gain without collapsing every excitatory cell onto one broadcast
//! current.

/// Inhibitory interneuron area for soft, continuous competition.
#[derive(Clone, Debug)]
pub struct InhibitoryInterneuronArea {
    pub n_excitatory: usize,
    pub n_inhibitory: usize,
    pub e_to_i_weight: f32,
    pub i_to_e_weight: f32,
    pub i_membranes: Vec<f32>,
    /// Row-major `n_inhibitory × n_excitatory` E→I projection.
    e_to_i_projection: Vec<f32>,
    /// Row-major `n_excitatory × n_inhibitory` I→E projection.
    i_to_e_projection: Vec<f32>,
    mean_i_to_e_row_sum: f32,
    /// Per-interneuron E→I row sums, precomputed in [`Self::new`].
    ///
    /// `e_to_i_projection` is built once and never mutated, so these sums are
    /// static. `compute_inhibition` used to recompute them on every call, which
    /// added a second full O(n_inhibitory × n_excitatory) pass and roughly
    /// doubled the cost of the E→I stage. Computed here with the same
    /// `row.iter().sum::<f32>()` over the same slice, so the summation order —
    /// and therefore the f32 result — is bit-identical to the old inline form.
    e_to_i_row_sums: Vec<f32>,
}

impl InhibitoryInterneuronArea {
    pub fn new(
        n_excitatory: usize,
        n_inhibitory: usize,
        e_to_i_weight: f32,
        i_to_e_weight: f32,
    ) -> Self {
        assert!(n_excitatory >= 1);
        assert!(n_inhibitory >= 1);
        assert!(e_to_i_weight.is_finite() && e_to_i_weight >= 0.0);
        assert!(i_to_e_weight.is_finite() && i_to_e_weight >= 0.0);

        // Stable index-derived projections avoid hidden RNG state while still
        // giving each interneuron and principal cell a distinct receptive field.
        let mut e_to_i_projection = vec![0.0; n_inhibitory * n_excitatory];
        for inh in 0..n_inhibitory {
            for exc in 0..n_excitatory {
                let code = (exc
                    .wrapping_mul(31)
                    .wrapping_add(inh.wrapping_mul(17))
                    .wrapping_add(7))
                    % 11;
                if code < 7 {
                    e_to_i_projection[inh * n_excitatory + exc] = 0.75 + 0.125 * (code % 3) as f32;
                }
            }
            // Every inhibitory neuron must receive at least one principal cell.
            let anchor = inh % n_excitatory;
            e_to_i_projection[inh * n_excitatory + anchor] =
                e_to_i_projection[inh * n_excitatory + anchor].max(1.0);
        }

        let mut i_to_e_projection = vec![0.0; n_excitatory * n_inhibitory];
        let mut total_i_to_e = 0.0f32;
        for exc in 0..n_excitatory {
            for inh in 0..n_inhibitory {
                let code = (exc
                    .wrapping_mul(19)
                    .wrapping_add(inh.wrapping_mul(37))
                    .wrapping_add(3))
                    % 13;
                if code < 8 {
                    let weight = 0.7 + 0.1 * (code % 4) as f32;
                    i_to_e_projection[exc * n_inhibitory + inh] = weight;
                    total_i_to_e += weight;
                }
            }
            // Every excitatory cell must receive inhibition.
            let anchor = exc % n_inhibitory;
            let slot = exc * n_inhibitory + anchor;
            if i_to_e_projection[slot] == 0.0 {
                i_to_e_projection[slot] = 1.0;
                total_i_to_e += 1.0;
            }
        }
        let mean_i_to_e_row_sum = (total_i_to_e / n_excitatory as f32).max(f32::EPSILON);

        // Same expression `compute_inhibition` used inline, evaluated once.
        let e_to_i_row_sums = (0..n_inhibitory)
            .map(|inh| {
                e_to_i_projection[inh * n_excitatory..(inh + 1) * n_excitatory]
                    .iter()
                    .sum::<f32>()
                    .max(f32::EPSILON)
            })
            .collect();

        Self {
            n_excitatory,
            n_inhibitory,
            e_to_i_weight,
            i_to_e_weight,
            i_membranes: vec![0.0; n_inhibitory],
            e_to_i_projection,
            i_to_e_projection,
            mean_i_to_e_row_sum,
            e_to_i_row_sums,
        }
    }

    /// Compute inhibitory feedback current for each excitatory cell given somatic voltages/rates.
    pub fn compute_inhibition(&mut self, e_activities: &[f32]) -> Vec<f32> {
        assert_eq!(e_activities.len(), self.n_excitatory);
        assert!(
            e_activities.iter().all(|x| x.is_finite() && *x >= 0.0),
            "excitatory activities must be finite and non-negative"
        );

        let population_ratio = self.n_excitatory as f32 / self.n_inhibitory as f32;
        for inh in 0..self.n_inhibitory {
            let row =
                &self.e_to_i_projection[inh * self.n_excitatory..(inh + 1) * self.n_excitatory];
            // Precomputed in `new()`; `e_to_i_projection` is never mutated.
            let row_sum = self.e_to_i_row_sums[inh];
            let receptive_activity = row
                .iter()
                .zip(e_activities)
                .map(|(weight, activity)| weight * activity)
                .sum::<f32>()
                / row_sum;
            let drive = receptive_activity * population_ratio * self.e_to_i_weight;
            self.i_membranes[inh] = 0.9 * self.i_membranes[inh] + drive;
        }

        (0..self.n_excitatory)
            .map(|exc| {
                let row =
                    &self.i_to_e_projection[exc * self.n_inhibitory..(exc + 1) * self.n_inhibitory];
                let projected = row
                    .iter()
                    .zip(&self.i_membranes)
                    .map(|(weight, membrane)| weight * membrane)
                    .sum::<f32>();
                projected * self.i_to_e_weight / self.mean_i_to_e_row_sum
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inhibitory_area_computes_graded_inhibition() {
        let mut area = InhibitoryInterneuronArea::new(10, 2, 0.5, 0.8);
        let e_acts = vec![1.0; 10];
        let inh1 = area.compute_inhibition(&e_acts);
        let inh2 = area.compute_inhibition(&e_acts);
        assert_eq!(inh1.len(), 10);
        assert!(
            inh2[0] > inh1[0],
            "inhibition should integrate over time with persistent excitation"
        );
    }

    #[test]
    fn heterogeneous_drive_produces_cell_specific_inhibition() {
        let mut area = InhibitoryInterneuronArea::new(32, 8, 0.5, 0.8);
        let mut activities = vec![0.05; 32];
        activities[3] = 2.0;
        activities[19] = 1.5;
        let inhibition = area.compute_inhibition(&activities);
        let lo = inhibition.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = inhibition.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            hi - lo > 1e-4,
            "cell-specific projections must not collapse to broadcast inhibition"
        );
    }

    #[test]
    fn inhibitory_projection_is_deterministic() {
        let activities = vec![0.25; 24];
        let mut a = InhibitoryInterneuronArea::new(24, 6, 0.5, 0.8);
        let mut b = InhibitoryInterneuronArea::new(24, 6, 0.5, 0.8);
        assert_eq!(
            a.compute_inhibition(&activities),
            b.compute_inhibition(&activities)
        );
    }
}
