//! A6 — ceiling health of the surviving matched arms.
//!
//! ceiling-health-ok: this binary reports the raw reference-vs-arm **ordering at
//! each training budget**, which is the measurement it exists to make. Replacing
//! that column with `guards::CeilingHealth::label()` would collapse a sensitivity
//! curve into a single verdict and destroy the finding — the point is precisely
//! to show *where* the ordering flips as the reference's budget rises.
//!
//! The hole `CeilingHealth` closes cannot bite here: the swept references run
//! from 0.9013 to 1.0000 against a chance of 0.5 (see
//! `RESULT_2026-08-19_A6_CEILING_HEALTH.md`), so no budget point is anywhere near
//! a dead reference. If a future sweep lowers the budget far enough to approach
//! chance, this exemption stops being true and must be revisited.
//!
//! # The question
//!
//! Two matched-architecture arms currently hold a PASS, and both clear the
//! gradient reference they are supposed to be bounded by:
//!
//! | schedule | arm | reference |
//! |---|---|---|
//! | `c1-dfa-c8c4fe0899908b84` | 0.9387 | 0.8963 |
//! | `c1-rl-42eddc9c801308e9`  | 0.9200 | 0.8887 |
//!
//! On the DFA schedule the broadcast-graded **control** reaches 0.9863 against
//! that same 0.8963. Two arms clear the ceiling and one of them is a control.
//! Either local rules genuinely beat backprop on this substrate, or the
//! SuperSpike BPTT references are undertrained and the matched side of the
//! transfer gap is not measuring what the paper says it measures.
//!
//! This binary answers it the only way that settles it: raise the **reference's
//! own** training budget while holding the forward, splits, seeds, and the arms
//! fixed, and report the reference-vs-arm ordering as a function of budget.
//!
//! # Why this is not just `--bptt-epochs`
//!
//! `runner_dfa_match::run_seed` and `runner_rl_match::run_seed` both read a
//! single `config.base.bptt_epochs` and hand it to the reference *and* every
//! arm. Raising it there raises everyone's budget, which cannot separate "the
//! reference was undertrained" from "everything was undertrained". Only
//! `bptt_lr` is reference-only in that path.
//!
//! So this binary drives the pieces directly — `freeze_trials`,
//! `samples_to_gradient_examples`, and the same arm constructors the canonical
//! runners use — with two independent budgets: a swept one for the reference and
//! the frozen canonical one for the arms.
//!
//! # Scope
//!
//! **Exploratory. Not a canonical run.** It writes no frozen manifest, claims no
//! `--config-hash`, and moves no preregistered threshold. Its output is a
//! sensitivity table plus an explicit undertrained / not-undertrained verdict.
//! Because it has no bit-identity requirement it is the one task in the week
//! plan that may legitimately run off the local machine.
//!
//! # Harness self-check
//!
//! The canonical budget point (`bptt_epochs`, `bptt_lr` straight off the frozen
//! config) is always swept, whether or not the caller lists it. If this harness
//! is faithful, that row must reproduce the published reference and arm numbers.
//! A row that does not reproduce them invalidates every other row, and the
//! verdict says so rather than reporting the sweep as if it stood.

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;

use binn_lab::{freeze_trials, samples_to_gradient_examples, DfaMatchConfig, RlMatchConfig};
use binn_learn::{
    GradientExample, MatchedBroadcastErr, MatchedDfa, MatchedGradient, MatchedRlFlat,
    MatchedRlGraded, MatchedRlReinforceFb, DEFAULT_MATCHED_BETA,
};

const PROTOCOL_VERSION: u64 = 1;
const EXPERIMENT_NAME: &str = "a6-ceiling-health";

/// Published reference means this sweep exists to interrogate, used only to
/// report reproduction drift in the canonical row. Never used as a threshold.
const PUBLISHED_DFA_REFERENCE: f32 = 0.8963;
const PUBLISHED_RL_REFERENCE: f32 = 0.8887;

/// One reference training budget.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Budget {
    epochs: usize,
    lr: f32,
}

impl Budget {
    fn label(&self) -> String {
        format!("e{}/lr{}", self.epochs, self.lr)
    }
}

/// Which matched suite to sweep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suite {
    Dfa,
    Rl,
}

