//! v147 — can the local arm learn *at all*, and is the v144 gap the optimiser?
//!
//! # What v144 actually compared
//!
//! ```text
//! ceiling:   train_bptt(..)      -> Adam::new(model)          // adaptive, momentum
//! treatment: train_feedback(..)  -> apply_sgd_step(.., 0.005) // plain SGD, one fixed lr
//! ```
//!
//! "BPTT 1.0000 vs matched RFB 0.2533" therefore varies **two** things at once:
//! the credit pathway *and* the optimiser. Adam is far more forgiving on
//! badly-scaled problems, and this codebase has produced three scale artifacts
//! in a week. On top of that, v144 trained on 200 examples × 20 epochs = 4 000
//! updates, with `lr` never swept — a REINFORCE-family local rule typically
//! needs an order of magnitude more.
//!
//! Before "local rules fail on shortcut-resistant tasks" can be claimed, those
//! confounds have to be removed.
//!
//! # Design
//!
//! Difficulty is fixed at the **easiest** setting, where v144 measured BPTT at
//! 1.0000. If the local arm cannot learn there, no harder setting matters.
//!
//! | Axis | Levels |
//! |---|---|
//! | Accessibility | `Accessible` (rate shortcut exists), `Immune` (v144 construction) |
//! | Arm | `feedback-sgd`, `bptt-sgd` (**optimiser-matched ceiling**), `bptt-adam` (best-achievable) |
//! | Budget | (200, 20), (1000, 20), (1000, 100) as (n_train, epochs) |
//! | Learning rate | 0.001 / 0.005 / 0.02 / 0.08 for both SGD arms |
//!
//! `bptt-sgd` is the honest one-variable contrast: same update rule, same step
//! size, only the gradients differ. `bptt-adam` is kept as a separate reference
//! and is **not** the comparison of record.
//!
//! # Questions, in dependency order
//!
//! - **Q1 (capability).** Does `feedback-sgd` ever clear chance on *Accessible*?
//!   If not, the multiclass local path is broken and nothing downstream stands.
//! - **Q2 (optimiser).** How much of the v144 gap is `bptt-adam` − `bptt-sgd`?
//!   That component is optimiser, not credit assignment.
//! - **Q3 (shortcut).** At matched optimiser and best budget/lr, how much does
//!   `feedback-sgd` lose going Accessible → Immune? *That* is the shortcut effect.
//!
//! Learning curves are recorded, not just final accuracy, so "never learned" is
//! distinguishable from "learned then collapsed". Realised step RMS and hidden
//! modulator RMS are reported per cell — the instrumentation `train_*` already
//! returns and v144 discarded.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use binn_data::{
    RateAccessibility, TemporalDifficulty, TemporalOrderSplit, TEMPORAL_ORDER_CHANCE,
    TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
use binn_lab::guards::{wilson_interval, Z_95};
use binn_lab::{mean, std_error, temporal_order_to_dense_examples};
use binn_learn::{
    mean_step_rms, random_feedback, train_bptt, train_bptt_sgd, train_feedback, SharedTemporalNet,
    ADAM_LR,
};

const PROTOCOL_VERSION: u64 = 147;
const MASTER_SEED: u64 = 0x7E4A_5147_0000_0001;

/// Easiest v144 setting: BPTT measured 1.0000 there.
const DIFFICULTY: TemporalDifficulty = TemporalDifficulty::new(0, 4);
/// Margin above chance required to call an arm "learning".
const LEARNS_MARGIN: f32 = 0.10;
/// Checkpoints per run, for the learning curve.
const CHECKPOINTS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    FeedbackSgd,
    BpttSgd,
    BpttAdam,
}

impl Arm {
    const fn label(self) -> &'static str {
        match self {
            Arm::FeedbackSgd => "feedback-sgd",
            Arm::BpttSgd => "bptt-sgd (matched)",
            Arm::BpttAdam => "bptt-adam (reference)",
        }
    }
    /// Adam ignores the swept learning rate.
    const fn uses_lr(self) -> bool {
        !matches!(self, Arm::BpttAdam)
    }
}

#[derive(Clone, Copy, Debug)]
struct Budget {
    n_train: usize,
    epochs: usize,
}

#[derive(Clone, Debug)]
struct Cell {
    accessibility: RateAccessibility,
    arm: Arm,
    budget: Budget,
    lr: f32,
    accs: Vec<f32>,
    curve: Vec<f32>,
    step_rms: f32,
    modulator_rms: f32,
    wall_secs: f64,
}

