//! Protocol-v148 paired shortcut-accessibility contrast.
//!
//! This is one experiment, not two separately selectable runs. Both variants
//! use the same four-class examples, model, initialization, feedback, BPTT
//! reference, seed, and schedule. The sole intervention is whether class
//! identity is available in the per-channel count vector.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_data::{
    RateAccessibility, TemporalDifficulty, TemporalOrderExample, TemporalOrderSplit,
    RATE_ACCESSIBLE_MARKER_EVENTS, TEMPORAL_DIFFICULTIES, TEMPORAL_ORDER_N_CLASSES,
    TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
use binn_learn::{
    random_feedback, train_bptt, train_feedback, DenseTemporalExample, InputRateClassifier,
    InputRateConfig, SharedTemporalNet, ShdExample,
};

const PROTOCOL_VERSION: u64 = 148;
const MASTER_SEED: u64 = 0x7E4A_5148_0000_0001;
const N_SEEDS: usize = 3;
const N_TRAIN: usize = 200;
const N_TEST: usize = 100;
const EPOCHS: usize = 20;
const HIDDEN: usize = 64;
const LOCAL_LR: f32 = 0.005;
const RAW_RATE_HIGH: f32 = 0.90;
const RAW_RATE_CHANCE_MAX: f32 = 0.30;
const BPTT_FLOOR: f32 = 0.90;
const LOCAL_HIGH: f32 = 0.80;
const LOCAL_CHANCE_MAX: f32 = 0.30;
const ACTIVE_RATE_MIN: f32 = 0.01;
const SATURATED_RATE_MIN: f32 = 0.99;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    RateAccessible,
    RateImmune,
}

impl Variant {
    const ALL: [Self; 2] = [Self::RateAccessible, Self::RateImmune];

