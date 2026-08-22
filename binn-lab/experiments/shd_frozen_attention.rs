//! Is 0.234 a limit of local credit assignment, or of the forward model's memory?
//!
//! # The question this sharpens
//!
//! `shd_arch_ablation` asks whether the `c1-shd-cal-*` DFA result is confounded
//! by a feed-forward fixed-threshold forward model, and answers it with the
//! recurrence x adaptation grid that ETLP's conclusion points at. That grid
//! changes the *dynamics* of the hidden layer, which changes both what the layer
//! can represent and how credit propagates through it. The two are not
//! separable there.
//!
//! This binary separates them. It attaches a **frozen** time-axis attention
//! read-out ([`binn_learn::shd_attention`]) to the unchanged feed-forward
//! forward model:
//!
//! * the block is drawn once and **never updated** — asserted bitwise, for all
//!   three rules, by `shd_alif::tests::training_never_moves_the_frozen_attention_block`;
//! * nothing is backpropagated through it, and it carries **no credit** to the
//!   hidden layer — the transported-feedback path reads only the hidden columns
//!   of `w_out`;
//! * the read-out stays one layer with a local error signal, so the arm is
//!   **exactly as local as the arm it extends**.
//!
//! So the only thing that changes is how much temporal structure the read-out
//! can *see*. If that alone moves the local arms, the binding constraint was the
//! forward model's 46 ms memory horizon
//! (`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md` aside; the horizon
//! itself is derived in `shd_attention`'s module documentation), not the
//! locality of the rule. If it does not, locality survives a genuine test it has
//! not previously been given.
//!
//! **This is the only attention arm in the project that speaks to Gate G2.** The
//! `+attn` arms of the matched instrument are BPTT references and say nothing
//! about local learning.
//!
//! # THIS BINARY CANNOT CURRENTLY RUN, AND THAT IS CORRECT
//!
//! It requests `CampaignKind::LocalLearning`, which
//! `binn_lab::authorize_campaign` refuses while `SHD_INSTRUMENT_STATE` is
//! `Uncalibrated`. That is a compile-time constant with **no flag and no
//! environment override**, by design: `SHD_INSTRUMENT_STATUS.md` blocks "new SHD
//! local-learning or architecture-ablation campaigns" outright, and its sibling
//! `shd-arch-ablation` is blocked identically.
//!
//! The instrument is uncalibrated because calibration criterion 4 requires
//! **three clean reference seeds, each at least 0.80**, and the converged
//! `ff+fixed` ceiling is 0.7378. Criterion 5 additionally requires a matched
//! Python/Rust configuration, and there is no Python mirror of the attention
//! axis (`scripts/shd_calibration/arms.py`).
//!
//! So this arm is implemented, tested and wired, and waits on the gate. Do not
//! flip `SHD_INSTRUMENT_STATE` to run it: that constant *is* the claim that the
//! instrument measures what it says it measures.
//!
//! Preregistration: `results/PREREG_2026-08-19_SHD_FROZEN_ATTENTION_LOCAL.md`.
//!
//! # Robustness properties, inherited deliberately from `shd_arch_ablation`
//!
//! * The report is rewritten after every cell, so a wall-clock kill leaves a
//!   valid partial report rather than nothing.
//! * Fixture data is fatal outside `--quick`: a full run that silently fell back
//!   to the smoke fixture would be indistinguishable from a real result.
//! * Every cell is checked for collapse, silence and saturation. A degenerate
//!   arm scores at chance, which would otherwise read as "attention does not
//!   help" and invert the conclusion.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use binn_data::{default_shd_dir, load_fixture, load_shd_split_capped, SHD_CHANCE};
use binn_lab::{mean_or_nan, shd_sample_to_example};
use binn_learn::shd_attention::AttentionConfig;
use binn_learn::{AlifEval, ShdAlifArm, ShdAlifConfig, ShdAlifRule, ShdExample};

const PROTOCOL_VERSION: u64 = 150;
const DEFAULT_SEEDS: usize = 12;
const DEFAULT_HIDDEN: usize = 128;
const DEFAULT_EPOCHS: usize = 15;
const DEFAULT_LR: f32 = 0.02;
const DEFAULT_ATTN_DIM: usize = 32;
const DEFAULT_ATTN_LAYERS: usize = 1;
/// Registered capped splits, matching the sibling ablation so the two are
/// readable side by side.
const CAPPED_TRAIN: usize = 2000;
const CAPPED_TEST: usize = 500;

/// The rules under test, plus the transported-feedback ceiling as a reference.
const RULES: [ShdAlifRule; 3] = [
    ShdAlifRule::Dfa,
    ShdAlifRule::BroadcastPm1,
    ShdAlifRule::EpropCeiling,
];

struct Cell {
    rule: ShdAlifRule,
    attention: bool,
    evals: Vec<AlifEval>,
    wall_secs: f64,
}

