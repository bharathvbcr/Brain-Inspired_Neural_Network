use binn_core::Csr;
use binn_engine::Engine;
use binn_hybrid_learn::{topology_signature, CreditFeatures};

/// A terminal-loss-only trace. No intermediate target labels are represented.
#[derive(Clone, Debug)]
pub struct TerminalTrace {
    pub activations: Vec<f32>,
    pub pre_activations: Vec<f32>,
    pub logits: Vec<f32>,
    pub prediction: usize,
}

#[derive(Clone, Debug)]
pub struct TeacherTargets {
    pub loss: f32,
    pub edge_deltas: Vec<f32>,
    pub post_credits: Vec<f32>,
    pub features: Vec<CreditFeatures>,
}

pub trait TerminalTeacher {
    fn targets(
        &self,
        model: &SparseTerminalModel,
        trace: &TerminalTrace,
        terminal_label: usize,
        learning_rate: f32,
    ) -> TeacherTargets;
}

/// Hand-coded reverse pass for the smooth surrogate used by H0.
#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalTraceTeacher;

impl TerminalTeacher for TerminalTraceTeacher {
    fn targets(
        &self,
        model: &SparseTerminalModel,
        trace: &TerminalTrace,
        terminal_label: usize,
        learning_rate: f32,
    ) -> TeacherTargets {
        assert!(terminal_label < model.output_size());
        let probabilities = softmax(&trace.logits);
        let loss = -probabilities[terminal_label].max(1e-12).ln();
        let mut cell_credit = vec![0.0f32; model.n_cells()];
        let output_start = model.layer_offsets[model.layer_sizes.len() - 1];
        for (index, probability) in probabilities.iter().copied().enumerate() {
            cell_credit[output_start + index] = f32::from(index == terminal_label) - probability;
        }

        for layer in (1..model.layer_sizes.len() - 1).rev() {
            let start = model.layer_offsets[layer];
            let end = start + model.layer_sizes[layer];
            for pre in start..end {
                let outgoing = edge_range(&model.engine.conn, pre);
                let downstream = outgoing
                    .clone()
                    .map(|edge| {
                        let post = model.engine.conn.col[edge] as usize;
                        model.engine.edge_w[edge] * cell_credit[post]
                    })
                    .sum::<f32>();
                let activation = trace.activations[pre];
                cell_credit[pre] = downstream * (1.0 - activation * activation);
            }
        }

        let mut edge_deltas = vec![0.0f32; model.engine.edge_w.len()];
        for pre in 0..model.n_cells() {
            for edge in edge_range(&model.engine.conn, pre) {
                let post = model.engine.conn.col[edge] as usize;
                let delta = learning_rate * trace.activations[pre] * cell_credit[post];
                edge_deltas[edge] = delta;
            }
        }
        TeacherTargets {
            loss,
            edge_deltas,
            post_credits: cell_credit
                .into_iter()
                .map(|credit| learning_rate * credit)
                .collect(),
            features: model.credit_features(trace, None),
        }
    }
}

/// A sparse, layer-structured smooth surrogate whose weights are stored in the
/// production `Engine` representation.
///
/// It is intentionally an H0 matched sparse surrogate, not a claim that the
/// hard event dynamics are differentiable.
pub struct SparseTerminalModel {
    pub engine: Engine,
    pub(crate) layer_sizes: Vec<usize>,
    pub(crate) layer_offsets: Vec<usize>,
}

impl Clone for SparseTerminalModel {
    fn clone(&self) -> Self {
        let mut engine = Engine::with_cells(self.n_cells());
        engine.set_connectivity(self.engine.conn.clone(), self.engine.edge_w.clone());
        Self {
            engine,
            layer_sizes: self.layer_sizes.clone(),
            layer_offsets: self.layer_offsets.clone(),
        }
    }
}

