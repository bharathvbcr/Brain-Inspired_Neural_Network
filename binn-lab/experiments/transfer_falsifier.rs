//! Protocol-v146 same-specification Rust/NumPy transfer falsifier.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use binn_core::Rng;
use binn_data::{
    time_shuffle, TransferBundle, TEMPORAL_DIFFICULTIES, TEMPORAL_ORDER_CHANCE,
    TEMPORAL_ORDER_N_CLASSES, TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
use binn_lab::guards::Z_95;
use binn_lab::{
    mean_or_nan, std_error, temporal_order_to_dense_examples, temporal_order_to_shd_examples,
    MicroTrace, TransferModel, TransferPole, TRANSFER_PROTOCOL_VERSION,
};
use binn_learn::{train_bptt, InputRateClassifier, InputRateConfig, SharedTemporalNet};

const MASTER_SEED: u64 = 0x7A4F_5146_0000_0001;
const SCIENTIFIC_SEEDS: usize = 10;
const SCIENTIFIC_TRAIN: usize = 1_000;
const SCIENTIFIC_TEST: usize = 500;
const SCIENTIFIC_EPOCHS: usize = 20;
const SCIENTIFIC_HIDDEN: usize = 128;
const LOCAL_LR: f32 = 0.005;
const FREEZE_PATH: &str = "results/temporal_task_calibration_v144.txt";

#[derive(Clone, Debug)]
struct SeedOutcome {
    rust_matched: f32,
    rust_live: f32,
    numpy_matched: f32,
    numpy_live: f32,
    raw_rate: f32,
    time_shuffle: f32,
    shuffled_label: f32,
    bptt: f32,
}

fn main() -> ExitCode {
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::Transfer) {
        eprintln!("transfer-falsifier: {error}");
        return ExitCode::from(3);
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("transfer-falsifier: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut quick = false;
    let mut out = PathBuf::from("results/transfer_falsifier_v146.md");
    let mut bundle_dir = PathBuf::from("results/transfer_v146_bundles");
    let args: Vec<String> = env::args().skip(1).collect();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--quick" => quick = true,
            "--out" => {
                index += 1;
                out = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--out requires a path".to_string())?,
                );
            }
            "--bundle-dir" => {
                index += 1;
                bundle_dir = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--bundle-dir requires a path".to_string())?,
                );
            }
            "-h" | "--help" => {
                println!("Usage: transfer-falsifier [--quick] [--out PATH] [--bundle-dir PATH]");
                return Ok(());
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }

    let n_seeds = if quick { 1 } else { SCIENTIFIC_SEEDS };
    let n_train = if quick { 40 } else { SCIENTIFIC_TRAIN };
    let n_test = if quick { 20 } else { SCIENTIFIC_TEST };
    let epochs = if quick { 1 } else { SCIENTIFIC_EPOCHS };
    let hidden = if quick { 16 } else { SCIENTIFIC_HIDDEN };
    let bptt_epochs = if quick { 3 } else { SCIENTIFIC_EPOCHS };
    fs::create_dir_all(&bundle_dir).map_err(|error| error.to_string())?;

    // The difficulty is frozen by the preceding calibration artifact. Until a
    // qualifying calibration exists, candidate 0 is used only by --quick and
    // the report remains PILOT. Scientific runs consume and validate the
    // frozen pair rather than silently falling back to a hard-coded candidate.
    let selected_difficulty = if quick {
        TEMPORAL_DIFFICULTIES[0]
    } else {
        read_freeze(Path::new(FREEZE_PATH))?
    };

    let numpy_script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/transfer_numpy.py");
    let mut outcomes = Vec::with_capacity(n_seeds);
    let mut micro_conformance = true;
    let mut replay = true;

    for seed_index in 0..n_seeds {
        let seed = MASTER_SEED ^ (seed_index as u64).wrapping_mul(0x1000_014D);
        let bundle = TransferBundle::generate(n_train, n_test, selected_difficulty, hidden, seed)?;
        let bundle_path = bundle_dir.join(format!("seed-{seed_index:02}.binntrf1"));
        bundle.write(&bundle_path)?;
        replay &= TransferBundle::read(&bundle_path)? == bundle;

        if seed_index == 0 {
            for (name, pole) in [
                ("matched", TransferPole::matched()),
                ("live", TransferPole::live()),
            ] {
                let mut rust_model = TransferModel::from_bundle(&bundle);
                let rust_trace = rust_model.micro_step(&bundle.train[0], pole, LOCAL_LR);
                let rust_path = bundle_dir.join(format!("micro-rust-{name}.json"));
                fs::write(&rust_path, render_micro(&rust_trace))
                    .map_err(|error| error.to_string())?;
                let first_numpy = bundle_dir.join(format!("micro-numpy-{name}.json"));
                let second_numpy = bundle_dir.join(format!("micro-numpy-{name}-replay.json"));
                run_numpy_micro(&numpy_script, &bundle_path, name, &rust_path, &first_numpy)?;
                run_numpy_micro(&numpy_script, &bundle_path, name, &rust_path, &second_numpy)?;
                let first = fs::read(&first_numpy).map_err(|error| error.to_string())?;
                let second = fs::read(&second_numpy).map_err(|error| error.to_string())?;
                replay &= first == second;
                micro_conformance &= true;
            }
        }

        let matched_pole = TransferPole::matched();
        let live_pole = TransferPole::live();
        let mut matched = TransferModel::from_bundle(&bundle);
        let mut live = TransferModel::from_bundle(&bundle);
        matched.train(&bundle.train, matched_pole, epochs, LOCAL_LR);
        live.train(&bundle.train, live_pole, epochs, LOCAL_LR);
        let matched_eval = matched.evaluate(&bundle.test, matched_pole);
        let live_eval = live.evaluate(&bundle.test, live_pole);
        if !matched_eval.no_test_update || !live_eval.no_test_update {
            return Err("Rust endpoint evaluation mutated parameters".into());
        }

        let numpy_matched =
            run_numpy_accuracy(&numpy_script, &bundle_path, "matched", epochs, LOCAL_LR)?;
        let numpy_live = run_numpy_accuracy(&numpy_script, &bundle_path, "live", epochs, LOCAL_LR)?;

        let raw_train = temporal_order_to_shd_examples(&bundle.train);
        let raw_test = temporal_order_to_shd_examples(&bundle.test);
        let mut raw = InputRateClassifier::new(
            InputRateConfig {
                n_in: TEMPORAL_ORDER_N_IN,
                n_classes: TEMPORAL_ORDER_N_CLASSES,
                lr: LOCAL_LR,
                epochs,
            },
            seed,
        );
        let raw_rate = raw.train_and_evaluate(&raw_train, &raw_test).accuracy;

        let shuffled_train = time_shuffle(&bundle.train, seed ^ 0x715A);
        let shuffled_test = time_shuffle(&bundle.test, seed ^ 0x7E57);
        let mut time_model = TransferModel::from_bundle(&bundle);
        time_model.train(&shuffled_train, matched_pole, epochs, LOCAL_LR);
        let time_shuffle_accuracy = time_model.evaluate(&shuffled_test, matched_pole).accuracy;

        let mut labels = bundle
            .train
            .iter()
            .map(|example| example.label)
            .collect::<Vec<_>>();
        let mut rng = Rng::new(seed ^ 0x5A1F_1AB3);
        for i in (1..labels.len()).rev() {
            let j = rng.gen_index(i + 1);
            labels.swap(i, j);
        }
        let mut label_train = bundle.train.clone();
        for (example, label) in label_train.iter_mut().zip(labels) {
            example.label = label;
        }
        let mut label_model = TransferModel::from_bundle(&bundle);
        label_model.train(&label_train, matched_pole, epochs, LOCAL_LR);
        let shuffled_label = label_model.evaluate(&bundle.test, matched_pole).accuracy;

        let deep_train = temporal_order_to_dense_examples(&bundle.train);
        let deep_test = temporal_order_to_dense_examples(&bundle.test);
        let mut bptt = SharedTemporalNet::new(
            TEMPORAL_ORDER_N_IN,
            TEMPORAL_ORDER_T,
            TEMPORAL_ORDER_N_CLASSES,
            &[hidden],
            0.9,
            1.0,
            5.0,
            seed,
        );
        train_bptt(&mut bptt, &deep_train, bptt_epochs);
        let bptt_accuracy = bptt.accuracy(&deep_test);

        outcomes.push(SeedOutcome {
            rust_matched: matched_eval.accuracy,
            rust_live: live_eval.accuracy,
            numpy_matched,
            numpy_live,
            raw_rate,
            time_shuffle: time_shuffle_accuracy,
            shuffled_label,
            bptt: bptt_accuracy,
        });
    }

    let rust_matched = values(&outcomes, |outcome| outcome.rust_matched);
    let rust_live = values(&outcomes, |outcome| outcome.rust_live);
    let rust_gap: Vec<f32> = rust_matched
        .iter()
        .zip(&rust_live)
        .map(|(matched, live)| matched - live)
        .collect();
    let numpy_matched = values(&outcomes, |outcome| outcome.numpy_matched);
    let numpy_live = values(&outcomes, |outcome| outcome.numpy_live);
    let numpy_gap: Vec<f32> = numpy_matched
        .iter()
        .zip(&numpy_live)
        .map(|(matched, live)| matched - live)
        .collect();
    let raw = values(&outcomes, |outcome| outcome.raw_rate);
    let time = values(&outcomes, |outcome| outcome.time_shuffle);
    let labels = values(&outcomes, |outcome| outcome.shuffled_label);
    let bptt = values(&outcomes, |outcome| outcome.bptt);

    let matched_mean = mean_or_nan(&rust_matched);
    let matched_lcb = lower_95(&rust_matched);
    let rust_gap_mean = mean_or_nan(&rust_gap);
    let rust_gap_lcb = lower_95(&rust_gap);
    let matched_valid = (0.40..=0.85).contains(&matched_mean)
        && matched_lcb > TEMPORAL_ORDER_CHANCE + 0.10
        && mean_or_nan(&raw) <= 0.28
        && mean_or_nan(&time) <= 0.28
        && mean_or_nan(&labels) <= 0.28
        && mean_or_nan(&bptt) + 0.05 >= matched_mean;
    let phenomenon = rust_gap_mean >= 0.10 && rust_gap_lcb > 0.05;
    let reproduction = (mean_or_nan(&numpy_matched) - matched_mean).abs() <= 0.05
        && (mean_or_nan(&numpy_live) - mean_or_nan(&rust_live)).abs() <= 0.05
        && (mean_or_nan(&numpy_gap) - rust_gap_mean).abs() <= 0.10
        && micro_conformance
        && replay;
    let verdict = if quick {
        "PILOT"
    } else if !micro_conformance {
        "INVALID_HARNESS — specification/engine divergence"
    } else if matched_valid && phenomenon && reproduction {
        "PASS"
    } else {
        "FAIL"
    };
    let report = format!(
        "# Same-specification Rust/NumPy transfer falsifier\n\n\
         **Protocol:** v{TRANSFER_PROTOCOL_VERSION}  \n\
         **Schedule:** {}  \n\
         **Verdict:** **{verdict}**  \n\
         **Micro-conformance:** {}  \n\
         **Byte-identical replay:** {}  \n\n\
         | Metric | Rust | NumPy |\n\
         |---|---:|---:|\n\
         | Matched accuracy | {:.4} | {:.4} |\n\
         | Live accuracy | {:.4} | {:.4} |\n\
         | Matched − live | {:.4} (LCB {:.4}) | {:.4} |\n\n\
         Controls: raw-rate {:.4}; time-shuffled {:.4}; shuffled-label {:.4}; BPTT {:.4}.  \n\
         Matched validity: **{}**. Transfer phenomenon: **{}**. Independent reproduction: **{}**.\n\n\
         Historical `0.51` is intentionally not used; it combined incompatible harnesses.\n",
        if quick {
            "QUICK / non-citable"
        } else {
            "10 paired seeds, 1000/500, 20 epochs"
        },
        yes_no(micro_conformance),
        yes_no(replay),
        matched_mean,
        mean_or_nan(&numpy_matched),
        mean_or_nan(&rust_live),
        mean_or_nan(&numpy_live),
        rust_gap_mean,
        rust_gap_lcb,
        mean_or_nan(&numpy_gap),
        mean_or_nan(&raw),
        mean_or_nan(&time),
        mean_or_nan(&labels),
        mean_or_nan(&bptt),
        yes_no(matched_valid),
        yes_no(phenomenon),
        yes_no(reproduction),
    );
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&out, &report).map_err(|error| error.to_string())?;
    println!("{report}");
    Ok(())
}

