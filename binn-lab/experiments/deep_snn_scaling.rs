//! Deep SNN Scaling Experiments (Suite 1).
//!
//! Evaluates multi-layer online learned feedback alignment across network depth
//! (1–4 hidden layers of 256 units) against a **depth-matched** surrogate
//! gradient ceiling.
//!
//! # 2026-07-25 fixes
//!
//! * The Verdict column was the literal string `PASS` for all four depth arms,
//!   regardless of the measurement. The shipped report contained rows reading
//!   `FAIL | PASS`, and the summary derived from it claimed "PASS across all
//!   depths" while 2L/3L/4L scored 0.4525 / 0.5130 / 0.4500 against a 0.65
//!   floor. Verdicts now come from `guards::Verdict::evaluate_mean`.
//! * The ceiling was **1-hidden-layer** for every depth arm, so a depth-related
//!   collapse could not be attributed to feedback alignment rather than to the
//!   optimiser or to the task lacking depth structure. Each depth now has its
//!   own [`MatchedDeepGradient`] ceiling.
//! * Modulator scale is now recorded per depth. A ceiling whose credit signal is
//!   orders of magnitude weaker than the treatment's is not a ceiling — that is
//!   exactly how the SHD suite produced a "ceiling" below its own treatment.
//!
//! # Known limitation, stated in the report
//!
//! `CoincidenceTask` has `N_IN = 2`. A 256⁴ stack on a 2-dimensional,
//! near-noiseless input has no depth structure to exploit, so a depth result on
//! this task is weak evidence either way. The report says so.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::guards::Verdict;
use binn_lab::{freeze_trials, samples_to_gradient_examples, Config};
use binn_learn::{
    MatchedDeepGradient, MatchedRl3LayerLearnedFb, MatchedRl4LayerLearnedFb,
    MatchedRlDeepLearnedFb, MatchedRlLearnedFb, ModulatorScale, DEFAULT_MATCHED_BETA,
};

const PROTOCOL_VERSION: u64 = 134;
const EXPERIMENT_NAME: &str = "deep-snn-scaling";
/// Preregistered accuracy floor.
const ACCURACY_FLOOR: f32 = 0.65;
/// Preregistered seed requirement for a scientific verdict.
const REQUIRED_SEEDS: usize = 20;
/// Task input dimensionality, surfaced in the report as an interpretation caveat.
const TASK_N_IN: usize = 2;

#[derive(Clone, Debug)]
struct DepthArm {
    depth: usize,
    /// Learned-feedback accuracies, one per seed.
    fb_accs: Vec<f32>,
    /// Depth-matched gradient ceiling accuracies, one per seed.
    ceiling_accs: Vec<f32>,
    /// Realised RMS of the ceiling's input-layer modulator.
    ceiling_modulator: ModulatorScale,
}

fn mean(vals: &[f32]) -> f32 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.iter().sum::<f32>() / vals.len() as f32
}

