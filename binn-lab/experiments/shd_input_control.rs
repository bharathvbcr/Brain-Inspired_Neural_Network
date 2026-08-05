//! Protocol-v143 paired SHD input-rate shortcut confirmation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_data::{default_shd_dir, load_fixture, load_shd_split_capped, ShdSample, SHD_CHANCE};
use binn_learn::{
    hierarchical_bootstrap, shuffle_labels, InputRateClassifier, InputRateConfig,
    PairedPredictions, ShdAlifArm, ShdAlifConfig, ShdAlifRule, ShdExample, ShdSuperSpikeCeiling,
    ShdTrainConfig,
};

const PROTOCOL_VERSION: u64 = 143;
const CAPPED_MASTER_SEED: u64 = 0x5AD0_0143_0000_0001;
const FULL_MASTER_SEED: u64 = 0x5AD0_0143_0000_1001;
const REQUIRED_SEEDS: usize = 10;
const EXTENDED_SEEDS: usize = 20;
const BOOTSTRAP_SEED: u64 = 0xB007_5143_0000_0001;
const BOOTSTRAP_DRAWS: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Comparison {
    CappedAlif,
    FullSuperSpike,
}

impl Comparison {
    const fn name(self) -> &'static str {
        match self {
            Self::CappedAlif => "capped-alif-ff-fixed",
            Self::FullSuperSpike => "full-superspike",
        }
    }

    const fn n_train(self) -> usize {
        match self {
            Self::CappedAlif => 2_000,
            Self::FullSuperSpike => 8_156,
        }
    }

    const fn n_test(self) -> usize {
        match self {
            Self::CappedAlif => 500,
            Self::FullSuperSpike => 2_264,
        }
    }

    const fn epochs(self) -> usize {
        match self {
            Self::CappedAlif => 15,
            Self::FullSuperSpike => 20,
        }
    }

    const fn lr(self) -> f32 {
        match self {
            Self::CappedAlif => 0.005,
            Self::FullSuperSpike => 0.02,
        }
    }

    const fn master_seed(self) -> u64 {
        match self {
            Self::CappedAlif => CAPPED_MASTER_SEED,
            Self::FullSuperSpike => FULL_MASTER_SEED,
        }
    }
}

