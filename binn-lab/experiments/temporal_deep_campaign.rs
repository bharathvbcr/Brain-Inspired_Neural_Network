//! Protocol-v144/v145 temporal-task calibration and shared-forward depth run.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_data::{
    time_shuffle, TemporalDifficulty, TemporalOrderSplit, TEMPORAL_DIFFICULTIES,
    TEMPORAL_ORDER_N_CLASSES, TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
use binn_lab::guards::CeilingHealth;
use binn_lab::{
    mean_or_nan, temporal_order_to_dense_examples, temporal_order_to_shd_examples, write_report,
};
use binn_learn::{
    random_feedback, train_bptt, train_feedback, train_learned_feedback, InputRateClassifier,
    InputRateConfig, SharedTemporalNet,
};

const CALIBRATION_PROTOCOL: u64 = 144;
const DEPTH_PROTOCOL: u64 = 145;
const CALIBRATION_MASTER_SEED: u64 = 0x7E4A_5144_0000_0001;
const SCIENTIFIC_MASTER_SEED: u64 = 0x7E4A_5145_0000_0001;
/// Constant-predictor rate for the temporal-order task; a BPTT ceiling must
/// clear it before anything can be measured against it.
const CHANCE: f32 = 1.0 / TEMPORAL_ORDER_N_CLASSES as f32;
const FREEZE_PATH: &str = "results/temporal_task_calibration_v144.txt";

#[derive(Clone, Copy, Debug, PartialEq)]
struct CalibrationResult {
    difficulty: TemporalDifficulty,
    rfb: f32,
    bptt: f32,
    raw: f32,
    shuffled: f32,
}

fn main() -> ExitCode {
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::LocalLearning) {
        eprintln!("temporal-deep-campaign: {error}");
        return ExitCode::from(3);
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("temporal-deep-campaign: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut depth_run = false;
    let mut out: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--quick" => quick = true,
            "--depth-run" => depth_run = true,
            "--out" => {
                index += 1;
                out = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--out requires a path".to_string())?,
                ));
            }
            "-h" | "--help" => {
                println!("Usage: temporal-deep-campaign [--quick] [--depth-run] [--out PATH]");
                return Ok(());
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }
    if depth_run {
        run_depth(
            quick,
            out.unwrap_or_else(|| PathBuf::from("results/temporal_depth_v145.md")),
        )
    } else {
        run_calibration(
            quick,
            out.unwrap_or_else(|| PathBuf::from("results/temporal_calibration_v144.md")),
        )
    }
}

