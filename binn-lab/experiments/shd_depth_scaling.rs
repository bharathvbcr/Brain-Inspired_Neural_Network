//! Does learned feedback alignment track its BPTT ceiling as depth grows, on a
//! task with real headroom? Protocol v151.
//!
//! # Why this binary exists
//!
//! `deep-snn-scaling` v136 asked exactly this question and answered it on
//! `CoincidenceTask`, which has `N_IN = 2`. Its ceiling reached **exactly
//! 1.0000** at depths 2–4, and its own report says what that costs:
//!
//! > A saturated reference has no headroom, so "the treatment tracks its
//! > ceiling" is close to "both arms solved an easy task". […] Moving the suite
//! > to an input-rich task remains open work.
//!
//! This is that move. Same comparison, same module, same discipline; 140 input
//! channels, 100 timesteps and 20 classes instead of 2 inputs, 8 timesteps and
//! 2 classes. The treatment ([`train_learned_feedback_adam`]) and the ceiling
//! ([`train_bptt`]) share one forward graph, one initialisation, one optimiser
//! and one frozen step size, and differ **only** in whether the gradients are
//! true or feedback-projected.
//!
//! Preregistration: `results/PREREG_2026-08-22_DEEP_DEPTH_ON_SHD.md`.
//!
//! # THIS BINARY'S CAMPAIGN CANNOT CURRENTLY RUN, AND THAT IS CORRECT
//!
//! `run` trains a non-gradient credit rule on SHD and reports its accuracy, so
//! it requests `CampaignKind::LocalLearning`, which `binn_lab::authorize_campaign`
//! refuses while `SHD_INSTRUMENT_STATE` is `Uncalibrated`.
//! `results/SHD_INSTRUMENT_STATUS.md` blocks "new SHD local-learning or
//! architecture-ablation campaigns" outright, and its siblings
//! `shd-arch-ablation`, `shd-frozen-attention` and `shd-scientific-sweep` are
//! refused identically. That constant has no flag and no environment override,
//! by design; **do not flip it to run this.** It *is* the claim that the
//! instrument measures what it says it measures, and flipping it would falsify
//! the work rather than unblock it. See
//! `results/BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md` for the same
//! situation stated in full.
//!
//! # `activity-probe` is authorized, and is not the campaign
//!
//! The one thing that must be known **before** a depth verdict could mean
//! anything is whether this architecture is inside its activity band at this
//! operating point at all — a stack that is silent above layer 1 produced two
//! withdrawn results in this workspace already. `activity-probe` measures
//! initialisation firing rate per layer and the realised cost per example, on
//! real SHD frames. It applies **no parameter update** — asserted by comparing
//! [`SharedTemporalNet::parameter_fingerprint`] before and after — and computes
//! no accuracy, so it is `CampaignKind::HarnessValidation`, the same class as
//! `shd-instrument temporal-sensitivity`. Cost is a timing, not a result:
//! `scripts/gate_f_rust.py` excludes `wall_secs` from its compared fields for
//! that reason.
//!
//! # Registered operating point, and why nothing in it was chosen here
//!
//! | knob | value | owner it comes from |
//! |---|---|---|
//! | contract | `fixed-t100` | the 216 recorded instrument cells (`rust__fixed-t100__…`) |
//! | geometry | `adjacent-sum-5`, 140 inputs | the same recorded cells |
//! | `alpha` | `exp(-dt_ms / MATCHED_PHYSICAL_TAU_MS)` | `shd_matched::loss_and_gradient` |
//! | threshold | `THETA_REST` | `binn_engine::cell` |
//! | surrogate beta | `DEFAULT_MATCHED_BETA` | `matched_local_baseline` |
//! | optimiser | Adam at the module's frozen `ADAM_LR` | `shared_bptt` |
//! | input values | raw event counts, unscaled | `shd_matched::loss_and_gradient` |
//! | activity band | `[ACTIVITY_MIN, ACTIVITY_MAX]` | `shd_alif` |
//! | ceiling health | `CeilingHealth` against the realised majority rate | `binn_lab::guards` |
//!
//! There is no free parameter set by this experiment. That is deliberate: the
//! only defensible way to run a comparison on an instrument this workspace has
//! twice caught measuring nothing is to inherit every constant from a named
//! owner and let the registered validity gates decide whether the result exists.
//!
//! # Why there is no SGD step-size ladder here
//!
//! v136 ran the full registered ladder for both arms and recorded that at
//! depth >= 2 the **SGD ceiling sits at exactly chance at every rung**, while
//! the Adam ceiling learns. A reference that cannot learn bounds nothing. The
//! ladder is therefore not re-run at 70x the cost per cell; the finding it
//! produced is cited instead, and the pair is matched at Adam.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use rayon::prelude::*;

use binn_data::{FrequencyGeometry, ShdEventContract, SHD_CHANCE, SHD_N_CLASSES};
use binn_engine::THETA_REST;
use binn_lab::guards::{CeilingHealth, Verdict};
use binn_lab::{
    authorize_campaign, class_histogram, contract_alpha, contract_timesteps,
    load_shd_dense_examples, majority_class_rate, mean, std_error, CampaignKind,
};
use binn_learn::shd_alif::{ACTIVITY_MAX, ACTIVITY_MIN};
use binn_learn::{
    random_feedback, train_bptt, train_learned_feedback_adam, DenseTemporalExample,
    SharedTemporalNet, DEFAULT_MATCHED_BETA,
};

const PROTOCOL_VERSION: u64 = 151;
const EXPERIMENT_NAME: &str = "shd-depth-scaling";

/// Depths under test, one treatment/ceiling pair each. Same grid as v136, so
/// the two runs are readable side by side.
const DEPTHS: [usize; 4] = [1, 2, 3, 4];
/// Preregistered seed requirement for a scientific verdict.
const REQUIRED_SEEDS: usize = 12;
/// Preregistered floor for citing a treatment arm as a positive: five times the
/// 0.05 chance rate of a 20-class task.
const ACCURACY_FLOOR: f32 = 0.25;
/// Preregistered two-sided band on `treatment - ceiling` within which the
/// treatment counts as tracking its ceiling. Unchanged from v136.
const GAP_TOLERANCE: f32 = 0.05;
/// **The condition v136 could not meet.** A ceiling above this has no headroom
/// left, so "the treatment tracks it" stops being a statement about credit
/// assignment. v136's ceiling read exactly 1.0000 at depths 2-4.
const HEADROOM_MAX: f32 = 0.95;
/// Feedback-alignment rate for the treatment, as in v136.
const FEEDBACK_LR: f32 = 0.01;
/// Registered capped splits, matching `shd-arch-ablation` and
/// `shd-frozen-attention` so the three are readable side by side.
const CAPPED_TRAIN: usize = 2000;
const CAPPED_TEST: usize = 500;
const DEFAULT_HIDDEN: usize = 128;
/// Matches the `__e20__` epoch budget of the 216 recorded instrument cells.
const DEFAULT_EPOCHS: usize = 20;
/// Samples the activity probe reads. Diagnostic only.
const DEFAULT_PROBE_SAMPLES: usize = 64;
/// Examples the cost probe times per arm and depth. Diagnostic only.
const DEFAULT_PROBE_STEPS: usize = 8;