    const fn accessibility(self) -> RateAccessibility {
        match self {
            Self::RateAccessible => RateAccessibility::Accessible,
            Self::RateImmune => RateAccessibility::Immune,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::RateAccessible => "A. rate-accessible",
            Self::RateImmune => "B. rate-immune",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PredictionHealth {
    accuracy: f32,
    classes: usize,
    majority: f32,
}

#[derive(Clone, Copy, Debug)]
struct ActivitySummary {
    mean_rate: f32,
    active_fraction: f32,
    saturated_fraction: f32,
}

#[derive(Clone, Copy, Debug)]
struct VariantResult {
    seed: u64,
    variant: Variant,
    raw_rate: PredictionHealth,
    local_train: PredictionHealth,
    local_test: PredictionHealth,
    bptt_train: PredictionHealth,
    bptt_test: PredictionHealth,
    local_activity: ActivitySummary,
    bptt_activity: ActivitySummary,
    end_modulator_rms: f32,
    replay: bool,
    no_test_update: bool,
    pair_contract: bool,
}

fn main() -> ExitCode {
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::LocalLearning) {
        eprintln!("shortcut-accessibility-contrast: {error}");
        return ExitCode::from(3);
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shortcut-accessibility-contrast: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let expected_hash = config_hash();
    let mut out = PathBuf::from("results/shortcut_accessibility_contrast_v148.md");
    let args: Vec<String> = env::args().skip(1).collect();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                index += 1;
                out = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--out requires a path".to_string())?,
                );
            }
            "--config-hash" => {
                index += 1;
                let requested = args
                    .get(index)
                    .ok_or_else(|| "--config-hash requires a value".to_string())?;
                if requested != &expected_hash {
                    return Err(format!(
                        "config hash mismatch: requested {requested}, expected {expected_hash}"
                    ));
                }
            }
            "-h" | "--help" => {
                println!(
                    "Usage: shortcut-accessibility-contrast \
                     [--config-hash {expected_hash}] [--out PATH]"
                );
                return Ok(());
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }

    let difficulty = easiest_difficulty();
    let mut results = Vec::with_capacity(N_SEEDS * Variant::ALL.len());
    for seed_index in 0..N_SEEDS {
        let seed = MASTER_SEED ^ (seed_index as u64).wrapping_mul(0x1000_00EF);
        let immune = TemporalOrderSplit::generate_with_rate_accessibility(
            N_TRAIN,
            N_TEST,
            difficulty,
            seed,
            RateAccessibility::Immune,
        )?;
        let accessible = TemporalOrderSplit::generate_with_rate_accessibility(
            N_TRAIN,
            N_TEST,
            difficulty,
            seed,
            RateAccessibility::Accessible,
        )?;
        let pair_contract = paired_intervention_is_exact(&immune, &accessible);
        if !pair_contract {
            return Err(format!(
                "paired task intervention contract failed at seed {seed}"
            ));
        }

        for variant in Variant::ALL {
            let split = match variant {
                Variant::RateAccessible => &accessible,
                Variant::RateImmune => &immune,
            };
            results.push(run_variant(seed, variant, split, pair_contract)?);
        }
    }

    let accessible = aggregate(&results, Variant::RateAccessible);
    let immune = aggregate(&results, Variant::RateImmune);
    let mechanical_health = results.iter().all(|result| {
        result.replay
            && result.no_test_update
            && result.pair_contract
            && activity_is_finite(result.local_activity)
            && activity_is_finite(result.bptt_activity)
            && result.end_modulator_rms.is_finite()
            && result.end_modulator_rms > 0.0
    });
    let reference_health = accessible.raw_rate.accuracy >= RAW_RATE_HIGH
        && immune.raw_rate.accuracy <= RAW_RATE_CHANCE_MAX
        && accessible.bptt_test.accuracy >= BPTT_FLOOR
        && immune.bptt_test.accuracy >= BPTT_FLOOR
        && accessible.raw_rate.classes == TEMPORAL_ORDER_N_CLASSES
        && accessible.bptt_test.classes == TEMPORAL_ORDER_N_CLASSES
        && immune.bptt_test.classes == TEMPORAL_ORDER_N_CLASSES;
    let outcome = classify_outcome(accessible.local_test.accuracy, immune.local_test.accuracy);
    let verdict = if !mechanical_health || !reference_health {
        "INVALID_HARNESS — task/reference/mechanical control failed"
    } else {
        outcome
    };

    let mut seed_rows = String::new();
    for result in &results {
        seed_rows.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {}/4 | {:.3} | {:.4} | {:.4} | {:.4} | {:.3} | {:.3e} | {} |\n",
            result.seed,
            result.variant.label(),
            result.raw_rate.accuracy,
            result.local_train.accuracy,
            result.local_test.accuracy,
            result.local_test.classes,
            result.local_test.majority,
            result.bptt_test.accuracy,
            result.local_activity.mean_rate,
            result.local_activity.active_fraction,
            result.local_activity.saturated_fraction,
            result.end_modulator_rms,
            yes_no(result.replay),
        ));
    }

    let report = format!(
        "# Shortcut-accessibility contrast\n\n\
         **Protocol:** v{PROTOCOL_VERSION}  \n\
         **Hash:** `{expected_hash}`  \n\
         **Schedule:** one paired experiment; {N_SEEDS} fresh seeds; \
         {N_TRAIN}/{N_TEST}; {EPOCHS} epochs; hidden={HIDDEN}; local lr={LOCAL_LR:.3}  \n\
         **Verdict:** **{verdict}**\n\n\
         ## Frozen intervention\n\n\
         Both variants use the same four-class multiclass local arm, true shared-forward \
         BPTT reference, initialization, immutable feedback, labels, nuisance realization, \
         seed, and schedule. Variant A adds {RATE_ACCESSIBLE_MARKER_EVENTS} fixed-total \
         events on the class-indexed channel; variant B is the exact byte-identical-count \
         v144 task at `(jitter=0, distractors=4)`. No variant can be run separately.\n\n\
         | Variant | Channel counts | Raw-rate test | Local train | Local test | BPTT train | BPTT test | Local hidden mean | Local active frac | Local saturated frac | BPTT hidden mean | End local modulator RMS |\n\
         |---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n\
         | A. rate-accessible | class-dependent; fixed total | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.3e} |\n\
         | B. rate-immune | byte-identical within quartet | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.3e} |\n\n\
         Hidden activity is evaluated on held-out examples after training. `active frac` is \
         the fraction of final hidden rates >= {ACTIVE_RATE_MIN:.2}; `saturated frac` is the \
         fraction >= {SATURATED_RATE_MIN:.2}. End modulator RMS is evaluated after training \
         over the frozen training set without applying updates.\n\n\
         ## Per-seed audit\n\n\
         | Seed | Variant | Raw rate | Local train | Local test | Local classes | Majority | BPTT test | Local hidden mean | Active frac | Saturated frac | End mod RMS | Replay |\n\
         |---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n\
         {seed_rows}\n\
         Mechanical health: **{}**. Reference health: **{}**. Every evaluation preserved \
         parameter fingerprints: **{}**. The paired task intervention was exact for every \
         seed: **{}**.\n\n\
         ## Frozen interpretation\n\n\
         - A local >= {LOCAL_HIGH:.2} and B local <= {LOCAL_CHANCE_MAX:.2}: shortcut-accessibility finding; positive control passed.\n\
         - Both local <= {LOCAL_CHANCE_MAX:.2}: multiclass local positive control failed; prior multiclass-local interpretations are void.\n\
         - Both local >= {LOCAL_HIGH:.2}: v144 was a difficulty artifact rather than a rate-shortcut result.\n\
         - Any intermediate pattern: stop; do not promote a claim or add a sweep.\n",
        accessible.raw_rate.accuracy,
        accessible.local_train.accuracy,
        accessible.local_test.accuracy,
        accessible.bptt_train.accuracy,
        accessible.bptt_test.accuracy,
        accessible.local_activity.mean_rate,
        accessible.local_activity.active_fraction,
        accessible.local_activity.saturated_fraction,
        accessible.bptt_activity.mean_rate,
        accessible.end_modulator_rms,
        immune.raw_rate.accuracy,
        immune.local_train.accuracy,
        immune.local_test.accuracy,
        immune.bptt_train.accuracy,
        immune.bptt_test.accuracy,
        immune.local_activity.mean_rate,
        immune.local_activity.active_fraction,
        immune.local_activity.saturated_fraction,
        immune.bptt_activity.mean_rate,
        immune.end_modulator_rms,
        yes_no(mechanical_health),
        yes_no(reference_health),
        yes_no(results.iter().all(|result| result.no_test_update)),
        yes_no(results.iter().all(|result| result.pair_contract)),
    );
    write_report(&out, &report)?;
    println!("{report}");
    Ok(())
}

