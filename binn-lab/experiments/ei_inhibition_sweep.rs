//! Dynamic E-I interneuron sweeps (Suite 3).
//!
//! # 2026-07-25 rewrite
//!
//! The previous version was not a test:
//!
//! * It drove the interneuron area with i.i.d. uniform noise
//!   (`rng.next_f32() * 2.0`) rather than structured network activity.
//! * Its only metric was `smoothness = 1/(1 + var)` over the pooled inhibition
//!   trace. With i.i.d. input that quantity is a deterministic function of the
//!   pooling arithmetic, not a property of the dynamics.
//! * Every row printed the literal `PASS`. There was no threshold anywhere in
//!   the file, so the sweep could not fail.
//! * The verdict asserted "stable, continuous soft-WTA competition", which the
//!   code never tested.
//!
//! This version drives the area with heterogeneous, seeded activity patterns and
//! evaluates three **falsifiable** criteria per sweep point. The implementation
//! uses deterministic heterogeneous E→I and I→E projections, so C3 measures
//! actual cell-specific competition rather than a broadcast gain-control scalar.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_areas::InhibitoryInterneuronArea;
use binn_core::Rng;
use binn_lab::guards::Verdict;

const PROTOCOL_VERSION: u64 = 135;
const EXPERIMENT_NAME: &str = "ei-inhibition-sweep";

/// Minimum relative spread of inhibition across excitatory cells for the
/// dynamics to count as *competitive* rather than uniform gain control.
const COMPETITION_MIN_SPREAD: f32 = 0.01;
/// Seeds per sweep point.
const N_SEEDS: usize = 10;
/// Integration steps per seed.
const N_STEPS: usize = 50;

