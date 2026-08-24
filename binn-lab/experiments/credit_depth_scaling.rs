//! Network depth on a task whose reference can fall (protocol v1).
//!
//! Registered in `results/PREREG_2026-08-23_DEPTH_ON_A_TASK_WITH_HEADROOM.md`.
//!
//! # Why this exists
//!
//! `deep-snn-scaling` v136 answered the depth question on `CoincidenceTask`,
//! where the ceiling reaches **exactly 1.0000** at depths 2-4. "The treatment
//! tracks its ceiling" is close to "both arms solved an easy task" when the
//! reference has nowhere to fall, and the same disease governs the matched-arch
//! schedule. SHD is the obvious remedy and is refused at the calibration gate;
//! that refusal is respected rather than routed around.
//!
//! `binn_data::credit_depth` supplies a compositional, order-sensitive,
//! terminal-reward task with tunable difficulty, written for depth studies and
//! never wired to one. At `n_states = 8`, task depth 4, the ceiling sits at
//! 0.4600 against a chance of 0.1250 — room to move in both directions.
//! See `results/MEASUREMENT_2026-08-23_A_TASK_WITH_HEADROOM.md`.
//!
//! # Task depth is fixed; network depth is the variable
//!
//! Confounding the two would make a "depth effect" unreadable. The task stays at
//! depth 4 for every arm; only the number of hidden layers moves.
//!
//! # Validity is gated, not reported
//!
//! The feasibility sweep flagged one cell where the treatment exceeded the
//! ceiling and one where the two agreed to four decimals — the shape of two arms
//! agreeing because both are degenerate. So this suite refuses a reading rather
//! than printing one: `CeilingHealth` at every depth, a saturation gate the v136
//! report lacked, and a per-arm readout audit on class collapse.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rayon::prelude::*;

use binn_core::Rng;
use binn_data::credit_depth::CreditDepthConfig;
use binn_engine::{DEFAULT_TAU_M, THETA_REST};
use binn_lab::guards::{CeilingHealth, Verdict};
use binn_lab::{
    credit_depth_chance, credit_depth_examples, credit_depth_input_width, mean, std_error,
};
use binn_learn::{
    random_feedback, train_bptt, train_learned_feedback_adam, DenseTemporalExample,
    SharedTemporalNet, DEFAULT_MATCHED_BETA, MAJORITY_PRED_MAX,
};

const PROTOCOL_VERSION: u64 = 1;
const EXPERIMENT_NAME: &str = "credit-depth-scaling";
/// Network depths under test. Registered.
const DEPTHS: [usize; 4] = [1, 2, 3, 4];
/// Registered seed requirement for a scientific verdict.
const REQUIRED_SEEDS: usize = 12;
/// A ceiling above this has no headroom, and the reading at that depth is void.
/// v136 had no such gate and reported gaps against a reference at exactly 1.0000.
const HEADROOM_MAX: f32 = 0.95;
/// Registered effect size for both two-sided hypotheses.
const EFFECT_BAR: f32 = 0.05;
/// Task knobs, fixed for the whole suite by the rule in prereg section 1.
const N_STATES: usize = 8;
const TASK_DEPTH: usize = 4;
const FEEDBACK_LR: f32 = 0.01;

fn task_config(seed: u64) -> CreditDepthConfig {
    CreditDepthConfig {
        seed,
        n_states: N_STATES,
        n_operations: 2,
        depth: TASK_DEPTH,
    }
}

fn build_model(
    cfg: &CreditDepthConfig,
    depth: usize,
    hidden: usize,
    seed: u64,
) -> SharedTemporalNet {
    SharedTemporalNet::new(
        credit_depth_input_width(cfg),
        cfg.depth,
        cfg.n_states,
        &vec![hidden; depth],
        (-1.0f32 / DEFAULT_TAU_M).exp(),
        THETA_REST,
        DEFAULT_MATCHED_BETA,
        seed,
    )
}