impl Suite {
    fn label(self) -> &'static str {
        match self {
            Self::Dfa => "c1-dfa",
            Self::Rl => "c1-rl",
        }
    }

    /// Name of the arm whose PASS is under interrogation.
    fn arm_label(self) -> &'static str {
        match self {
            Self::Dfa => "MatchedDfa",
            Self::Rl => "MatchedRlReinforceFb",
        }
    }

    fn control_label(self) -> &'static str {
        match self {
            Self::Dfa => "MatchedBroadcastErr (control)",
            Self::Rl => "MatchedRlGraded",
        }
    }

    fn third_label(self) -> Option<&'static str> {
        match self {
            Self::Dfa => None,
            Self::Rl => Some("MatchedRlFlat (±1 baseline)"),
        }
    }

    fn published_reference(self) -> f32 {
        match self {
            Self::Dfa => PUBLISHED_DFA_REFERENCE,
            Self::Rl => PUBLISHED_RL_REFERENCE,
        }
    }
}

/// The frozen substrate a suite is swept against. Cloned from the canonical
/// config so the forward, splits, and seed lineage are bit-for-bit the ones the
/// published numbers came from.
struct Substrate {
    suite: Suite,
    seeds: Vec<u64>,
    n_hidden: usize,
    eta: f32,
    lambda: f32,
    beta: f32,
    /// Canonical epochs — the *arms'* budget, held fixed across the sweep.
    arm_epochs: usize,
    /// Canonical reference budget, always included as the self-check row.
    canonical: Budget,
    /// Frozen per-seed train/test splits.
    splits: Vec<(Vec<GradientExample>, Vec<GradientExample>)>,
}

fn beta_of(surrogate_beta: f32) -> f32 {
    if surrogate_beta > 0.0 {
        surrogate_beta
    } else {
        DEFAULT_MATCHED_BETA
    }
}

impl Substrate {
    fn build(suite: Suite, n_seeds: Option<usize>, quick: bool) -> Self {
        // Take the canonical config verbatim, then override only the seed count
        // when the caller asks for a shorter pilot.
        let (base, canonical, seeds, n_hidden, eta, lambda, beta, arm_epochs) = match suite {
            Suite::Dfa => {
                let mut config = if quick {
                    DfaMatchConfig::quick()
                } else {
                    DfaMatchConfig::scientific()
                };
                if let Some(n) = n_seeds {
                    config.base.n_seeds = n;
                }
                let b = config.base.clone();
                (
                    b.clone(),
                    Budget {
                        epochs: b.bptt_epochs,
                        lr: b.bptt_lr,
                    },
                    config.seeds(),
                    b.n_hidden,
                    b.eta,
                    b.lambda,
                    beta_of(b.surrogate_beta),
                    b.bptt_epochs,
                )
            }
            Suite::Rl => {
                let mut config = if quick {
                    RlMatchConfig::quick()
                } else {
                    RlMatchConfig::scientific()
                };
                if let Some(n) = n_seeds {
                    config.base.n_seeds = n;
                }
                let b = config.base.clone();
                (
                    b.clone(),
                    Budget {
                        epochs: b.bptt_epochs,
                        lr: b.bptt_lr,
                    },
                    config.seeds(),
                    b.n_hidden,
                    b.eta,
                    b.lambda,
                    beta_of(b.surrogate_beta),
                    b.bptt_epochs,
                )
            }
        };

        // Freeze the splits once. Every budget point and every arm reads these
        // same examples, which is what "holding the forward and splits fixed"
        // has to mean for the comparison to be about the budget.
        let splits = seeds
            .iter()
            .map(|&seed| {
                let split = freeze_trials(&base, seed);
                (
                    samples_to_gradient_examples(&split.train),
                    samples_to_gradient_examples(&split.test),
                )
            })
            .collect();

        Self {
            suite,
            seeds,
            n_hidden,
            eta,
            lambda,
            beta,
            arm_epochs,
            canonical,
            splits,
        }
    }