impl Cell {
    fn accuracies(&self) -> Vec<f32> {
        self.evals.iter().map(|e| e.accuracy).collect()
    }
    fn mean_acc(&self) -> f32 {
        mean_or_nan(&self.accuracies())
    }
    fn degenerate(&self) -> usize {
        self.evals.iter().filter(|e| e.is_degenerate()).count()
    }
    fn label(&self) -> String {
        format!(
            "{}{}",
            self.rule.label(),
            if self.attention { " +frozen-attn" } else { "" }
        )
    }
}

fn sd(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return f32::NAN;
    }
    let m = mean_or_nan(values);
    (values.iter().map(|v| (v - m) * (v - m)).sum::<f32>() / (values.len() - 1) as f32).sqrt()
}

fn print_help() {
    eprintln!(
        "shd-frozen-attention [--quick] [--seeds N] [--hidden N] [--epochs N] \\\n\
        \x20                    [--attn-dim N] [--attn-layers N] [--full] --out FILE\n\n\
         Frozen-attention local-learning arm. The attention block is never\n\
         updated; only the local rule learns."
    );
}

fn main() -> ExitCode {
    // Same authorization class as the sibling ablation: this trains local rules.
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::LocalLearning) {
        eprintln!("{error}");
        return ExitCode::from(2);
    }

    let mut quick = false;
    let mut full = false;
    let mut seeds = DEFAULT_SEEDS;
    let mut hidden = DEFAULT_HIDDEN;
    let mut epochs = DEFAULT_EPOCHS;
    let mut attn_dim = DEFAULT_ATTN_DIM;
    let mut attn_layers = DEFAULT_ATTN_LAYERS;
    let mut out: Option<PathBuf> = None;

    let argv: Vec<String> = env::args().skip(1).collect();
    // A closure capturing the cursor cannot coexist with indexing it, so the
    // cursor is advanced explicitly. Longer, but it compiles and it is obvious
    // which arm consumed a value.
    let mut i = 0usize;
    while i < argv.len() {
        let flag = argv[i].as_str();
        let value = |i: &mut usize| -> Option<String> {
            *i += 1;
            argv.get(*i).cloned()
        };
        match flag {
            "--quick" => quick = true,
            "--full" => full = true,
            "--seeds" | "--hidden" | "--epochs" | "--attn-dim" | "--attn-layers" => {
                let raw = match value(&mut i) {
                    Some(v) => v,
                    None => {
                        eprintln!("{flag} requires a value");
                        return ExitCode::from(2);
                    }
                };
                let parsed: usize = match raw.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("{flag} requires a non-negative integer, got {raw:?}");
                        return ExitCode::from(2);
                    }
                };
                // `--attn-dim`/`--attn-layers` are validated by
                // `AttentionConfig::new` below, which knows the real
                // constraints; the rest must simply be positive.
                if parsed == 0 && !matches!(flag, "--attn-dim" | "--attn-layers") {
                    eprintln!("{flag} must be positive");
                    return ExitCode::from(2);
                }
                match flag {
                    "--seeds" => seeds = parsed,
                    "--hidden" => hidden = parsed,
                    "--epochs" => epochs = parsed,
                    "--attn-dim" => attn_dim = parsed,
                    _ => attn_layers = parsed,
                }
            }
            "--out" => match value(&mut i) {
                Some(v) => out = Some(PathBuf::from(v)),
                None => {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                }
            },
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument {other:?}");
                print_help();
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let attention = match AttentionConfig::new(attn_dim, attn_layers) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    if quick {
        seeds = seeds.min(2);
        epochs = epochs.min(3);
    }

    println!("SHD frozen-attention local arm — protocol v{PROTOCOL_VERSION}");
    println!(
        "seeds={seeds} hidden={hidden} epochs={epochs} lr={DEFAULT_LR} \
         attention=d{attn_dim}xL{attn_layers}\n"
    );

    let (max_train, max_test) = if full {
        (None, None)
    } else {
        (Some(CAPPED_TRAIN), Some(CAPPED_TEST))
    };
    let dir = default_shd_dir();
    let split = match load_shd_split_capped(&dir, max_train, max_test) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("SHD cache unavailable ({e}); falling back to fixture");
            match load_fixture() {
                Ok(s) => s,
                Err(e2) => {
                    eprintln!("fixture unavailable too: {e2}");
                    return ExitCode::from(1);
                }
            }
        }
    };
    if split.fixture && !quick {
        eprintln!(
            "\nFATAL: loaded the smoke FIXTURE, not real SHD.\n\
             A scientific run on fixture data is indistinguishable from a real one\n\
             in the report, so this is a hard error rather than a warning.\n\
             Use --quick to allow the fixture."
        );
        return ExitCode::from(3);
    }

    let train: Vec<ShdExample> = split.train.iter().map(shd_sample_to_example).collect();
    let test: Vec<ShdExample> = split.test.iter().map(shd_sample_to_example).collect();
    if train.is_empty() || test.is_empty() {
        eprintln!("empty SHD split");
        return ExitCode::from(1);
    }
    let n_classes = split.n_classes;
    let chance = if n_classes == 20 {
        SHD_CHANCE
    } else {
        1.0 / n_classes as f32
    };
    println!(
        "loaded: n_train={} n_test={} n_in={} T={} classes={n_classes} fixture={}\n",
        train.len(),
        test.len(),
        split.n_in,
        split.t,
        split.fixture
    );
    drop(split);

    // ---- Grid. Baseline before treatment, per rule, so a timeout still leaves
    // a comparable pair rather than a lone treatment number. ----
    let mut cells: Vec<Cell> = Vec::new();
    for rule in RULES {
        for use_attention in [false, true] {
            let started = Instant::now();
            let cfg = ShdAlifConfig::feedforward_fixed(hidden, n_classes, DEFAULT_LR, epochs)
                .with_frozen_attention(if use_attention { Some(attention) } else { None });
            let mut evals = Vec::with_capacity(seeds);
            for seed_index in 0..seeds {
                // Seeds are shared between the two arms of a pair, so the
                // contrast is paired rather than merely matched in distribution.
                let seed = 5_170_001 + seed_index as u64;
                let mut arm = ShdAlifArm::new(&train[0], &cfg, rule, seed);
                evals.push(arm.train_and_evaluate_detailed(epochs, &train, &test));
            }
            let cell = Cell {
                rule,
                attention: use_attention,
                evals,
                wall_secs: started.elapsed().as_secs_f64(),
            };
            println!(
                "  {:<28} mean {:.4} ± {:.4}   degenerate {}/{}   {:.0}s",
                cell.label(),
                cell.mean_acc(),
                sd(&cell.accuracies()),
                cell.degenerate(),
                cell.evals.len(),
                cell.wall_secs
            );
            cells.push(cell);
            if let Some(path) = &out {
                // Rewritten after every cell: a kill leaves a valid partial.
                let _ = fs::write(
                    path,
                    render(&cells, chance, seeds, hidden, epochs, attention),
                );
            }
        }
    }

    if let Some(path) = &out {
        match fs::write(
            path,
            render(&cells, chance, seeds, hidden, epochs, attention),
        ) {
            Ok(()) => println!("\nreport -> {}", path.display()),
            Err(e) => {
                eprintln!("failed to write report: {e}");
                return ExitCode::from(1);
            }
        }
    }
    ExitCode::SUCCESS
}