fn run_numpy_micro(
    script: &Path,
    bundle: &Path,
    pole: &str,
    rust_trace: &Path,
    out: &Path,
) -> Result<(), String> {
    let status = Command::new("python3")
        .arg(script)
        .args(["--bundle", &bundle.display().to_string()])
        .args(["--pole", pole, "--micro"])
        .args(["--compare-rust", &rust_trace.display().to_string()])
        .args(["--out", &out.display().to_string()])
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "NumPy micro-conformance failed for {pole}: {status}"
        ))
    }
}

fn run_numpy_accuracy(
    script: &Path,
    bundle: &Path,
    pole: &str,
    epochs: usize,
    lr: f32,
) -> Result<f32, String> {
    let output = Command::new("python3")
        .arg(script)
        .args(["--bundle", &bundle.display().to_string()])
        .args(["--pole", pole])
        .args(["--epochs", &epochs.to_string()])
        .args(["--lr", &lr.to_string(), "--accuracy-only"])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    String::from_utf8(output.stdout)
        .map_err(|error| error.to_string())?
        .trim()
        .parse()
        .map_err(|error| format!("invalid NumPy accuracy: {error}"))
}

fn render_micro(trace: &MicroTrace) -> String {
    format!(
        "{{\"eligibility\":{},\"event_ticks\":{},\"final_thresholds\":{},\
         \"prediction\":{},\"recipients\":{},\"weight_delta\":{},\"winners_by_tick\":{}}}\n",
        float_array(&trace.eligibility),
        integer_array(&trace.event_ticks),
        float_array(&trace.final_thresholds),
        trace.prediction,
        integer_array(&trace.recipients),
        float_array(&trace.weight_delta),
        nested_integer_array(&trace.winners_by_tick),
    )
}