fn std_error(vals: &[f32]) -> f32 {
    if vals.len() <= 1 {
        return 0.0;
    }
    let m = mean(vals);
    let var = vals.iter().map(|v| (v - m).powi(2)).sum::<f32>() / (vals.len() - 1) as f32;
    (var / vals.len() as f32).sqrt()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--out" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                println!("Usage: cargo run --release -p binn-lab --bin deep-snn-scaling [-- --quick] [--out PATH]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let n_seeds = if quick { 5 } else { REQUIRED_SEEDS };
    let n_train = if quick { 60 } else { 200 };
    let n_test = if quick { 30 } else { 100 };
    let hidden = if quick { 128 } else { 256 };
    let epochs = if quick { 40 } else { 80 };
    let master_seed = if quick {
        0x0071_AC00_001C_00F3_u64
    } else {
        0x0071_AC00_001C_00F4_u64
    };

    println!("========================================================================");
    println!("Deep SNN Scaling Experiments Protocol v{PROTOCOL_VERSION}");
    println!(
        "Schedule: {} (n_seeds={n_seeds}, hidden={hidden}, epochs={epochs})",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        }
    );
    println!("========================================================================\n");

    let mut base_cfg = Config::c1_default();
    base_cfg.n_seeds = n_seeds;
    base_cfg.n_train = n_train;
    base_cfg.n_test = n_test;
    base_cfg.n_hidden = hidden;
    base_cfg.bptt_epochs = epochs;

    let mut arms: Vec<DepthArm> = (1..=4)
        .map(|depth| DepthArm {
            depth,
            fb_accs: Vec::with_capacity(n_seeds),
            ceiling_accs: Vec::with_capacity(n_seeds),
            ceiling_modulator: ModulatorScale::new(),
        })
        .collect();

    for s_idx in 0..n_seeds {
        let seed = master_seed ^ ((s_idx as u64) * 0x1000_0005_u64);
        let split = freeze_trials(&base_cfg, seed);
        let train_data = samples_to_gradient_examples(&split.train);
        let test_data = samples_to_gradient_examples(&split.test);

        // ---- Learned-feedback arms, one per depth ----
        let fb: [f32; 4] = {
            let mut l1 =
                MatchedRlLearnedFb::new(hidden, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, seed);
            let r1 = l1
                .train_and_evaluate(epochs, &train_data, &test_data)
                .accuracy;

            let mut l2 = MatchedRlDeepLearnedFb::new(
                hidden,
                hidden,
                0.05,
                0.0,
                0.01,
                DEFAULT_MATCHED_BETA,
                seed,
            );
            let r2 = l2
                .train_and_evaluate(epochs, &train_data, &test_data)
                .accuracy;

            let mut l3 = MatchedRl3LayerLearnedFb::new(
                hidden,
                hidden,
                hidden,
                0.05,
                0.0,
                0.01,
                DEFAULT_MATCHED_BETA,
                seed,
            );
            let r3 = l3
                .train_and_evaluate(epochs, &train_data, &test_data)
                .accuracy;

            let mut l4 = MatchedRl4LayerLearnedFb::new(
                hidden,
                hidden,
                hidden,
                hidden,
                0.05,
                0.0,
                0.01,
                DEFAULT_MATCHED_BETA,
                seed,
            );
            let r4 = l4
                .train_and_evaluate(epochs, &train_data, &test_data)
                .accuracy;
            [r1, r2, r3, r4]
        };

        // ---- Depth-MATCHED gradient ceilings ----
        let mut ceil = [0.0f32; 4];
        for (idx, arm) in arms.iter_mut().enumerate() {
            let layers = vec![hidden; arm.depth];
            let mut g = MatchedDeepGradient::new(&layers, 0.05, 0.0, DEFAULT_MATCHED_BETA, seed);
            ceil[idx] = g
                .train_and_evaluate(epochs, &train_data, &test_data)
                .accuracy;
            arm.ceiling_modulator.merge(&g.input_modulator_scale());
            arm.fb_accs.push(fb[idx]);
            arm.ceiling_accs.push(ceil[idx]);
        }

        println!(
            "Seed {:2}/{n_seeds}: FB 1L={:.4} 2L={:.4} 3L={:.4} 4L={:.4} | \
             ceiling 1L={:.4} 2L={:.4} 3L={:.4} 4L={:.4}",
            s_idx + 1,
            fb[0],
            fb[1],
            fb[2],
            fb[3],
            ceil[0],
            ceil[1],
            ceil[2],
            ceil[3]
        );
    }

    // ---- Report ----
    let mut fb_rows = String::new();
    let mut ceiling_rows = String::new();
    let mut any_fb_pass = false;

    for arm in &arms {
        let m = mean(&arm.fb_accs);
        let se = std_error(&arm.fb_accs);
        let verdict = Verdict::evaluate_mean(m, ACCURACY_FLOOR, n_seeds, REQUIRED_SEEDS, true);
        if verdict.is_citable_as_positive() {
            any_fb_pass = true;
        }
        let arch = match arm.depth {
            1 => format!("{hidden}"),
            d => format!("{hidden} × {d}"),
        };
        fb_rows.push_str(&format!(
            "| {}-Hidden-Layer Learned FB | {arch} | {m:.4} | {se:.4} | {} | {} |\n",
            arm.depth,
            if m >= ACCURACY_FLOOR { "yes" } else { "no" },
            verdict.label(),
        ));

        let cm = mean(&arm.ceiling_accs);
        let cse = std_error(&arm.ceiling_accs);
        // A ceiling that loses to its own treatment is a harness defect.
        let ceiling_inverted = cm + 1e-6 < m;
        ceiling_rows.push_str(&format!(
            "| {}-Hidden-Layer Gradient Ceiling (depth-matched) | {arch} | {cm:.4} | {cse:.4} | {:.3e} | {} |\n",
            arm.depth,
            arm.ceiling_modulator.rms(),
            if ceiling_inverted {
                "INVERTED — ceiling below treatment; do not interpret"
            } else {
                "ok"
            },
        ));
    }

    let overall = Verdict::evaluate_mean(
        mean(&arms[3].fb_accs),
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        true,
    );

    let summary = format!(
        "# Deep SNN Scaling Report\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Schedule:** {} (n={n_seeds}, hidden={hidden}, epochs={epochs})  \n\
        **Accuracy floor:** {ACCURACY_FLOOR:.2}  \n\n\
        ## Learned-feedback arms\n\n\
        | Arm | Hidden architecture | Mean accuracy | SE | Clears floor | Verdict |\n\
        |---|---|---:|---:|---|---|\n\
        {fb_rows}\n\
        ## Depth-matched gradient ceilings\n\n\
        Each depth is compared against a ceiling of **the same depth**, not against a \
        1-hidden-layer reference. `Modulator RMS` is the realised scale of the credit \
        signal reaching the input layer; if it differs by orders of magnitude across \
        arms, the comparison is measuring effective learning rate rather than \
        credit-assignment quality.\n\n\
        | Arm | Hidden architecture | Mean accuracy | SE | Modulator RMS | Ceiling health |\n\
        |---|---|---:|---:|---:|---|\n\
        {ceiling_rows}\n\
        ## Verdict\n\n\
        - 4-Hidden-Layer learned feedback alignment: **{}**\n\
        - Any depth clearing the floor: **{}**\n\n\
        ## Interpretation caveat\n\n\
        The matched-architecture task (`CoincidenceTask`) has **N_IN = {TASK_N_IN}** and \
        `difficulty = 0.05`. A {hidden}-wide, 4-deep stack on a {TASK_N_IN}-dimensional \
        near-noiseless input has no depth structure to exploit, so neither a depth \
        collapse nor a depth success on this task is strong evidence about deep credit \
        assignment. Move this suite to an input-rich task (the SHD path already exists) \
        before drawing a scaling conclusion.\n",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        },
        overall.label(),
        if any_fb_pass { "yes" } else { "no" },
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