fn render(
    cells: &[Cell],
    chance: f32,
    seeds: usize,
    hidden: usize,
    epochs: usize,
    attention: AttentionConfig,
) -> String {
    let mut report = format!(
        "# SHD frozen-attention local arm\n\n\
         **Protocol version:** {PROTOCOL_VERSION}  \n\
         **Preregistration:** `PREREG_2026-08-19_SHD_FROZEN_ATTENTION_LOCAL.md`  \n\
         **Seeds:** {seeds} · **hidden:** {hidden} · **epochs:** {epochs} · \
         **attention:** d{}xL{} (frozen)  \n\
         **Chance:** {chance:.4}\n\n\
         The attention block is drawn once and never updated. No credit reaches \
         the hidden layer through it; the read-out remains a single layer trained \
         by the same local rule. Bitwise-asserted by \
         `shd_alif::tests::training_never_moves_the_frozen_attention_block`.\n\n\
         | Rule | Frozen attention | Mean acc | SD | Degenerate seeds | Wall s |\n\
         |---|---|---:|---:|---:|---:|\n",
        attention.d_model, attention.layers
    );
    for cell in cells {
        report.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {}/{} | {:.0} |\n",
            cell.rule.label(),
            if cell.attention { "yes" } else { "no" },
            cell.mean_acc(),
            sd(&cell.accuracies()),
            cell.degenerate(),
            cell.evals.len(),
            cell.wall_secs
        ));
    }

    report.push_str(
        "\n## Paired contrasts\n\n| Rule | without | with | delta |\n|---|---:|---:|---:|\n",
    );
    for rule in RULES {
        let base = cells.iter().find(|c| c.rule == rule && !c.attention);
        let treat = cells.iter().find(|c| c.rule == rule && c.attention);
        if let (Some(base), Some(treat)) = (base, treat) {
            report.push_str(&format!(
                "| {} | {:.4} | {:.4} | **{:+.4}** |\n",
                rule.label(),
                base.mean_acc(),
                treat.mean_acc(),
                treat.mean_acc() - base.mean_acc()
            ));
        }
    }

    let degenerate: usize = cells.iter().map(Cell::degenerate).sum();
    let total: usize = cells.iter().map(|c| c.evals.len()).sum();
    report.push_str(&format!(
        "\n## Validity\n\n\
         Degenerate cells (collapsed, silent, saturated or diverged): **{degenerate} of {total}**. \
         A degenerate arm scores near chance, which reads as \"attention does not help\" \
         unless it is counted separately — so it is.\n"
    ));
    if degenerate > 0 {
        report.push_str("\n| Rule | Attention | Seed index | Defects |\n|---|---|---:|---|\n");
        for cell in cells {
            for (index, eval) in cell.evals.iter().enumerate() {
                if eval.is_degenerate() {
                    report.push_str(&format!(
                        "| {} | {} | {index} | {} |\n",
                        cell.rule.label(),
                        if cell.attention { "yes" } else { "no" },
                        eval.defects().join(", ")
                    ));
                }
            }
        }
    }
    report
}
