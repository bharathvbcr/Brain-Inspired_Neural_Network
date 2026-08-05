//! U18-U20 / Gate G5 efficiency harness + P2 Engine F1/F5 systems.
//!
//! Reports deterministic partitioned-engine parity, reset-aware scan timing /
//! barrier headroom (F1), and C1 sparse-vs-dense work-per-accuracy with
//! activity≠compute accounting (F5). It never calls the modeled work proxy
//! "energy". Does **not** reopen G2/G4.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use binn_core::Csr;
use binn_data::{Metrics, WorkCosts};
use binn_engine::{Engine, ParallelismProfile, PartitionPlan, PARALLEL_CELL_THRESHOLD};
use binn_lab::{ConditionLabel, Config, Runner};
use binn_learn::forward_scan_training;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let enabled = args.iter().any(|arg| arg == "--enable-efficiency")
        || env::var("BINN_OVERRIDE_G2_FOR")
            .map(|v| {
                v.split(',')
                    .any(|p| matches!(p.trim().to_ascii_lowercase().as_str(), "u20" | "all"))
            })
            .unwrap_or(false);
    let quick = args.iter().any(|arg| arg == "--quick");
    let out = args
        .windows(2)
        .find(|pair| pair[0] == "--out")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| {
            PathBuf::from(if quick {
                "results/u20_efficiency_quick.md"
            } else {
                "results/u20_efficiency.md"
            })
        });
    let f1f5_out = args
        .windows(2)
        .find(|pair| pair[0] == "--f1-f5-out")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| {
            PathBuf::from(if quick {
                "results/f1_f5_systems_quick.md"
            } else {
                "results/f1_f5_systems.md"
            })
        });
    if !enabled {
        eprintln!("U18-U20 require --enable-efficiency (post-G2 exploratory override)");
        return ExitCode::from(2);
    }

    let n_cells = if quick { 128 } else { 512 };
    let (parallel_parity, sequential_secs, partitioned_secs, cut_edges, profile, always_par_secs) =
        partitioned_benchmark(n_cells);
    let scan_steps = if quick { 10_000 } else { 250_000 };
    let (scan_secs, scan_trace) = scan_benchmark(scan_steps);

    let config = if quick {
        Config::c1_quick()
    } else {
        Config::c1_default()
    };
    let report = Runner::new().run_c1(&config);
    let local = budget(&report.budgets, ConditionLabel::LocalAssembly);
    let dense = budget(&report.budgets, ConditionLabel::DenseLocal);
    let matched = report
        .budgets
        .iter()
        .find(|(label, _)| *label == ConditionLabel::DenseMatched)
        .map(|(_, budget)| budget);
    let matched_values: Vec<f32> = report
        .seeds
        .iter()
        .filter_map(|seed| seed.dense_matched)
        .collect();
    let matched_accuracy = (!matched_values.is_empty())
        .then(|| matched_values.iter().sum::<f32>() / matched_values.len() as f32);
    let accuracy_matched = matched_accuracy
        .map(|accuracy| report.summary.mean_local + 1e-6 >= accuracy)
        .unwrap_or(false);
    let work_better = matched
        .map(|budget| local.work_per_accuracy < budget.work_per_accuracy)
        .unwrap_or_else(|| local.work_per_accuracy < dense.work_per_accuracy);
    let g5_pass = !quick && accuracy_matched && work_better;

    let local_f5 = Metrics::activity_compute_account(
        local.work,
        WorkCosts::unit(),
        local.n_cells,
        report.mean_activity_sparsity,
    );
    let dense_sparsity = report
        .seeds
        .iter()
        .map(|s| s.dense_activity_sparsity)
        .sum::<f32>()
        / report.seeds.len().max(1) as f32;
    let dense_f5 = Metrics::activity_compute_account(
        dense.work,
        WorkCosts::unit(),
        dense.n_cells,
        dense_sparsity,
    );
    let matched_f5 = matched.map(|b| {
        Metrics::activity_compute_account(b.work, WorkCosts::unit(), b.n_cells, dense_sparsity)
    });

    let speedup = if partitioned_secs > 0.0 {
        sequential_secs / partitioned_secs
    } else {
        0.0
    };
    let always_par_speedup = if always_par_secs > 0.0 {
        sequential_secs / always_par_secs
    } else {
        0.0
    };

    let markdown = format!(
        "# U18-U20 / Gate G5 — throughput and efficiency\n\n\
         **Exploratory post-G2 override.** C1 Gate G2 FAIL remains unchanged.\n\n\
         - schedule: {}\n\
         - C1 config hash: `{}`\n\
         - verdict: **{}**\n\
         - activity sparsity: {:.4}\n\
         - local accuracy: {:.4}\n\
         - parameter-matched dense accuracy: {}\n\n\
         ## U18 partitioned delta engine (F1 adaptive)\n\n\
         | parity with sequential | graph cut edges | sequential seconds | adaptive partitioned seconds | always-rayon seconds |\n\
         |---|---:|---:|---:|---:|\n\
         | {} | {} | {:.6} | {:.6} | {:.6} |\n\n\
         Parallel threshold: `PARALLEL_CELL_THRESHOLD={}` distinct cells/tick.\n\n\
         ## U19 reset-aware scan training (F1 barriers)\n\n\
         | scanned steps | reset-free segments | sequential reset barriers | mean seg len | max seg len | barrier fraction | scan headroom | wall seconds |\n\
         |---:|---:|---:|---:|---:|---:|---:|---:|\n\
         | {} | {} | {} | {:.2} | {} | {:.6} | {:.6} | {:.6} |\n\n\
         ## U20 measured work disclosure\n\n\
         | condition | accuracy | modeled work/accuracy | wall seconds | peak RSS bytes |\n\
         |---|---:|---:|---:|---:|\n\
         | local assembly | {:.4} | {:.4} | {:.6} | {} |\n\
         | dense local | {:.4} | {:.4} | {:.6} | {} |\n\
         | dense parameter-matched | {} | {} | {} | {} |\n\n\
         ## F5 activity≠compute accounting\n\n\
         | condition | activity sparsity | event_work | naive_activity_work (N×a) | work_vs_activity_ratio | source_spikes | synaptic_deliveries |\n\
         |---|---:|---:|---:|---:|---:|---:|\n\
         | local assembly | {:.4} | {:.1} | {:.1} | {:.2} | {} | {} |\n\
         | dense local | {:.4} | {:.1} | {:.1} | {:.2} | {} | {} |\n\
         | dense parameter-matched | {} | {} | {} | {} | {} | {} |\n\n\
         Modeled work uses disjoint source-spike, delivery, cell-update, and \
         plasticity-update counters. It is a work proxy, **not hardware energy**. \
         Gate G5 requires lower work/accuracy at matched accuracy and disclosed sparsity. \
         F5 ratio ≫ 1 means sparse activity understates event/queue work.\n",
        if quick { "PILOT" } else { "scientific" },
        config.hash_string(),
        if quick {
            "PILOT"
        } else if g5_pass {
            "PASS"
        } else {
            "FAIL"
        },
        report.mean_activity_sparsity,
        report.summary.mean_local,
        matched_accuracy
            .map(|accuracy| format!("{accuracy:.4}"))
            .unwrap_or_else(|| "not-run".into()),
        parallel_parity,
        cut_edges,
        sequential_secs,
        partitioned_secs,
        always_par_secs,
        PARALLEL_CELL_THRESHOLD,
        scan_steps,
        scan_trace.segments,
        scan_trace.reset_barriers,
        scan_trace.mean_segment_len,
        scan_trace.max_segment_len,
        scan_trace.barrier_fraction,
        scan_trace.scan_headroom,
        scan_secs,
        report.summary.mean_local,
        local.work_per_accuracy,
        local.wall_secs,
        local.peak_rss_bytes,
        report.summary.mean_dense,
        dense.work_per_accuracy,
        dense.wall_secs,
        dense.peak_rss_bytes,
        matched_accuracy
            .map(|accuracy| format!("{accuracy:.4}"))
            .unwrap_or_else(|| "not-run".into()),
        matched
            .map(|b| format!("{:.4}", b.work_per_accuracy))
            .unwrap_or_else(|| "not-run".into()),
        matched
            .map(|b| format!("{:.6}", b.wall_secs))
            .unwrap_or_else(|| "not-run".into()),
        matched
            .map(|b| b.peak_rss_bytes.to_string())
            .unwrap_or_else(|| "not-run".into()),
        report.mean_activity_sparsity,
        local_f5.event_work,
        local_f5.naive_activity_work,
        local_f5.work_vs_activity_ratio,
        local.work.source_spikes,
        local.work.synaptic_deliveries,
        dense_sparsity,
        dense_f5.event_work,
        dense_f5.naive_activity_work,
        dense_f5.work_vs_activity_ratio,
        dense.work.source_spikes,
        dense.work.synaptic_deliveries,
        matched_accuracy
            .map(|_| format!("{dense_sparsity:.4}"))
            .unwrap_or_else(|| "not-run".into()),
        matched_f5
            .map(|a| format!("{:.1}", a.event_work))
            .unwrap_or_else(|| "not-run".into()),
        matched_f5
            .map(|a| format!("{:.1}", a.naive_activity_work))
            .unwrap_or_else(|| "not-run".into()),
        matched_f5
            .map(|a| format!("{:.2}", a.work_vs_activity_ratio))
            .unwrap_or_else(|| "not-run".into()),
        matched
            .map(|b| b.work.source_spikes.to_string())
            .unwrap_or_else(|| "not-run".into()),
        matched
            .map(|b| b.work.synaptic_deliveries.to_string())
            .unwrap_or_else(|| "not-run".into()),
    );
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(error) = fs::write(&out, &markdown) {
        eprintln!("failed to write {}: {error}", out.display());
        return ExitCode::from(1);
    }

    let systems_note = format!(
        "# Engine F1 / F5 systems (P2 engineering)\n\n\
         **Non-claims:** engineering measurement only — not biology, not a G2/G4 reopen, \
         not a neuromorphic energy claim. Modeled work is a software work proxy.\n\n\
         - schedule: {}\n\
         - companion U18–U20 note: `{}`\n\
         - C1 config hash: `{}` (unchanged kill-gate family)\n\n\
         ## F1 — reset-barrier / parallelism headroom\n\n\
         ### Same-tick engine delta buckets\n\n\
         | metric | value |\n\
         |---|---:|\n\
         | sequential wall (s) | {:.6} |\n\
         | adaptive partitioned wall (s) | {:.6} |\n\
         | always-rayon partitioned wall (s) | {:.6} |\n\
         | adaptive / sequential speedup | {:.3} |\n\
         | always-rayon / sequential speedup | {:.3} |\n\
         | parity with sequential | {} |\n\
         | ticks with events | {} |\n\
         | parallel ticks (≥ threshold) | {} |\n\
         | sequential thin ticks | {} |\n\
         | mean width (distinct cells/tick) | {:.2} |\n\
         | max width | {} |\n\
         | width headroom (mean/threshold, cap 1) | {:.3} |\n\
         | PARALLEL_CELL_THRESHOLD | {} |\n\n\
         **Reading:** width headroom near 1 means many buckets meet the parallel \
         threshold; values ≪ 1 mean the stream is thin-tick dominated. Cross-tick \
         spike reset / fan-out remains a sequential barrier. Adaptive path skips \
         rayon on thin ticks (safe; determinism preserved).\n\n\
         ### Reset-aware scan (U19)\n\n\
         | metric | value |\n\
         |---|---:|\n\
         | steps | {} |\n\
         | reset barriers | {} |\n\
         | segments | {} |\n\
         | mean segment length | {:.2} |\n\
         | max segment length | {} |\n\
         | parallelizable steps (len > chunk) | {} |\n\
         | barrier fraction | {:.6} |\n\
         | scan headroom (1 − barrier fraction) | {:.6} |\n\
         | wall seconds | {:.6} |\n\n\
         ## F5 — activity ≠ compute\n\n\
         | condition | activity | event_work | naive N×a | ratio |\n\
         |---|---:|---:|---:|---:|\n\
         | local | {:.4} | {:.1} | {:.1} | {:.2} |\n\
         | dense-local | {:.4} | {:.1} | {:.1} | {:.2} |\n\n\
         Ratio ≫ 1: counting active cells understates queue/delivery/update work.\n\n\
         ## How to reproduce\n\n\
         ```bash\n\
         cargo run --release -p binn-lab --bin efficiency -- --enable-efficiency --out results/u20_efficiency.md\n\
         cargo bench -p binn-engine --bench f1_parallelism\n\
         cargo test -p binn-engine -p binn-learn --lib\n\
         cargo test -p binn-lab --test override_refuse\n\
         # optional Polars summary:\n\
         cargo test -p binn-lab --features tables harvest -- --nocapture\n\
         ```\n\n\
         ## Remaining limits\n\n\
         - Cross-tick causality (synaptic delay + spike reset) is still sequential.\n\
         - Scan headroom is a timeline fraction, not measured wall-clock speedup.\n\
         - Adaptive rayon helps thin streams vs always-rayon; delta-bucket grouping \
           still costs vs bare `step_until`, so sequential often remains fastest on CPU.\n\
         - Synaptic fan-out can widen ticks and erase thin-tick headroom — F1 timing \
           microbench uses a no-cascade schedule to isolate the barrier.\n\
         - G5 FAIL / G2 FAIL stand; this note does not reinterpret kill gates.\n",
        if quick { "PILOT" } else { "scientific" },
        out.display(),
        config.hash_string(),
        sequential_secs,
        partitioned_secs,
        always_par_secs,
        speedup,
        always_par_speedup,
        parallel_parity,
        profile.ticks_with_events,
        profile.parallel_ticks,
        profile.sequential_ticks,
        profile.mean_width(),
        profile.max_width,
        profile.width_headroom(),
        PARALLEL_CELL_THRESHOLD,
        scan_steps,
        scan_trace.reset_barriers,
        scan_trace.segments,
        scan_trace.mean_segment_len,
        scan_trace.max_segment_len,
        scan_trace.parallelizable_steps,
        scan_trace.barrier_fraction,
        scan_trace.scan_headroom,
        scan_secs,
        report.mean_activity_sparsity,
        local_f5.event_work,
        local_f5.naive_activity_work,
        local_f5.work_vs_activity_ratio,
        dense_sparsity,
        dense_f5.event_work,
        dense_f5.naive_activity_work,
        dense_f5.work_vs_activity_ratio,
    );
    if let Some(parent) = f1f5_out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(error) = fs::write(&f1f5_out, systems_note) {
        eprintln!("failed to write {}: {error}", f1f5_out.display());
        return ExitCode::from(1);
    }

    // Optional Polars CSV summary when built with `--features tables`.
    #[cfg(feature = "tables")]
    {
        if let Err(error) = write_polars_summary(
            &f1f5_out.with_extension("csv"),
            sequential_secs,
            partitioned_secs,
            always_par_secs,
            &profile,
            &scan_trace,
            &local_f5,
            &dense_f5,
        ) {
            eprintln!("polars summary skipped: {error}");
        }
    }

    println!("U18 sequential parity: {parallel_parity}");
    println!("F1 adaptive speedup: {speedup:.3}x (always-rayon {always_par_speedup:.3}x)");
    println!(
        "F1 scan headroom: {:.4} (barriers {})",
        scan_trace.scan_headroom, scan_trace.reset_barriers
    );
    println!(
        "F5 local work_vs_activity: {:.2}",
        local_f5.work_vs_activity_ratio
    );
    println!(
        "G5 verdict: {}",
        if quick {
            "PILOT"
        } else if g5_pass {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!("results note: {}", out.display());
    println!("F1/F5 systems note: {}", f1f5_out.display());
    ExitCode::SUCCESS
}

fn budget(
    budgets: &[(ConditionLabel, binn_lab::BudgetDisclosure)],
    label: ConditionLabel,
) -> &binn_lab::BudgetDisclosure {
    budgets
        .iter()
        .find(|(candidate, _)| *candidate == label)
        .map(|(_, budget)| budget)
        .expect("condition budget")
}

fn build_connected_engines(n_cells: usize) -> (Csr, Engine, Engine, Engine) {
    // Disconnected graph for F1 timing: force-spikes do not cascade into
    // wide delivery buckets. Connectivity is still installed (empty CSR) so the
    // partitioned path exercises the real engine API; parity is checked against
    // sequential on the same schedule.
    let conn = Csr::from_adjacency(&(0..n_cells).map(|_| Vec::<u32>::new()).collect::<Vec<_>>());
    let weights: Vec<f32> = Vec::new();
    let mut sequential = Engine::with_cells(n_cells);
    sequential.set_connectivity(conn.clone(), weights.clone());
    let mut partitioned = Engine::with_cells(n_cells);
    partitioned.set_connectivity(conn.clone(), weights.clone());
    let mut always_par = Engine::with_cells(n_cells);
    always_par.set_connectivity(conn.clone(), weights);
    // Mostly thin ticks (1 cell), with occasional wide bursts (≥ threshold).
    let n_events = n_cells * 4;
    for i in 0..n_events {
        let in_burst = i % 64 < PARALLEL_CELL_THRESHOLD;
        let (cell, tick) = if in_burst {
            let burst_id = i / 64;
            let slot = i % PARALLEL_CELL_THRESHOLD;
            ((slot % n_cells) as u32, 1 + burst_id as u64)
        } else {
            ((i * 17 % n_cells) as u32, 1_000 + i as u64)
        };
        sequential.force_spike(cell, tick);
        partitioned.force_spike(cell, tick);
        always_par.force_spike(cell, tick);
    }
    (conn, sequential, partitioned, always_par)
}

fn partitioned_benchmark(n_cells: usize) -> (bool, f64, f64, usize, ParallelismProfile, f64) {
    let (conn, mut sequential, mut partitioned, mut always_par) = build_connected_engines(n_cells);
    let until = 1_000 + (n_cells * 4) as u64;
    let start = Instant::now();
    let seq_spikes = sequential.step_until(until);
    let sequential_secs = start.elapsed().as_secs_f64();
    let plan = PartitionPlan::degree_balanced(&conn, 4);
    let start = Instant::now();
    let (par_spikes, profile) = partitioned.step_until_partitioned_profiled(until, &plan);
    let partitioned_secs = start.elapsed().as_secs_f64();
    // Legacy always-rayon path on the same schedule (threshold=1 ⇒ every
    // non-empty bucket uses rayon). Fair before/after for F1 adaptive.
    let start = Instant::now();
    let (always_spikes, _) = always_par.step_until_partitioned_threshold(until, &plan, 1);
    let always_par_secs = start.elapsed().as_secs_f64();
    assert_eq!(
        always_spikes, seq_spikes,
        "always-rayon path must preserve sequential observables"
    );

    (
        seq_spikes == par_spikes && sequential.work() == partitioned.work(),
        sequential_secs,
        partitioned_secs,
        plan.cut_edges(),
        profile,
        always_par_secs,
    )
}

fn scan_benchmark(n_steps: usize) -> (f64, binn_learn::ScanTrainingTrace) {
    let inputs: Vec<f32> = (0..n_steps)
        .map(|i| ((i * 37 % 101) as f32 / 101.0) - 0.5)
        .collect();
    let resets: Vec<bool> = (0..n_steps).map(|i| i % 997 == 996).collect();
    let start = Instant::now();
    let trace = forward_scan_training(&inputs, &resets, 0.0, 0.0, 20.0, 1.0);
    (start.elapsed().as_secs_f64(), trace)
}

#[cfg(feature = "tables")]
fn write_polars_summary(
    path: &std::path::Path,
    sequential_secs: f64,
    partitioned_secs: f64,
    always_par_secs: f64,
    profile: &ParallelismProfile,
    scan: &binn_learn::ScanTrainingTrace,
    local_f5: &binn_data::ActivityComputeAccount,
    dense_f5: &binn_data::ActivityComputeAccount,
) -> Result<(), String> {
    use binn_lab::harvest::write_csv;
    use polars::prelude::*;

    let mut df = DataFrame::new(vec![
        Series::new(
            "metric".into(),
            vec![
                "sequential_secs",
                "adaptive_partitioned_secs",
                "always_rayon_secs",
                "mean_width",
                "width_headroom",
                "scan_barrier_fraction",
                "scan_headroom",
                "local_work_vs_activity",
                "dense_work_vs_activity",
            ],
        )
        .into(),
        Series::new(
            "value".into(),
            vec![
                sequential_secs,
                partitioned_secs,
                always_par_secs,
                profile.mean_width(),
                profile.width_headroom(),
                scan.barrier_fraction,
                scan.scan_headroom,
                local_f5.work_vs_activity_ratio,
                dense_f5.work_vs_activity_ratio,
            ],
        )
        .into(),
    ])
    .map_err(|e| e.to_string())?;
    write_csv(&mut df, path).map_err(|e| e.to_string())
}
