//! Deep SNN Scaling Experiments (Suite 1), protocol v136.
//!
//! Evaluates multi-layer learned feedback alignment across network depth (1-4
//! hidden layers) against a depth-matched surrogate-gradient ceiling.
//!
//! # 2026-08-22 rewrite (v136) — why the previous instrument was replaced
//!
//! v134/v135 compared [`binn_learn::MatchedDeepGradient`] (ceiling) against the
//! `MatchedRl*LearnedFb` family (treatment). Both sides were defective, in ways
//! that made the comparison meaningless rather than merely noisy. Registered and
//! measured in `results/PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md`:
//!
//! * **Both sides were silent above layer 1 at initialisation.** The inter-layer
//!   signal is a rate, `s[l-1]/T`, but the hidden weights were initialised at a
//!   scale for a unit-variance input. Every layer above the first emitted zero
//!   spikes, so the readout saw a zero vector and both classes produced a
//!   bit-identical logit.
//! * **The treatment had no readout bias.** With non-negative rate features its
//!   decision boundary was pinned at the origin. Measured: at width 256 the
//!   hidden representation separated the classes by 735 spikes and the arm still
//!   scored exactly 0.5000, because both logits landed on the same side of zero.
//! * **Raising the initialisation was not enough for the ceiling.** With every
//!   layer inside the activity band the deep stack spikes, carries the class
//!   signal at initialisation (separation 5 and 6 at depths 2 and 3) — and
//!   training then destroys it, saturating both layers to identical, class-blind
//!   patterns. The eligibility is sign-definite, so a hidden unit can learn only
//!   a scalar gain, which then runs away. That is a defect in the credit rule,
//!   not in the operating point, and it is not fixable by initialisation.
//!
//! So this suite now runs on [`binn_learn::shared_bptt`], which was written as
//! the validated replacement for exactly this and had **no callers**. It gives a
//! genuinely shared forward with an explicit readout bias and exact reverse-mode
//! gradients, so treatment and ceiling can share one forward, one initialisation
//! and one optimiser, and differ only in whether the gradients are true or
//! feedback-projected. The old suite varied the credit pathway and the optimiser
//! at once.
//!
//! # The optimiser is matched at Adam, and the choice read the ceiling only
//!
//! `shared_bptt` offers an SGD-matched pair ([`train_learned_feedback`] /
//! [`train_bptt_sgd`]) and an Adam pair ([`train_learned_feedback_adam`] /
//! [`train_bptt`]). The SGD pair is only useful where SGD can train the
//! architecture at all, and on this stack it cannot: at depth ≥ 2 the **ceiling**
//! sits at exactly 0.5000 at every rung of the registered step-size ladder, while
//! the Adam ceiling reaches 1.0000. A reference that cannot learn bounds nothing.
//!
//! The headline pair is therefore Adam, at the frozen `ADAM_LR`, with **no step
//! size to choose and nothing tuned on either arm**. The full SGD ladder is run
//! for both arms at every depth and reported, so the reader can see exactly what
//! the optimiser choice excluded. The selection that led here read the ceiling
//! arm only, never the treatment — registered in
//! `results/PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` §7a.
//!
//! # Known limitation, stated in the report
//!
//! `CoincidenceTask` has `N_IN = 2`. A wide, deep stack on a two-dimensional,
//! near-noiseless input has no depth structure to exploit, so a depth result on
//! this task is weak evidence either way. The report says so.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rayon::prelude::*;

use binn_engine::{DEFAULT_TAU_M, THETA_REST};
use binn_lab::guards::{CeilingHealth, Verdict};
use binn_lab::{freeze_trials, mean, samples_to_dense_temporal_examples, std_error, Config};
use binn_learn::{
    random_feedback, train_bptt, train_bptt_sgd, train_learned_feedback,
    train_learned_feedback_adam, DenseTemporalExample, SharedTemporalNet, DEFAULT_MATCHED_BETA,
};

const PROTOCOL_VERSION: u64 = 136;
const EXPERIMENT_NAME: &str = "deep-snn-scaling";
/// Preregistered accuracy floor. Unchanged from v134.
const ACCURACY_FLOOR: f32 = 0.65;
/// Preregistered seed requirement for a scientific verdict. Unchanged from v134.
const REQUIRED_SEEDS: usize = 20;
/// Task input dimensionality, surfaced in the report as an interpretation caveat.
const TASK_N_IN: usize = 2;
/// `CoincidenceTask` is two-class, so this is the constant-predictor rate every
/// gradient ceiling must clear before it can bound anything.
const CHANCE: f32 = 0.5;
/// Number of classes in the matched task.
const N_CLASSES: usize = 2;
/// Depths under test, one arm each.
const DEPTHS: [usize; 4] = [1, 2, 3, 4];
/// SGD step sizes. Every rung is run and reported; see the module header for how
/// the headline rung is picked.
const LR_LADDER: [f32; 5] = [1e-3, 3e-3, 1e-2, 3e-2, 1e-1];
/// Feedback-alignment rate for the learned-feedback treatment, as in v134.
const FEEDBACK_LR: f32 = 0.01;

