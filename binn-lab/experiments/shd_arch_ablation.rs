//! SHD architecture ablation: is 0.234 a locality limit or an architecture limit?
//!
//! # The question
//!
//! `c1-shd-cal-*` reports DFA ≈ 0.234 on SHD (20 classes, chance 0.05) from a
//! **feed-forward, fixed-threshold** LIF with no `W_rec`. Published local rules
//! on the same dataset reach far higher:
//!
//! | Method | Locality | SHD accuracy |
//! |---|---|---|
//! | BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
//! | e-prop (local in time, non-local in space) | partial | 0.808 |
//! | ETLP (fully local three-factor, hardware-targeted) | full | 0.746 |
//! | this project, feed-forward fixed-θ DFA | full | 0.234 |
//!
//! ETLP's own conclusion is that *"threshold adaptation in spiking neurons and a
//! recurrent topology are necessary to learn spatio-temporal patterns with a
//! rich temporal structure"*. Both are absent from the current architecture, so
//! the 0.234 figure is confounded.
//!
//! Preregistration: `results/PREREG_2026-07-25_SHD_ARCH_ABLATION.md`.
//!
//! # Robustness properties of this harness
//!
//! * **Cells run in H1-critical order.** `ff+fixed` and `rec+alif` (the only two
//!   the preregistered H1 contrast needs) run first, so a timeout still answers
//!   the question.
//! * **The report is rewritten after every cell.** A wall-clock kill leaves a
//!   valid partial report rather than nothing.
//! * **Fixture data is fatal outside `--quick`.** A full run that silently fell
//!   back to the smoke fixture would look like a real result.
//! * **Every cell is checked for collapse, silence and saturation.** A recurrent
//!   net that dies or runs away scores at chance, which would otherwise be
//!   misread as "recurrence does not help" — inverting the conclusion.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use binn_data::{default_shd_dir, load_fixture, load_shd_split_capped, ShdSample, SHD_CHANCE};
use binn_lab::guards::{wilson_interval, Verdict, Z_95};
use binn_learn::{
    shuffle_labels, AlifEval, ModulatorScale, ShdAlifArm, ShdAlifConfig, ShdAlifRule, ShdExample,
    MODULATOR_PARITY_TOLERANCE,
};

const PROTOCOL_VERSION: u64 = 142;
const EXPERIMENT_NAME: &str = "shd-arch-ablation";

/// Preregistered: minimum absolute gain of `rec+alif` over `ff+fixed` for H1.
const H1_MIN_ARCH_GAIN: f32 = 0.10;
/// Preregistered: DFA accuracy required for H2.
const H2_MIN_ACCURACY: f32 = 0.50;
/// Preregistered: shuffled-label control must stay below `chance + this`.
const CONTROL_MAX_EXCESS: f32 = 0.05;

/// Published reference points, for the report's context table.
const REF_BPTT_DELAYS: f32 = 0.951;
const REF_EPROP: f32 = 0.808;
const REF_ETLP: f32 = 0.746;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Arch {
    recurrent: bool,
    adaptive: bool,
}

impl Arch {
    const fn label(self) -> &'static str {
        match (self.recurrent, self.adaptive) {
            (false, false) => "ff+fixed",
            (false, true) => "ff+alif",
            (true, false) => "rec+fixed",
            (true, true) => "rec+alif",
        }
    }
}

/// The two architectures the preregistered H1 contrast needs, first.
const ARCH_ORDER: [Arch; 4] = [
    Arch {
        recurrent: false,
        adaptive: false,
    }, // H1 baseline
    Arch {
        recurrent: true,
        adaptive: true,
    }, // H1 treatment
    Arch {
        recurrent: true,
        adaptive: false,
    }, // interaction
    Arch {
        recurrent: false,
        adaptive: true,
    }, // interaction
];

#[derive(Clone, Debug)]
struct Cell {
    arch: Arch,
    rule: ShdAlifRule,
    lr: f32,
    accs: Vec<f32>,
    evals: Vec<AlifEval>,
    modulator: ModulatorScale,
    wall_secs: f64,
}