#[derive(Clone, Debug)]
struct EiSweepPoint {
    e_ratio: usize,
    weight_i_to_e: f32,
    mean_inhibition: f32,
    /// `(max − min) / mean` of inhibition across excitatory cells.
    competition_spread: f32,
    /// Coefficient of variation of the pooled inhibition trace over time.
    temporal_cv: f32,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                println!(
                    "Usage: cargo run --release -p binn-lab --bin ei-inhibition-sweep [--out PATH]"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    println!("========================================================================");
    println!("Dynamic E-I Interneuron Sweeps Protocol v{PROTOCOL_VERSION}");
    println!("========================================================================\n");

    let e_ratios = [4usize, 8, 16];
    let weights_i_to_e = [0.2f32, 0.5, 1.0];
    let n_ex = 100usize;

    let mut points = Vec::new();

    for &e in &e_ratios {
        for &w_ie in &weights_i_to_e {
            let mut mean_inh_acc = 0.0f32;
            let mut spread_acc = 0.0f32;
            let mut cv_acc = 0.0f32;

            for s in 0..N_SEEDS {
                let mut rng = Rng::new(0x0071_AC00u64 ^ (s as u64).wrapping_mul(0x9E37_79B9));
                let n_inh = (n_ex / e).max(1);
                let mut area = InhibitoryInterneuronArea::new(n_ex, n_inh, 0.5, w_ie);

                let mut trace = Vec::with_capacity(N_STEPS);
                let mut last_inh = vec![0.0f32; n_ex];

                for _ in 0..N_STEPS {
                    // Heterogeneous, sparse activity: a minority of strongly
                    // driven cells over a weakly active background. This is the
                    // regime in which competition would matter; i.i.d. uniform
                    // noise is not.
                    let e_acts: Vec<f32> = (0..n_ex)
                        .map(|_| {
                            if rng.next_f32() < 0.1 {
                                1.0 + rng.next_f32()
                            } else {
                                0.05 * rng.next_f32()
                            }
                        })
                        .collect();
                    last_inh = area.compute_inhibition(&e_acts);
                    trace.push(last_inh.iter().sum::<f32>() / n_ex as f32);
                }

                let mean_inh = trace.iter().sum::<f32>() / trace.len() as f32;
                let var =
                    trace.iter().map(|v| (v - mean_inh).powi(2)).sum::<f32>() / trace.len() as f32;
                let cv = if mean_inh.abs() > 1e-9 {
                    var.sqrt() / mean_inh
                } else {
                    0.0
                };

                let hi = last_inh.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let lo = last_inh.iter().copied().fold(f32::INFINITY, f32::min);
                let per_cell_mean = last_inh.iter().sum::<f32>() / n_ex as f32;
                let spread = if per_cell_mean.abs() > 1e-9 {
                    (hi - lo) / per_cell_mean
                } else {
                    0.0
                };

                mean_inh_acc += mean_inh;
                spread_acc += spread;
                cv_acc += cv;
            }

            let n = N_SEEDS as f32;
            let point = EiSweepPoint {
                e_ratio: e,
                weight_i_to_e: w_ie,
                mean_inhibition: mean_inh_acc / n,
                competition_spread: spread_acc / n,
                temporal_cv: cv_acc / n,
            };
            println!(
                "E:I = {}:1 | W(I->E) = {:.1} => mean inh = {:.4} | competition spread = {:.6} | temporal CV = {:.4}",
                point.e_ratio,
                point.weight_i_to_e,
                point.mean_inhibition,
                point.competition_spread,
                point.temporal_cv
            );
            points.push(point);
        }
    }

    // ---- Falsifiable criteria ----

    // C1: mean inhibition increases monotonically with W(I->E) at fixed E:I.
    let mut c1_ok = true;
    for &e in &e_ratios {
        let row: Vec<&EiSweepPoint> = points.iter().filter(|p| p.e_ratio == e).collect();
        for w in row.windows(2) {
            if w[1].mean_inhibition <= w[0].mean_inhibition {
                c1_ok = false;
            }
        }
    }

    // C2: mean inhibition increases monotonically with E:I ratio at fixed W.
    let mut c2_ok = true;
    for &w_ie in &weights_i_to_e {
        let col: Vec<&EiSweepPoint> = points
            .iter()
            .filter(|p| (p.weight_i_to_e - w_ie).abs() < 1e-6)
            .collect();
        for w in col.windows(2) {
            if w[1].mean_inhibition <= w[0].mean_inhibition {
                c2_ok = false;
            }
        }
    }

    // C3: inhibition is differential across excitatory cells (true competition).
    let c3_ok = points
        .iter()
        .all(|p| p.competition_spread >= COMPETITION_MIN_SPREAD);

    let v1 = Verdict::evaluate_mean(f32::from(c1_ok), 1.0, N_SEEDS, N_SEEDS, true);
    let v2 = Verdict::evaluate_mean(f32::from(c2_ok), 1.0, N_SEEDS, N_SEEDS, true);
    let v3 = Verdict::evaluate_mean(f32::from(c3_ok), 1.0, N_SEEDS, N_SEEDS, true);

    let rows = points
        .iter()
        .map(|p| {
            let ok = p.competition_spread >= COMPETITION_MIN_SPREAD;
            format!(
                "| {}:1 | {:.1} | {:.4} | {:.6} | {:.4} | {} |",
                p.e_ratio,
                p.weight_i_to_e,
                p.mean_inhibition,
                p.competition_spread,
                p.temporal_cv,
                Verdict::evaluate_mean(f32::from(ok), 1.0, N_SEEDS, N_SEEDS, true).label(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let summary = format!(
        "# Dynamic E-I Interneuron Sweeps Report\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Seeds per point:** {N_SEEDS}; **steps per seed:** {N_STEPS}  \n\
        **Drive:** heterogeneous sparse activity (10% strongly driven, 90% weak background)\n\n\
        ## Sweep\n\n\
        `Competition spread` is `(max − min) / mean` of the inhibitory current across \
        excitatory cells. Uniform gain control gives 0; genuine competition requires \
        ≥ {COMPETITION_MIN_SPREAD}.\n\n\
        | E:I ratio | W(I→E) | Mean inhibition | Competition spread | Temporal CV | Per-point verdict |\n\
        |---|---:|---:|---:|---:|---|\n\
        {rows}\n\n\
        ## Criteria\n\n\
        | Criterion | Statement | Verdict |\n\
        |---|---|---|\n\
        | C1 | Mean inhibition increases with W(I→E) at fixed E:I | {} |\n\
        | C2 | Mean inhibition increases with E:I ratio at fixed W(I→E) | {} |\n\
        | C3 | Inhibition is differential across excitatory cells (soft-WTA competition) | {} |\n\n\
        ## Verdict\n\n\
        - Graded, monotone gain control: **{}**\n\
        - Soft-WTA *competition*: **{}**\n\n\
        ## Mechanism disclosure\n\n\
        `InhibitoryInterneuronArea` uses deterministic heterogeneous E→I receptive fields \
        and I→E projections. Projection normalization is population-level rather than \
        per-cell, so individual excitatory cells receive distinct inhibitory currents. \
        C3 still comes from the measured spread above; it is not assumed from the design.\n",
        v1.label(),
        v2.label(),
        v3.label(),
        if c1_ok && c2_ok { "supported" } else { "not supported" },
        if c3_ok { "supported" } else { "NOT supported" },
    );

    println!("\n{summary}");

    if let Some(path) = out {
        if let Err(e) = fs::write(&path, &summary) {
            eprintln!("Failed to write output report to {}: {e}", path.display());
            return ExitCode::from(1);
        }
        println!("Report saved to: {}", path.display());
    }

    ExitCode::SUCCESS
}