fn float_array(values: &[f32]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("{value:.9}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn integer_array<T: std::fmt::Display>(values: &[T]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn nested_integer_array(values: &[Vec<u32>]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| integer_array(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn values(outcomes: &[SeedOutcome], get: impl Fn(&SeedOutcome) -> f32) -> Vec<f32> {
    outcomes.iter().map(get).collect()
}

/// Lower 95% bound on the mean of `values`; NaN for an empty slice.
///
/// The spread comes from the crate's [`binn_lab::std_error`], which is
/// Bessel-corrected and reports `0.0` below two samples, so a single seed
/// returns that seed's own value and needs no special case here.
///
/// Empty is NaN rather than a number, matching [`mean_or_nan`] in this binary:
/// a bound computed from nothing must be visible in the report. It also used
/// to be the one place in the repository that divided by `len() - 1` without a
/// zero guard, which made an empty slice panic under `debug` and yield NaN
/// under `release` — the same input, two answers, decided by the build
/// profile.
fn lower_95(values: &[f32]) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    mean_or_nan(values) - Z_95 * std_error(values)
}

fn read_freeze(path: &Path) -> Result<binn_data::TemporalDifficulty, String> {
    let text = fs::read_to_string(path).map_err(|_| {
        format!(
            "scientific run refused: calibration freeze {} is absent",
            path.display()
        )
    })?;
    let jitter_radius = parse_freeze_field(&text, "jitter")?;
    let distractor_events = parse_freeze_field(&text, "distractors")?;
    let difficulty = binn_data::TemporalDifficulty::new(jitter_radius, distractor_events);
    if !TEMPORAL_DIFFICULTIES.contains(&difficulty) {
        return Err("calibration freeze names a non-preregistered difficulty".into());
    }
    Ok(difficulty)
}