const MASTER_SEED_FULL: u64 = 0x0071_AC00_0151_00F1_u64;
const MASTER_SEED_QUICK: u64 = 0x0071_AC00_0151_00F0_u64;

fn registered_contract() -> ShdEventContract {
    ShdEventContract::fixed(100).expect("fixed-t100 is a registered contract")
}

const REGISTERED_GEOMETRY: FrequencyGeometry = FrequencyGeometry::AdjacentSum5;

/// Default location of the count-preserving event caches written by
/// `scripts/shd_calibration/data.py`.
fn default_events_dir() -> PathBuf {
    PathBuf::from("data/shd/events")
}

fn build_model(
    depth: usize,
    hidden: usize,
    n_in: usize,
    timesteps: usize,
    seed: u64,
) -> SharedTemporalNet {
    let widths = vec![hidden; depth];
    SharedTemporalNet::new(
        n_in,
        timesteps,
        SHD_N_CLASSES,
        &widths,
        contract_alpha(registered_contract()),
        THETA_REST,
        DEFAULT_MATCHED_BETA,
        seed,
    )
}

/// One trainable unit of the grid. Every cell is independent given the seed.
#[derive(Clone, Copy, Debug)]
enum Job {
    /// **Headline treatment.** Learned feedback alignment at `depth`, Adam.
    Treatment { depth_idx: usize },
    /// **Headline ceiling.** True BPTT at `depth`, Adam — same optimiser, same
    /// hyper-parameters, differing from the treatment only in whether the
    /// gradients are true or feedback-projected.
    Ceiling { depth_idx: usize },
}

/// What one job produced.
#[derive(Clone, Debug, Default)]
struct JobOutcome {
    accuracy: f32,
    /// Realised RMS of the credit modulator reaching each hidden layer, index 0
    /// being the input layer. Empty for the ceiling arms, which do not project
    /// through a feedback matrix.
    modulator_rms: Vec<f32>,
    /// Distinct classes the arm ever predicted on the held-out split. `1` is a
    /// collapse to a constant predictor.
    distinct_predicted: usize,
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
        let n_in = train[0].n_in;
        let timesteps = train[0].timesteps;
        match self {
            Job::Treatment { depth_idx } => {
                let mut model = build_model(DEPTHS[depth_idx], hidden, n_in, timesteps, seed);
                let mut feedback = random_feedback(&model, seed);
                train_learned_feedback_adam(&mut model, &mut feedback, train, epochs, FEEDBACK_LR);
                // Read at the trained parameters, on the held-out split, without
                // applying an update. Index 0 is the input layer: the deepest
                // the credit has to travel, and the first place an attenuating
                // transport shows up.
                JobOutcome {
                    accuracy: model.accuracy(test),
                    modulator_rms: model.feedback_modulator_rms(test, &feedback),
                    distinct_predicted: distinct_predicted(&model, test),
                }
            }
            Job::Ceiling { depth_idx } => {
                let mut model = build_model(DEPTHS[depth_idx], hidden, n_in, timesteps, seed);
                train_bptt(&mut model, train, epochs);
                JobOutcome {
                    accuracy: model.accuracy(test),
                    modulator_rms: Vec::new(),
                    distinct_predicted: distinct_predicted(&model, test),
                }
            }
        }
    }
}

/// How many distinct classes an arm ever predicts.
///
/// On a 20-class task an arm can sit far above the two-class notion of chance
/// and still be a constant predictor of the majority class, so this is reported
/// beside every accuracy. `binn_lab::guards::ReadoutAudit` owns this check for
/// **binary** readouts and cannot be reused here; the collapse threshold comes
/// from `shd_alif::MAJORITY_PRED_MAX`, which is the multi-class owner.
fn distinct_predicted(model: &SharedTemporalNet, test: &[DenseTemporalExample]) -> usize {
    let mut seen = vec![false; SHD_N_CLASSES];
    for example in test {
        let predicted = model.forward(example).prediction as usize;
        if predicted < SHD_N_CLASSES {
            seen[predicted] = true;
        }
    }
    seen.into_iter().filter(|&s| s).count()
}

/// Mean firing rate of the deepest layer of `model`, and the fraction of its
/// units outside the activity band.
///
/// `SharedTemporalNet::forward` exposes the final layer's cumulative rate at the
/// last step, which is exactly `(1/T) * sum_t spike_t` for that layer. Layer `L`
/// is therefore read by building a depth-`L+1` model: the constructor draws
/// layers in order from one seeded stream, so layer `L` of a depth-`d` model is
/// bit-identical for every `d > L`. Pinned by
/// `prefix_layers_are_identical_across_depths`.
fn layer_activity(model: &SharedTemporalNet, samples: &[DenseTemporalExample]) -> (f32, f32, f32) {
    let mut total = 0.0f64;
    let mut count = 0usize;
    let mut silent = 0usize;
    let mut saturated = 0usize;
    for example in samples {
        for rate in model.forward(example).final_rates {
            total += f64::from(rate);
            count += 1;
            if rate < ACTIVITY_MIN {
                silent += 1;
            } else if rate > ACTIVITY_MAX {
                saturated += 1;
            }
        }
    }
    let n = count.max(1) as f64;
    (
        (total / n) as f32,
        (silent as f64 / n) as f32,
        (saturated as f64 / n) as f32,
    )
}

/// The headline pair at one depth, across seeds.
#[derive(Clone, Debug, Default)]
struct DepthArm {
    treatment: Vec<f32>,
    ceiling: Vec<f32>,
    /// One row per seed, one column per hidden layer.
    modulator: Vec<Vec<f32>>,
    treatment_distinct: Vec<usize>,
    ceiling_distinct: Vec<usize>,
}

/// Column-wise mean of a ragged-free matrix of per-layer values.
fn mean_per_layer(rows: &[Vec<f32>]) -> Vec<f32> {
    let Some(width) = rows.iter().map(Vec::len).max() else {
        return Vec::new();
    };
    (0..width)
        .map(|column| {
            let column_values: Vec<f32> = rows
                .iter()
                .filter_map(|row| row.get(column).copied())
                .collect();
            mean(&column_values)
        })
        .collect()
}

fn format_layer_values(values: &[f32]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(|value| format!("{value:.3e}"))
        .collect::<Vec<_>>()
        .join(" / ")
}