    /// Train the arms once per seed at the canonical budget. They do not depend
    /// on the swept reference budget, so computing them once is both correct and
    /// what keeps the sweep affordable.
    fn arm_accuracies(&self, index: usize) -> ArmRow {
        let (train, test) = &self.splits[index];
        let seed = self.seeds[index];
        match self.suite {
            Suite::Dfa => {
                let mut arm =
                    MatchedDfa::new(self.n_hidden, self.eta, self.lambda, self.beta, seed);
                let arm_acc = arm
                    .train_and_evaluate(self.arm_epochs, train, test)
                    .accuracy;
                let mut control =
                    MatchedBroadcastErr::new(self.n_hidden, self.eta, self.lambda, self.beta, seed);
                let control_acc = control
                    .train_and_evaluate(self.arm_epochs, train, test)
                    .accuracy;
                ArmRow {
                    arm: arm_acc,
                    control: control_acc,
                    third: None,
                }
            }
            Suite::Rl => {
                let mut arm = MatchedRlReinforceFb::new(
                    self.n_hidden,
                    self.eta,
                    self.lambda,
                    self.beta,
                    seed,
                );
                let arm_acc = arm
                    .train_and_evaluate(self.arm_epochs, train, test)
                    .accuracy;
                let mut graded =
                    MatchedRlGraded::new(self.n_hidden, self.eta, self.lambda, self.beta, seed);
                let graded_acc = graded
                    .train_and_evaluate(self.arm_epochs, train, test)
                    .accuracy;
                let mut flat =
                    MatchedRlFlat::new(self.n_hidden, self.eta, self.lambda, self.beta, seed);
                let flat_acc = flat
                    .train_and_evaluate(self.arm_epochs, train, test)
                    .accuracy;
                ArmRow {
                    arm: arm_acc,
                    control: graded_acc,
                    third: Some(flat_acc),
                }
            }
        }
    }

    /// Train the gradient reference at one budget for one seed.
    fn reference_accuracy(&self, index: usize, budget: Budget) -> f32 {
        let (train, test) = &self.splits[index];
        let mut gradient = MatchedGradient::new_feedforward(
            self.n_hidden,
            budget.lr,
            self.beta,
            self.seeds[index],
        );
        gradient
            .train_and_evaluate(budget.epochs, train, test)
            .accuracy
    }
}

#[derive(Clone, Copy, Debug)]
struct ArmRow {
    arm: f32,
    control: f32,
    third: Option<f32>,
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

/// Standard error of the mean. Reported so a reader can tell an ordering flip
/// from sampling noise instead of guessing.
fn standard_error(values: &[f32]) -> f32 {
    let n = values.len();
    if n < 2 {
        return f32::NAN;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m) * (v - m)).sum::<f32>() / (n - 1) as f32;
    (variance / n as f32).sqrt()
}

/// Run `tasks` across `jobs` OS threads and return results in input order.
///
/// Every task is an independent, deterministically seeded train/evaluate, so the
/// thread split cannot change any number — only how long it takes. Results are
/// reassembled by index, never by completion order.
fn parallel_map<T, R, F>(tasks: Vec<T>, jobs: usize, f: F) -> Vec<R>
where
    T: Send + Sync,
    R: Send + Default + Clone,
    F: Fn(&T) -> R + Sync,
{
    let jobs = jobs.max(1).min(tasks.len().max(1));
    let mut out = vec![R::default(); tasks.len()];
    let chunk = tasks.len().div_ceil(jobs).max(1);
    let f = &f;
    thread::scope(|scope| {
        for (task_chunk, out_chunk) in tasks.chunks(chunk).zip(out.chunks_mut(chunk)) {
            scope.spawn(move || {
                for (task, slot) in task_chunk.iter().zip(out_chunk.iter_mut()) {
                    *slot = f(task);
                }
            });
        }
    });
    out
}

/// One suite's swept result.
struct SuiteResult {
    suite: Suite,
    seeds: usize,
    arm_epochs: usize,
    canonical: Budget,
    arms: ArmRow,
    /// Budget -> (mean reference, SE, per-seed accuracies).
    references: Vec<(Budget, f32, f32)>,
}

