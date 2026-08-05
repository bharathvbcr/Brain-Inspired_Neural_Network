//! Multi-channel neuromodulation experiment (Suite 2).
//!
//! Evaluates multi-channel credit assignment combining:
//! - **Dopamine (DA)**: global reward-prediction error
//! - **Acetylcholine (ACh)**: somatic membrane-proximity gating
//! - **Noradrenaline (NE)**: surprise multiplier
//!
//! # 2026-07-25 rewrite
//!
//! The previous version was not an experiment. It made two calls to
//! `compute_signal` with hand-picked scalars, printed the mean of each returned
//! vector, hardcoded `PASS` in both rows, and asserted a conclusion
//! ("integrates spatial addressability with membrane proximity gating") that
//! nothing in the file tested. There was no dataset, no seeds, no control, and
//! no outcome that could have been negative.
//!
//! This version states four falsifiable properties of the modulator and tests
//! each across seeds with an explicit criterion.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_core::Rng;
use binn_lab::guards::Verdict;
use binn_learn::{CreditSignal, MultiChannelNeuromodulator};

const PROTOCOL_VERSION: u64 = 136;
const EXPERIMENT_NAME: &str = "multi-channel-neuromod";

const N_CELLS: usize = 64;
const N_SEEDS: usize = 20;
const THETA: f32 = 1.0;
const BETA: f32 = 5.0;
/// Minimum fraction of seeds a property must hold on to be reported as supported.
const PROPERTY_FLOOR: f32 = 0.95;

fn mean(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f32>() / v.len() as f32
}

fn mean_abs(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().map(|x| x.abs()).sum::<f32>() / v.len() as f32
}