fn run_calibration(quick: bool, out: PathBuf) -> Result<(), String> {
    let n_seeds = if quick { 1 } else { 3 };
    let n_train = if quick { 40 } else { 200 };
    let n_test = if quick { 20 } else { 100 };
    let epochs = if quick { 3 } else { 20 };
    let hidden = if quick { 16 } else { 64 };
    let mut results = Vec::new();
    for difficulty in TEMPORAL_DIFFICULTIES {
        let mut rfb = Vec::new();
        let mut bptt = Vec::new();
        let mut raw = Vec::new();
        let mut shuffled = Vec::new();
        for seed_index in 0..n_seeds {
            let seed = CALIBRATION_MASTER_SEED
                ^ (seed_index as u64).wrapping_mul(0x1000_009D)
                ^ ((difficulty.jitter_radius as u64) << 32)
                ^ difficulty.distractor_events as u64;
            let split = TemporalOrderSplit::generate(n_train, n_test, difficulty, seed)?;
            let train = temporal_order_to_dense_examples(&split.train);
            let test = temporal_order_to_dense_examples(&split.test);
            let initial = SharedTemporalNet::new(
                TEMPORAL_ORDER_N_IN,
                TEMPORAL_ORDER_T,
                TEMPORAL_ORDER_N_CLASSES,
                &[hidden],
                0.9,
                1.0,
                5.0,
                seed,
            );
            let mut treatment = initial.clone();
            let mut ceiling = initial.clone();
            if treatment.forward(&train[0]) != ceiling.forward(&train[0]) {
                return Err("shared-forward clone parity failed".into());
            }
            let feedback = random_feedback(&initial, seed);
            train_feedback(&mut treatment, &feedback, &train, epochs, 0.005);
            train_bptt(&mut ceiling, &train, epochs);
            rfb.push(treatment.accuracy(&test));
            bptt.push(ceiling.accuracy(&test));

            let raw_train = temporal_order_to_shd_examples(&split.train);
            let raw_test = temporal_order_to_shd_examples(&split.test);
            let mut raw_model = InputRateClassifier::new(
                InputRateConfig {
                    n_in: TEMPORAL_ORDER_N_IN,
                    n_classes: TEMPORAL_ORDER_N_CLASSES,
                    lr: 0.005,
                    epochs,
                },
                seed,
            );
            raw.push(raw_model.train_and_evaluate(&raw_train, &raw_test).accuracy);

            let shuffled_train =
                temporal_order_to_dense_examples(&time_shuffle(&split.train, seed ^ 0x715A));
            let shuffled_test =
                temporal_order_to_dense_examples(&time_shuffle(&split.test, seed ^ 0x7E57));
            let mut shuffled_model = initial.clone();
            train_bptt(&mut shuffled_model, &shuffled_train, epochs);
            shuffled.push(shuffled_model.accuracy(&shuffled_test));
        }
        results.push(CalibrationResult {
            difficulty,
            rfb: mean_or_nan(&rfb),
            bptt: mean_or_nan(&bptt),
            raw: mean_or_nan(&raw),
            shuffled: mean_or_nan(&shuffled),
        });
    }
    let selected = results
        .iter()
        .rev()
        .find(|result| {
            (0.55..=0.80).contains(&result.rfb)
                && (0.65..=0.90).contains(&result.bptt)
                && result.raw <= 0.28
                && result.shuffled <= 0.28
        })
        .copied();
    let verdict = if quick {
        "PILOT"
    } else if selected.is_some() {
        "PASS"
    } else {
        "INVALID_TASK — no frozen candidate satisfies all calibration gates"
    };
    let mut rows = String::new();
    for result in &results {
        rows.push_str(&format!(
            "| ({}, {}) | {:.4} | {:.4} | {:.4} | {:.4} | {} |\n",
            result.difficulty.jitter_radius,
            result.difficulty.distractor_events,
            result.rfb,
            result.bptt,
            result.raw,
            result.shuffled,
            if Some(*result) == selected {
                "selected"
            } else {
                "no"
            },
        ));
    }
    let report = format!(
        "# Shortcut-resistant temporal task calibration\n\n\
         **Protocol:** v{CALIBRATION_PROTOCOL}  \n\
         **Schedule:** {}  \n\
         **Verdict:** **{verdict}**\n\n\
         | (jitter, distractors) | Matched RFB | BPTT | Raw rate | Time shuffled | Selected |\n\
         |---|---:|---:|---:|---:|---|\n\
         {rows}\n\
         Raw-rate insufficiency is structural: every quartet has byte-identical channel counts.\n",
        if quick {
            "QUICK / non-citable"
        } else {
            "calibration-only seed family"
        },
    );
    write_report(&out, &report)?;
    if !quick {
        match selected {
            Some(result) => {
                let hash = calibration_hash(result.difficulty);
                fs::write(
                    FREEZE_PATH,
                    format!(
                        "protocol={CALIBRATION_PROTOCOL}\nhash=temporal-v144-{hash:016x}\n\
                         jitter={}\ndistractors={}\n",
                        result.difficulty.jitter_radius, result.difficulty.distractor_events
                    ),
                )
                .map_err(|error| error.to_string())?;
            }
            None => {
                // Never leave a stale freeze after a failed fresh calibration.
                if Path::new(FREEZE_PATH).is_file() {
                    return Err(format!(
                        "calibration invalid but an existing {FREEZE_PATH} must be adjudicated manually"
                    ));
                }
            }
        }
    }
    println!("{report}");
    Ok(())
}