fn run_suite(
    suite: Suite,
    budgets: &[Budget],
    n_seeds: Option<usize>,
    quick: bool,
    jobs: usize,
    arm_epochs: Option<usize>,
) -> SuiteResult {
    let mut substrate = Substrate::build(suite, n_seeds, quick);
    // Raising only the reference's budget invites the obvious objection that the
    // arms were simply given less compute. `--arm-epochs` answers it by putting
    // both sides on the same budget, which turns "the reference was
    // undertrained" into a claim that survives a matched-compute reading.
    if let Some(e) = arm_epochs {
        substrate.arm_epochs = e;
    }
    let n = substrate.seeds.len();

    // Arms: one task per seed, canonical budget, computed once.
    let arm_rows = parallel_map((0..n).collect::<Vec<_>>(), jobs, |&i| {
        Some(substrate.arm_accuracies(i))
    });
    let arm_rows: Vec<ArmRow> = arm_rows.into_iter().map(|r| r.expect("arm row")).collect();
    let arms = ArmRow {
        arm: mean(&arm_rows.iter().map(|r| r.arm).collect::<Vec<_>>()),
        control: mean(&arm_rows.iter().map(|r| r.control).collect::<Vec<_>>()),
        third: if arm_rows.iter().all(|r| r.third.is_some()) {
            Some(mean(
                &arm_rows
                    .iter()
                    .map(|r| r.third.expect("third arm"))
                    .collect::<Vec<_>>(),
            ))
        } else {
            None
        },
    };

    // References: one task per (budget, seed).
    let tasks: Vec<(usize, usize)> = budgets
        .iter()
        .enumerate()
        .flat_map(|(b, _)| (0..n).map(move |i| (b, i)))
        .collect();
    let accuracies = parallel_map(tasks.clone(), jobs, |&(b, i)| {
        Some(substrate.reference_accuracy(i, budgets[b]))
    });

    let mut per_budget: BTreeMap<usize, Vec<f32>> = BTreeMap::new();
    for ((b, _), acc) in tasks.iter().zip(accuracies) {
        per_budget
            .entry(*b)
            .or_default()
            .push(acc.expect("reference accuracy"));
    }
    let references = budgets
        .iter()
        .enumerate()
        .map(|(b, budget)| {
            let values = per_budget.get(&b).cloned().unwrap_or_default();
            (*budget, mean(&values), standard_error(&values))
        })
        .collect();

    SuiteResult {
        suite,
        seeds: n,
        arm_epochs: substrate.arm_epochs,
        canonical: substrate.canonical,
        arms,
        references,
    }
}