/// One trainable unit of the grid. Every cell is independent given the seed.
#[derive(Clone, Copy, Debug)]
enum Job {
    /// **Headline treatment.** Learned feedback alignment at `depth`, Adam.
    TreatmentAdam { depth_idx: usize },
    /// **Headline ceiling.** True BPTT at `depth`, Adam — same optimiser, same
    /// hyper-parameters, differing from the treatment only in whether the
    /// gradients are true or feedback-projected.
    CeilingAdam { depth_idx: usize },
    /// Diagnostic: the same treatment under plain SGD at `LR_LADDER[lr_idx]`.
    TreatmentSgd { depth_idx: usize, lr_idx: usize },
    /// Diagnostic: the same ceiling under plain SGD at the same step size.
    CeilingSgd { depth_idx: usize, lr_idx: usize },
}

/// What one job produced.
#[derive(Clone, Copy, Debug, Default)]
struct JobOutcome {
    accuracy: f32,
    /// Realised RMS of the credit modulator reaching the **input** layer.
    /// Zero for the ceiling arms, which do not project through a feedback matrix.
    input_modulator_rms: f32,
}

fn build_model(depth: usize, hidden: usize, seed: u64) -> SharedTemporalNet {
    let widths = vec![hidden; depth];
    SharedTemporalNet::new(
        TASK_N_IN,
        binn_learn::REFERENCE_SEQUENCE_LEN,
        N_CLASSES,
        &widths,
        (-1.0f32 / DEFAULT_TAU_M).exp(),
        THETA_REST,
        DEFAULT_MATCHED_BETA,
        seed,
    )
}

impl Job {
    fn run(
        self,
        hidden: usize,
        epochs: usize,
        seed: u64,
        train: &[DenseTemporalExample],
        test: &[DenseTemporalExample],
    ) -> JobOutcome {
        match self {
            Job::TreatmentAdam { depth_idx } => {
                let mut model = build_model(DEPTHS[depth_idx], hidden, seed);
                let mut feedback = random_feedback(&model, seed);
                train_learned_feedback_adam(&mut model, &mut feedback, train, epochs, FEEDBACK_LR);
                // The modulator is read at the trained parameters, on the held-out
                // split, without applying an update. Layer 0 is the input layer:
                // it is the deepest the credit has to travel, and the first place
                // an attenuating transport shows up.
                let rms = model.feedback_modulator_rms(test, &feedback);
                JobOutcome {
                    accuracy: model.accuracy(test),
                    input_modulator_rms: rms.first().copied().unwrap_or(0.0),
                }
            }
            Job::CeilingAdam { depth_idx } => {
                let mut model = build_model(DEPTHS[depth_idx], hidden, seed);
                train_bptt(&mut model, train, epochs);
                JobOutcome {
                    accuracy: model.accuracy(test),
                    input_modulator_rms: 0.0,
                }
            }
            Job::TreatmentSgd { depth_idx, lr_idx } => {
                let mut model = build_model(DEPTHS[depth_idx], hidden, seed);
                let mut feedback = random_feedback(&model, seed);
                train_learned_feedback(
                    &mut model,
                    &mut feedback,
                    train,
                    epochs,
                    LR_LADDER[lr_idx],
                    FEEDBACK_LR,
                );
                JobOutcome {
                    accuracy: model.accuracy(test),
                    input_modulator_rms: 0.0,
                }
            }
            Job::CeilingSgd { depth_idx, lr_idx } => {
                let mut model = build_model(DEPTHS[depth_idx], hidden, seed);
                train_bptt_sgd(&mut model, train, epochs, LR_LADDER[lr_idx]);
                JobOutcome {
                    accuracy: model.accuracy(test),
                    input_modulator_rms: 0.0,
                }
            }
        }
    }
}

/// The full job list, in a fixed order so the ordered fold is deterministic.
fn job_grid() -> Vec<Job> {
    let mut jobs = Vec::new();
    for depth_idx in 0..DEPTHS.len() {
        jobs.push(Job::TreatmentAdam { depth_idx });
        jobs.push(Job::CeilingAdam { depth_idx });
        for lr_idx in 0..LR_LADDER.len() {
            jobs.push(Job::TreatmentSgd { depth_idx, lr_idx });
            jobs.push(Job::CeilingSgd { depth_idx, lr_idx });
        }
    }
    jobs
}

