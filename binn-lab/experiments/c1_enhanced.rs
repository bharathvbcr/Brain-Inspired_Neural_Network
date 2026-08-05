//! Live spiking engine experiment binary & SpMV throughput benchmark.
//!
//! Evaluates the live-engine enhancements (soft-WTA annealing, finite adaptive
//! thresholds, dendritic compartments, trial-reset hygiene) with a
//! reward-modulated linear readout over the k-WTA winner set, and benchmarks
//! SpMV throughput across the backends that are *actually available*.
//!
//! # 2026-07-25 rewrite
//!
//! The previous version of this binary was not measuring learning:
//!
//! * The sample was never injected into the engine. `engine.reset_state()` was
//!   called and then membrane values were read, so every trial saw identical
//!   state.
//! * `predicted = !winners.is_empty()` is constant by construction — `soft_k_wta`
//!   returns a non-empty set whenever `k > 0` and `scores` is non-empty.
//! * The training loop applied no weight updates.
//!
//! The reported "0.8500 accuracy" was therefore the positive-class fraction of a
//! 20-sample test split. This version injects the stimulus, learns a readout,
//! evaluates on ≥ `MIN_EVAL_SAMPLES` held-out samples, and refuses to write a
//! report if [`ReadoutAudit`] detects a degenerate readout.

#![allow(clippy::needless_range_loop)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use binn_areas::wta::{soft_k_wta, WtaAnnealer};
use binn_core::metal_backend::{benchmarkable_backends, SpmvBackend, SpmvBackendConfig};
use binn_core::sparse::Csr;
use binn_core::Rng;
use binn_data::{Sample, SynthConfig, SyntheticStream};
use binn_engine::{CellId, Engine};
use binn_lab::guards::{ReadoutAudit, StimulusProbe, Verdict};

/// Preregistered accuracy floor for the live readout arm.
const ACCURACY_FLOOR: f32 = 0.65;
/// Ticks of stimulus presentation per trial.
const STIM_TICKS: u64 = 8;
/// Hidden cells driven by the stimulus.
const HIDDEN: usize = 64;
/// k-WTA winner budget.
const K_WTA: usize = 8;
/// Input drive gain (keeps membranes in an informative sub/near-threshold band).
const INPUT_GAIN: f32 = 0.35;
/// Readout learning rate.
const ETA_READOUT: f32 = 0.05;

fn sigmoid(z: f32) -> f32 {
    1.0 / (1.0 + (-z).exp())
}

/// Fixed, seeded input projection: `n_features -> HIDDEN`.
fn input_projection(n_features: usize, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed ^ 0x1_9E37_79B9);
    (0..HIDDEN * n_features)
        .map(|_| rng.next_f32() * 2.0 - 1.0)
        .collect()
}

/// Drive the engine with one sample and return the readout score vector.
///
/// This is the step the previous version omitted entirely.
fn present_stimulus(engine: &mut Engine, w_in: &[f32], sample: &Sample) -> Vec<(CellId, f32)> {
    engine.reset_state();
    let n_features = sample.values.len();

    for t in 0..STIM_TICKS {
        for i in 0..HIDDEN {
            let mut drive = 0.0f32;
            for j in 0..n_features {
                drive += w_in[i * n_features + j] * sample.values[j];
            }
            let amount = drive * INPUT_GAIN;
            if amount.abs() > 1e-6 {
                engine.inject_weighted(i as CellId, 0, t, amount);
            }
        }
    }
    engine.step_until(STIM_TICKS);

    (0..HIDDEN as CellId)
        .map(|id| {
            let c = engine.cell(id);
            (id, c.v + c.dendritic_coincidence_score())
        })
        .collect()
}

/// Reward-modulated linear readout over the k-WTA winner set.
struct WinnerReadout {
    w: Vec<f32>,
    b: f32,
    rng: Rng,
}

impl WinnerReadout {
    fn new(seed: u64) -> Self {
        Self {
            w: vec![0.0; HIDDEN],
            b: 0.0,
            rng: Rng::new(seed ^ 0x00C1_E5A0_0001),
        }
    }