fn run_variant(
    seed: u64,
    variant: Variant,
    split: &TemporalOrderSplit,
    pair_contract: bool,
) -> Result<VariantResult, String> {
    assert_eq!(
        variant.accessibility(),
        match variant {
            Variant::RateAccessible => RateAccessibility::Accessible,
            Variant::RateImmune => RateAccessibility::Immune,
        }
    );
    let train = as_dense(&split.train);
    let test = as_dense(&split.test);
    let (raw_train, raw_test) = as_shd(&split.train, &split.test);

    let mut raw = InputRateClassifier::new(
        InputRateConfig {
            n_in: TEMPORAL_ORDER_N_IN,
            n_classes: TEMPORAL_ORDER_N_CLASSES,
            lr: LOCAL_LR,
            epochs: EPOCHS,
        },
        seed,
    );
    let raw_rate_report = raw.train_and_evaluate(&raw_train, &raw_test);
    let raw_rate = PredictionHealth {
        accuracy: raw_rate_report.accuracy,
        classes: raw_rate_report.n_distinct_predicted,
        majority: raw_rate_report.majority_pred_frac,
    };

    let initial = new_model(seed);
    let feedback = random_feedback(&initial, seed);
    let mut local = initial.clone();
    let local_diagnostics = train_feedback(&mut local, &feedback, &train, EPOCHS, LOCAL_LR);
    if local_diagnostics.is_empty() {
        return Err("local training emitted no diagnostics".into());
    }
    let local_train = prediction_health(&local, &train);
    let local_test = prediction_health(&local, &test);
    let local_activity = activity_summary(&local, &test);
    let end_modulator_rms = local
        .feedback_modulator_rms(&train, &feedback)
        .into_iter()
        .next()
        .ok_or_else(|| "local arm emitted no modulator layer".to_string())?;

    let mut local_replay = initial.clone();
    let local_replay_diagnostics =
        train_feedback(&mut local_replay, &feedback, &train, EPOCHS, LOCAL_LR);
    let local_replay_ok = local.parameter_fingerprint() == local_replay.parameter_fingerprint()
        && local_diagnostics == local_replay_diagnostics;

    let mut bptt = initial.clone();
    let bptt_diagnostics = train_bptt(&mut bptt, &train, EPOCHS);
    if bptt_diagnostics.is_empty() {
        return Err("BPTT training emitted no diagnostics".into());
    }
    let bptt_train = prediction_health(&bptt, &train);
    let bptt_test = prediction_health(&bptt, &test);
    let bptt_activity = activity_summary(&bptt, &test);

    let mut bptt_replay = initial;
    let bptt_replay_diagnostics = train_bptt(&mut bptt_replay, &train, EPOCHS);
    let bptt_replay_ok = bptt.parameter_fingerprint() == bptt_replay.parameter_fingerprint()
        && bptt_diagnostics == bptt_replay_diagnostics;

    Ok(VariantResult {
        seed,
        variant,
        raw_rate,
        local_train,
        local_test,
        bptt_train,
        bptt_test,
        local_activity,
        bptt_activity,
        end_modulator_rms,
        replay: local_replay_ok && bptt_replay_ok,
        no_test_update: raw_rate_report.no_test_update,
        pair_contract,
    })
}