/// Accuracies for one `(depth, lr)` SGD diagnostic cell across seeds.
#[derive(Clone, Debug, Default)]
struct SgdCell {
    treatment: Vec<f32>,
    ceiling: Vec<f32>,
}

/// The headline Adam pair at one depth, across seeds.
#[derive(Clone, Debug, Default)]
struct AdamArm {
    treatment: Vec<f32>,
    ceiling: Vec<f32>,
    treatment_modulator: Vec<f32>,
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
    let hidden = if quick { 64 } else { 128 };
    let epochs = if quick { 20 } else { 60 };
    let master_seed = if quick {
        0x0071_AC00_0136_00F3_u64
    } else {
        0x0071_AC00_0136_00F4_u64
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

    let jobs = job_grid();

    // Every cell is seeded solely by `seed` and reads the split immutably, so no
    // cell can observe any other. rayon's `map(..).collect()` preserves input
    // order and there is no cross-seed accumulator inside the parallel region,
    // so the result is order-independent and reproducible at any thread count.
    let per_seed: Vec<Vec<JobOutcome>> = (0..n_seeds)
        .into_par_iter()
        .map(|s_idx| {
            let seed = master_seed ^ ((s_idx as u64) * 0x1000_0005_u64);
            let split = freeze_trials(&base_cfg, seed);
            let train_data = samples_to_dense_temporal_examples(&split.train, TASK_N_IN);
            let test_data = samples_to_dense_temporal_examples(&split.test, TASK_N_IN);
            jobs.par_iter()
                .map(|job| job.run(hidden, epochs, seed, &train_data, &test_data))
                .collect()
        })
        .collect();

    // Ordered fold, outside the parallel region.
    let mut sgd: Vec<Vec<SgdCell>> = vec![vec![SgdCell::default(); LR_LADDER.len()]; DEPTHS.len()];
    let mut adam: Vec<AdamArm> = vec![AdamArm::default(); DEPTHS.len()];
    for outcomes in &per_seed {
        for (job, outcome) in jobs.iter().zip(outcomes) {
            match *job {
                Job::TreatmentAdam { depth_idx } => {
                    adam[depth_idx].treatment.push(outcome.accuracy);
                    adam[depth_idx]
                        .treatment_modulator
                        .push(outcome.input_modulator_rms);
                }
                Job::CeilingAdam { depth_idx } => adam[depth_idx].ceiling.push(outcome.accuracy),
                Job::TreatmentSgd { depth_idx, lr_idx } => {
                    sgd[depth_idx][lr_idx].treatment.push(outcome.accuracy)
                }
                Job::CeilingSgd { depth_idx, lr_idx } => {
                    sgd[depth_idx][lr_idx].ceiling.push(outcome.accuracy)
                }
            }
        }
    }

    // Best SGD rung per depth, on the ceiling alone. Reported as a diagnostic;
    // it decides nothing.
    let best_rung: Vec<usize> = (0..DEPTHS.len())
        .map(|d| {
            (0..LR_LADDER.len())
                .max_by(|&a, &b| {
                    mean(&sgd[d][a].ceiling)
                        .partial_cmp(&mean(&sgd[d][b].ceiling))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .expect("the ladder is non-empty")
        })
        .collect();

    let mut headline_rows = String::new();
    let mut ladder_rows = String::new();
    let mut any_treatment_pass = false;
    let mut any_ceiling_defect = false;

    for (d, &depth) in DEPTHS.iter().enumerate() {
        let arm = &adam[d];
        let t_mean = mean(&arm.treatment);
        let t_se = std_error(&arm.treatment);
        let c_mean = mean(&arm.ceiling);
        let c_se = std_error(&arm.ceiling);

        let health = CeilingHealth::evaluate(c_mean, t_mean, CHANCE);
        if !health.is_usable() {
            any_ceiling_defect = true;
        }
        let verdict = Verdict::evaluate_mean(
            t_mean,
            ACCURACY_FLOOR,
            n_seeds,
            REQUIRED_SEEDS,
            health.is_usable(),
        );
        if verdict.is_citable_as_positive() {
            any_treatment_pass = true;
        }

        headline_rows.push_str(&format!(
            "| {depth} | {hidden} x {depth} | {t_mean:.4} | {t_se:.4} | {c_mean:.4} | \
             {c_se:.4} | {:+.4} | {:.3e} | {} | {} |\n",
            t_mean - c_mean,
            mean(&arm.treatment_modulator),
            health.label(),
            verdict.label(),
        ));

        for (l, lr) in LR_LADDER.iter().enumerate() {
            ladder_rows.push_str(&format!(
                "| {depth} | {lr:.0e} | {:.4} | {:.4} | {} |\n",
                mean(&sgd[d][l].treatment),
                mean(&sgd[d][l].ceiling),
                if l == best_rung[d] { "best rung" } else { "" },
            ));
        }
    }

    let deepest = DEPTHS.len() - 1;
    let deepest_arm = &adam[deepest];
    let deepest_health = CeilingHealth::evaluate(
        mean(&deepest_arm.ceiling),
        mean(&deepest_arm.treatment),
        CHANCE,
    );
    let overall = Verdict::evaluate_mean(
        mean(&deepest_arm.treatment),
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        deepest_health.is_usable(),
    );

    // A defect banner is emitted before any number, so a reader who stops at the
    // first table cannot miss it. Empty when every reference cleared chance.
    let harness_banner = if any_ceiling_defect {
        "> **HARNESS DEFECT - do not interpret any comparison below.** At least one \
         depth-matched ceiling failed its health check: a reference must clear chance \
         before anything can be measured against it. Arm verdicts are reported as \
         `INVALID_HARNESS` rather than PASS or FAIL for exactly this reason.\n\n"
    } else {
        ""
    };

    let summary = format!(
        "# Deep SNN Scaling Report\n\n\
        {harness_banner}\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Schedule:** {} (n={n_seeds}, hidden={hidden}, epochs={epochs})  \n\
        **Accuracy floor:** {ACCURACY_FLOOR:.2}  \n\
        **Preregistration:** `results/PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` \
        section 7  \n\n\
        Treatment and ceiling share one forward graph, one initialisation, one \
        optimiser and one step size. They differ in the credit pathway and in nothing \
        else: the treatment projects the readout error through a learned feedback \
        matrix, the ceiling uses exact reverse-mode gradients. The Adam ceiling is the \
        best-achievable reference for the same architecture and does **not** decide \
        ceiling health, because it differs from the treatment in two things at once.\n\n\
        ## Headline, per depth\n\n\
        Both arms use Adam at the module's frozen settings. There is no step size to \
        choose and nothing was tuned on either arm.\n\n\
        | Depth | Hidden | Treatment | SE | Ceiling | SE | Gap | \
        Input modulator RMS | Ceiling health | Verdict |\n\
        |---:|---|---:|---:|---:|---:|---:|---:|---|---|\n\
        {headline_rows}\n\
        `Gap` is treatment minus ceiling; negative means the treatment is below its \
        own reference. `Input modulator RMS` is the realised scale of the credit \
        signal reaching the input layer - the deepest the credit has to travel. If it \
        collapses with depth, the comparison is measuring effective step size rather \
        than credit-assignment quality.\n\n\
        ## Why Adam, and what plain SGD does\n\n\
        The optimiser is matched across the two arms either way. Adam is used because \
        plain SGD cannot train this architecture at depth: the table below is the full \
        registered step-size ladder, run for **both** arms at every depth. The \
        selection that led here read the **ceiling** only - a reference that cannot \
        learn bounds nothing - and never the treatment.\n\n\
        | Depth | SGD lr | Treatment | Ceiling | |\n\
        |---:|---:|---:|---:|---|\n\
        {ladder_rows}\n\
        ## Verdict\n\n\
        - Deepest ({}-layer) learned feedback alignment: **{}**\n\
        - Any depth clearing the floor: **{}**\n\n\
        ## Interpretation caveat\n\n\
        The matched-architecture task (`CoincidenceTask`) has **N_IN = {TASK_N_IN}** and \
        `difficulty = 0.05`. A {hidden}-wide, {}-deep stack on a {TASK_N_IN}-dimensional \
        near-noiseless input has no depth structure to exploit, so neither a depth \
        collapse nor a depth success on this task is strong evidence about deep credit \
        assignment. Move this suite to an input-rich task before drawing a scaling \
        conclusion.\n\n\
        ## Provenance\n\n\
        v134 and v135 of this suite are **withdrawn** and are not comparable with \
        anything here. They compared a ceiling that was silent above layer 1 against a \
        treatment that was silent above layer 1 **and** had no readout bias, so its \
        decision boundary was pinned at the origin. See \
        `results/PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` sections 1-2 for \
        the measurements, and `results/RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md` \
        for the original withdrawal.\n",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        },
        DEPTHS[deepest],
        overall.label(),
        if any_treatment_pass { "yes" } else { "no" },
        DEPTHS[deepest],
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