impl SparseTerminalModel {
    pub fn new(layer_sizes: Vec<usize>, seed: u64) -> Self {
        assert!(layer_sizes.len() >= 2);
        assert!(layer_sizes.iter().all(|&size| size > 0));
        let mut layer_offsets = Vec::with_capacity(layer_sizes.len());
        let mut n_cells = 0usize;
        for &size in &layer_sizes {
            layer_offsets.push(n_cells);
            n_cells += size;
        }
        let mut rows = vec![Vec::new(); n_cells];
        for layer in 0..layer_sizes.len() - 1 {
            let pre_start = layer_offsets[layer];
            let post_start = layer_offsets[layer + 1];
            for row in rows.iter_mut().skip(pre_start).take(layer_sizes[layer]) {
                row.extend((post_start..post_start + layer_sizes[layer + 1]).map(|v| v as u32));
            }
        }
        let conn = Csr::from_adjacency(&rows);
        let mut rng = LocalRng::new(seed);
        let weights = (0..conn.nnz())
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * 0.22)
            .collect::<Vec<_>>();
        let mut engine = Engine::with_cells(n_cells);
        engine.set_connectivity(conn, weights);
        Self {
            engine,
            layer_sizes,
            layer_offsets,
        }
    }

    pub fn n_cells(&self) -> usize {
        self.engine.num_cells()
    }

    pub fn output_size(&self) -> usize {
        *self.layer_sizes.last().expect("layers")
    }

    pub fn signature(&self) -> u64 {
        topology_signature(&self.engine)
    }

    pub fn forward(&self, input: &[f32]) -> TerminalTrace {
        assert_eq!(input.len(), self.layer_sizes[0]);
        let mut activations = vec![0.0f32; self.n_cells()];
        let mut pre_activations = vec![0.0f32; self.n_cells()];
        activations[..input.len()].copy_from_slice(input);
        pre_activations[..input.len()].copy_from_slice(input);
        for layer in 1..self.layer_sizes.len() {
            let start = self.layer_offsets[layer];
            let end = start + self.layer_sizes[layer];
            for post in start..end {
                let mut sum = 0.0f32;
                for (pre, &activation) in activations.iter().enumerate().take(start) {
                    if let Some(edge) = edge_index(&self.engine.conn, pre, post) {
                        sum += self.engine.edge_w[edge] * activation;
                    }
                }
                pre_activations[post] = sum;
                activations[post] = if layer + 1 == self.layer_sizes.len() {
                    sum
                } else {
                    sum.tanh()
                };
            }
        }
        let output_start = *self.layer_offsets.last().expect("output offset");
        let logits = activations[output_start..].to_vec();
        let prediction = logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or(0);
        TerminalTrace {
            activations,
            pre_activations,
            logits,
            prediction,
        }
    }

    pub fn credit_features(
        &self,
        trace: &TerminalTrace,
        broadcast_reward: Option<f32>,
    ) -> Vec<CreditFeatures> {
        let mut features = Vec::with_capacity(self.engine.edge_w.len());
        for pre in 0..self.n_cells() {
            for edge in edge_range(&self.engine.conn, pre) {
                let post = self.engine.conn.col[edge] as usize;
                features.push(CreditFeatures {
                    pre_trace: trace.activations[pre],
                    post_trace: trace.activations[post],
                    eligibility: trace.activations[pre] * trace.activations[post],
                    weight: self.engine.edge_w[edge],
                    pre_membrane: trace.pre_activations[pre],
                    post_membrane: trace.pre_activations[post],
                    pre_threshold: 0.0,
                    post_threshold: 0.0,
                    pre_activity: trace.activations[pre].abs(),
                    post_activity: trace.activations[post].abs(),
                    structural_id: ((pre as u32) << 16) ^ post as u32,
                    broadcast_reward,
                });
            }
        }
        features
    }

    pub fn apply_deltas(&mut self, deltas: &[f32]) {
        assert_eq!(deltas.len(), self.engine.edge_w.len());
        for (edge, &delta) in deltas.iter().enumerate() {
            let updated = (self.engine.edge_w[edge] + delta).clamp(-8.0, 8.0);
            self.engine.edge_w[edge] = updated;
            self.engine.syn.as_mut_slice()[edge].weight = updated;
        }
    }
}

pub(crate) fn edge_index(conn: &Csr, pre: usize, post: usize) -> Option<usize> {
    edge_range(conn, pre).find(|&edge| conn.col[edge] as usize == post)
}

pub(crate) fn edge_range(conn: &Csr, pre: usize) -> std::ops::Range<usize> {
    conn.row_ptr[pre] as usize..conn.row_ptr[pre + 1] as usize
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values = logits
        .iter()
        .map(|value| (*value - max).exp())
        .collect::<Vec<_>>();
    let sum = values.iter().sum::<f32>().max(1e-12);
    for value in &mut values {
        *value /= sum;
    }
    values
}

#[derive(Clone, Copy)]
pub(crate) struct LocalRng(u64);

impl LocalRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.0 = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    pub(crate) fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    pub(crate) fn index(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_teacher_matches_finite_difference() {
        let model = SparseTerminalModel::new(vec![3, 4, 2], 7);
        let input = [0.3, -0.7, 0.9];
        let trace = model.forward(&input);
        let targets = TerminalTraceTeacher.targets(&model, &trace, 1, 1.0);
        let epsilon = 1e-3f32;
        for edge in 0..model.engine.edge_w.len() {
            let mut plus = model.clone();
            plus.engine.edge_w[edge] += epsilon;
            plus.engine.syn.as_mut_slice()[edge].weight = plus.engine.edge_w[edge];
            let plus_loss = TerminalTraceTeacher
                .targets(&plus, &plus.forward(&input), 1, 1.0)
                .loss;
            let mut minus = model.clone();
            minus.engine.edge_w[edge] -= epsilon;
            minus.engine.syn.as_mut_slice()[edge].weight = minus.engine.edge_w[edge];
            let minus_loss = TerminalTraceTeacher
                .targets(&minus, &minus.forward(&input), 1, 1.0)
                .loss;
            let descent = -(plus_loss - minus_loss) / (2.0 * epsilon);
            assert!(
                (targets.edge_deltas[edge] - descent).abs() < 2e-3,
                "edge {edge}: analytic={} numeric={descent}",
                targets.edge_deltas[edge]
            );
        }
    }
}