#[derive(Clone, Debug)]
struct SeedRecord {
    seed: u64,
    input_accuracy: f32,
    hidden_accuracy: f32,
    shuffled_label_accuracy: f32,
    input_predictions: Vec<u32>,
    hidden_predictions: Vec<u32>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shd-input-control: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut extend = false;
    let mut comparisons = vec![Comparison::CappedAlif, Comparison::FullSuperSpike];
    let mut out_dir = PathBuf::from("results/shd_0c1_v143");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--quick" => quick = true,
            "--extend" => extend = true,
            "--comparison" => {
                index += 1;
                comparisons = match args.get(index).map(String::as_str) {
                    Some("capped") => vec![Comparison::CappedAlif],
                    Some("full") => vec![Comparison::FullSuperSpike],
                    Some("both") => vec![Comparison::CappedAlif, Comparison::FullSuperSpike],
                    _ => return Err("--comparison requires capped|full|both".into()),
                };
            }
            "--out-dir" => {
                index += 1;
                out_dir = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--out-dir requires a path".to_string())?,
                );
            }
            "-h" | "--help" => {
                println!(
                    "Usage: shd-input-control [--quick] [--extend] \
                     [--comparison capped|full|both] [--out-dir PATH]"
                );
                return Ok(());
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }
    if quick && extend {
        return Err("--quick and --extend are mutually exclusive".into());
    }
    fs::create_dir_all(&out_dir).map_err(|error| error.to_string())?;
    let mut decisions = Vec::new();
    for comparison in comparisons {
        decisions.push(run_comparison(comparison, quick, extend, &out_dir)?);
    }
    if !quick && decisions.len() == 2 && decisions.iter().all(|(_, equivalent)| *equivalent) {
        fs::write(
            out_dir.join("SHD_0C1_DECISION.md"),
            "# SHD 0c-1 decision\n\n\
             **PASS — raw-rate shortcut confirmed against both frozen comparators.**\n\n\
             The SHD claim axis is superseded. Stop SHD activity tuning, feedback parity, \
             and additional architecture ablations. Historical reports remain byte-for-byte \
             unchanged and must be read through this status artifact.\n",
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn run_comparison(
    comparison: Comparison,
    quick: bool,
    extend: bool,
    out_dir: &Path,
) -> Result<(Comparison, bool), String> {
    let (n_seeds, n_train, n_test, epochs, hidden, fixture_allowed) = if quick {
        (1, 24, 8, 1, 16, true)
    } else {
        (
            if extend {
                EXTENDED_SEEDS
            } else {
                REQUIRED_SEEDS
            },
            comparison.n_train(),
            comparison.n_test(),
            comparison.epochs(),
            128,
            false,
        )
    };
    let split = if quick {
        load_fixture()?
    } else {
        load_shd_split_capped(
            &default_shd_dir(),
            Some(comparison.n_train()),
            Some(comparison.n_test()),
        )?
    };
    if !fixture_allowed && split.fixture {
        return Err("confirmatory SHD run refused fixture data".into());
    }
    if !quick && (split.train.len() != n_train || split.test.len() != n_test) {
        return Err(format!(
            "official SHD size mismatch: expected {n_train}/{n_test}, got {}/{}",
            split.train.len(),
            split.test.len()
        ));
    }
    let train: Vec<ShdExample> = split.train.iter().map(to_example).collect();
    let test: Vec<ShdExample> = split.test.iter().map(to_example).collect();
    let labels: Vec<u32> = test.iter().map(|example| example.label).collect();
    let mut records = Vec::with_capacity(n_seeds);

    for seed_index in 0..n_seeds {
        let seed = comparison.master_seed() ^ (seed_index as u64).wrapping_mul(0x1000_00B5);
        let input_cfg = InputRateConfig {
            n_in: split.n_in,
            n_classes: split.n_classes,
            lr: if quick { 0.01 } else { comparison.lr() },
            epochs,
        };
        let mut input = InputRateClassifier::new(input_cfg, seed);
        let input_report = input.train_and_evaluate(&train, &test);
        if !input_report.no_test_update {
            return Err("input-only evaluation mutated parameters".into());
        }

        let (hidden_accuracy, hidden_predictions) = match comparison {
            Comparison::CappedAlif => {
                let config = ShdAlifConfig::feedforward_fixed(
                    hidden,
                    split.n_classes,
                    if quick { 0.01 } else { comparison.lr() },
                    epochs,
                );
                let mut hidden_model = ShdAlifArm::new(&train[0], &config, ShdAlifRule::Dfa, seed);
                let report = hidden_model.train_and_evaluate(epochs, &train, &test);
                let predictions = hidden_model.predictions(&test);
                if predictions != hidden_model.predictions(&test) {
                    return Err("ALIF test prediction replay failed".into());
                }
                (report.accuracy, predictions)
            }
            Comparison::FullSuperSpike => {
                let config = ShdTrainConfig {
                    hidden,
                    n_classes: split.n_classes,
                    lr: if quick { 0.01 } else { comparison.lr() },
                    beta: 5.0,
                    epochs,
                };
                let mut hidden_model = ShdSuperSpikeCeiling::new(&train[0], config, seed);
                let report = hidden_model.train_and_evaluate(epochs, &train, &test);
                let predictions = hidden_model.predictions(&test);
                if predictions != hidden_model.predictions(&test) {
                    return Err("SuperSpike test prediction replay failed".into());
                }
                (report.accuracy, predictions)
            }
        };

        let shuffled_train = shuffle_labels(&train, seed ^ 0x5A1F);
        let mut shuffled_input = InputRateClassifier::new(input_cfg, seed);
        let shuffled_label_accuracy = shuffled_input
            .train_and_evaluate(&shuffled_train, &test)
            .accuracy;
        records.push(SeedRecord {
            seed,
            input_accuracy: input_report.accuracy,
            hidden_accuracy,
            shuffled_label_accuracy,
            input_predictions: input_report.predictions,
            hidden_predictions,
        });
    }

    let pairs: Vec<PairedPredictions> = records
        .iter()
        .map(|record| PairedPredictions {
            labels: labels.clone(),
            input_only: record.input_predictions.clone(),
            hidden: record.hidden_predictions.clone(),
        })
        .collect();
    let summary = hierarchical_bootstrap(
        &pairs,
        BOOTSTRAP_SEED ^ comparison.master_seed(),
        if quick { 1_000 } else { BOOTSTRAP_DRAWS },
    )?;
    let input_predictions: Vec<u32> = records
        .iter()
        .flat_map(|record| record.input_predictions.iter().copied())
        .collect();
    let hidden_predictions: Vec<u32> = records
        .iter()
        .flat_map(|record| record.hidden_predictions.iter().copied())
        .collect();
    let input_degenerate = degeneracy(&input_predictions, split.n_classes);
    let hidden_degenerate = degeneracy(&hidden_predictions, split.n_classes);
    let shuffled_label_mean = mean(
        &records
            .iter()
            .map(|record| record.shuffled_label_accuracy)
            .collect::<Vec<_>>(),
    );
    let validity = !input_degenerate
        && !hidden_degenerate
        && shuffled_label_mean <= SHD_CHANCE + 0.05
        && records.len() == n_seeds;
    let verdict = if quick {
        "PILOT"
    } else if !validity {
        "INVALID_HARNESS"
    } else if summary.equivalent {
        "PASS — input-only equivalent"
    } else if summary.hidden_clearly_better {
        "FAIL — hidden comparator clearly better; retain SHD"
    } else {
        "INCONCLUSIVE — extend unchanged protocol to 20 seeds"
    };
    let effective_lr = if quick { 0.01 } else { comparison.lr() };
    let hash = protocol_hash(
        comparison,
        n_seeds,
        n_train,
        n_test,
        epochs,
        effective_lr,
        quick,
    );
    let report = format!(
        "# SHD paired input-rate control\n\n\
         **Protocol:** v{PROTOCOL_VERSION}  \n\
         **Hash:** `shd-0c1-{hash:016x}`  \n\
         **Comparison:** {}  \n\
         **Schedule:** {n_train}/{n_test}, {epochs} epochs, lr {:.3}, {n_seeds} seeds  \n\
         **Data:** {}  \n\
         **Verdict:** **{verdict}**\n\n\
         Input-only accuracy: {:.4}. Hidden accuracy: {:.4}.  \n\
         Hidden − input-only: mean {:.4}, hierarchical-bootstrap 95% CI [{:.4}, {:.4}].  \n\
         Shuffled-label input control: {:.4}. Input degenerate: {}. Hidden degenerate: {}.  \n\
         No test-time updates and deterministic prediction replay: **yes**.\n\n\
         Equivalence requires mean < 0.02 and upper 95% bound < 0.05. \
         The unregistered pilot is not used in this verdict.\n",
        comparison.name(),
        effective_lr,
        if split.fixture {
            "fixture / non-citable"
        } else {
            "official SHD"
        },
        mean(
            &records
                .iter()
                .map(|record| record.input_accuracy)
                .collect::<Vec<_>>()
        ),
        mean(
            &records
                .iter()
                .map(|record| record.hidden_accuracy)
                .collect::<Vec<_>>()
        ),
        summary.mean_hidden_minus_input,
        summary.lower_95,
        summary.upper_95,
        shuffled_label_mean,
        yes_no(input_degenerate),
        yes_no(hidden_degenerate),
    );
    let report_path = out_dir.join(format!("{}.md", comparison.name()));
    fs::write(&report_path, &report).map_err(|error| error.to_string())?;
    let prediction_path = out_dir.join(format!("{}-predictions.tsv", comparison.name()));
    fs::write(&prediction_path, render_predictions(&records, &labels))
        .map_err(|error| error.to_string())?;
    println!("{report}");
    Ok((comparison, validity && summary.equivalent))
}

fn degeneracy(predictions: &[u32], n_classes: usize) -> bool {
    if predictions.is_empty() {
        return true;
    }
    let mut counts = vec![0usize; n_classes];
    for &prediction in predictions {
        if let Some(count) = counts.get_mut(prediction as usize) {
            *count += 1;
        } else {
            return true;
        }
    }
    let distinct = counts.iter().filter(|&&count| count > 0).count();
    let majority = counts.iter().copied().max().unwrap_or(0) as f32 / predictions.len() as f32;
    distinct < 5 || majority >= 0.95
}

fn render_predictions(records: &[SeedRecord], labels: &[u32]) -> String {
    let mut text = String::from("seed\texample\tlabel\tinput_only\thidden\n");
    for record in records {
        for (index, label) in labels.iter().enumerate() {
            text.push_str(&format!(
                "{}\t{index}\t{}\t{}\t{}\n",
                record.seed,
                label,
                record.input_predictions[index],
                record.hidden_predictions[index],
            ));
        }
    }
    text
}

#[allow(clippy::too_many_arguments)]
fn protocol_hash(
    comparison: Comparison,
    n_seeds: usize,
    n_train: usize,
    n_test: usize,
    epochs: usize,
    lr: f32,
    fixture: bool,
) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for value in [
        PROTOCOL_VERSION,
        comparison.master_seed(),
        n_train as u64,
        n_test as u64,
        epochs as u64,
        lr.to_bits() as u64,
        n_seeds as u64,
        fixture as u64,
        0x0200_0500, // equivalence mean 0.02, upper 0.05
    ] {
        hash ^= value;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn to_example(sample: &ShdSample) -> ShdExample {
    ShdExample {
        frames: sample.frames.clone(),
        t: sample.t,
        n_in: sample.n_in,
        label: sample.label,
    }
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
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
    fn comparison_hashes_and_seed_extensions_are_distinct() {
        let capped = protocol_hash(
            Comparison::CappedAlif,
            REQUIRED_SEEDS,
            2_000,
            500,
            15,
            0.005,
            false,
        );
        let full = protocol_hash(
            Comparison::FullSuperSpike,
            REQUIRED_SEEDS,
            8_156,
            2_264,
            20,
            0.02,
            false,
        );
        let extended = protocol_hash(
            Comparison::CappedAlif,
            EXTENDED_SEEDS,
            2_000,
            500,
            15,
            0.005,
            false,
        );
        let quick = protocol_hash(Comparison::CappedAlif, 1, 24, 8, 1, 0.01, true);
        assert_ne!(capped, full);
        assert_ne!(capped, extended);
        assert_ne!(capped, quick);
    }

    #[test]
    fn degeneracy_gate_requires_five_classes_and_no_majority_collapse() {
        assert!(degeneracy(&[0, 1, 2, 3], 20));
        assert!(!degeneracy(&[0, 1, 2, 3, 4, 0, 1, 2, 3, 4], 20));
        let mut collapsed = vec![0; 96];
        collapsed.extend([1, 2, 3, 4]);
        assert!(degeneracy(&collapsed, 20));
    }
}