fn render(results: &[SuiteResult], quick: bool) -> String {
    let mut md = String::new();
    md.push_str("# A6 — ceiling health of the surviving matched arms\n\n");
    let _ = writeln!(
        md,
        "`{EXPERIMENT_NAME}` protocol v{PROTOCOL_VERSION}. **Exploratory sensitivity sweep — \
         not a canonical run, no frozen manifest, no `--config-hash` claim.**\n"
    );
    if quick {
        md.push_str("> **PILOT schedule.** Reduced seeds/size. Not a scientific verdict.\n\n");
    }
    md.push_str(
        "The reference's training budget is swept while the forward, the frozen splits, the \
         seed lineage, and every arm's budget are held fixed. Only `MatchedGradient` sees the \
         swept `epochs`/`lr`; the arms are trained once at the canonical budget.\n\n",
    );

    for result in results {
        let _ = writeln!(md, "## `{}`\n", result.suite.label());
        let _ = writeln!(
            md,
            "- seeds: **{}** · arm budget (fixed): **{} epochs** · canonical reference budget: \
             **{}**",
            result.seeds,
            result.arm_epochs,
            result.canonical.label()
        );
        let _ = writeln!(
            md,
            "- {} (fixed): **{:.4}**",
            result.suite.arm_label(),
            result.arms.arm
        );
        let _ = writeln!(
            md,
            "- {} (fixed): **{:.4}**",
            result.suite.control_label(),
            result.arms.control
        );
        if let (Some(label), Some(value)) = (result.suite.third_label(), result.arms.third) {
            let _ = writeln!(md, "- {label} (fixed): **{value:.4}**");
        }
        md.push('\n');

        md.push_str("| reference budget | reference mean | SE | vs arm | vs control |\n");
        md.push_str("|---|---|---|---|---|\n");
        for (budget, mean_ref, se) in &result.references {
            let canonical_mark = if *budget == result.canonical {
                " *(canonical)*"
            } else {
                ""
            };
            let vs_arm = if mean_ref > &result.arms.arm {
                "**reference above**"
            } else {
                "arm above"
            };
            let vs_control = if mean_ref > &result.arms.control {
                "**reference above**"
            } else {
                "control above"
            };
            let _ = writeln!(
                md,
                "| `{}`{} | {:.4} | {:.4} | {} | {} |",
                budget.label(),
                canonical_mark,
                mean_ref,
                se,
                vs_arm,
                vs_control
            );
        }
        md.push('\n');

        // Harness self-check first: if the canonical row does not reproduce the
        // published reference, nothing else in this table is interpretable.
        let canonical_row = result
            .references
            .iter()
            .find(|(b, _, _)| *b == result.canonical);
        let published = result.suite.published_reference();
        match canonical_row {
            Some((_, mean_ref, _)) => {
                let drift = (mean_ref - published).abs();
                if drift <= 0.02 {
                    let _ = writeln!(
                        md,
                        "**Harness check: reproduces.** Canonical row {mean_ref:.4} vs published \
                         {published:.4} (drift {drift:.4})."
                    );
                } else {
                    let _ = writeln!(
                        md,
                        "**Harness check: DOES NOT REPRODUCE.** Canonical row {mean_ref:.4} vs \
                         published {published:.4} (drift {drift:.4}). Treat every row below as \
                         uninterpretable until this is explained — the sweep is measuring a \
                         different substrate than the published number."
                    );
                }
            }
            None => md.push_str("**Harness check: canonical row missing — cannot validate.**\n"),
        }
        md.push('\n');

        let best = result
            .references
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .copied();
        match best {
            Some((budget, mean_ref, _)) if mean_ref > result.arms.arm => {
                let _ = writeln!(
                    md,
                    "**Verdict: at this arm budget, more reference compute inverts the \
                     ordering.** At `{}` the reference reaches {:.4}, above the {} arm's {:.4} \
                     (arms held at {} epochs). \n\n> **Scope — read before quoting this.** The \
                     arms did not receive the extra compute. This row shows the reference was \
                     undertrained; it does *not* show the reference was *uniquely* undertrained. \
                     Re-run with `--arm-epochs` equal to the reference budget before claiming \
                     the published {:.4} is a budget artifact — if the arms rise too, the \
                     finding is that the whole schedule is undertrained, which is a different \
                     claim with different consequences for the paper.",
                    budget.label(),
                    mean_ref,
                    result.suite.arm_label(),
                    result.arms.arm,
                    result.arm_epochs,
                    published
                );
            }
            Some((budget, mean_ref, _)) => {
                let _ = writeln!(
                    md,
                    "**Verdict: no budget tested lifts the reference above the arm.** Best is \
                     `{}` at {:.4}, still below the {} arm's {:.4}. On this evidence the \
                     published {:.4} is not simply undertrained — but note this is a bounded \
                     sweep, not a proof of convergence.",
                    budget.label(),
                    mean_ref,
                    result.suite.arm_label(),
                    result.arms.arm,
                    published
                );
            }
            None => md.push_str("**Verdict: no budgets ran.**\n"),
        }
        if result.arms.control > result.arms.arm {
            let _ = writeln!(
                md,
                "\n> The {} still sits above the arm under test ({:.4} vs {:.4}). Whatever the \
                 reference does, that ordering is its own open question.",
                result.suite.control_label(),
                result.arms.control,
                result.arms.arm
            );
        }
        md.push('\n');
    }

    md.push_str(
        "## Reading this table\n\n\
         A row where the reference sits below the arm is only evidence about *that* budget. \
         The sweep is bounded above by the largest budget listed; it cannot show the reference \
         has converged, only that it had not overtaken the arm by the budgets tested. Report \
         both numbers, never the conclusion alone.\n",
    );
    md
}

fn parse_list<T: std::str::FromStr>(raw: &str, what: &str) -> Result<Vec<T>, String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<T>()
                .map_err(|_| format!("could not parse {what} value {s:?}"))
        })
        .collect()
}