impl Cell {
    fn mean_acc(&self) -> f32 {
        mean(&self.accs)
    }

    fn mean_activity(&self) -> f32 {
        mean(
            &self
                .evals
                .iter()
                .map(|e| e.mean_activity)
                .collect::<Vec<_>>(),
        )
    }

    /// Union of degeneracy defects across seeds.
    fn defects(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = Vec::new();
        for e in &self.evals {
            for d in e.defects() {
                if !v.contains(&d) {
                    v.push(d);
                }
            }
        }
        v
    }

    fn is_degenerate(&self) -> bool {
        !self.defects().is_empty()
    }

    fn defect_text(&self) -> String {
        let d = self.defects();
        if d.is_empty() {
            "ok".to_string()
        } else {
            d.join("; ")
        }
    }
}

fn mean(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f32>() / v.len() as f32
}

fn std_error(v: &[f32]) -> f32 {
    if v.len() <= 1 {
        return 0.0;
    }
    let m = mean(v);
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / (v.len() - 1) as f32;
    (var / v.len() as f32).sqrt()
}

fn seed_ci(v: &[f32]) -> (f32, f32) {
    let m = mean(v);
    let se = std_error(v);
    ((m - Z_95 * se).max(0.0), (m + Z_95 * se).min(1.0))
}

fn to_example(s: &ShdSample) -> ShdExample {
    ShdExample {
        frames: s.frames.clone(),
        t: s.t,
        n_in: s.n_in,
        label: s.label,
    }
}

/// Scalar split metadata, so the heavyweight `ShdSplit` can be freed after
/// conversion. At T=100 × n_in=700 each sample is 280 KB, so holding both the
/// source split and the converted copy costs ~1.4 GB at the 2000/500 caps.
#[derive(Clone, Copy, Debug)]
struct SplitMeta {
    n_in: usize,
    t: usize,
    n_classes: usize,
    fixture: bool,
    n_train: usize,
    n_test: usize,
}