fn aggregate(results: &[VariantResult], variant: Variant) -> VariantResult {
    let selected: Vec<_> = results
        .iter()
        .copied()
        .filter(|result| result.variant == variant)
        .collect();
    assert_eq!(selected.len(), N_SEEDS);
    VariantResult {
        seed: 0,
        variant,
        raw_rate: aggregate_health(selected.iter().map(|result| result.raw_rate)),
        local_train: aggregate_health(selected.iter().map(|result| result.local_train)),
        local_test: aggregate_health(selected.iter().map(|result| result.local_test)),
        bptt_train: aggregate_health(selected.iter().map(|result| result.bptt_train)),
        bptt_test: aggregate_health(selected.iter().map(|result| result.bptt_test)),
        local_activity: aggregate_activity(selected.iter().map(|result| result.local_activity)),
        bptt_activity: aggregate_activity(selected.iter().map(|result| result.bptt_activity)),
        end_modulator_rms: mean(selected.iter().map(|result| result.end_modulator_rms)),
        replay: selected.iter().all(|result| result.replay),
        no_test_update: selected.iter().all(|result| result.no_test_update),
        pair_contract: selected.iter().all(|result| result.pair_contract),
    }
}

fn aggregate_health(values: impl Iterator<Item = PredictionHealth>) -> PredictionHealth {
    let values: Vec<_> = values.collect();
    PredictionHealth {
        accuracy: mean(values.iter().map(|value| value.accuracy)),
        classes: values.iter().map(|value| value.classes).min().unwrap_or(0),
        majority: mean(values.iter().map(|value| value.majority)),
    }
}

fn aggregate_activity(values: impl Iterator<Item = ActivitySummary>) -> ActivitySummary {
    let values: Vec<_> = values.collect();
    ActivitySummary {
        mean_rate: mean(values.iter().map(|value| value.mean_rate)),
        active_fraction: mean(values.iter().map(|value| value.active_fraction)),
        saturated_fraction: mean(values.iter().map(|value| value.saturated_fraction)),
    }
}