impl Cell {
    fn mean_acc(&self) -> f32 {
        mean(&self.accs)
    }
    fn learns(&self) -> bool {
        self.mean_acc() >= TEMPORAL_ORDER_CHANCE + LEARNS_MARGIN
    }
}

fn accessibility_label(a: RateAccessibility) -> &'static str {
    match a {
        RateAccessibility::Accessible => "accessible",
        RateAccessibility::Immune => "immune",
    }
}

fn main() -> ExitCode {
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::Optimizer) {
        eprintln!("temporal-optimizer-control: {error}");
        return ExitCode::from(3);
    }
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut wide = false;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--wide" => wide = true,
            "--out" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(v));
            }
            "-h" | "--help" => {
                println!(
                    "Usage: cargo run --release -p binn-lab --bin temporal-optimizer-control -- \\\n\
                     \x20 [--quick] [--wide] [--out PATH]\n\n\
                     v147: separates the credit pathway from the optimiser and the training\n\
                     budget, on the easiest v144 difficulty.\n\n\
                     --quick  smoke, non-citable (~1 min)\n\
                     (default) 3 seeds, 4 lrs, budgets to 100k updates (~15 min)\n\
                     --wide    10 seeds, 6 lrs, budgets to 1M updates (~6-12 h).\n\
                     \x20         Use when a negative Q1 must not be dismissible as\n\
                     \x20         under-powered."
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    // `--wide` exists so a negative Q1 cannot later be dismissed as
    // under-powered: 10 seeds, six learning rates spanning two and a half
    // decades, and a top budget of 1M updates per run. If the local arm cannot
    // clear chance anywhere in that grid on the *easiest* difficulty with a rate
    // shortcut available, the result is about the rule, not the schedule.
    let n_seeds = if quick {
        1
    } else if wide {
        10
    } else {
        3
    };
    let n_test = if quick { 40 } else { 200 };
    let hidden = if quick { 16 } else { 64 };
    let budgets: Vec<Budget> = if quick {
        vec![Budget {
            n_train: 100,
            epochs: 5,
        }]
    } else if wide {
        vec![
            Budget {
                n_train: 200,
                epochs: 20,
            },
            Budget {
                n_train: 1000,
                epochs: 100,
            },
            Budget {
                n_train: 5000,
                epochs: 200,
            },
        ]
    } else {
        vec![
            Budget {
                n_train: 200,
                epochs: 20,
            },
            Budget {
                n_train: 1000,
                epochs: 20,
            },
            Budget {
                n_train: 1000,
                epochs: 100,
            },
        ]
    };
    let lrs: Vec<f32> = if quick {
        vec![0.005, 0.02]
    } else if wide {
        vec![0.0005, 0.001, 0.005, 0.02, 0.08, 0.2]
    } else {
        vec![0.001, 0.005, 0.02, 0.08]
    };

    println!("========================================================================");
    println!("v{PROTOCOL_VERSION} temporal optimiser / budget control");
    println!(
        "difficulty=({}, {}) seeds={n_seeds} hidden={hidden} n_test={n_test}",
        DIFFICULTY.jitter_radius, DIFFICULTY.distractor_events
    );
    println!("========================================================================\n");

    let mut cells: Vec<Cell> = Vec::new();

    for accessibility in [RateAccessibility::Accessible, RateAccessibility::Immune] {
        for arm in [Arm::FeedbackSgd, Arm::BpttSgd, Arm::BpttAdam] {
            for budget in &budgets {
                // Both arms of the `if` must yield the same type, so the
                // single-element branch has to be a `Vec` too.
                #[allow(clippy::useless_vec)]
                let arm_lrs: Vec<f32> = if arm.uses_lr() {
                    lrs.clone()
                } else {
                    vec![ADAM_LR]
                };
                for &lr in &arm_lrs {
                    let t0 = Instant::now();
                    let mut accs = Vec::with_capacity(n_seeds);
                    // Fixed length (`CHECKPOINTS` is a const) and only ever indexed
                    // and iterated, so an array is a drop-in with identical arithmetic.
                    let mut curve_acc = [0.0f32; CHECKPOINTS];
                    let mut step_total = 0.0f32;
                    let mut mod_total = 0.0f32;

                    for s in 0..n_seeds {
                        let seed = MASTER_SEED ^ (s as u64).wrapping_mul(0x1000_009D);
                        let split = match TemporalOrderSplit::generate_with_rate_accessibility(
                            budget.n_train,
                            n_test,
                            DIFFICULTY,
                            seed,
                            accessibility,
                        ) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("split generation failed: {e}");
                                return ExitCode::from(1);
                            }
                        };
                        let train = temporal_order_to_dense_examples(&split.train);
                        let test = temporal_order_to_dense_examples(&split.test);

                        let mut model = SharedTemporalNet::new(
                            TEMPORAL_ORDER_N_IN,
                            TEMPORAL_ORDER_T,
                            binn_data::TEMPORAL_ORDER_N_CLASSES,
                            &[hidden],
                            0.9,
                            1.0,
                            5.0,
                            seed,
                        );
                        let feedback = random_feedback(&model, seed);

                        // Train in chunks so the learning curve is observable.
                        let chunk = (budget.epochs / CHECKPOINTS).max(1);
                        let mut diagnostics = Vec::new();
                        #[allow(clippy::needless_range_loop)]
                        for c in 0..CHECKPOINTS {
                            let mut d = match arm {
                                Arm::FeedbackSgd => {
                                    train_feedback(&mut model, &feedback, &train, chunk, lr)
                                }
                                Arm::BpttSgd => train_bptt_sgd(&mut model, &train, chunk, lr),
                                Arm::BpttAdam => train_bptt(&mut model, &train, chunk),
                            };
                            diagnostics.append(&mut d);
                            curve_acc[c] += model.accuracy(&test);
                        }

                        accs.push(model.accuracy(&test));
                        step_total += mean_step_rms(&diagnostics);
                        mod_total += mean(&model.feedback_modulator_rms(&test, &feedback));
                    }

                    let n = n_seeds as f32;
                    let cell = Cell {
                        accessibility,
                        arm,
                        budget: *budget,
                        lr,
                        accs,
                        curve: curve_acc.iter().map(|v| v / n).collect(),
                        step_rms: step_total / n,
                        modulator_rms: mod_total / n,
                        wall_secs: t0.elapsed().as_secs_f64(),
                    };
                    println!(
                        "  {:<11} {:<22} n={:<5} ep={:<4} lr={:<7.4} acc={:.4} step_rms={:.3e} ({:.1}s)",
                        accessibility_label(accessibility),
                        arm.label(),
                        budget.n_train,
                        budget.epochs,
                        lr,
                        cell.mean_acc(),
                        cell.step_rms,
                        cell.wall_secs
                    );
                    cells.push(cell);
                }
            }
        }
    }

    let report = render(&cells, n_seeds, n_test, hidden, quick);
    println!("\n{report}");
    if let Some(path) = &out {
        if let Err(e) = fs::write(path, &report) {
            eprintln!("failed to write {}: {e}", path.display());
            return ExitCode::from(1);
        }
        println!("Report saved to: {}", path.display());
    }
    ExitCode::SUCCESS
}