fn usage() -> String {
    format!(
        "{EXPERIMENT_NAME} — A6 reference ceiling-health sweep\n\n\
         Options:\n  \
         --suite dfa|rl|both   which matched suite to sweep (default: both)\n  \
         --epochs a,b,c        reference epoch budgets (default: 80,160,320,640)\n  \
         --lr a,b,c            reference learning rates (default: canonical only)\n  \
         --seeds N             override seed count (default: the config's own)\n  \
         --arm-epochs N        train the arms at N epochs too (default: canonical)\n  \
         --jobs N              worker threads (default: available parallelism)\n  \
         --quick               PILOT schedule, not a scientific verdict\n  \
         --out PATH            write the markdown report here\n"
    )
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut suite_arg = "both".to_string();
    let mut epochs_arg = "80,160,320,640".to_string();
    let mut lr_arg: Option<String> = None;
    let mut seeds_arg: Option<usize> = None;
    let mut jobs_arg: Option<usize> = None;
    let mut arm_epochs_arg: Option<usize> = None;
    let mut quick = false;
    let mut out: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        let take = |i: &mut usize, flag: &str| -> Result<String, String> {
            *i += 1;
            args.get(*i)
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        let result = match args[i].as_str() {
            "--suite" => take(&mut i, "--suite").map(|v| suite_arg = v),
            "--epochs" => take(&mut i, "--epochs").map(|v| epochs_arg = v),
            "--lr" => take(&mut i, "--lr").map(|v| lr_arg = Some(v)),
            "--seeds" => take(&mut i, "--seeds").and_then(|v| {
                v.parse()
                    .map(|n| seeds_arg = Some(n))
                    .map_err(|_| "--seeds needs an integer".to_string())
            }),
            "--jobs" => take(&mut i, "--jobs").and_then(|v| {
                v.parse()
                    .map(|n| jobs_arg = Some(n))
                    .map_err(|_| "--jobs needs an integer".to_string())
            }),
            "--out" => take(&mut i, "--out").map(|v| out = Some(PathBuf::from(v))),
            "--arm-epochs" => take(&mut i, "--arm-epochs").and_then(|v| {
                v.parse()
                    .map(|n| arm_epochs_arg = Some(n))
                    .map_err(|_| "--arm-epochs needs an integer".to_string())
            }),
            "--quick" => {
                quick = true;
                Ok(())
            }
            "--help" | "-h" => {
                print!("{}", usage());
                return ExitCode::SUCCESS;
            }
            other => Err(format!("unknown argument {other:?}\n\n{}", usage())),
        };
        if let Err(message) = result {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
        i += 1;
    }

    let suites: Vec<Suite> = match suite_arg.as_str() {
        "dfa" => vec![Suite::Dfa],
        "rl" => vec![Suite::Rl],
        "both" => vec![Suite::Dfa, Suite::Rl],
        other => {
            eprintln!("error: --suite must be dfa|rl|both, got {other:?}");
            return ExitCode::FAILURE;
        }
    };

    let epochs: Vec<usize> = match parse_list(&epochs_arg, "--epochs") {
        Ok(v) if !v.is_empty() => v,
        Ok(_) => {
            eprintln!("error: --epochs is empty");
            return ExitCode::FAILURE;
        }
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };
    if epochs.contains(&0) {
        eprintln!("error: --epochs values must be >= 1");
        return ExitCode::FAILURE;
    }
    let explicit_lrs: Option<Vec<f32>> = match lr_arg.as_deref().map(|raw| parse_list(raw, "--lr"))
    {
        Some(Ok(v)) if !v.is_empty() => Some(v),
        Some(Ok(_)) => {
            eprintln!("error: --lr is empty");
            return ExitCode::FAILURE;
        }
        Some(Err(message)) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
        None => None,
    };

    let jobs = jobs_arg.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });

    println!("{EXPERIMENT_NAME} v{PROTOCOL_VERSION} — jobs={jobs}, quick={quick}");
    let mut results = Vec::new();
    for suite in suites {
        // Build once just to read the canonical budget, so the self-check row is
        // always present even when the caller lists neither its epochs nor its lr.
        let probe = Substrate::build(suite, seeds_arg, quick);
        let lrs = explicit_lrs
            .clone()
            .unwrap_or_else(|| vec![probe.canonical.lr]);
        let mut budgets: Vec<Budget> = Vec::new();
        budgets.push(probe.canonical);
        for &e in &epochs {
            for &lr in &lrs {
                let candidate = Budget { epochs: e, lr };
                if !budgets.contains(&candidate) {
                    budgets.push(candidate);
                }
            }
        }
        drop(probe);

        println!(
            "  {} — {} budget points x {} seeds",
            suite.label(),
            budgets.len(),
            seeds_arg
                .map(|n| n.to_string())
                .unwrap_or_else(|| "config".into())
        );
        let started = std::time::Instant::now();
        let result = run_suite(suite, &budgets, seeds_arg, quick, jobs, arm_epochs_arg);
        println!(
            "  {} done in {:.1}s",
            suite.label(),
            started.elapsed().as_secs_f32()
        );
        results.push(result);
    }

    let markdown = render(&results, quick);
    match out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("error: could not create {}: {e}", parent.display());
                        return ExitCode::FAILURE;
                    }
                }
            }
            if let Err(e) = fs::write(&path, &markdown) {
                eprintln!("error: could not write {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
            println!("wrote {}", path.display());
        }
        None => print!("{markdown}"),
    }
    ExitCode::SUCCESS
}