/// Whether a reference still has room above it for the treatment to fall into.
///
/// This is the gate `deep-snn-scaling` v136 did not have. Its ceiling read
/// exactly 1.0000 at depths 2-4, and its own report says what that costs: on a
/// saturated reference, "the treatment tracks its ceiling" is close to "both
/// arms solved an easy task". A ceiling above [`HEADROOM_MAX`] cannot support a
/// depth reading whatever the gap says.
fn ceiling_has_headroom(ceiling_mean: f32) -> bool {
    ceiling_mean <= HEADROOM_MAX
}

/// The registered validity gates V-1..V-5. Any one of them voids the depth
/// reading in either direction; see the preregistration section 5.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Validity {
    /// **V-1.** Some depth-matched ceiling failed [`CeilingHealth`].
    ceiling_defect: bool,
    /// **V-2.** Some ceiling is above [`HEADROOM_MAX`] — the defect this
    /// experiment exists to avoid.
    ceiling_saturated: bool,
    /// **V-3.** Some hidden layer's initialisation firing rate is outside the
    /// activity band.
    layer_outside_band: bool,
    /// **V-4.** Some seed's arm predicted a single class on the held-out split.
    constant_predictor: bool,
    /// **V-5.** The evaluation split does not carry all 20 classes.
    incomplete_eval_classes: bool,
}

impl Validity {
    /// Whether any registered gate fired, i.e. whether outcome O-0 applies.
    const fn voided(self) -> bool {
        self.ceiling_defect
            || self.ceiling_saturated
            || self.layer_outside_band
            || self.constant_predictor
            || self.incomplete_eval_classes
    }
}

/// The registered outcome names of the preregistration section 7, decided in the
/// order that document fixes rather than by a reader looking at the table.
///
/// `gaps` is `treatment - ceiling` per depth, in the [`DEPTHS`] order.
fn registered_outcome(
    validity: Validity,
    gaps: &[f32],
    treatment_means: &[f32],
    ceiling_means: &[f32],
) -> &'static str {
    if validity.voided() {
        return "O-0 - a registered validity gate fired; no depth verdict is issued";
    }
    let deepest = gaps.len().saturating_sub(1);
    let drift = gaps[deepest] - gaps[0];
    let within = gaps.iter().all(|gap| gap.abs() <= GAP_TOLERANCE);
    let above_tolerance_positive = gaps.iter().any(|&gap| gap > GAP_TOLERANCE);
    if above_tolerance_positive {
        // CeilingHealth classifies an inverted pair as a defect, so this is
        // normally unreachable; it is named so that it can never be presented
        // as a finding if it ever is reached.
        return "O-4 - treatment above the reference that bounds it; not a finding";
    }
    let both_below_floor = treatment_means.iter().all(|&m| m < ACCURACY_FLOOR)
        && ceiling_means.iter().all(|&m| m < ACCURACY_FLOOR);
    if both_below_floor {
        return "O-5 - both arms below the accuracy floor; a result about the budget, \
                not about feedback alignment";
    }
    if within {
        return "O-1 - the treatment tracks its ceiling at every depth; no depth \
                penalty detected on SHD";
    }
    if drift < -GAP_TOLERANCE {
        "O-2 - a depth penalty for learned feedback alignment"
    } else {
        "O-3 - a constant cost of feedback projection, not a depth effect"
    }
}

/// **O-6.** Whether the credit reaching the input layer collapses with depth.
///
/// If the modulator RMS at layer 0 falls by more than an order of magnitude
/// between the shallowest and the deepest arm, any gap is confounded with
/// effective step size and the caveat is attached to the headline rather than
/// to a footnote.
fn modulator_collapses_with_depth(shallowest: f32, deepest: f32) -> bool {
    shallowest.is_finite() && deepest.is_finite() && deepest * 10.0 < shallowest
}

/// Banner emitted **before any number**, so a reader who stops at the first
/// table cannot miss it. Empty exactly when every registered gate held.
fn validity_banner(validity: &Validity) -> String {
    let mut banner = String::new();
    if validity.ceiling_defect {
        banner.push_str(
            "> **HARNESS DEFECT - do not interpret any comparison below.** At least one \
             depth-matched ceiling failed its health check: a reference must clear the \
             realised majority-class rate before anything can be measured against it. \
             Arm verdicts are reported as `INVALID_HARNESS` rather than as a pass or a \
             failure for exactly this reason.\n\n",
        );
    }
    if validity.ceiling_saturated {
        banner.push_str(&format!(
            "> **NO HEADROOM - the depth reading does not count.** At least one ceiling \
             is above {HEADROOM_MAX:.2}, which is the condition this experiment exists to \
             avoid: on a saturated reference a treatment that tracks its ceiling is close \
             to both arms having solved an easy task. That is exactly what happened to \
             `deep-snn-scaling` v136, whose ceiling read 1.0000 at depths 2-4.\n\n",
        ));
    }
    if validity.layer_outside_band {
        banner.push_str(&format!(
            "> **OPERATING POINT OUTSIDE THE ACTIVITY BAND.** At least one hidden layer's \
             initialisation firing rate is outside `[{ACTIVITY_MIN}, {ACTIVITY_MAX}]`. A \
             silent or saturated layer carries no class signal, and two withdrawn results \
             in this workspace came from exactly that.\n\n",
        ));
    }
    if validity.constant_predictor {
        banner.push_str(
            "> **CONSTANT PREDICTOR.** At least one seed's arm assigned every held-out \
             utterance to a single class. On a 20-class task such an arm can still print \
             an accuracy that looks like learning; it is not one.\n\n",
        );
    }
    if validity.incomplete_eval_classes {
        banner.push_str(
            "> **EVALUATION SPLIT IS NOT WHAT IT CLAIMS.** Not every class is present in \
             the held-out split, so the realised chance rate is not the registered one \
             and no accuracy below is on the registered scale.\n\n",
        );
    }
    banner
}

fn print_help() {
    eprintln!(
        "{EXPERIMENT_NAME} COMMAND\n\n\
         Commands:\n\
           run             depth-matched treatment-vs-ceiling campaign on SHD\n\
           activity-probe  initialisation activity and per-example cost; trains nothing\n\n\
         Options for `run`:\n\
           --quick                pilot schedule (fewer seeds, narrower, fewer epochs)\n\
           --seeds N --hidden N --epochs N --train N --test N\n\
           --events DIR           directory holding train.events / test.events\n\
           --out FILE             write the markdown report here\n\n\
         Options for `activity-probe`:\n\
           --hidden N --samples N --steps N --events DIR --out FILE\n\n\
         `run` requests CampaignKind::LocalLearning and is refused while\n\
         SHD_INSTRUMENT_STATE is Uncalibrated. That is the gate working, not a\n\
         defect; see results/SHD_INSTRUMENT_STATUS.md."
    );
}