/// Collect the per-cell credit vector for one RPE value.
fn signal_for(modulator: &MultiChannelNeuromodulator, rpe: f32, v_soma: &[f32]) -> Vec<f32> {
    let credit = modulator.compute_signal(1.0, rpe, v_soma, THETA, BETA);
    (0..v_soma.len())
        .map(|i| credit.for_post(i as u32))
        .collect()
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
                println!("Usage: cargo run --release -p binn-lab --bin multi-channel-neuromod [--out PATH]");
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
    println!("Multi-Channel Neuromodulation Protocol v{PROTOCOL_VERSION}");
    println!("========================================================================\n");

    // P1 magnitude monotonicity, P2 sign reversal, P3 ACh proximity gating,
    // P4 spatial addressability (per-cell variation).
    let mut p1_hits = 0usize;
    let mut p2_hits = 0usize;
    let mut p3_hits = 0usize;
    let mut p4_hits = 0usize;

    let mut high_means = Vec::with_capacity(N_SEEDS);
    let mut low_means = Vec::with_capacity(N_SEEDS);
    let mut negative_means = Vec::with_capacity(N_SEEDS);

    for s in 0..N_SEEDS {
        let seed = 0x0071_AC00u64 ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut rng = Rng::new(seed);
        // Heterogeneous membrane voltages spanning θ, so ACh gating has a
        // gradient to act on.
        let v_soma: Vec<f32> = (0..N_CELLS).map(|_| rng.next_f32() * 2.0).collect();

        let modulator = MultiChannelNeuromodulator::new(N_CELLS, seed, 0.01);

        let sig_high = signal_for(&modulator, 0.8, &v_soma);
        let sig_low = signal_for(&modulator, 0.1, &v_soma);
        let sig_neg = signal_for(&modulator, -0.8, &v_soma);

        let mu_high = mean_abs(&sig_high);
        let mu_low = mean_abs(&sig_low);
        let mu_neg = mean(&sig_neg);
        high_means.push(mu_high);
        low_means.push(mu_low);
        negative_means.push(mu_neg);

        // P1: a larger |RPE| produces a larger mean absolute credit.
        if mu_high > mu_low {
            p1_hits += 1;
        }
        // P2: hold every other input fixed and require vector anti-symmetry.
        if sig_high
            .iter()
            .zip(&sig_neg)
            .all(|(pos, neg)| (pos + neg).abs() < 1e-6)
        {
            p2_hits += 1;
        }
        // P3: isolate ACh and hold feedback/RPE fixed while moving every cell
        // from the threshold to one full voltage unit away.
        let near = modulator.components(0.8, &vec![THETA; N_CELLS], THETA, BETA);
        let far = modulator.components(0.8, &vec![THETA + 1.0; N_CELLS], THETA, BETA);
        if near
            .acetylcholine
            .values()
            .iter()
            .zip(far.acetylcholine.values())
            .all(|(near_i, far_i)| near_i.abs() > far_i.abs())
        {
            p3_hits += 1;
        }
        // P4: credit is spatially addressed, i.e. not identical across cells.
        // A broadcast scalar would fail this.
        let hi = sig_high.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let lo = sig_high.iter().copied().fold(f32::INFINITY, f32::min);
        if (hi - lo).abs() > 1e-6 {
            p4_hits += 1;
        }
    }

    let n = N_SEEDS as f32;
    let (r1, r2, r3, r4) = (
        p1_hits as f32 / n,
        p2_hits as f32 / n,
        p3_hits as f32 / n,
        p4_hits as f32 / n,
    );
    let v1 = Verdict::evaluate_mean(r1, PROPERTY_FLOOR, N_SEEDS, N_SEEDS, true);
    let v2 = Verdict::evaluate_mean(r2, PROPERTY_FLOOR, N_SEEDS, N_SEEDS, true);
    let v3 = Verdict::evaluate_mean(r3, PROPERTY_FLOOR, N_SEEDS, N_SEEDS, true);
    let v4 = Verdict::evaluate_mean(r4, PROPERTY_FLOOR, N_SEEDS, N_SEEDS, true);

    println!(
        "P1 magnitude monotonicity : {p1_hits}/{N_SEEDS} ({})",
        v1.label()
    );
    println!(
        "P2 sign reversal          : {p2_hits}/{N_SEEDS} ({})",
        v2.label()
    );
    println!(
        "P3 ACh proximity gating   : {p3_hits}/{N_SEEDS} ({})",
        v3.label()
    );
    println!(
        "P4 spatial addressability : {p4_hits}/{N_SEEDS} ({})",
        v4.label()
    );

    let all_supported = [v1, v2, v3, v4].iter().all(|v| v.is_citable_as_positive());

    let summary = format!(
        "# Multi-Channel Neuromodulation Report\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Seeds:** {N_SEEDS}; **cells:** {N_CELLS}; **θ:** {THETA}; **β:** {BETA}  \n\
        **Property floor:** {PROPERTY_FLOOR:.2} of seeds  \n\n\
        ## Falsifiable properties\n\n\
        | Property | Statement | Seeds holding | Rate | Verdict |\n\
        |---|---|---:|---:|---|\n\
        | P1 | Larger \\|RPE\\| yields larger mean credit magnitude | {p1_hits}/{N_SEEDS} | {r1:.4} | {} |\n\
        | P2 | Reversing RPE sign reverses credit direction | {p2_hits}/{N_SEEDS} | {r2:.4} | {} |\n\
        | P3 | ACh gating concentrates credit near threshold | {p3_hits}/{N_SEEDS} | {r3:.4} | {} |\n\
        | P4 | Credit is per-cell addressed, not a broadcast scalar | {p4_hits}/{N_SEEDS} | {r4:.4} | {} |\n\n\
        ## Descriptive means\n\n\
        - Mean absolute credit at RPE = 0.8: {:.6}\n\
        - Mean absolute credit at RPE = 0.1: {:.6}\n\
        - Mean signed credit at RPE = -0.8 (descriptive): {:.6}\n\n\
        ## Verdict\n\n\
        - All four properties supported: **{}**\n\n\
        ## Caveats\n\n\
        - This is a **property test of the modulator**, not a learning result. It says \
        nothing about downstream task accuracy and must not be cited as evidence that \
        multi-channel neuromodulation improves credit assignment.\n\
        - P2 holds feedback, voltages and channel weights fixed and tests vector \
        anti-symmetry. P3 uses the explicit per-channel decomposition and holds feedback \
        and RPE fixed; neither property is inferred by subtracting mixed signals.\n",
        v1.label(),
        v2.label(),
        v3.label(),
        v4.label(),
        mean(&high_means),
        mean(&low_means),
        mean(&negative_means),
        if all_supported { "yes" } else { "no" },
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