fn run_depth(quick: bool, out: PathBuf) -> Result<(), String> {
    let difficulty = read_freeze(Path::new(FREEZE_PATH))?;
    let n_seeds = if quick { 1 } else { 10 };
    let n_train = if quick { 40 } else { 1_000 };
    let n_test = if quick { 20 } else { 500 };
    let epochs = if quick { 3 } else { 40 };
    let width = if quick { 16 } else { 128 };
    let mut rows = String::new();
    let mut any_inversion = false;
    for depth in 1..=4 {
        let mut treatment_acc = Vec::new();
        let mut ceiling_acc = Vec::new();
        for seed_index in 0..n_seeds {
            let seed = SCIENTIFIC_MASTER_SEED ^ (seed_index as u64).wrapping_mul(0x1000_00D5);
            let split = TemporalOrderSplit::generate(n_train, n_test, difficulty, seed)?;
            let train = temporal_order_to_dense_examples(&split.train);
            let test = temporal_order_to_dense_examples(&split.test);
            let widths = vec![width; depth];
            let initial = SharedTemporalNet::new(
                TEMPORAL_ORDER_N_IN,
                TEMPORAL_ORDER_T,
                TEMPORAL_ORDER_N_CLASSES,
                &widths,
                0.9,
                1.0,
                5.0,
                seed,
            );
            let mut treatment = initial.clone();
            let mut ceiling = initial.clone();
            let mut feedback = random_feedback(&initial, seed);
            train_learned_feedback(&mut treatment, &mut feedback, &train, epochs, 0.005, 0.01);
            train_bptt(&mut ceiling, &train, epochs);
            treatment_acc.push(treatment.accuracy(&test));
            ceiling_acc.push(ceiling.accuracy(&test));
        }
        let treatment = mean_or_nan(&treatment_acc);
        let ceiling = mean_or_nan(&ceiling_acc);
        // 2026-08-21: was a bare `ceiling + 0.01 < treatment` inversion test,
        // which is silent when the BPTT ceiling never learned and the treatment
        // is below it. `CeilingHealth` tests the reference against chance first.
        let health = CeilingHealth::evaluate(ceiling, treatment, CHANCE);
        any_inversion |= !health.is_usable();
        rows.push_str(&format!(
            "| {depth} | {width} × {depth} | {treatment:.4} | {ceiling:.4} | {} |\n",
            health.label()
        ));
    }
    let verdict = if quick {
        "PILOT"
    } else if any_inversion {
        "INVALID_HARNESS"
    } else {
        "PASS (mechanical schedule complete; scientific interpretation remains exploratory)"
    };
    let report = format!(
        "# Shared-forward temporal depth experiment\n\n\
         **Protocol:** v{DEPTH_PROTOCOL}  \n\
         **Schedule:** {}  \n\
         **Verdict:** **{verdict}**\n\n\
         | Depth | Widths | Learned feedback | True BPTT | Ceiling health |\n\
         |---:|---|---:|---:|---|\n\
         {rows}",
        if quick {
            "QUICK / non-citable"
        } else {
            "10 fresh seeds, 40 epochs"
        }
    );
    write_report(&out, &report)?;
    println!("{report}");
    Ok(())
}

fn read_freeze(path: &Path) -> Result<TemporalDifficulty, String> {
    let text = fs::read_to_string(path).map_err(|_| {
        format!(
            "depth run refused: missing calibration freeze {}",
            path.display()
        )
    })?;
    let jitter = parse_field(&text, "jitter")?;
    let distractors = parse_field(&text, "distractors")?;
    let difficulty = TemporalDifficulty::new(jitter, distractors);
    if !TEMPORAL_DIFFICULTIES.contains(&difficulty) {
        return Err("calibration freeze names a non-preregistered difficulty".into());
    }
    Ok(difficulty)
}

fn parse_field(text: &str, name: &str) -> Result<usize, String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
        .ok_or_else(|| format!("calibration freeze missing {name}"))?
        .parse()
        .map_err(|error| format!("invalid {name}: {error}"))
}

fn calibration_hash(difficulty: TemporalDifficulty) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for value in [
        CALIBRATION_PROTOCOL,
        CALIBRATION_MASTER_SEED,
        difficulty.jitter_radius as u64,
        difficulty.distractor_events as u64,
    ] {
        hash ^= value;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_hashes_are_candidate_specific() {
        let hashes: Vec<_> = TEMPORAL_DIFFICULTIES
            .iter()
            .copied()
            .map(calibration_hash)
            .collect();
        for (index, hash) in hashes.iter().enumerate() {
            assert!(!hashes[..index].contains(hash));
        }
    }

    #[test]
    fn freeze_parser_rejects_unregistered_grid_points() {
        let path =
            std::env::temp_dir().join(format!("binn-temporal-freeze-{}.txt", std::process::id()));
        fs::write(&path, "jitter=2\ndistractors=7\n").unwrap();
        let result = read_freeze(&path);
        fs::remove_file(path).unwrap();
        assert!(result.is_err());
    }
}