#[derive(Clone, Debug)]
struct Options {
    quick: bool,
    seeds: Option<usize>,
    hidden: Option<usize>,
    epochs: Option<usize>,
    train: Option<usize>,
    test: Option<usize>,
    samples: Option<usize>,
    steps: Option<usize>,
    events: PathBuf,
    out: Option<PathBuf>,
}

fn parse_options(argv: &[String]) -> Result<Options, String> {
    let mut options = Options {
        quick: false,
        seeds: None,
        hidden: None,
        epochs: None,
        train: None,
        test: None,
        samples: None,
        steps: None,
        events: default_events_dir(),
        out: None,
    };
    let mut i = 0usize;
    while i < argv.len() {
        let flag = argv[i].as_str();
        match flag {
            "--quick" => options.quick = true,
            "--seeds" | "--hidden" | "--epochs" | "--train" | "--test" | "--samples"
            | "--steps" => {
                i += 1;
                let raw = argv
                    .get(i)
                    .ok_or_else(|| format!("{flag} requires a value"))?;
                let parsed: usize = raw
                    .parse()
                    .map_err(|_| format!("{flag} requires a non-negative integer, got {raw:?}"))?;
                if parsed == 0 {
                    return Err(format!("{flag} must be positive"));
                }
                match flag {
                    "--seeds" => options.seeds = Some(parsed),
                    "--hidden" => options.hidden = Some(parsed),
                    "--epochs" => options.epochs = Some(parsed),
                    "--train" => options.train = Some(parsed),
                    "--test" => options.test = Some(parsed),
                    "--samples" => options.samples = Some(parsed),
                    _ => options.steps = Some(parsed),
                }
            }
            "--events" => {
                i += 1;
                let raw = argv
                    .get(i)
                    .ok_or_else(|| "--events requires a path".to_string())?;
                options.events = PathBuf::from(raw);
            }
            "--out" => {
                i += 1;
                let raw = argv
                    .get(i)
                    .ok_or_else(|| "--out requires a path".to_string())?;
                options.out = Some(PathBuf::from(raw));
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
        i += 1;
    }
    Ok(options)
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let command = argv.first().map(String::as_str).unwrap_or("help");
    if matches!(command, "help" | "-h" | "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }
    // Authorization is decided from the command alone and **before** the option
    // list is parsed, so a blocked campaign cannot be reached by a malformed
    // argument, and a typo in a flag cannot be mistaken for the gate refusing.
    let outcome = match command_kind(command) {
        Err(error) => Err(error),
        Ok(kind) => authorize_campaign(kind).and_then(|()| {
            let options = parse_options(&argv[1..])?;
            match command {
                "run" => run(&options),
                _ => probe(&options),
            }
        }),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

/// The authorization class each command belongs to.
///
/// `run` trains a non-gradient credit rule on SHD and reports its accuracy, so
/// it is a local-learning campaign. `activity-probe` applies no update and
/// computes no accuracy, so it is harness validation — the same class as
/// `shd-instrument temporal-sensitivity`.
fn command_kind(command: &str) -> Result<CampaignKind, String> {
    match command {
        "run" => Ok(CampaignKind::LocalLearning),
        "activity-probe" => Ok(CampaignKind::HarnessValidation),
        other => Err(format!("unknown command: {other}")),
    }
}

/// Load the registered capped splits once. The split is a property of the
/// dataset, not of the seed: unlike v136's synthetic task there is nothing to
/// resample, so seeds vary initialisation only.
fn load_splits(
    events: &Path,
    n_train: usize,
    n_test: usize,
) -> Result<(Vec<DenseTemporalExample>, Vec<DenseTemporalExample>), String> {
    let contract = registered_contract();
    let timesteps = contract_timesteps(contract)
        .ok_or_else(|| "the registered contract must carry a fixed step count".to_string())?;
    let train = load_shd_dense_examples(
        &events.join("train.events"),
        contract,
        REGISTERED_GEOMETRY,
        timesteps,
        Some(n_train),
    )?;
    let test = load_shd_dense_examples(
        &events.join("test.events"),
        contract,
        REGISTERED_GEOMETRY,
        timesteps,
        Some(n_test),
    )?;
    Ok((train, test))
}

fn probe(options: &Options) -> Result<(), String> {
    let hidden = options.hidden.unwrap_or(DEFAULT_HIDDEN);
    let samples = options.samples.unwrap_or(DEFAULT_PROBE_SAMPLES);
    let steps = options.steps.unwrap_or(DEFAULT_PROBE_STEPS);
    let contract = registered_contract();
    let (train, _) = load_splits(&options.events, samples.max(steps), samples.max(steps))?;
    let n_in = train[0].n_in;
    let timesteps = train[0].timesteps;
    let probe_set = &train[..samples.min(train.len())];
    let step_set = &train[..steps.min(train.len())];

    let mut activity_rows = String::new();
    let mut cost_rows = String::new();
    for (depth_idx, &depth) in DEPTHS.iter().enumerate() {
        let model = build_model(depth, hidden, n_in, timesteps, MASTER_SEED_FULL);
        let before = model.parameter_fingerprint();
        let (rate, silent, saturated) = layer_activity(&model, probe_set);
        let band = if !(ACTIVITY_MIN..=ACTIVITY_MAX).contains(&rate) {
            "OUTSIDE BAND"
        } else {
            "inside"
        };
        activity_rows.push_str(&format!(
            "| {} | {rate:.4} | {silent:.4} | {saturated:.4} | {band} |\n",
            depth_idx
        ));

        let feedback = random_feedback(&model, MASTER_SEED_FULL);
        let ceiling_start = Instant::now();
        for example in step_set {
            let _ = model.loss_and_gradients(example);
        }
        let ceiling_secs = ceiling_start.elapsed().as_secs_f64() / step_set.len() as f64;
        let treatment_start = Instant::now();
        for example in step_set {
            let _ = model.feedback_gradients(example, &feedback);
        }
        let treatment_secs = treatment_start.elapsed().as_secs_f64() / step_set.len() as f64;
        // Nothing above may have moved a parameter: `model` is held by shared
        // reference and `feedback` is never realigned.
        if model.parameter_fingerprint() != before {
            return Err(format!(
                "activity-probe mutated model parameters at depth {depth}; it must not train"
            ));
        }
        cost_rows.push_str(&format!(
            "| {depth} | {ceiling_secs:.4} | {treatment_secs:.4} |\n"
        ));
    }

    let report = format!(
        "# SHD depth scaling — instrument probe (no training, no accuracy)\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME} `activity-probe`  \n\
        **Authorization:** `CampaignKind::HarnessValidation`  \n\
        **Contract / geometry:** `{}` / `{}`  \n\
        **Hidden width:** {hidden}, **samples:** {}, **timed examples per arm:** {}  \n\n\
        No parameter is updated and no accuracy is computed. \
        `SharedTemporalNet::parameter_fingerprint` is compared before and after every \
        measurement and the probe fails if it moved.\n\n\
        ## Initialisation activity, per hidden layer\n\n\
        Layer `L` is read from a depth-`L+1` model, whose first `L+1` layers are \
        bit-identical to those of any deeper model at the same seed. The band is \
        `[{ACTIVITY_MIN}, {ACTIVITY_MAX}]`, owned by `binn_learn::shd_alif`.\n\n\
        | Layer | Mean firing rate | Silent fraction | Saturated fraction | Band |\n\
        |---:|---:|---:|---:|---|\n\
        {activity_rows}\n\
        ## Cost per example, forward plus gradient\n\n\
        Seconds per example, single-threaded, no optimiser step. Timings are not \
        results — `scripts/gate_f_rust.py` excludes `wall_secs` from its compared \
        fields for exactly this reason.\n\n\
        | Depth | Ceiling (true BPTT) | Treatment (feedback-projected) |\n\
        |---:|---:|---:|\n\
        {cost_rows}",
        contract.id(),
        REGISTERED_GEOMETRY.id(),
        probe_set.len(),
        step_set.len(),
    );
    println!("\n{report}");
    if let Some(path) = &options.out {
        fs::write(path, &report).map_err(|e| format!("write {}: {e}", path.display()))?;
        println!("Report saved to: {}", path.display());
    }
    Ok(())
}

fn run(options: &Options) -> Result<(), String> {
    let quick = options.quick;
    let n_seeds = options
        .seeds
        .unwrap_or(if quick { 3 } else { REQUIRED_SEEDS });
    let hidden = options
        .hidden
        .unwrap_or(if quick { 64 } else { DEFAULT_HIDDEN });
    let epochs = options
        .epochs
        .unwrap_or(if quick { 5 } else { DEFAULT_EPOCHS });
    let n_train = options
        .train
        .unwrap_or(if quick { 400 } else { CAPPED_TRAIN });
    let n_test = options
        .test
        .unwrap_or(if quick { 200 } else { CAPPED_TEST });
    let master_seed = if quick {
        MASTER_SEED_QUICK
    } else {
        MASTER_SEED_FULL
    };
    let contract = registered_contract();
    let started = Instant::now();

    println!("========================================================================");
    println!("SHD Depth Scaling Protocol v{PROTOCOL_VERSION}");
    println!(
        "Schedule: {} (n_seeds={n_seeds}, hidden={hidden}, epochs={epochs}, \
         n_train={n_train}, n_test={n_test})",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        }
    );
    println!("========================================================================\n");

    let (train, test) = load_splits(&options.events, n_train, n_test)?;
    let majority = majority_class_rate(&test, SHD_N_CLASSES);
    let test_histogram = class_histogram(&test, SHD_N_CLASSES);
    let classes_present = test_histogram.iter().filter(|&&c| c > 0).count();
    let n_in = train[0].n_in;
    let timesteps = train[0].timesteps;

    // Initialisation activity, measured before any training, on a prefix of the
    // training split. A stack that is silent above layer 1 has produced two
    // withdrawn results in this workspace; this makes that visible in the report
    // rather than in a post-mortem.
    let probe_set = &train[..DEFAULT_PROBE_SAMPLES.min(train.len())];
    let mut activity_rows = String::new();
    let mut any_layer_outside_band = false;
    for (depth_idx, _) in DEPTHS.iter().enumerate() {
        let model = build_model(DEPTHS[depth_idx], hidden, n_in, timesteps, master_seed);
        let (rate, silent, saturated) = layer_activity(&model, probe_set);
        let inside = (ACTIVITY_MIN..=ACTIVITY_MAX).contains(&rate);
        if !inside {
            any_layer_outside_band = true;
        }
        activity_rows.push_str(&format!(
            "| {depth_idx} | {rate:.4} | {silent:.4} | {saturated:.4} | {} |\n",
            if inside { "inside" } else { "OUTSIDE BAND" }
        ));
    }

    let jobs: Vec<Job> = (0..DEPTHS.len())
        .flat_map(|depth_idx| [Job::Treatment { depth_idx }, Job::Ceiling { depth_idx }])
        .collect();

    // Every cell is seeded solely by `seed` and reads both splits immutably, so
    // no cell can observe any other. rayon's `map(..).collect()` preserves input
    // order and there is no cross-seed accumulator inside the parallel region,
    // so the result is order-independent and reproducible at any thread count.
    let per_seed: Vec<Vec<JobOutcome>> = (0..n_seeds)
        .into_par_iter()
        .map(|s_idx| {
            let seed = master_seed ^ ((s_idx as u64) * 0x1000_0005_u64);
            jobs.par_iter()
                .map(|job| job.run(hidden, epochs, seed, &train, &test))
                .collect()
        })
        .collect();

    // Ordered fold, outside the parallel region.
    let mut arms: Vec<DepthArm> = vec![DepthArm::default(); DEPTHS.len()];
    for outcomes in &per_seed {
        for (job, outcome) in jobs.iter().zip(outcomes) {
            match *job {
                Job::Treatment { depth_idx } => {
                    arms[depth_idx].treatment.push(outcome.accuracy);
                    arms[depth_idx]
                        .modulator
                        .push(outcome.modulator_rms.clone());
                    arms[depth_idx]
                        .treatment_distinct
                        .push(outcome.distinct_predicted);
                }
                Job::Ceiling { depth_idx } => {
                    arms[depth_idx].ceiling.push(outcome.accuracy);
                    arms[depth_idx]
                        .ceiling_distinct
                        .push(outcome.distinct_predicted);
                }
            }
        }
    }

    let mut headline_rows = String::new();
    let mut modulator_rows = String::new();
    let mut any_ceiling_defect = false;
    let mut any_ceiling_saturated = false;
    let mut any_constant_predictor = false;
    let mut gaps: Vec<f32> = Vec::new();
    let mut treatment_means: Vec<f32> = Vec::new();
    let mut ceiling_means: Vec<f32> = Vec::new();
    let mut input_modulator: Vec<f32> = Vec::new();
    for (d, &depth) in DEPTHS.iter().enumerate() {
        let arm = &arms[d];
        let t_mean = mean(&arm.treatment);
        let c_mean = mean(&arm.ceiling);
        let health = CeilingHealth::evaluate(c_mean, t_mean, majority);
        if !health.is_usable() {
            any_ceiling_defect = true;
        }
        if !ceiling_has_headroom(c_mean) {
            any_ceiling_saturated = true;
        }
        gaps.push(t_mean - c_mean);
        treatment_means.push(t_mean);
        ceiling_means.push(c_mean);
        let per_layer = mean_per_layer(&arm.modulator);
        input_modulator.push(per_layer.first().copied().unwrap_or(f32::NAN));
        if arm
            .treatment_distinct
            .iter()
            .chain(&arm.ceiling_distinct)
            .any(|&distinct| distinct <= 1)
        {
            any_constant_predictor = true;
        }
        let verdict = Verdict::evaluate_mean(
            t_mean,
            ACCURACY_FLOOR,
            n_seeds,
            REQUIRED_SEEDS,
            health.is_usable(),
        );
        headline_rows.push_str(&format!(
            "| {depth} | {t_mean:.4} | {:.4} | {c_mean:.4} | {:.4} | {:+.4} | {} | {} | {} | {} |\n",
            std_error(&arm.treatment),
            std_error(&arm.ceiling),
            t_mean - c_mean,
            if ceiling_has_headroom(c_mean) {
                "ok"
            } else {
                "SATURATED"
            },
            arm.treatment_distinct.iter().min().copied().unwrap_or(0),
            arm.ceiling_distinct.iter().min().copied().unwrap_or(0),
            health.label(),
        ));
        modulator_rows.push_str(&format!(
            "| {depth} | {} | {} |\n",
            format_layer_values(&per_layer),
            verdict.label(),
        ));
    }

    let deepest = DEPTHS.len() - 1;
    let deepest_arm = &arms[deepest];
    let deepest_health = CeilingHealth::evaluate(
        mean(&deepest_arm.ceiling),
        mean(&deepest_arm.treatment),
        majority,
    );
    let overall = Verdict::evaluate_mean(
        mean(&deepest_arm.treatment),
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        deepest_health.is_usable(),
    );
    let gap_drift = gaps[deepest] - gaps[0];
    let tracks = gaps.iter().all(|gap| gap.abs() <= GAP_TOLERANCE);

    let validity = Validity {
        ceiling_defect: any_ceiling_defect,
        ceiling_saturated: any_ceiling_saturated,
        layer_outside_band: any_layer_outside_band,
        constant_predictor: any_constant_predictor,
        incomplete_eval_classes: classes_present != SHD_N_CLASSES,
    };
    let banner = validity_banner(&validity);
    let outcome = registered_outcome(validity, &gaps, &treatment_means, &ceiling_means);
    let modulator_collapse = modulator_collapses_with_depth(
        input_modulator[0],
        input_modulator[input_modulator.len() - 1],
    );

    let summary = format!(
        "# SHD Depth Scaling Report\n\n\
        {banner}\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Schedule:** {} (n={n_seeds}, hidden={hidden}, epochs={epochs})  \n\
        **Data:** SHD `{}` / `{}`, {n_in} inputs x {timesteps} steps, \
        {} train / {} test  \n\
        **Chance:** {SHD_CHANCE:.4}; **realised majority-class rate:** {majority:.4}; \
        **classes present in the eval split:** {classes_present} of {SHD_N_CLASSES}  \n\
        **Accuracy floor:** {ACCURACY_FLOOR:.2}; **gap tolerance:** {GAP_TOLERANCE:.2}; \
        **headroom bar:** ceiling <= {HEADROOM_MAX:.2}  \n\
        **Preregistration:** `results/PREREG_2026-08-22_DEEP_DEPTH_ON_SHD.md`  \n\
        **Wall time:** {:.1} s\n\n\
        Treatment and ceiling share one forward graph, one initialisation, one \
        optimiser and one frozen step size. They differ in the credit pathway and in \
        nothing else: the treatment projects the readout error through a learned \
        feedback matrix, the ceiling uses exact reverse-mode gradients. The split is a \
        property of the dataset rather than of the seed, so seeds vary initialisation \
        only.\n\n\
        ## Initialisation activity, per hidden layer\n\n\
        Measured before training, on a {}-sample prefix of the training split. Band \
        `[{ACTIVITY_MIN}, {ACTIVITY_MAX}]`, owned by `binn_learn::shd_alif`.\n\n\
        | Layer | Mean firing rate | Silent fraction | Saturated fraction | Band |\n\
        |---:|---:|---:|---:|---|\n\
        {activity_rows}\n\
        ## Headline, per depth\n\n\
        `Gap` is treatment minus ceiling; negative means the treatment is below its own \
        reference. `Min distinct` is the smallest number of classes any seed's arm ever \
        predicted, over {SHD_N_CLASSES}: a `1` is a constant predictor whatever its \
        accuracy reads.\n\n\
        | Depth | Treatment | SE | Ceiling | SE | Gap | Headroom | Min distinct (T) | \
        Min distinct (C) | Ceiling health |\n\
        |---:|---:|---:|---:|---:|---:|---|---:|---:|---|\n\
        {headline_rows}\n\
        ## Credit modulator reaching each layer, and the arm verdict\n\n\
        RMS of the feedback modulator at the trained parameters, on the held-out split, \
        without applying an update. Layer 0 is the input layer - the deepest the credit \
        has to travel, and the first place an attenuating transport shows up. If it \
        collapses with depth, the comparison is measuring effective step size rather \
        than credit-assignment quality.\n\n\
        | Depth | Modulator RMS by layer (0 = input) | Treatment verdict |\n\
        |---:|---|---|\n\
        {modulator_rows}\n\
        ## Verdict\n\n\
        - **Registered outcome: {}**\n\
        - Deepest ({}-layer) learned feedback alignment: **{}**\n\
        - Treatment within +/-{GAP_TOLERANCE:.2} of its ceiling at **every** depth: **{}**\n\
        - Gap drift, depth {} minus depth {}: **{:+.4}**\n\
        - Credit reaching the input layer collapses with depth (O-6): **{}**{}\n\n\
        ## What this may not claim\n\n\
        - **Nothing, if a banner is printed above.** A defective ceiling, a saturated \
        ceiling, an out-of-band operating point, a constant predictor and an incomplete \
        evaluation split each independently void the depth reading, and they are \
        reported before the numbers for that reason.\n\
        - **It is not comparable with `deep-snn-scaling` v136.** Different task, \
        different input dimensionality, different class count and a different chance \
        rate. The two share a module, not a measurement.\n\
        - **It does not touch the calibration matrix.** `SHD_INSTRUMENT_STATE` is \
        untouched and no recorded cell is re-derived here.\n\
        - **Seeds measure initialisation variance only.** The SHD split is fixed, so \
        the standard errors above do not include sampling variability of the data.\n",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        },
        contract.id(),
        REGISTERED_GEOMETRY.id(),
        train.len(),
        test.len(),
        started.elapsed().as_secs_f64(),
        probe_set.len(),
        outcome,
        DEPTHS[deepest],
        overall.label(),
        if tracks { "yes" } else { "no" },
        DEPTHS[deepest],
        DEPTHS[0],
        gap_drift,
        if modulator_collapse { "yes" } else { "no" },
        if modulator_collapse {
            " - the gap is confounded with effective step size and the outcome above \
             must be read with that attached, not as a statement about credit quality"
        } else {
            ""
        },
    );

    println!("\n{summary}");
    if let Some(path) = &options.out {
        fs::write(path, &summary).map_err(|e| format!("write {}: {e}", path.display()))?;
        println!("Report saved to: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dense(n_in: usize, timesteps: usize, label: u32, seed: u64) -> DenseTemporalExample {
        // Deterministic pseudo-counts in the same range real SHD frames occupy
        // under `fixed-t100` / `adjacent-sum-5`: mostly zero, occasionally 1-3.
        let mut state = seed | 1;
        let frames = (0..timesteps * n_in)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                match (state >> 60) % 8 {
                    0 => 1.0,
                    1 => 2.0,
                    _ => 0.0,
                }
            })
            .collect();
        DenseTemporalExample {
            frames,
            timesteps,
            n_in,
            label,
        }
    }

    /// The activity probe reads layer `L` from a depth-`L+1` model. That is only
    /// sound because the constructor draws hidden layers in order from one
    /// seeded stream, so the first `L+1` layers of a deeper model are identical.
    /// If that ever stops holding, every per-layer activity number this
    /// experiment reports becomes a different layer's number.
    #[test]
    fn prefix_layers_are_identical_across_depths() {
        let n_in = 7;
        let hidden = 5;
        let seed = 0x0BAD_C0DE;
        let shallow = build_model(1, hidden, n_in, 4, seed);
        let deep = build_model(4, hidden, n_in, 4, seed);
        let prefix = hidden * n_in + hidden;
        assert_eq!(
            shallow.parameter_values()[..prefix],
            deep.parameter_values()[..prefix],
            "layer 0 differs between a depth-1 and a depth-4 model at the same seed"
        );
        // And the models are genuinely different beyond that prefix, so the
        // assertion above is not comparing two identical objects.
        assert_ne!(shallow.parameter_count(), deep.parameter_count());
    }

    #[test]
    fn layer_activity_reports_a_silent_stack_as_silent() {
        let samples: Vec<DenseTemporalExample> = (0..4).map(|s| dense(6, 5, 0, 100 + s)).collect();
        let mut silent = SharedTemporalNet::new(
            6,
            5,
            SHD_N_CLASSES,
            &[4],
            0.5,
            1.0e6,
            DEFAULT_MATCHED_BETA,
            9,
        );
        let (rate, silent_fraction, saturated_fraction) = layer_activity(&silent, &samples);
        assert!(
            rate < ACTIVITY_MIN,
            "a stack whose threshold is unreachable must read as silent, got {rate}"
        );
        assert!((silent_fraction - 1.0).abs() < 1e-6);
        assert!(saturated_fraction < 1e-6);

        // The same probe at the registered threshold is not silent, so the check
        // above is measuring the stack rather than always firing.
        silent = SharedTemporalNet::new(
            6,
            5,
            SHD_N_CLASSES,
            &[4],
            0.5,
            0.01,
            DEFAULT_MATCHED_BETA,
            9,
        );
        let (live_rate, live_silent, _) = layer_activity(&silent, &samples);
        assert!(live_rate > ACTIVITY_MIN, "control stack read {live_rate}");
        assert!(live_silent < 1.0);
    }

    /// `distinct_predicted` counts **classes**, not samples. Counting samples
    /// would make the collapse check unfireable, because a healthy 500-sample
    /// split would read 500 either way.
    #[test]
    fn distinct_predicted_counts_classes_not_samples() {
        let model = build_model(1, 4, 6, 5, 31);
        let one = dense(6, 5, 0, 7);
        let repeated = vec![one.clone(), one.clone(), one.clone(), one];
        assert_eq!(
            distinct_predicted(&model, &repeated),
            1,
            "a deterministic model on four copies of one utterance predicts one class"
        );
        let varied: Vec<DenseTemporalExample> =
            (0..6).map(|s| dense(6, 5, (s % 3) as u32, 7 + s)).collect();
        let distinct = distinct_predicted(&model, &varied);
        assert!(distinct >= 1, "an arm always predicts something");
        assert!(distinct <= SHD_N_CLASSES.min(varied.len()));
    }

    /// The campaign itself cannot run — the gate refuses it — so the training
    /// call path is exercised here at toy scale instead. This is plumbing, not a
    /// measurement: the accuracies it produces are meaningless and are asserted
    /// only to be well-formed. What it does establish is that both arms execute,
    /// that the treatment reports **one modulator per hidden layer**, and that
    /// the ceiling reports none.
    #[test]
    fn both_arms_execute_and_only_the_treatment_reports_a_modulator_per_layer() {
        let n_in = 6;
        let timesteps = 5;
        let train: Vec<DenseTemporalExample> = (0..6)
            .map(|s| dense(n_in, timesteps, (s % 3) as u32, 41 + s))
            .collect();
        let test: Vec<DenseTemporalExample> = (0..4)
            .map(|s| dense(n_in, timesteps, (s % 3) as u32, 907 + s))
            .collect();

        // depth_idx 2 is DEPTHS[2] == 3 hidden layers.
        let treatment = Job::Treatment { depth_idx: 2 }.run(4, 1, 5, &train, &test);
        assert_eq!(
            treatment.modulator_rms.len(),
            DEPTHS[2],
            "the treatment must report one modulator per hidden layer"
        );
        assert!(treatment.modulator_rms.iter().all(|v| v.is_finite()));
        assert!((0.0..=1.0).contains(&treatment.accuracy));
        assert!(treatment.distinct_predicted >= 1);

        let ceiling = Job::Ceiling { depth_idx: 2 }.run(4, 1, 5, &train, &test);
        assert!(
            ceiling.modulator_rms.is_empty(),
            "the ceiling does not project through a feedback matrix"
        );
        assert!((0.0..=1.0).contains(&ceiling.accuracy));
    }

    #[test]
    fn mean_per_layer_averages_columns_not_rows() {
        let rows = vec![vec![1.0, 10.0], vec![3.0, 20.0]];
        assert_eq!(mean_per_layer(&rows), vec![2.0, 15.0]);
        assert!(mean_per_layer(&[]).is_empty());
        assert_eq!(format_layer_values(&[]), "-");
        assert_eq!(format_layer_values(&[1.5e-3]), "1.500e-3");
    }

    /// Each registered validity gate must produce its own banner, must void the
    /// depth reading, and a clean run must produce none. A banner that could not
    /// fire would let a saturated ceiling be read as a depth result — the exact
    /// defect this experiment was built to avoid.
    #[test]
    fn every_validity_gate_has_its_own_banner_voids_the_reading_and_a_clean_run_has_none() {
        assert!(validity_banner(&Validity::default()).is_empty());
        assert!(!Validity::default().voided());

        type SetGate = fn(&mut Validity);
        let cases: [(SetGate, &str); 5] = [
            (|v| v.ceiling_defect = true, "HARNESS DEFECT"),
            (|v| v.ceiling_saturated = true, "NO HEADROOM"),
            (|v| v.layer_outside_band = true, "ACTIVITY BAND"),
            (|v| v.constant_predictor = true, "CONSTANT PREDICTOR"),
            (|v| v.incomplete_eval_classes = true, "NOT WHAT IT CLAIMS"),
        ];
        let mut total = 0usize;
        for (set, marker) in cases {
            let mut validity = Validity::default();
            set(&mut validity);
            let banner = validity_banner(&validity);
            assert!(banner.contains(marker), "missing banner for {marker}");
            assert!(validity.voided(), "{marker} must void the depth reading");
            assert_eq!(
                registered_outcome(validity, &[0.0; 4], &[0.9; 4], &[0.9; 4]),
                "O-0 - a registered validity gate fired; no depth verdict is issued"
            );
            total += banner.len();
        }
        // The saturated banner must name the run it exists because of.
        let saturated = Validity {
            ceiling_saturated: true,
            ..Validity::default()
        };
        assert!(validity_banner(&saturated).contains("v136"));

        let all = validity_banner(&Validity {
            ceiling_defect: true,
            ceiling_saturated: true,
            layer_outside_band: true,
            constant_predictor: true,
            incomplete_eval_classes: true,
        });
        assert_eq!(all.len(), total, "banners must concatenate, not replace");
    }

    /// The registered outcome names of section 7 must be decided by the code, in
    /// the order that document fixes, rather than chosen by a reader.
    #[test]
    fn registered_outcomes_are_decided_in_the_order_the_prereg_fixes() {
        let clean = Validity::default();
        let high = [0.7f32; 4];

        // O-1: every gap inside tolerance.
        assert!(
            registered_outcome(clean, &[-0.01, 0.0, -0.02, -0.03], &high, &high).starts_with("O-1")
        );
        // O-2: gap negative and drifting further with depth.
        assert!(
            registered_outcome(clean, &[-0.01, -0.05, -0.10, -0.20], &high, &high)
                .starts_with("O-2")
        );
        // O-3: outside tolerance but flat.
        assert!(
            registered_outcome(clean, &[-0.20, -0.19, -0.21, -0.22], &high, &high)
                .starts_with("O-3")
        );
        // O-4: treatment above the reference that bounds it.
        assert!(registered_outcome(clean, &[0.0, 0.0, 0.0, 0.30], &high, &high).starts_with("O-4"));
        // O-5 dominates a gap reading: both arms below the floor.
        let low = [ACCURACY_FLOOR - 0.01; 4];
        assert!(registered_outcome(clean, &[0.0; 4], &low, &low).starts_with("O-5"));
        // ... but a ceiling above the floor is not O-5, even with a low treatment.
        assert!(registered_outcome(clean, &[0.0; 4], &low, &high).starts_with("O-1"));
    }

    /// O-6 is a caveat on the headline, and has to be able to fire.
    #[test]
    fn a_modulator_that_collapses_by_an_order_of_magnitude_is_flagged() {
        assert!(modulator_collapses_with_depth(1.0e-1, 9.0e-3));
        assert!(!modulator_collapses_with_depth(1.0e-1, 1.1e-2));
        assert!(!modulator_collapses_with_depth(1.0e-1, 1.0e-1));
        assert!(!modulator_collapses_with_depth(f32::NAN, 1.0e-3));
    }

    /// The headroom bar has to be able to reject the reading v136 produced, and
    /// has to accept the level this instrument's own converged ceiling reaches.
    #[test]
    fn the_headroom_bar_rejects_the_v136_ceiling_and_accepts_this_instruments() {
        // v136, depths 2-4.
        assert!(!ceiling_has_headroom(std::hint::black_box(1.0000)));
        // The converged `ff+fixed` ceiling recorded in SHD_INSTRUMENT_STATUS.md.
        assert!(ceiling_has_headroom(std::hint::black_box(0.7378)));
        assert!(ceiling_has_headroom(std::hint::black_box(HEADROOM_MAX)));
        assert!(!ceiling_has_headroom(std::hint::black_box(0.9501)));
    }

    /// The class each command belongs to, asserted here rather than only in the
    /// gate's own tests, so that re-labelling this campaign to something the
    /// gate happens to permit shows up as a failure in the experiment that did
    /// the re-labelling.
    #[test]
    fn the_campaign_is_a_local_learning_campaign_and_the_probe_is_not() {
        assert_eq!(command_kind("run"), Ok(CampaignKind::LocalLearning));
        assert_eq!(
            command_kind("activity-probe"),
            Ok(CampaignKind::HarnessValidation)
        );
        assert!(command_kind("nope").is_err());
        if binn_lab::SHD_INSTRUMENT_STATE == binn_lab::InstrumentState::Uncalibrated {
            assert!(
                authorize_campaign(CampaignKind::LocalLearning).is_err(),
                "the campaign must stay refused while the instrument is uncalibrated"
            );
            assert!(authorize_campaign(CampaignKind::HarnessValidation).is_ok());
        }
    }

    #[test]
    fn options_reject_a_zero_a_missing_value_and_an_unknown_flag() {
        let ok = parse_options(&["--seeds".into(), "4".into(), "--quick".into()])
            .expect("a well-formed option list parses");
        assert_eq!(ok.seeds, Some(4));
        assert!(ok.quick);
        assert!(parse_options(&["--seeds".into(), "0".into()]).is_err());
        assert!(parse_options(&["--seeds".into()]).is_err());
        assert!(parse_options(&["--nope".into()]).is_err());
        assert!(parse_options(&["--out".into()]).is_err());
    }

    /// The registered operating point is inherited, not chosen here. If any of
    /// these drifts, the report's provenance table is a lie.
    #[test]
    fn the_registered_operating_point_matches_the_recorded_instrument_cells() {
        let contract = registered_contract();
        assert_eq!(contract.id(), "fixed-t100");
        assert_eq!(REGISTERED_GEOMETRY.id(), "adjacent-sum-5");
        assert_eq!(REGISTERED_GEOMETRY.n_inputs(), 140);
        assert_eq!(contract_timesteps(contract), Some(100));
        assert_eq!(SHD_N_CLASSES, 20);
        assert!((SHD_CHANCE - 0.05).abs() < 1e-6);
        // `alpha` is the contract's own step against the matched physical tau,
        // not the synthetic task's `exp(-1/tau)`.
        let expected = (-contract.dt_ms() / binn_learn::MATCHED_PHYSICAL_TAU_MS).exp();
        assert!((contract_alpha(contract) - expected).abs() < 1e-7);
    }
}