fn best(cells: &[Cell], accessibility: RateAccessibility, arm: Arm) -> Option<&Cell> {
    cells
        .iter()
        .filter(|c| c.accessibility == accessibility && c.arm == arm)
        .max_by(|a, b| {
            a.mean_acc()
                .partial_cmp(&b.mean_acc())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn render(cells: &[Cell], n_seeds: usize, n_test: usize, hidden: usize, quick: bool) -> String {
    let mut rows = String::new();
    for c in cells {
        let curve = c
            .curve
            .iter()
            .map(|v| format!("{v:.3}"))
            .collect::<Vec<_>>()
            .join(" → ");
        let _ = writeln!(
            rows,
            "| {} | {} | {} | {} | {:.4} | {:.4} | {:.4} | {curve} | {:.3e} | {:.3e} |",
            accessibility_label(c.accessibility),
            c.arm.label(),
            c.budget.n_train,
            c.budget.epochs,
            c.lr,
            c.mean_acc(),
            std_error(&c.accs),
            c.step_rms,
            c.modulator_rms,
        );
    }

    let fb_acc = best(cells, RateAccessibility::Accessible, Arm::FeedbackSgd);
    let fb_imm = best(cells, RateAccessibility::Immune, Arm::FeedbackSgd);
    let sgd_acc = best(cells, RateAccessibility::Accessible, Arm::BpttSgd);
    let adam_acc = best(cells, RateAccessibility::Accessible, Arm::BpttAdam);

    let q1 = fb_acc.map(|c| c.learns()).unwrap_or(false);
    let optimiser_gap = match (adam_acc, sgd_acc) {
        (Some(a), Some(s)) => a.mean_acc() - s.mean_acc(),
        _ => f32::NAN,
    };
    let shortcut_gap = match (fb_acc, fb_imm) {
        (Some(a), Some(i)) => a.mean_acc() - i.mean_acc(),
        _ => f32::NAN,
    };

    let (lo, hi) = fb_acc
        .map(|c| {
            wilson_interval(
                (c.mean_acc() * n_test as f32).round() as usize,
                n_test,
                Z_95,
            )
        })
        .unwrap_or((f32::NAN, f32::NAN));

    let interpretation = if !q1 {
        "**Q1 FAILS — the local arm does not learn even where a rate shortcut exists.** \
         The multiclass feedback path is broken or mis-scaled; the v144 chance result says \
         nothing about shortcut resistance. Debug `feedback_gradients` before any claim. \
         Compare `step_rms` between `feedback-sgd` and `bptt-sgd` at the same lr: if they \
         differ by an order of magnitude, the feedback projection is rescaling the update \
         and the learning rates are not comparable."
            .to_string()
    } else if shortcut_gap.is_finite() && shortcut_gap >= LEARNS_MARGIN {
        format!(
            "**Q1 passes, Q3 supported.** The local arm learns the rate-accessible task \
             ({:.4}, 95% CI [{lo:.4}, {hi:.4}]) and loses {shortcut_gap:+.4} going to the \
             rate-immune construction. That is the shortcut effect, measured at matched \
             optimiser. Of the original v144 gap, {optimiser_gap:+.4} is attributable to \
             Adam-vs-SGD alone and must be subtracted before any credit-assignment claim.",
            fb_acc.map(|c| c.mean_acc()).unwrap_or(f32::NAN)
        )
    } else {
        format!(
            "**Q1 passes, Q3 not supported.** The local arm learns the accessible task but \
             loses only {shortcut_gap:+.4} on the immune one — so shortcut-resistance is not \
             what breaks it. Optimiser contribution to the v144 gap: {optimiser_gap:+.4}. \
             Re-examine whether v144's chance result came from budget or learning rate rather \
             than task structure."
        )
    };

    format!(
        "# v{PROTOCOL_VERSION} — temporal optimiser / budget control\n\n\
        **Schedule:** {} · difficulty ({}, {}) · seeds {n_seeds} · hidden {hidden} · n_test {n_test}  \n\
        **Chance:** {TEMPORAL_ORDER_CHANCE:.4} · **learns threshold:** chance + {LEARNS_MARGIN:.2}\n\n\
        ## Why this run exists\n\n\
        v144 compared `train_bptt` (**Adam**) against `train_feedback` (**plain SGD, lr fixed \
        at 0.005**) on 200 examples × 20 epochs, with no learning-rate sweep. That varies the \
        credit pathway, the optimiser and the budget simultaneously. `bptt-sgd` below is the \
        optimiser-matched ceiling — same update rule, same step size, only the gradients \
        differ. It, not `bptt-adam`, is the comparison of record.\n\n\
        ## Grid\n\n\
        | Accessibility | Arm | n_train | epochs | lr | Mean acc | SE | Learning curve | Step RMS | Modulator RMS |\n\
        |---|---|---:|---:|---:|---:|---:|---|---:|---:|\n\
        {rows}\n\
        ## Questions\n\n\
        | ID | Question | Measured | Verdict |\n\
        |---|---|---|---|\n\
        | Q1 | Does `feedback-sgd` learn the **rate-accessible** task at all? | best {:.4} | {} |\n\
        | Q2 | How much of the v144 gap is optimiser? (`bptt-adam` − `bptt-sgd`) | {optimiser_gap:+.4} | descriptive |\n\
        | Q3 | Shortcut effect at matched optimiser (accessible − immune, feedback arm) | {shortcut_gap:+.4} | descriptive |\n\n\
        ## Interpretation\n\n\
        {interpretation}\n\n\
        ## Non-claims\n\n\
        - Q2 and Q3 are **descriptive**; neither is a preregistered hypothesis test.\n\
        - A `--quick` run is non-citable.\n\
        - This run fixes difficulty at the easiest setting. It says nothing about harder ones.\n\
        - `bptt-adam` is a best-achievable reference, **not** a matched ceiling. Do not quote \
        a gap against it as a credit-assignment result.\n",
        if quick { "QUICK / non-citable" } else { "scientific" },
        DIFFICULTY.jitter_radius,
        DIFFICULTY.distractor_events,
        fb_acc.map(|c| c.mean_acc()).unwrap_or(f32::NAN),
        if q1 { "learns" } else { "does not learn" },
    )
}
