//! Multi-area structural scaling experiment binary & SpMV throughput benchmark.
//!
//! Evaluates multi-area k-WTA networks across 2, 4 and 8 areas with fused
//! inter-area projections, soft-to-hard WTA annealing, and online feedback
//! alignment on the inter-area forward weights.
//!
//! # 2026-07-25 rewrite
//!
//! The previous version did not measure learning or scaling:
//!
//! * Area 0's activation was the literal constant `(0..k)` for every sample, so
//!   nothing about the stimulus ever entered the network.
//! * `predicted = !prev_winners.is_empty()` is constant by construction.
//! * The "online feedback update" passed `vec![1.0; n]` for both pre- and
//!   post-synaptic activity, and `update_inter_area_feedback` never wrote to the
//!   forward weights anyway.
//! * Each area count drew its dataset from a *different* stream seed
//!   (`toy(100 + n_areas)`) with a 20-sample test split, so the reported
//!   0.75 / 0.40 / 0.90 were the positive-class fractions of three different
//!   tiny datasets — 95% CI half-widths of ±0.19, ±0.22 and ±0.13.
//!
//! This version encodes the sample into area-0 activity, propagates it, learns
//! both the readout and the inter-area forward weights, holds the dataset seed
//! **fixed across area counts**, evaluates on ≥ `MIN_EVAL_SAMPLES` held-out
//! samples, and aborts rather than reporting a degenerate readout.

#![allow(clippy::needless_range_loop)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use binn_areas::multi_area::{InterAreaStepOpts, MultiAreaNetwork};
use binn_areas::wta::{k_wta, soft_k_wta, WtaAnnealer};
use binn_core::metal_backend::{benchmarkable_backends, SpmvBackend, SpmvBackendConfig};
use binn_core::sparse::Csr;
use binn_core::Rng;
use binn_data::{Sample, SynthConfig, SyntheticStream};
use binn_engine::{CellId, Engine};
use binn_lab::guards::{ReadoutAudit, StimulusProbe, Verdict};
use binn_learn::multi_area_learn::{winners_to_activity, MultiAreaLearner};

/// Preregistered accuracy floor.
const ACCURACY_FLOOR: f32 = 0.65;
/// Dataset seed, held FIXED across area counts so M is the only thing varying.
const DATASET_SEED: u64 = 100;
const CELLS_PER_AREA: usize = 32;
const K_WTA: usize = 4;
const ETA_READOUT: f32 = 0.1;

fn sigmoid(z: f32) -> f32 {
    1.0 / (1.0 + (-z).exp())
}

/// Fixed seeded projection from sample features into area-0 cell drive.
fn stimulus_projection(n_features: usize, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed ^ 0x00A4_EA00_0001);
    (0..CELLS_PER_AREA * n_features)
        .map(|_| rng.next_f32() * 2.0 - 1.0)
        .collect()
}

/// Seeded sparse inter-area adjacency + heterogeneous weights.
///
/// The previous `(r + c) % 3 == 0` lattice with constant weight `0.2` collapsed
/// distinct area-0 winner sets into a handful of identical charge fingerprints
/// (empirically ~4 on the toy stream), so the readout froze on the majority
/// class and tripped [`ReadoutAudit::assert_non_degenerate`]. Random sparse
/// connectivity with per-edge weights keeps the destination charge
/// stimulus-dependent.
fn inter_area_projection(seed: u64) -> (Csr, Vec<f32>) {
    let mut rng = Rng::new(seed ^ 0x00A4_EA00_0003);
    let mut adj = vec![Vec::new(); CELLS_PER_AREA];
    for r in 0..CELLS_PER_AREA {
        for c in 0..CELLS_PER_AREA {
            // ~1/3 density, matching the old lattice's nominal sparsity.
            if rng.next_f32() < 0.34 {
                adj[r].push(c as u32);
            }
        }
        if adj[r].is_empty() {
            adj[r].push(rng.gen_index(CELLS_PER_AREA) as u32);
        }
    }
    let csr = Csr::from_adjacency(&adj);
    let weights: Vec<f32> = (0..csr.nnz())
        .map(|_| 0.05 + 0.35 * rng.next_f32())
        .collect();
    (csr, weights)
}

/// Encode a sample as continuous area-0 drive scores (also reused as a weak
/// residual bias for deeper areas so multi-hop k-WTA cannot erase the stimulus).
fn encode_area0_scores(w_stim: &[f32], sample: &Sample) -> Vec<(CellId, f32)> {
    let n_features = sample.values.len();
    (0..CELLS_PER_AREA)
        .map(|i| {
            let mut drive = 0.0f32;
            for j in 0..n_features {
                drive += w_stim[i * n_features + j] * sample.values[j];
            }
            (i as CellId, drive)
        })
        .collect()
}