/// What one arm produced at one depth and seed.
#[derive(Clone, Copy, Debug, Default)]
struct ArmOutcome {
    accuracy: f32,
    /// Distinct classes the arm actually predicted.
    classes_predicted: usize,
    /// Share of predictions falling in the single most-predicted class.
    majority_prediction: f32,
}

/// Score an arm, recording *how* it predicted and not only how well.
///
/// An accuracy alone cannot distinguish a learner from a constant predictor, and
/// this workspace has repeatedly published the latter as the former.
fn score(model: &SharedTemporalNet, test: &[DenseTemporalExample], n_classes: usize) -> ArmOutcome {
    let mut histogram = vec![0usize; n_classes];
    let mut correct = 0usize;
    for example in test {
        let prediction = model.forward(example).prediction as usize;
        if prediction < n_classes {
            histogram[prediction] += 1;
        }
        if prediction as u32 == example.label {
            correct += 1;
        }
    }
    let top = histogram.iter().copied().max().unwrap_or(0);
    ArmOutcome {
        accuracy: correct as f32 / test.len() as f32,
        classes_predicted: histogram.iter().filter(|&&c| c > 0).count(),
        majority_prediction: top as f32 / test.len() as f32,
    }
}

/// Reasons an arm's accuracy is not interpretable. Empty when healthy.
fn readout_defects(outcome: &ArmOutcome) -> Vec<&'static str> {
    let mut defects = Vec::new();
    if outcome.classes_predicted <= 1 {
        defects.push("CONSTANT PREDICTOR (one class)");
    } else if outcome.majority_prediction > MAJORITY_PRED_MAX {
        defects.push("NEAR-COLLAPSED (>95% of predictions in one class)");
    }
    if !outcome.accuracy.is_finite() {
        defects.push("NON-FINITE ACCURACY");
    }
    defects
}

#[derive(Clone, Copy, Debug)]
enum Job {
    Treatment { depth_idx: usize },
    Ceiling { depth_idx: usize },
}

impl Job {
    fn run(
        self,
        hidden: usize,
        epochs: usize,
        seed: u64,
        cfg: &CreditDepthConfig,
        train: &[DenseTemporalExample],
        test: &[DenseTemporalExample],
    ) -> ArmOutcome {
        match self {
            Job::Treatment { depth_idx } => {
                let mut model = build_model(cfg, DEPTHS[depth_idx], hidden, seed);
                let mut feedback = random_feedback(&model, seed);
                train_learned_feedback_adam(&mut model, &mut feedback, train, epochs, FEEDBACK_LR);
                score(&model, test, cfg.n_states)
            }
            Job::Ceiling { depth_idx } => {
                let mut model = build_model(cfg, DEPTHS[depth_idx], hidden, seed);
                train_bptt(&mut model, train, epochs);
                score(&model, test, cfg.n_states)
            }
        }
    }
}

fn job_grid() -> Vec<Job> {
    let mut jobs = Vec::new();
    for depth_idx in 0..DEPTHS.len() {
        jobs.push(Job::Treatment { depth_idx });
        jobs.push(Job::Ceiling { depth_idx });
    }
    jobs
}

#[derive(Clone, Debug, Default)]
struct Arm {
    treatment: Vec<ArmOutcome>,
    ceiling: Vec<ArmOutcome>,
}

