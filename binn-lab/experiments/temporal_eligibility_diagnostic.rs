//! Protocol-v147 local temporal-eligibility mechanism diagnostic.

use binn_lab::{mean_or_nan, temporal_order_to_dense_examples, write_report};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_data::{
    TemporalDifficulty, TemporalOrderSplit, TEMPORAL_DIFFICULTIES, TEMPORAL_ORDER_N_CLASSES,
    TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
use binn_learn::{
    random_feedback, train_bptt, train_feedback, DenseTemporalExample, SharedTemporalNet,
    StepDiagnostics,
};

const PROTOCOL_VERSION: u64 = 147;
const MASTER_SEED: u64 = 0x7E4A_5147_0000_0001;
const OVERFIT_SEED: u64 = 0x7E4A_5147_0F17_0001;
const DIAGNOSTIC_SEEDS: usize = 3;
const N_TRAIN: usize = 200;
const N_TEST: usize = 100;
const EPOCHS: usize = 20;
const HIDDEN: usize = 64;
const LR: f32 = 0.005;
const OVERFIT_TRAIN: usize = 40;
const OVERFIT_EPOCHS: usize = 200;
const OVERFIT_FLOOR: f32 = 0.95;
const LEARNING_FLOOR: f32 = 0.55;
const BPTT_NEAR_PERFECT: f32 = 0.90;
const CHANCE_LIKE_MAX: f32 = 0.30;

#[derive(Clone, Copy, Debug)]
struct ScaleSummary {
    hidden_gradient_rms: f32,
    hidden_step_rms: f32,
    readout_gradient_rms: f32,
    readout_step_rms: f32,
}

#[derive(Clone, Copy, Debug)]
struct PredictionHealth {
    accuracy: f32,
    classes: usize,
    majority: f32,
}

#[derive(Clone, Copy, Debug)]
struct SeedResult {
    seed: u64,
    rfb_train: f32,
    rfb_test: PredictionHealth,
    bptt_train: f32,
    bptt_test: PredictionHealth,
    rfb_scale: ScaleSummary,
    bptt_scale: ScaleSummary,
    replay: bool,
}

fn main() -> ExitCode {
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::LocalLearning) {
        eprintln!("temporal-eligibility-diagnostic: {error}");
        return ExitCode::from(3);
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("temporal-eligibility-diagnostic: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut out = PathBuf::from("results/temporal_eligibility_diagnostic_v147.md");
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
            "-h" | "--help" => {
                println!("Usage: temporal-eligibility-diagnostic [--out PATH]");
                return Ok(());
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }

    let difficulty = easiest_difficulty();
    let overfit_split = TemporalOrderSplit::generate(OVERFIT_TRAIN, 20, difficulty, OVERFIT_SEED)?;
    let overfit_train = temporal_order_to_dense_examples(&overfit_split.train);
    let overfit_initial = new_model(OVERFIT_SEED);
    let overfit_feedback = random_feedback(&overfit_initial, OVERFIT_SEED);
    let mut overfit_model = overfit_initial.clone();
    let overfit_diagnostics = train_feedback(
        &mut overfit_model,
        &overfit_feedback,
        &overfit_train,
        OVERFIT_EPOCHS,
        LR,
    );
    let overfit_health = prediction_health(&overfit_model, &overfit_train);
    let overfit_scale = summarize(&overfit_diagnostics)?;
    let mut overfit_replay = overfit_initial.clone();
    let replay_diagnostics = train_feedback(
        &mut overfit_replay,
        &overfit_feedback,
        &overfit_train,
        OVERFIT_EPOCHS,
        LR,
    );
    let overfit_replay_ok = overfit_model.parameter_fingerprint()
        == overfit_replay.parameter_fingerprint()
        && overfit_diagnostics == replay_diagnostics
        && overfit_health.accuracy == overfit_replay.accuracy(&overfit_train);
    let overfit_gate = overfit_health.accuracy > OVERFIT_FLOOR
        && overfit_health.classes == TEMPORAL_ORDER_N_CLASSES
        && overfit_health.majority < 0.95
        && scales_are_live(overfit_scale)
        && overfit_replay_ok;

    let mut results = Vec::with_capacity(DIAGNOSTIC_SEEDS);
    for seed_index in 0..DIAGNOSTIC_SEEDS {
        let seed = MASTER_SEED ^ (seed_index as u64).wrapping_mul(0x1000_00E7);
        let split = TemporalOrderSplit::generate(N_TRAIN, N_TEST, difficulty, seed)?;
        let train = temporal_order_to_dense_examples(&split.train);
        let test = temporal_order_to_dense_examples(&split.test);
        let initial = new_model(seed);
        let feedback = random_feedback(&initial, seed);
        let mut rfb = initial.clone();
        let rfb_diagnostics = train_feedback(&mut rfb, &feedback, &train, EPOCHS, LR);
        let rfb_train = rfb.accuracy(&train);
        let rfb_test = prediction_health(&rfb, &test);
        let rfb_scale = summarize(&rfb_diagnostics)?;

        let mut replay = initial.clone();
        let replay_diagnostics = train_feedback(&mut replay, &feedback, &train, EPOCHS, LR);
        let replay_ok = rfb.parameter_fingerprint() == replay.parameter_fingerprint()
            && rfb_diagnostics == replay_diagnostics
            && rfb_train == replay.accuracy(&train)
            && rfb_test.accuracy == replay.accuracy(&test);

        let mut bptt = initial;
        let bptt_diagnostics = train_bptt(&mut bptt, &train, EPOCHS);
        let bptt_train = bptt.accuracy(&train);
        let bptt_test = prediction_health(&bptt, &test);
        let bptt_scale = summarize(&bptt_diagnostics)?;
        results.push(SeedResult {
            seed,
            rfb_train,
            rfb_test,
            bptt_train,
            bptt_test,
            rfb_scale,
            bptt_scale,
            replay: replay_ok,
        });
    }

    let mean_rfb_train = mean(results.iter().map(|result| result.rfb_train));
    let mean_rfb_test = mean(results.iter().map(|result| result.rfb_test.accuracy));
    let mean_bptt_train = mean(results.iter().map(|result| result.bptt_train));
    let mean_bptt_test = mean(results.iter().map(|result| result.bptt_test.accuracy));
    let all_healthy = results.iter().all(|result| {
        result.rfb_test.classes == TEMPORAL_ORDER_N_CLASSES
            && result.rfb_test.majority < 0.95
            && result.bptt_test.classes == TEMPORAL_ORDER_N_CLASSES
            && result.bptt_test.majority < 0.95
            && scales_are_live(result.rfb_scale)
            && scales_are_live(result.bptt_scale)
            && result.replay
    });
    let learns = overfit_gate
        && all_healthy
        && mean_rfb_test >= LEARNING_FLOOR
        && mean_bptt_test >= BPTT_NEAR_PERFECT;
    let stopped_at_chance = mean_rfb_test <= CHANCE_LIKE_MAX && mean_bptt_test >= BPTT_NEAR_PERFECT;
    let verdict = if learns {
        "PASS — temporal eligibility restores matched-feedback learning"
    } else if stopped_at_chance {
        "FAIL — corrected matched feedback remains at chance; stop this design"
    } else if !overfit_gate || !all_healthy {
        "INVALID_HARNESS — mechanical or replay gate failed"
    } else {
        "FAIL — mechanism missed the frozen learning gate; stop and reassess"
    };

    let mut rows = String::new();
    for result in &results {
        rows.push_str(&format!(
            "| {} | {:.4} | {:.4} | {}/4 | {:.3} | {:.3e} | {:.3e} | {:.3e} | {:.3e} | {:.4} | {:.4} | {}/4 | {} |\n",
            result.seed,
            result.rfb_train,
            result.rfb_test.accuracy,
            result.rfb_test.classes,
            result.rfb_test.majority,
            result.rfb_scale.hidden_gradient_rms,
            result.rfb_scale.hidden_step_rms,
            result.rfb_scale.readout_gradient_rms,
            result.rfb_scale.readout_step_rms,
            result.bptt_train,
            result.bptt_test.accuracy,
            result.bptt_test.classes,
            yes_no(result.replay),
        ));
    }
    let report = format!(
        "# Temporal eligibility mechanism diagnostic\n\n\
         **Protocol:** v{PROTOCOL_VERSION}\n\
         **Hash:** `temporal-elig-v147-{:016x}`\n\
         **Difficulty:** `(jitter=0, distractors=4)` only\n\
         **Schedule:** {DIAGNOSTIC_SEEDS} fresh seeds, {N_TRAIN}/{N_TEST}, \
         {EPOCHS} epochs, hidden={HIDDEN}, lr={LR:.3}\n\
         **Verdict:** **{verdict}**\n\n\
         ## Mandatory pre-calibration overfit gate\n\n\
         Training accuracy {:.4}; predicted classes {}/4; majority {:.3}; \
         hidden gradient RMS {:.3e}; hidden step RMS {:.3e}; \
         readout gradient RMS {:.3e}; readout step RMS {:.3e}; replay {}.\n\
         Gate (> {:.2} with all classes, live gradients/steps, and exact replay): **{}**.\n\n\
         ## Frozen easiest-candidate diagnostic\n\n\
         | Seed | RFB train | RFB test | RFB classes | Majority | Hidden grad RMS | Hidden step RMS | Readout grad RMS | Readout step RMS | BPTT train | BPTT test | BPTT classes | Replay |\n\
         |---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n\
         {rows}\n\
         Means: RFB train {mean_rfb_train:.4}, RFB test {mean_rfb_test:.4}; \
         BPTT train {mean_bptt_train:.4}, BPTT test {mean_bptt_test:.4}.\n\
         RFB learning requires mean test >= {LEARNING_FLOOR:.2}; \
         chance-like stop is <= {CHANCE_LIKE_MAX:.2} while BPTT >= {BPTT_NEAR_PERFECT:.2}.\n\n\
         This is a mechanism diagnostic, not a replacement calibration. It runs \
         no optimizer sweep and no difficulty sweep. Protocols v145/v146 remain blocked.\n",
        protocol_hash(),
        overfit_health.accuracy,
        overfit_health.classes,
        overfit_health.majority,
        overfit_scale.hidden_gradient_rms,
        overfit_scale.hidden_step_rms,
        overfit_scale.readout_gradient_rms,
        overfit_scale.readout_step_rms,
        yes_no(overfit_replay_ok),
        OVERFIT_FLOOR,
        yes_no(overfit_gate),
    );
    write_report(&out, &report)?;
    let decision = if learns {
        "# v147 decision\n\n**ADVANCE.** The corrected matched-feedback treatment learned \
         under the frozen mechanism gate. Register a fresh v148 calibration and \
         successor v149/v150 protocols before further scientific execution.\n"
    } else {
        "# v147 decision\n\n**STOP.** The corrected matched-feedback treatment did not \
         clear the frozen mechanism gate. Do not create v148-v150 or run v145/v146; \
         reassess the treatment mechanism.\n"
    };
    write_report(
        Path::new("results/TEMPORAL_ELIGIBILITY_V147_DECISION.md"),
        decision,
    )?;
    println!("{report}");
    Ok(())
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

fn summarize(diagnostics: &[StepDiagnostics]) -> Result<ScaleSummary, String> {
    if diagnostics.is_empty() {
        return Err("training emitted no step diagnostics".into());
    }
    Ok(ScaleSummary {
        hidden_gradient_rms: mean(diagnostics.iter().map(|step| step.layer_gradient_rms[0])),
        hidden_step_rms: mean(diagnostics.iter().map(|step| step.layer_step_rms[0])),
        readout_gradient_rms: mean(diagnostics.iter().map(|step| step.readout_gradient_rms)),
        readout_step_rms: mean(diagnostics.iter().map(|step| step.readout_step_rms)),
    })
}

fn scales_are_live(scale: ScaleSummary) -> bool {
    [
        scale.hidden_gradient_rms,
        scale.hidden_step_rms,
        scale.readout_gradient_rms,
        scale.readout_step_rms,
    ]
    .iter()
    .all(|value| value.is_finite() && *value > 0.0)
}

/// Mean of an iterator; **NaN** for an empty one.
///
/// This had no empty guard at all, so an empty iterator fell out of `0.0 / 0.0`
/// as NaN. That is [`binn_lab::mean_or_nan`]'s contract exactly, so the
/// behaviour is unchanged and the choice is now stated rather than incidental -
/// it is deliberately NOT `binn_lab::mean`, which would report `0.0`.
fn mean(values: impl Iterator<Item = f32>) -> f32 {
    let values: Vec<f32> = values.collect();
    mean_or_nan(&values)
}

fn protocol_hash() -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for value in [
        PROTOCOL_VERSION,
        MASTER_SEED,
        OVERFIT_SEED,
        DIAGNOSTIC_SEEDS as u64,
        N_TRAIN as u64,
        N_TEST as u64,
        EPOCHS as u64,
        HIDDEN as u64,
        LR.to_bits() as u64,
        OVERFIT_TRAIN as u64,
        OVERFIT_EPOCHS as u64,
        0x5445_4D50_454C_4947,
    ] {
        hash ^= value;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
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
    fn protocol_is_easiest_only_and_hash_is_stable() {
        assert_eq!(easiest_difficulty(), TemporalDifficulty::new(0, 4));
        assert_eq!(protocol_hash(), protocol_hash());
        assert_ne!(MASTER_SEED, OVERFIT_SEED);
    }

    #[test]
    fn prediction_health_detects_class_collapse_without_mutation() {
        let split = TemporalOrderSplit::generate(40, 20, easiest_difficulty(), 7).unwrap();
        let examples = temporal_order_to_dense_examples(&split.test);
        let model = new_model(7);
        let before = model.parameter_fingerprint();
        let health = prediction_health(&model, &examples);
        assert!((1..=TEMPORAL_ORDER_N_CLASSES).contains(&health.classes));
        assert_eq!(before, model.parameter_fingerprint());
    }
}