fn parse_freeze_field(text: &str, name: &str) -> Result<usize, String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
        .ok_or_else(|| format!("calibration freeze missing {name}"))?
        .parse()
        .map_err(|error| format!("invalid calibration freeze {name}: {error}"))
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

    /// An empty sample must not be answered differently by `debug` and
    /// `release`.
    ///
    /// `lower_95` divided by `values.len() - 1` behind a guard that only
    /// covered `len() == 1`, so a zero-length slice underflowed: a panic under
    /// `debug`, and `usize::MAX` as the divisor under `release`, which fell out
    /// as NaN. A number this binary reports must not depend on the build
    /// profile. Pinned here with the single- and multi-sample cases, which the
    /// fix leaves bit-identical.
    #[test]
    fn lower_95_is_nan_on_an_empty_sample_and_unchanged_otherwise() {
        assert!(
            lower_95(&[]).is_nan(),
            "an empty sample has no lower bound; it must not be a number"
        );

        // One seed has no spread, so the bound is that seed's own value.
        for single in [0.0f32, 0.42, -1.5, 1.0] {
            assert_eq!(lower_95(&[single]).to_bits(), single.to_bits());
        }

        // Two or more: the longhand the fix replaced, bit for bit.
        fn reference(values: &[f32]) -> f32 {
            let average = values.iter().sum::<f32>() / values.len() as f32;
            let variance = values
                .iter()
                .map(|value| (value - average).powi(2))
                .sum::<f32>()
                / (values.len() - 1) as f32;
            average - 1.96 * (variance / values.len() as f32).sqrt()
        }
        for values in [
            &[0.80f32, 0.84][..],
            &[0.71, 0.68, 0.74, 0.70, 0.69],
            &[0.5, 0.5, 0.5],
        ] {
            assert_eq!(
                lower_95(values).to_bits(),
                reference(values).to_bits(),
                "lower_95 drifted from the pre-fix arithmetic on {values:?}"
            );
        }
    }

    #[test]
    fn scientific_freeze_selects_registered_nonzero_candidate() {
        let path = std::env::temp_dir().join(format!(
            "binn-transfer-freeze-{}-registered.txt",
            std::process::id()
        ));
        fs::write(&path, "jitter=2\ndistractors=12\n").unwrap();
        let difficulty = read_freeze(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(difficulty, TEMPORAL_DIFFICULTIES[2]);
    }

    #[test]
    fn scientific_freeze_rejects_unregistered_candidate() {
        let path = std::env::temp_dir().join(format!(
            "binn-transfer-freeze-{}-invalid.txt",
            std::process::id()
        ));
        fs::write(&path, "jitter=2\ndistractors=11\n").unwrap();
        let result = read_freeze(&path);
        fs::remove_file(path).unwrap();
        assert!(result.is_err());
    }
}