fn classify_outcome(accessible_local: f32, immune_local: f32) -> &'static str {
    if accessible_local >= LOCAL_HIGH && immune_local <= LOCAL_CHANCE_MAX {
        "PASS — local learning depends on shortcut accessibility"
    } else if accessible_local <= LOCAL_CHANCE_MAX && immune_local <= LOCAL_CHANCE_MAX {
        "INVALID_HARNESS — multiclass local path failed its rate-accessible positive control"
    } else if accessible_local >= LOCAL_HIGH && immune_local >= LOCAL_HIGH {
        "FAIL — both variants learn; v144 was a difficulty artifact"
    } else {
        "FAIL — intermediate contrast; stop without a claim or follow-up sweep"
    }
}

fn paired_intervention_is_exact(
    immune: &TemporalOrderSplit,
    accessible: &TemporalOrderSplit,
) -> bool {
    if immune.seed != accessible.seed
        || immune.difficulty != accessible.difficulty
        || immune.train.len() != accessible.train.len()
        || immune.test.len() != accessible.test.len()
    {
        return false;
    }
    immune
        .train
        .iter()
        .chain(&immune.test)
        .zip(accessible.train.iter().chain(&accessible.test))
        .all(|(base, marked)| {
            if base.label != marked.label {
                return false;
            }
            let mut recovered = marked.clone();
            for event in 0..RATE_ACCESSIBLE_MARKER_EVENTS {
                let time = event * TEMPORAL_ORDER_T / RATE_ACCESSIBLE_MARKER_EVENTS;
                recovered.frames[time * TEMPORAL_ORDER_N_IN + base.label as usize] -= 1.0;
            }
            let base_total = base.frames.iter().sum::<f32>();
            let marked_total = marked.frames.iter().sum::<f32>();
            recovered == *base && marked_total - base_total == RATE_ACCESSIBLE_MARKER_EVENTS as f32
        })
}

fn prediction_health(
    model: &SharedTemporalNet,
    examples: &[DenseTemporalExample],
) -> PredictionHealth {
    let before = model.parameter_fingerprint();
    let mut counts = [0usize; TEMPORAL_ORDER_N_CLASSES];
    let mut correct = 0usize;
    for example in examples {
        let prediction = model.forward(example).prediction as usize;
        counts[prediction] += 1;
        correct += usize::from(prediction == example.label as usize);
    }
    assert_eq!(before, model.parameter_fingerprint());
    PredictionHealth {
        accuracy: correct as f32 / examples.len() as f32,
        classes: counts.iter().filter(|&&count| count > 0).count(),
        majority: counts.iter().copied().max().unwrap_or(0) as f32 / examples.len() as f32,
    }
}

fn activity_summary(
    model: &SharedTemporalNet,
    examples: &[DenseTemporalExample],
) -> ActivitySummary {
    let before = model.parameter_fingerprint();
    let rates: Vec<f32> = examples
        .iter()
        .flat_map(|example| model.forward(example).final_rates)
        .collect();
    assert_eq!(before, model.parameter_fingerprint());
    let n = rates.len() as f32;
    ActivitySummary {
        mean_rate: rates.iter().sum::<f32>() / n,
        active_fraction: rates
            .iter()
            .filter(|&&rate| rate >= ACTIVE_RATE_MIN)
            .count() as f32
            / n,
        saturated_fraction: rates
            .iter()
            .filter(|&&rate| rate >= SATURATED_RATE_MIN)
            .count() as f32
            / n,
    }
}

fn activity_is_finite(summary: ActivitySummary) -> bool {
    [
        summary.mean_rate,
        summary.active_fraction,
        summary.saturated_fraction,
    ]
    .iter()
    .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
}