/// Per-area stimulus residual. Area 0 is fully stimulus-driven; deeper areas
/// get a weak copy so winner identity cannot collapse across samples.
const STIM_RESIDUAL_GAIN: f32 = 0.25;

fn area_stimulus_bias(w_stim: &[f32], sample: &Sample, area_idx: usize) -> Vec<f32> {
    let scores = encode_area0_scores(w_stim, sample);
    let gain = if area_idx == 0 {
        1.0
    } else {
        STIM_RESIDUAL_GAIN
    };
    scores.into_iter().map(|(_, s)| s * gain).collect()
}

fn encode_area0_winners(
    w_stim: &[f32],
    sample: &Sample,
    temperature: f32,
    seed: u64,
) -> Vec<CellId> {
    let scores = encode_area0_scores(w_stim, sample);
    if temperature > 0.0 {
        soft_k_wta(&scores, K_WTA, temperature, seed)
    } else {
        k_wta(&scores, K_WTA)
    }
}

/// Reward-modulated linear readout over the final area's winner set.
struct AreaReadout {
    w: Vec<f32>,
    b: f32,
    offset: CellId,
}

impl AreaReadout {
    fn new(len: usize, offset: CellId, _seed: u64) -> Self {
        Self {
            w: vec![0.0; len],
            b: 0.0,
            offset,
        }
    }

    fn logit(&self, winners: &[CellId]) -> f32 {
        let mut z = self.b;
        for &c in winners {
            if c >= self.offset {
                let local = (c - self.offset) as usize;
                if local < self.w.len() {
                    z += self.w[local];
                }
            }
        }
        z
    }

    fn predict(&self, winners: &[CellId]) -> bool {
        self.logit(winners) > 0.0
    }