fn accuracies(outcomes: &[ArmOutcome]) -> Vec<f32> {
    outcomes.iter().map(|o| o.accuracy).collect()
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
                println!("Usage: cargo run --release -p binn-lab --bin credit-depth-scaling [-- --quick] [--out PATH]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let n_seeds = if quick { 3 } else { REQUIRED_SEEDS };
    let n_train = if quick { 200 } else { 600 };
    let n_test = if quick { 100 } else { 300 };
    let hidden = if quick { 32 } else { 64 };
    let epochs = if quick { 15 } else { 40 };
    let master_seed = 0x00C4_D3F7_0001_u64;
    let chance = credit_depth_chance(&task_config(0));

    println!("========================================================================");
    println!("Credit-Depth Scaling — protocol v{PROTOCOL_VERSION}");
    println!(
        "Schedule: {} (n_seeds={n_seeds}, hidden={hidden}, epochs={epochs}, \
         n_states={N_STATES}, task_depth={TASK_DEPTH}, chance={chance:.4})",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        }
    );
    println!("========================================================================\n");

    let jobs = job_grid();

    // Each cell is seeded solely by `seed` and reads its split immutably, so no
    // cell can observe another. `map(..).collect()` preserves input order and
    // there is no cross-seed accumulator inside the parallel region.
    let per_seed: Vec<Vec<ArmOutcome>> = (0..n_seeds)
        .into_par_iter()
        .map(|s_idx| {
            let seed = master_seed ^ ((s_idx as u64) * 0x1000_0005_u64);
            let cfg = task_config(seed);
            let mut rng = Rng::new(seed ^ 0x0DA7_A000);
            let train = credit_depth_examples(&cfg, &mut rng, n_train);
            let test = credit_depth_examples(&cfg, &mut rng, n_test);
            jobs.par_iter()
                .map(|job| job.run(hidden, epochs, seed, &cfg, &train, &test))
                .collect()
        })
        .collect();

    let mut arms: Vec<Arm> = vec![Arm::default(); DEPTHS.len()];
    for outcomes in &per_seed {
        for (job, outcome) in jobs.iter().zip(outcomes) {
            match *job {
                Job::Treatment { depth_idx } => arms[depth_idx].treatment.push(*outcome),
                Job::Ceiling { depth_idx } => arms[depth_idx].ceiling.push(*outcome),
            }
        }
    }

    let mut rows = String::new();
    let mut audit_rows = String::new();
    let mut any_defect = false;
    let mut gaps = Vec::new();
    let mut ceilings = Vec::new();

    for (d, &depth) in DEPTHS.iter().enumerate() {
        let t = accuracies(&arms[d].treatment);
        let c = accuracies(&arms[d].ceiling);
        let (t_mean, c_mean) = (mean(&t), mean(&c));
        gaps.push(t_mean - c_mean);
        ceilings.push(c_mean);

        let health = CeilingHealth::evaluate(c_mean, t_mean, chance);
        // V-2: a ceiling with no headroom voids the reading at this depth. v136
        // lacked this gate and reported gaps against a reference at 1.0000.
        let saturated = c_mean > HEADROOM_MAX;
        let usable = health.is_usable() && !saturated;
        if !usable {
            any_defect = true;
        }

        let verdict =
            Verdict::evaluate_mean(t_mean, chance + EFFECT_BAR, n_seeds, REQUIRED_SEEDS, usable);
        rows.push_str(&format!(
            "| {depth} | {hidden} x {depth} | {t_mean:.4} | {:.4} | {c_mean:.4} | {:.4} | \
             {:+.4} | {} | {} | {} |\n",
            std_error(&t),
            std_error(&c),
            t_mean - c_mean,
            if saturated { "**SATURATED**" } else { "ok" },
            health.label(),
            verdict.label(),
        ));

        for (label, outcomes) in [
            ("treatment", &arms[d].treatment),
            ("ceiling", &arms[d].ceiling),
        ] {
            let classes = mean(
                &outcomes
                    .iter()
                    .map(|o| o.classes_predicted as f32)
                    .collect::<Vec<_>>(),
            );
            let majority = mean(
                &outcomes
                    .iter()
                    .map(|o| o.majority_prediction)
                    .collect::<Vec<_>>(),
            );
            let worst = outcomes
                .iter()
                .map(readout_defects)
                .find(|d| !d.is_empty())
                .map(|d| d.join("; "))
                .unwrap_or_else(|| "none".to_string());
            if worst != "none" {
                any_defect = true;
            }
            audit_rows.push_str(&format!(
                "| {depth} | {label} | {classes:.2} | {majority:.4} | {worst} |\n"
            ));
        }
    }

    let d1 = (gaps[DEPTHS.len() - 1] - gaps[0]).abs();
    let d2 = (ceilings[DEPTHS.len() - 1] - ceilings[0]).abs();

    let banner = if any_defect {
        "> **HARNESS DEFECT - do not interpret any comparison below.** At least one \
         depth failed a validity gate: the ceiling did not clear chance, it was \
         inverted, it saturated, or an arm's readout collapsed. Verdicts are \
         `INVALID_HARNESS` for exactly this reason.\n\n"
    } else {
        ""
    };

    let summary = format!(
        "# Credit-Depth Scaling Report\n\n\
        {banner}\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Preregistration:** `results/PREREG_2026-08-23_DEPTH_ON_A_TASK_WITH_HEADROOM.md`  \n\
        **Schedule:** {} (n={n_seeds}, hidden={hidden}, epochs={epochs})  \n\
        **Task:** `CreditDepthTask`, n_states={N_STATES}, task depth {TASK_DEPTH} \
        (fixed), chance {chance:.4}  \n\n\
        Treatment and ceiling share one forward graph, one initialisation, one \
        optimiser and one step size, and differ only in whether the gradients are \
        true or feedback-projected. **Task depth is held fixed**; the variable is \
        network depth.\n\n\
        ## Per network depth\n\n\
        | Depth | Hidden | Treatment | SE | Ceiling | SE | Gap | Headroom | Ceiling health | Verdict |\n\
        |---:|---|---:|---:|---:|---:|---:|---|---|---|\n\
        {rows}\n\
        `Gap` is treatment minus ceiling. `Headroom` fails when the ceiling exceeds \
        {HEADROOM_MAX}, which voids the reading at that depth — the gate v136 did \
        not have.\n\n\
        ## Readout audit\n\n\
        An accuracy cannot distinguish a learner from a constant predictor. \
        `Classes` is the mean number of distinct classes an arm actually \
        predicted, out of {N_STATES}; `Majority` is the share in the single \
        most-predicted class.\n\n\
        | Depth | Arm | Classes | Majority | Defects |\n\
        |---:|---|---:|---:|---|\n\
        {audit_rows}\n\
        ## Registered hypotheses\n\n\
        - **D-1** *(two-sided)* network depth changes the gap: \
        |gap(4) - gap(1)| = **{d1:.4}**, bar {EFFECT_BAR} -> **{}**\n\
        - **D-2** *(two-sided)* network depth changes the ceiling: \
        |ceiling(4) - ceiling(1)| = **{d2:.4}**, bar {EFFECT_BAR} -> **{}**\n\n\
        {}\
        ## Interpretation caveat\n\n\
        This is a compositional symbolic task, not an input-rich sensory one. \
        Whatever it finds is about credit assignment through composed \
        transformations and transfers to SHD only as a hypothesis. Task depth 4 \
        is one point; at task depth 8 the ceiling falls to 0.2750 and that regime \
        is untested.\n",
        if quick { "QUICK / PILOT" } else { "FULL SCIENTIFIC" },
        if d1 >= EFFECT_BAR { "SUPPORTED" } else { "NOT SUPPORTED" },
        if d2 >= EFFECT_BAR { "SUPPORTED" } else { "NOT SUPPORTED" },
        if any_defect {
            "**No hypothesis verdict may be read while the harness banner is present.**\n\n"
        } else {
            ""
        },
    );

    println!("\n{summary}");

    if let Some(path) = out {
        if let Err(e) = fs::write(&path, &summary) {
            eprintln!("Failed to write report to {}: {e}", path.display());
            return ExitCode::from(1);
        }
        println!("Report saved to: {}", path.display());
    }
    ExitCode::SUCCESS
}