fn new_model(seed: u64) -> SharedTemporalNet {
    SharedTemporalNet::new(
        TEMPORAL_ORDER_N_IN,
        TEMPORAL_ORDER_T,
        TEMPORAL_ORDER_N_CLASSES,
        &[HIDDEN],
        0.9,
        1.0,
        5.0,
        seed,
    )
}

fn easiest_difficulty() -> TemporalDifficulty {
    let difficulty = TEMPORAL_DIFFICULTIES[0];
    assert_eq!(difficulty, TemporalDifficulty::new(0, 4));
    difficulty
}

fn as_dense(examples: &[TemporalOrderExample]) -> Vec<DenseTemporalExample> {
    examples
        .iter()
        .map(|example| DenseTemporalExample {
            frames: example.frames.clone(),
            timesteps: TEMPORAL_ORDER_T,
            n_in: TEMPORAL_ORDER_N_IN,
            label: example.label,
        })
        .collect()
}

fn as_shd(
    train: &[TemporalOrderExample],
    test: &[TemporalOrderExample],
) -> (Vec<ShdExample>, Vec<ShdExample>) {
    let convert = |example: &TemporalOrderExample| ShdExample {
        frames: example.frames.clone(),
        t: TEMPORAL_ORDER_T,
        n_in: TEMPORAL_ORDER_N_IN,
        label: example.label,
    };
    (
        train.iter().map(convert).collect(),
        test.iter().map(convert).collect(),
    )
}

fn config_hash() -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for value in [
        PROTOCOL_VERSION,
        MASTER_SEED,
        N_SEEDS as u64,
        N_TRAIN as u64,
        N_TEST as u64,
        EPOCHS as u64,
        HIDDEN as u64,
        LOCAL_LR.to_bits() as u64,
        RATE_ACCESSIBLE_MARKER_EVENTS as u64,
        RAW_RATE_HIGH.to_bits() as u64,
        RAW_RATE_CHANCE_MAX.to_bits() as u64,
        BPTT_FLOOR.to_bits() as u64,
        LOCAL_HIGH.to_bits() as u64,
        LOCAL_CHANCE_MAX.to_bits() as u64,
        0x5348_4F52_5443_5554,
    ] {
        hash ^= value;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("shortcut-access-v148-{hash:016x}")
}

fn mean(values: impl Iterator<Item = f32>) -> f32 {
    let values: Vec<_> = values.collect();
    assert!(!values.is_empty());
    values.iter().sum::<f32>() / values.len() as f32
}

fn write_report(path: &Path, report: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, report).map_err(|error| error.to_string())
}

const fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_contains_both_variants_and_no_variant_selector() {
        assert_eq!(Variant::ALL.len(), 2);
        assert_ne!(
            Variant::RateAccessible.accessibility(),
            Variant::RateImmune.accessibility()
        );
    }

    #[test]
    fn paired_contract_accepts_only_the_frozen_marker_intervention() {
        let difficulty = easiest_difficulty();
        let immune = TemporalOrderSplit::generate_with_rate_accessibility(
            40,
            20,
            difficulty,
            7,
            RateAccessibility::Immune,
        )
        .unwrap();
        let mut accessible = TemporalOrderSplit::generate_with_rate_accessibility(
            40,
            20,
            difficulty,
            7,
            RateAccessibility::Accessible,
        )
        .unwrap();
        assert!(paired_intervention_is_exact(&immune, &accessible));
        accessible.train[0].frames[0] += 1.0;
        assert!(!paired_intervention_is_exact(&immune, &accessible));
    }

    #[test]
    fn outcome_table_is_frozen() {
        assert!(classify_outcome(0.90, 0.25).starts_with("PASS"));
        assert!(classify_outcome(0.25, 0.25).starts_with("INVALID_HARNESS"));
        assert!(classify_outcome(0.90, 0.85).starts_with("FAIL"));
        assert!(classify_outcome(0.60, 0.25).starts_with("FAIL"));
    }

    #[test]
    fn protocol_hash_is_stable() {
        assert_eq!(config_hash(), "shortcut-access-v148-953d6f24133cafb6");
    }
}