fn main() -> ExitCode {
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::LocalLearning) {
        eprintln!("shd-arch-ablation: {error}");
        return ExitCode::from(3);
    }
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut full = false;
    let mut lr_sweep = false;
    let mut out: Option<PathBuf> = None;
    let mut hidden = 128usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--full" => full = true,
            "--lr-sweep" => lr_sweep = true,
            "--hidden" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(h) if h >= 1 => hidden = h,
                    _ => {
                        eprintln!("--hidden requires a positive integer");
                        return ExitCode::from(2);
                    }
                }
            }
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
                    "Usage: cargo run --release -p binn-lab --bin shd-arch-ablation -- \\\n\
                     \x20 [--quick] [--full] [--lr-sweep] [--hidden N] [--out PATH]\n\n\
                     --quick     fixture-scale smoke (minutes); fixture data allowed\n\
                     --full      official uncapped splits (8156/2264)\n\
                     --lr-sweep  DFA only, 3 learning rates, reduced schedule — a cheap\n\
                     \x20           pilot that de-risks the inherited lr=0.02 confound\n\
                     (default)   capped 2000/500, both rules, one learning rate"
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

    let base_lr = 0.005f32;
    let (n_seeds, epochs, lrs, rules): (usize, usize, Vec<f32>, Vec<ShdAlifRule>) = if lr_sweep {
        (
            2,
            8,
            vec![base_lr / 4.0, base_lr, base_lr * 4.0],
            vec![ShdAlifRule::Dfa],
        )
    } else if quick {
        (
            2,
            3,
            vec![base_lr],
            vec![ShdAlifRule::Dfa, ShdAlifRule::EpropCeiling],
        )
    } else {
        (
            3,
            15,
            vec![base_lr],
            vec![ShdAlifRule::Dfa, ShdAlifRule::EpropCeiling],
        )
    };

    let (max_train, max_test) = if full {
        (None, None)
    } else if quick {
        (Some(200), Some(100))
    } else {
        (Some(2000), Some(500))
    };

    println!("========================================================================");
    println!("SHD Architecture Ablation — protocol v{PROTOCOL_VERSION}");
    println!(
        "mode={} hidden={hidden} seeds={n_seeds} epochs={epochs} lrs={lrs:?} caps={max_train:?}/{max_test:?}",
        if lr_sweep {
            "LR-SWEEP PILOT"
        } else if quick {
            "QUICK"
        } else if full {
            "FULL SPLITS"
        } else {
            "CAPPED SCIENTIFIC"
        }
    );
    println!("========================================================================\n");

    // ---- Data ----
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

    // A full run on fixture data would look exactly like a real result. Refuse.
    if split.fixture && !quick {
        eprintln!(
            "\nFATAL: loaded the smoke FIXTURE, not real SHD.\n\
             A scientific run on fixture data is indistinguishable from a real one in the \n\
             report, so this is a hard error rather than a warning.\n\n\
             Fix: convert the official corpus, then re-run.\n\
             \x20 PKG_CONFIG_PATH=\"$(brew --prefix hdf5)/lib/pkgconfig:${{PKG_CONFIG_PATH:-}}\" \\\n\
             \x20   cargo run --locked --release -p binn-data --features shd-convert \\\n\
             \x20     --bin convert-shd -- --cache-dir data/shd\n\n\
             Or set BINN_SHD_DIR to an existing cache. Use --quick to allow the fixture."
        );
        return ExitCode::from(3);
    }

    let mut split = split;
    let train: Vec<ShdExample> = split.train.iter().map(to_example).collect();
    let test: Vec<ShdExample> = split.test.iter().map(to_example).collect();
    if train.is_empty() || test.is_empty() {
        eprintln!("empty SHD split");
        return ExitCode::from(1);
    }

    let meta = SplitMeta {
        n_in: split.n_in,
        t: split.t,
        n_classes: split.n_classes,
        fixture: split.fixture,
        n_train: train.len(),
        n_test: test.len(),
    };
    // Free the source split: the converted copy is all that is needed from here.
    split.train = Vec::new();
    split.test = Vec::new();
    drop(split);

    println!(
        "loaded: n_train={} n_test={} n_in={} T={} classes={} fixture={}\n",
        meta.n_train, meta.n_test, meta.n_in, meta.t, meta.n_classes, meta.fixture
    );

    let chance = if meta.n_classes == 20 {
        SHD_CHANCE
    } else {
        1.0 / meta.n_classes as f32
    };

    let base_cfg = ShdAlifConfig {
        hidden,
        n_classes: meta.n_classes,
        lr: base_lr,
        beta: 5.0,
        epochs,
        recurrent: false,
        adaptive: false,
        tau_a: binn_learn::DEFAULT_TAU_A,
        beta_a: binn_learn::DEFAULT_BETA_A,
    };

    // ---- Grid, in H1-critical order ----
    let mut cells: Vec<Cell> = Vec::new();
    let total = ARCH_ORDER.len() * rules.len() * lrs.len();
    let mut done = 0usize;

    for &rule in &rules {
        for arch in ARCH_ORDER {
            for &lr in &lrs {
                let mut cfg = base_cfg
                    .with_recurrent(arch.recurrent)
                    .with_adaptive(arch.adaptive);
                cfg.lr = lr;

                let mut cell = Cell {
                    arch,
                    rule,
                    lr,
                    accs: Vec::with_capacity(n_seeds),
                    evals: Vec::with_capacity(n_seeds),
                    modulator: ModulatorScale::new(),
                    wall_secs: 0.0,
                };
                let t0 = Instant::now();
                for s in 0..n_seeds {
                    let seed =
                        0x00A1_1F00_0000_0001u64 ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                    let mut arm = ShdAlifArm::new(&train[0], &cfg, rule, seed);
                    let e = arm.train_and_evaluate_detailed(epochs, &train, &test);
                    println!(
                        "  {:>9} lr={lr:<6.4} {:<24} seed {}/{}: acc={:.4} act={:.4} classes={} [{}]",
                        arch.label(),
                        rule.label(),
                        s + 1,
                        n_seeds,
                        e.accuracy,
                        e.mean_activity,
                        e.n_distinct_predicted,
                        if e.is_degenerate() { "DEGENERATE" } else { "ok" }
                    );
                    cell.accs.push(e.accuracy);
                    cell.evals.push(e);
                    cell.modulator.merge(&arm.modulator_scale());
                }
                cell.wall_secs = t0.elapsed().as_secs_f64();
                done += 1;
                println!(
                    "  {:>9} lr={lr:<6.4} {:<24} MEAN={:.4} ({:.1}s) [{}/{}]\n",
                    arch.label(),
                    rule.label(),
                    cell.mean_acc(),
                    cell.wall_secs,
                    done,
                    total
                );
                cells.push(cell);

                // Rewrite the report after every cell: a timeout must not
                // destroy completed work.
                if let Some(p) = &out {
                    let partial = render(
                        &cells,
                        &[],
                        meta,
                        hidden,
                        n_seeds,
                        epochs,
                        &lrs,
                        chance,
                        lr_sweep,
                        quick,
                        full,
                        true,
                    );
                    let _ = fs::write(p, &partial);
                }
            }
        }
    }

    // ---- Shuffled-label negative control on the best DFA cell ----
    let best_dfa = cells
        .iter()
        .filter(|c| c.rule == ShdAlifRule::Dfa && !c.is_degenerate())
        .max_by(|a, b| {
            a.mean_acc()
                .partial_cmp(&b.mean_acc())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();

    let mut control_accs: Vec<f32> = Vec::new();
    if let Some(bc) = &best_dfa {
        let mut cfg = base_cfg
            .with_recurrent(bc.arch.recurrent)
            .with_adaptive(bc.arch.adaptive);
        cfg.lr = bc.lr;
        println!(
            "Shuffled-label control on {} (lr={}) ...",
            bc.arch.label(),
            bc.lr
        );
        for s in 0..n_seeds {
            let seed = 0x00C0_4750_0000_0001u64 ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let shuffled = shuffle_labels(&train, seed);
            let mut arm = ShdAlifArm::new(&train[0], &cfg, ShdAlifRule::Dfa, seed);
            let e = arm.train_and_evaluate_detailed(epochs, &shuffled, &test);
            control_accs.push(e.accuracy);
            println!(
                "  control seed {}/{}: acc={:.4}",
                s + 1,
                n_seeds,
                e.accuracy
            );
        }
    } else {
        eprintln!("WARNING: every DFA cell was degenerate; skipping the control.");
    }

    let report = render(
        &cells,
        &control_accs,
        meta,
        hidden,
        n_seeds,
        epochs,
        &lrs,
        chance,
        lr_sweep,
        quick,
        full,
        false,
    );
    println!("\n{report}");

    if let Some(path) = &out {
        if let Err(e) = fs::write(path, &report) {
            eprintln!("Failed to write report to {}: {e}", path.display());
            return ExitCode::from(1);
        }
        println!("Report saved to: {}", path.display());
    }

    ExitCode::SUCCESS
}

/// Best non-degenerate cell for an architecture under a rule, across learning rates.
fn best_cell(cells: &[Cell], arch: Arch, rule: ShdAlifRule) -> Option<&Cell> {
    cells
        .iter()
        .filter(|c| c.arch == arch && c.rule == rule && !c.is_degenerate())
        .max_by(|a, b| {
            a.mean_acc()
                .partial_cmp(&b.mean_acc())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[allow(clippy::too_many_arguments)]
fn render(
    cells: &[Cell],
    control_accs: &[f32],
    meta: SplitMeta,
    hidden: usize,
    n_seeds: usize,
    epochs: usize,
    lrs: &[f32],
    chance: f32,
    lr_sweep: bool,
    quick: bool,
    full: bool,
    partial: bool,
) -> String {
    // ---- Grid table ----
    let mut rows = String::new();
    for c in cells {
        let (lo, hi) = seed_ci(&c.accs);
        rows.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | [{lo:.4}, {hi:.4}] | {:.4} | {:.3e} | {:.1} | {} |\n",
            c.arch.label(),
            c.rule.label(),
            c.lr,
            c.mean_acc(),
            std_error(&c.accs),
            c.mean_activity(),
            c.modulator.rms(),
            c.wall_secs,
            c.defect_text(),
        ));
    }

    // ---- H1 / H2 ----
    let baseline_cell = best_cell(cells, ARCH_ORDER[0], ShdAlifRule::Dfa);
    let treat_cell = best_cell(cells, ARCH_ORDER[1], ShdAlifRule::Dfa);
    let baseline = baseline_cell.map(|c| c.mean_acc()).unwrap_or(f32::NAN);
    let best_arch = treat_cell.map(|c| c.mean_acc()).unwrap_or(f32::NAN);
    let arch_gain = best_arch - baseline;
    let ci_base = baseline_cell
        .map(|c| seed_ci(&c.accs))
        .unwrap_or((f32::NAN, f32::NAN));
    let ci_best = treat_cell
        .map(|c| seed_ci(&c.accs))
        .unwrap_or((f32::NAN, f32::NAN));
    let cis_disjoint = ci_best.0 > ci_base.1;

    // ---- Validity gates ----
    let control_mean = mean(control_accs);
    let control_ok = control_accs.is_empty() || control_mean <= chance + CONTROL_MAX_EXCESS;

    let mut worst_ratio = 0.0f32;
    for arch in ARCH_ORDER {
        if let (Some(d), Some(e)) = (
            best_cell(cells, arch, ShdAlifRule::Dfa),
            best_cell(cells, arch, ShdAlifRule::EpropCeiling),
        ) {
            let r = ModulatorScale::ratio(&d.modulator, &e.modulator);
            if r.is_finite() && r > worst_ratio {
                worst_ratio = r;
            }
        }
    }
    let parity_ok = worst_ratio == 0.0 || worst_ratio <= MODULATOR_PARITY_TOLERANCE;

    let h1_cells_present = baseline_cell.is_some() && treat_cell.is_some();
    let degenerate_cells: Vec<&Cell> = cells.iter().filter(|c| c.is_degenerate()).collect();
    let h1_cells_healthy = h1_cells_present;

    let nonconfirmatory = partial || quick || lr_sweep;
    let harness_valid = control_ok && parity_ok && h1_cells_healthy && !nonconfirmatory;

    let h1 = if nonconfirmatory {
        Verdict::Underpowered
    } else if !harness_valid {
        Verdict::InvalidHarness
    } else if arch_gain >= H1_MIN_ARCH_GAIN && cis_disjoint {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    let h2 = if nonconfirmatory {
        Verdict::Underpowered
    } else if !harness_valid {
        Verdict::InvalidHarness
    } else if best_arch >= H2_MIN_ACCURACY {
        Verdict::Pass
    } else {
        Verdict::Fail
    };

    // ---- Ceiling health ----
    let mut inversions = String::new();
    for arch in ARCH_ORDER {
        if let (Some(d), Some(e)) = (
            best_cell(cells, arch, ShdAlifRule::Dfa),
            best_cell(cells, arch, ShdAlifRule::EpropCeiling),
        ) {
            let (md, me) = (d.mean_acc(), e.mean_acc());
            let ratio = ModulatorScale::ratio(&d.modulator, &e.modulator);
            inversions.push_str(&format!(
                "| {} | {md:.4} | {me:.4} | {ratio:.2} | {} |\n",
                arch.label(),
                if me + 1e-6 < md {
                    "INVERTED — ceiling below treatment"
                } else {
                    "ok"
                },
            ));
        }
    }
    if inversions.is_empty() {
        inversions.push_str("| — | — | — | — | no comparable pairs yet |\n");
    }

    let control_line = if control_accs.is_empty() {
        "not run".to_string()
    } else {
        let (lo, hi) = wilson_interval(
            (control_mean * meta.n_test as f32).round() as usize,
            meta.n_test,
            Z_95,
        );
        format!(
            "{control_mean:.4} (95% CI [{lo:.4}, {hi:.4}]); chance {chance:.4}; \
             threshold {:.4}; **{}**",
            chance + CONTROL_MAX_EXCESS,
            if control_ok { "ok" } else { "LEAK DETECTED" }
        )
    };

    let degeneracy_line = if degenerate_cells.is_empty() {
        "No cell was degenerate.".to_string()
    } else {
        let names: Vec<String> = degenerate_cells
            .iter()
            .map(|c| {
                format!(
                    "`{} / {} / lr={}` ({})",
                    c.arch.label(),
                    c.rule.label(),
                    c.lr,
                    c.defect_text()
                )
            })
            .collect();
        format!(
            "**{} of {} cells degenerate** — their accuracies are NOT interpretable as \
             statements about the credit rule: {}",
            degenerate_cells.len(),
            cells.len(),
            names.join(", ")
        )
    };

    let mode = if lr_sweep {
        "LR-SWEEP PILOT (DFA only, reduced schedule — a pilot, not a confirmatory run)"
    } else if quick {
        "QUICK smoke"
    } else if full {
        "FULL official splits"
    } else {
        "CAPPED scientific (2000/500)"
    };

    let partial_banner = if partial {
        "> **PARTIAL REPORT.** Written mid-run so a timeout cannot destroy completed work. \
         Verdicts are held at `UNDERPOWERED` until the run finishes.\n\n"
    } else {
        ""
    };

    format!(
        "# SHD Architecture Ablation (C1-SHD-ARCH)\n\n\
        {partial_banner}\
        **Protocol version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION_V142.md`  \n\
        **Mode:** {mode}  \n\
        **Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a \
        feed-forward fixed-threshold forward model?  \n\
        **Schedule:** hidden={hidden}, seeds={n_seeds}, epochs={epochs}, lrs={lrs:?}  \n\
        **Data:** n_train={}, n_test={}, n_in={}, T={}, classes={}, chance={chance:.4}, \
        fixture={}  \n\n\
        ## Ablation grid\n\n\
        Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run \
        still answers the preregistered contrast. Where several learning rates were run, \
        H1/H2 use the **best non-degenerate cell per architecture**.\n\n\
        | Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |\n\
        |---|---|---:|---:|---:|---|---:|---:|---:|---|\n\
        {rows}\n\
        ### Degeneracy\n\n\
        {degeneracy_line}\n\n\
        A collapsed, silent or saturated arm scores near chance. Read as a bare number that \
        is indistinguishable from \"this credit rule does not work\", which would invert the \
        conclusion — hence the explicit health column.\n\n\
        ## Ceiling health\n\n\
        | Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |\n\
        |---|---:|---:|---:|---|\n\
        {inversions}\n\
        Parity tolerance {MODULATOR_PARITY_TOLERANCE:.2}; worst observed {worst_ratio:.2}.\n\n\
        ## Negative control (shuffled labels)\n\n\
        {control_line}\n\n\
        ## Preregistered hypotheses\n\n\
        | ID | Statement | Measured | Verdict |\n\
        |---|---|---|---|\n\
        | H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ {H1_MIN_ARCH_GAIN:.2} with disjoint 95% CIs | gain {arch_gain:+.4} ({baseline:.4} → {best_arch:.4}), CIs disjoint = {cis_disjoint} | {} |\n\
        | H2 | Best-architecture DFA reaches ≥ {H2_MIN_ACCURACY:.2} | {best_arch:.4} | {} |\n\n\
        Validity gates: control {}, modulator parity {}, H1 cells present {}.\n\n\
        ## Published reference points (same dataset, not run here)\n\n\
        | Method | Locality | SHD accuracy |\n\
        |---|---|---:|\n\
        | BPTT + learned delays (Hammouamri et al. 2023) | none | {REF_BPTT_DELAYS:.3} |\n\
        | e-prop | local in time, non-local in space | {REF_EPROP:.3} |\n\
        | ETLP (Quintana et al. 2024) | fully local three-factor | {REF_ETLP:.3} |\n\
        | this run, best DFA architecture | fully local three-factor | {best_arch:.3} |\n\n\
        ## Interpretation\n\n\
        {}\n\n\
        ## Non-claims\n\n\
        - **Not SOTA** and not a like-for-like ETLP comparison: different eligibility \
        formulation, different schedule, capped splits unless `--full`.\n\
        - **Not Gate G2.**\n\
        - The surrogate ALIF eligibility includes both `ε_v` and the adaptation \
        cross-term `ε_a`; fixed-threshold cells use its `β_a = 0` limit.\n\
        - An `--lr-sweep` run is a **pilot**. It may inform which learning rate a \
        confirmatory run uses; it may not itself be reported as the confirmatory result.\n\
        - `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.\n",
        meta.n_train,
        meta.n_test,
        meta.n_in,
        meta.t,
        meta.n_classes,
        meta.fixture,
        h1.label(),
        h2.label(),
        if control_ok { "ok" } else { "FAILED" },
        if parity_ok { "ok" } else { "FAILED" },
        if h1_cells_present { "ok" } else { "MISSING" },
        interpretation(
            h1,
            h2,
            arch_gain,
            best_arch,
            baseline,
            nonconfirmatory,
        ),
    )
}

fn interpretation(
    h1: Verdict,
    h2: Verdict,
    gain: f32,
    best: f32,
    baseline: f32,
    nonconfirmatory: bool,
) -> String {
    if nonconfirmatory {
        return "This is a partial, quick, or learning-rate-pilot schedule. It may verify \
                execution and validity guards, but no H1/H2 scientific verdict is meaningful."
            .to_string();
    }
    match (h1, h2) {
        (Verdict::InvalidHarness, _) | (_, Verdict::InvalidHarness) => {
            "**INVALID_HARNESS.** A validity gate failed — the shuffled-label control, the \
             modulator-parity check, or one of the two H1 cells being degenerate. No \
             architecture or locality conclusion may be drawn. Fix the flagged gate and \
             re-run before reading any number above."
                .to_string()
        }
        (Verdict::Pass, Verdict::Pass) => format!(
            "**Architecture was the binding constraint.** Adding recurrence and threshold \
             adaptation moved DFA from {baseline:.4} to {best:.4} ({gain:+.4}). The prior \
             `c1-shd-cal-*` figure characterised a feed-forward rate readout, not local \
             credit assignment. Obligations: restate the SHD claim axis; mark the h128/256/512 \
             reports superseded on architecture grounds as well as ceiling grounds; re-run \
             the width and depth sweeps on `rec+alif` before any scaling claim."
        ),
        (Verdict::Pass, _) => format!(
            "**Architecture matters, but does not close the gap.** DFA improved from \
             {baseline:.4} to {best:.4} ({gain:+.4}) — real, but short of \
             {H2_MIN_ACCURACY:.2} and far from ETLP's {REF_ETLP:.3}. Next, in order: \
             (1) implement the exact ALIF `ε_a` eligibility term, which currently biases \
             against this arm; (2) per-arm learning-rate sweep at the winning architecture; \
             (3) only then attribute the residual to locality."
        ),
        _ => format!(
            "**Architecture is NOT the binding constraint under protocol v142.** Recurrence and adaptation moved \
             DFA by only {gain:+.4} ({baseline:.4} → {best:.4}). The feed-forward confound is \
             ruled out and the limit lies in the credit pathway or the eligibility \
             formulation. The ALIF adaptation cross-term is included. This is a legitimate \
             negative result only at the preregistered learning rate; use the separately \
             declared pilot to decide whether a fresh held-out confirmation is warranted."
        ),
    }
}

#[cfg(test)]
mod report_tests {
    use super::*;

    fn meta() -> SplitMeta {
        SplitMeta {
            n_in: 16,
            t: 8,
            n_classes: 4,
            fixture: true,
            n_train: 20,
            n_test: 10,
        }
    }

    #[test]
    fn quick_and_lr_pilot_reports_are_underpowered() {
        for (lr_sweep, quick) in [(false, true), (true, false)] {
            let report = render(
                &[],
                &[],
                meta(),
                16,
                2,
                1,
                &[0.005],
                0.25,
                lr_sweep,
                quick,
                false,
                false,
            );
            assert!(report.contains("| UNDERPOWERED |"));
            assert!(!report.contains("legitimate negative result"));
        }
    }
}