    /// Returns the prediction error used to drive inter-area plasticity.
    ///
    /// Supervised logistic update on the linear readout (same form as a
    /// reward-prediction error with a greedy policy). REINFORCE on this tiny
    /// winner-set feature was collapsing to a constant majority-class
    /// predictor under the quick schedule.
    fn update(&mut self, winners: &[CellId], truth: bool) -> f32 {
        let p = sigmoid(self.logit(winners));
        let y = f32::from(truth);
        let err = y - p;
        for &c in winners {
            if c >= self.offset {
                let local = (c - self.offset) as usize;
                if local < self.w.len() {
                    self.w[local] += ETA_READOUT * err;
                }
            }
        }
        self.b += ETA_READOUT * err;
        err
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_quick = args.iter().any(|a| a == "--quick");
    let bench_scaling = args.iter().any(|a| a == "--bench-scaling");
    let eval_scaling = args.iter().any(|a| a == "--eval-scaling") || !bench_scaling;

    let out_path = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let mut report_lines = Vec::new();
    report_lines.push("# Multi-Area Structural Scaling Report\n\n".to_string());

    if eval_scaling {
        println!("=== Evaluating Multi-Area Structural Scaling ===");
        let n_epochs = if is_quick { 10 } else { 30 };
        let n_total = if is_quick { 200 } else { 400 };
        let n_train = n_total * 3 / 4;
        let area_counts = if is_quick { vec![2, 4] } else { vec![2, 4, 8] };

        // Dataset seed is FIXED: area count is the only independent variable.
        // Binary, roughly balanced labels so a constant predictor sits near 0.5
        // (toy defaults are 3-class; `label != 0` then has ~2/3 majority).
        let mut stream = SyntheticStream::new(SynthConfig {
            seed: DATASET_SEED,
            n_features: 8,
            n_classes: 2,
            sequence_len: 1,
            difficulty: 0.15,
            depth: 1,
        });
        let dataset: Vec<Sample> = (0..n_total).map(|_| stream.next_sample()).collect();
        let (train_samples, test_samples) = dataset.split_at(n_train);
        let n_features = dataset[0].values.len();
        let w_stim = stimulus_projection(n_features, DATASET_SEED);

        report_lines.push(format!(
            "## Learning accuracy across multi-area depths\n\n\
             Dataset seed fixed at `{DATASET_SEED}` across all area counts \
             (n_train={}, n_test={}); area count is the only independent variable.\n\n",
            train_samples.len(),
            test_samples.len()
        ));
        report_lines.push(
            "| Area count (M) | Total cells | Train accuracy | Test accuracy | 95% CI | \
             Constant-predictor baseline | Verdict |\n\
             |---:|---:|---:|---:|---|---:|---|\n"
                .to_string(),
        );

        let mut audits = Vec::new();

        for n_areas in area_counts {
            let annealer = WtaAnnealer::new(2.0, 0.1, n_epochs);
            let mut net = MultiAreaNetwork::new(n_areas, CELLS_PER_AREA, K_WTA);

            for i in 0..(n_areas - 1) {
                let (csr, weights) =
                    inter_area_projection(DATASET_SEED ^ ((i as u64 + 1) * 0x9E37_79B9));
                net.add_projection(i, i + 1, csr, weights);
            }

            let total_cells = n_areas * CELLS_PER_AREA;
            let mut engine = Engine::with_cells(total_cells);
            let mut learner = MultiAreaLearner::new(0.05, 0.01, 0.0, 20.0, 42);
            let final_offset = ((n_areas - 1) * CELLS_PER_AREA) as CellId;
            let mut readout = AreaReadout::new(CELLS_PER_AREA, final_offset, 42);

            let mut final_train_acc = 0.0f32;

            for epoch in 0..n_epochs {
                let temp = annealer.temperature_at(epoch);
                let mut epoch_correct = 0usize;

                for (s_idx, sample) in train_samples.iter().enumerate() {
                    engine.reset_state();

                    // Stimulus-dependent area-0 activation (annealed soft-WTA).
                    let mut prev_winners = encode_area0_winners(
                        &w_stim,
                        sample,
                        temp,
                        1000 + epoch as u64 * 7919 + s_idx as u64,
                    );
                    let mut winner_trace: Vec<Vec<CellId>> = vec![prev_winners.clone()];

                    for i in 0..(n_areas - 1) {
                        let bias = area_stimulus_bias(&w_stim, sample, i + 1);
                        prev_winners = net.fused_inter_area_step(
                            &mut engine,
                            i,
                            i + 1,
                            &prev_winners,
                            InterAreaStepOpts::new(
                                temp,
                                2000 + epoch as u64 * 7919 + s_idx as u64 + i as u64,
                            )
                            .with_bias(&bias),
                        );
                        winner_trace.push(prev_winners.clone());
                    }

                    let truth = sample.label != Some(0);
                    if readout.predict(&prev_winners) == truth {
                        epoch_correct += 1;
                    }
                    let rpe = readout.update(&prev_winners, truth);

                    // Feedback-aligned plasticity driven by REAL per-area
                    // activity, not `vec![1.0; n]` dummies.
                    for i in 0..(n_areas - 1) {
                        let pre_offset = (i * CELLS_PER_AREA) as CellId;
                        let post_offset = ((i + 1) * CELLS_PER_AREA) as CellId;
                        let pre_act =
                            winners_to_activity(&winner_trace[i], pre_offset, CELLS_PER_AREA);
                        let post_act =
                            winners_to_activity(&winner_trace[i + 1], post_offset, CELLS_PER_AREA);
                        if let Some(proj) = net.projections.get_mut(i) {
                            learner.update_inter_area_weights(
                                &proj.conn,
                                &mut proj.weights,
                                &proj.feedback_b,
                                rpe,
                                &pre_act,
                                &post_act,
                            );
                            learner.update_inter_area_feedback(
                                &proj.conn,
                                &mut proj.feedback_b,
                                rpe,
                                &pre_act,
                                &post_act,
                            );
                        }
                    }
                }

                final_train_acc = epoch_correct as f32 / train_samples.len() as f32;
            }

            // ---- Held-out evaluation, fully audited ----
            let mut probe = StimulusProbe::new();
            let mut predictions = Vec::with_capacity(test_samples.len());
            let mut truths = Vec::with_capacity(test_samples.len());

            for (s_idx, sample) in test_samples.iter().enumerate() {
                engine.reset_state();
                // Hard WTA at eval: deterministic, score-driven winners.
                let mut prev_winners = encode_area0_winners(&w_stim, sample, 0.0, 0);
                for i in 0..(n_areas - 1) {
                    let bias = area_stimulus_bias(&w_stim, sample, i + 1);
                    prev_winners = net.fused_inter_area_step(
                        &mut engine,
                        i,
                        i + 1,
                        &prev_winners,
                        InterAreaStepOpts::new(0.0, 9999 + s_idx as u64).with_bias(&bias),
                    );
                }
                // If the stimulus is not propagating, every final winner set is
                // identical and the audit fails instead of reporting balance.
                probe.observe_indices(&prev_winners);
                predictions.push(readout.predict(&prev_winners));
                truths.push(sample.label != Some(0));
            }

            let audit = ReadoutAudit::new(&predictions, &truths, Some(&probe));
            println!("Areas M={n_areas:2} | {total_cells:4} cells | train={final_train_acc:.4} | {audit}");
            audit.assert_non_degenerate(&format!("multi-area-M{n_areas}"));
            let verdict = Verdict::evaluate(&audit, ACCURACY_FLOOR, true);

            report_lines.push(format!(
                "| M = {n_areas} | {total_cells} | {final_train_acc:.4} | {:.4} | [{:.4}, {:.4}] | {:.4} | {} |\n",
                audit.accuracy,
                audit.accuracy_lcb95,
                audit.accuracy_ucb95,
                audit.constant_predictor_accuracy,
                verdict.label(),
            ));
            audits.push((n_areas, audit));
        }

        report_lines.push("\n### Readout audit\n\n".to_string());
        report_lines.push(format!("{}\n", ReadoutAudit::markdown_header()));
        for (m, audit) in &audits {
            report_lines.push(format!("{}\n", audit.markdown_row(&format!("M = {m}"))));
        }

        // Scaling is a claim about differences between area counts. State
        // explicitly whether the CIs even permit such a claim.
        if audits.len() >= 2 {
            let (m_lo, a_lo) = &audits[0];
            let (m_hi, a_hi) = &audits[audits.len() - 1];
            let separated = a_hi.accuracy_lcb95 > a_lo.accuracy_ucb95
                || a_lo.accuracy_lcb95 > a_hi.accuracy_ucb95;
            report_lines.push(format!(
                "\n**Scaling separation (M = {m_lo} vs M = {m_hi}):** {}. \
                 M={m_lo} 95% CI [{:.4}, {:.4}]; M={m_hi} 95% CI [{:.4}, {:.4}]. \
                 A scaling claim requires non-overlapping intervals.\n\n",
                if separated {
                    "intervals are disjoint"
                } else {
                    "intervals OVERLAP — no scaling effect is supported by this data"
                },
                a_lo.accuracy_lcb95,
                a_lo.accuracy_ucb95,
                a_hi.accuracy_lcb95,
                a_hi.accuracy_ucb95,
            ));
        }
    }

    if bench_scaling || args.len() == 1 {
        let arms = benchmarkable_backends();
        println!("\n=== Multi-area SpMV throughput benchmark ===");
        println!(
            "Available backends: {}",
            arms.iter()
                .map(|b| b.label())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let counts = if is_quick { vec![2, 4] } else { vec![4, 8, 16] };
        let cells_per_area = 500;

        report_lines.push("## Multi-area throughput\n\n".to_string());
        if arms.len() < 2 {
            report_lines.push(format!(
                "> Only one backend is available (**{}**). Metal GPU dispatch is unimplemented \
                 (`binn_core::metal_backend::METAL_GPU_DISPATCH_IMPLEMENTED == false`), so no \
                 GPU column is emitted.\n\n",
                arms[0].label()
            ));
        }

        let mut header = String::from("| Area count (M) | Total cells | nnz |");
        let mut sep = String::from("|---:|---:|---:|");
        for a in &arms {
            header.push_str(&format!(" {} (ms/step) |", a.label()));
            sep.push_str("---:|");
        }
        report_lines.push(format!("{header}\n{sep}\n"));

        for m in counts {
            let total_cells = m * cells_per_area;
            let density = 0.05;
            let nnz_per_row = ((total_cells as f32) * density) as usize;
            let mut adj = vec![Vec::new(); total_cells];
            for r in 0..total_cells {
                for i in 0..nnz_per_row {
                    adj[r].push(((r + i * 3) % total_cells) as u32);
                }
            }
            let csr = Csr::from_adjacency(&adj);
            let weights = vec![0.1f32; csr.nnz()];
            let x = vec![1.0f32; total_cells];
            let iterations = if is_quick { 30 } else { 100 };

            let mut row = format!("| M = {m} | {total_cells} | {} |", csr.nnz());
            for &arm in &arms {
                let backend = SpmvBackend::new(SpmvBackendConfig {
                    backend: arm,
                    batch_size: 1024,
                });
                let mut y = vec![0.0f32; total_cells];
                backend.spmv(&csr, &weights, &x, &mut y);

                let mut y = vec![0.0f32; total_cells];
                let start = Instant::now();
                for _ in 0..iterations {
                    backend.spmv(&csr, &weights, &x, &mut y);
                }
                let ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
                println!(
                    "M = {m:2} ({total_cells:5} cells): {} = {ms:.3} ms/step",
                    backend.label()
                );
                row.push_str(&format!(" {ms:.3} |"));
            }
            report_lines.push(format!("{row}\n"));
        }
        report_lines.push("\n".to_string());
    }

    if let Some(path) = out_path {
        let mut file = File::create(&path).expect("failed to create report file");
        for line in &report_lines {
            file.write_all(line.as_bytes()).expect("write error");
        }
        println!("\nReport written to: {path}");
    }
}