    fn logit(&self, winners: &[CellId]) -> f32 {
        self.b + winners.iter().map(|&i| self.w[i as usize]).sum::<f32>()
    }

    fn predict(&self, winners: &[CellId]) -> bool {
        self.logit(winners) > 0.0
    }

    /// REINFORCE update: sample an action, reward it, credit only the winners.
    fn update(&mut self, winners: &[CellId], truth: bool) {
        let p = sigmoid(self.logit(winners));
        let action = self.rng.next_f32() < p;
        let reward = if action == truth { 1.0f32 } else { -1.0 };
        let directional = reward * (f32::from(action) - p);
        for &i in winners {
            self.w[i as usize] += ETA_READOUT * directional;
        }
        self.b += ETA_READOUT * directional;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_quick = args.iter().any(|a| a == "--quick");
    let bench_spmv = args
        .iter()
        .any(|a| a == "--bench-spmv" || a == "--bench-metal");
    let eval_enhanced = args.iter().any(|a| a == "--eval-enhanced") || !bench_spmv;

    let out_path = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let mut report_lines = Vec::new();
    report_lines.push("# Live Spiking Engine Report\n\n".to_string());

    if eval_enhanced {
        println!("=== Evaluating Live Spiking Engine (stimulus-driven readout) ===");
        let n_epochs = if is_quick { 10 } else { 40 };
        let n_total = if is_quick { 200 } else { 400 };
        let n_train = n_total * 3 / 4;
        let annealer = WtaAnnealer::new(2.0, 0.1, n_epochs);

        let mut stream = SyntheticStream::new(SynthConfig::toy(42));
        let dataset: Vec<Sample> = (0..n_total).map(|_| stream.next_sample()).collect();
        let (train_samples, test_samples) = dataset.split_at(n_train);

        let n_features = dataset[0].values.len();
        let w_in = input_projection(n_features, 42);

        let mut engine = Engine::with_cells(HIDDEN + 2);
        let mut readout = WinnerReadout::new(42);

        let start_time = Instant::now();

        for epoch in 0..n_epochs {
            let temp = annealer.temperature_at(epoch);
            let mut epoch_correct = 0usize;

            for (i, sample) in train_samples.iter().enumerate() {
                let scores = present_stimulus(&mut engine, &w_in, sample);
                let winners = soft_k_wta(&scores, K_WTA, temp, 1337 + epoch as u64 + i as u64);
                let truth = sample.label != Some(0);
                if readout.predict(&winners) == truth {
                    epoch_correct += 1;
                }
                readout.update(&winners, truth);
            }

            if epoch % 5 == 0 || epoch + 1 == n_epochs {
                println!(
                    "Epoch {:2}/{} (T = {:.3}): train acc = {:.4}",
                    epoch + 1,
                    n_epochs,
                    temp,
                    epoch_correct as f32 / train_samples.len() as f32
                );
            }
        }

        // ---- Held-out evaluation, fully audited ----
        let mut probe = StimulusProbe::new();
        let mut predictions = Vec::with_capacity(test_samples.len());
        let mut truths = Vec::with_capacity(test_samples.len());

        for sample in test_samples {
            let scores = present_stimulus(&mut engine, &w_in, sample);
            // Fingerprint what the readout is about to consume. If the stimulus
            // is not reaching the network, every fingerprint is identical and
            // the audit fails loudly instead of reporting class balance.
            let score_values: Vec<f32> = scores.iter().map(|(_, s)| *s).collect();
            probe.observe_f32(&score_values);

            let winners = soft_k_wta(&scores, K_WTA, 0.1, 9999);
            predictions.push(readout.predict(&winners));
            truths.push(sample.label != Some(0));
        }

        let audit = ReadoutAudit::new(&predictions, &truths, Some(&probe));
        let elapsed = start_time.elapsed().as_secs_f32();

        println!("\nLive readout audit: {audit}");
        // Hard stop: a degenerate readout must never reach a report.
        audit.assert_non_degenerate("live-engine-readout");

        let verdict = Verdict::evaluate(&audit, ACCURACY_FLOOR, true);

        report_lines.push(format!(
            "## Live readout (stimulus-driven)\n\n\
             - **Test accuracy**: {:.4} (95% CI [{:.4}, {:.4}], n={})\n\
             - **Constant-predictor baseline**: {:.4}\n\
             - **Beats constant predictor (95%)**: {}\n\
             - **Distinct pre-readout states**: {} / {} samples\n\
             - **Accuracy floor**: {ACCURACY_FLOOR:.2}\n\
             - **Verdict**: {}\n\
             - **Execution time**: {:.3}s\n\n",
            audit.accuracy,
            audit.accuracy_lcb95,
            audit.accuracy_ucb95,
            audit.n,
            audit.constant_predictor_accuracy,
            audit.beats_constant_predictor(),
            probe.n_distinct_states(),
            probe.n_observed(),
            verdict.label(),
            elapsed,
        ));

        report_lines.push("### Readout audit\n\n".to_string());
        report_lines.push(format!("{}\n", ReadoutAudit::markdown_header()));
        report_lines.push(format!("{}\n\n", audit.markdown_row("live-engine-readout")));
    }

    if bench_spmv || args.len() == 1 {
        let arms = benchmarkable_backends();
        println!("\n=== SpMV throughput benchmark ===");
        println!(
            "Available backends: {}",
            arms.iter()
                .map(|b| b.label())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let sizes = if is_quick {
            vec![500, 2000]
        } else {
            vec![1000, 5000, 10000]
        };

        report_lines.push("## SpMV throughput\n\n".to_string());
        if arms.len() < 2 {
            report_lines.push(format!(
                "> Only one backend is available (**{}**), so no cross-substrate speedup is \
                 reported. Metal GPU dispatch is unimplemented \
                 (`binn_core::metal_backend::METAL_GPU_DISPATCH_IMPLEMENTED == false`); the \
                 harness refuses to emit a GPU column produced by CPU code.\n\n",
                arms[0].label()
            ));
        }

        let mut header = String::from("| Network size (N) | nnz |");
        let mut sep = String::from("|---:|---:|");
        for a in &arms {
            header.push_str(&format!(" {} (ms/iter) |", a.label()));
            sep.push_str("---:|");
        }
        report_lines.push(format!("{header}\n{sep}\n"));

        for n in sizes {
            let density = 0.05;
            let nnz_per_row = ((n as f32) * density) as usize;
            let mut adj = vec![Vec::new(); n];
            for r in 0..n {
                for i in 0..nnz_per_row {
                    adj[r].push(((r + i * 3) % n) as u32);
                }
            }
            let csr = Csr::from_adjacency(&adj);
            let weights = vec![0.1f32; csr.nnz()];
            let x = vec![1.0f32; n];
            let iterations = if is_quick { 50 } else { 200 };

            let mut row = format!("| N = {n} | {} |", csr.nnz());
            for &arm in &arms {
                let backend = SpmvBackend::new(SpmvBackendConfig {
                    backend: arm,
                    batch_size: 1024,
                });
                // Warm-up so the first arm does not absorb allocation costs.
                let mut y = vec![0.0f32; n];
                backend.spmv(&csr, &weights, &x, &mut y);

                let mut y = vec![0.0f32; n];
                let start = Instant::now();
                for _ in 0..iterations {
                    backend.spmv(&csr, &weights, &x, &mut y);
                }
                let ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
                println!("N = {n:5}: {} = {ms:.3} ms/iter", backend.label());
                row.push_str(&format!(" {ms:.3} |"));
            }
            report_lines.push(format!("{row}\n"));
        }
        report_lines.push("\n".to_string());
    }

    if let Some(path) = out_path {
        let mut file = File::create(&path).expect("failed to create report output file");
        for line in &report_lines {
            file.write_all(line.as_bytes()).expect("write report error");
        }
        println!("\nReport written to: {path}");
    }
}
