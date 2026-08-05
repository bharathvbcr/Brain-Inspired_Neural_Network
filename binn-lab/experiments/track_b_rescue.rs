//! Track B — Low-Cost Rescue Experiments binary.
//!
//! Preregistered evaluation of Track B low-cost rescue arms:
//! - E1.1: Continuous RPE Critic broadcast (`MatchedRlRpe`)
//! - E1.3: Online Learned Feedback alignment matrix (`MatchedRlLearnedFb`)
//! - E4.2: Surrogate-weighted eligibility trace evaluation (`SurrogateEligibility`)
//!
//! Contrasts: Matched RL-Flat (baseline ±1 broadcast), Matched REINFORCE×B_i, Matched Gradient ceiling.
//! Isolated from canonical kill-gates.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::guards::{gap_closed_clamped, gap_closed_exceeds_ceiling, Verdict};
use binn_lab::{freeze_trials, samples_to_gradient_examples, Config};
use binn_learn::{
    MatchedGradient, MatchedRlFlat, MatchedRlGraded, MatchedRlLearnedFb, MatchedRlReinforceFb,
    MatchedRlRpe, DEFAULT_MATCHED_BETA,
};

const PROTOCOL_VERSION: u64 = 131;
const TRACK_B_EXPERIMENT: &str = "track-b-rescue";

/// Preregistered accuracy floor.
const ACCURACY_FLOOR: f32 = 0.65;
/// Preregistered gap-closed LCB threshold.
const GAP_LCB_THRESHOLD: f32 = 0.5;
/// Preregistered seed requirement for a scientific verdict.
const REQUIRED_SEEDS: usize = 20;
/// Chance level of the binary matched task; the "dense" pole of gap-closed.
const DENSE_CHANCE: f32 = 0.5;
/// Minimum reference−dense separation for gap-closed to be identifiable.
///
/// Matches `Config::g2_min_reference_gap` used by the C1 runner.
const MIN_REFERENCE_GAP: f32 = 0.15;

/// Per-seed gap-closed, clamped to `[0, 1]` and gated on reference separation.
///
/// # The bug this replaces
///
/// Both rescue harnesses computed `(acc − 0.5) / (grad − 0.5).max(1e-4)` with no
/// clamp and no separation gate, which is how gap-closed values of **1.0155**
/// and **1.0244** — i.e. "closed 102% of the gap to the ceiling" — reached
/// shipped reports. `runner.rs` has always clamped to `[0, 1]`; these two did
/// not. A value above 1 means the arm beat the reference it is meant to be
/// bounded by, which is a harness warning (saturated task / undertrained
/// ceiling), not a result.
struct GapSeries {
    values: Vec<f32>,
    /// Seeds where the reference was too close to chance to identify a gap.
    unidentifiable: usize,
    /// Seeds where the arm exceeded the ceiling before clamping.
    exceeded_ceiling: usize,
}

fn gap_series(pairs: &[(f32, f32)]) -> GapSeries {
    let mut values = Vec::with_capacity(pairs.len());
    let mut unidentifiable = 0usize;
    let mut exceeded_ceiling = 0usize;
    for &(local, reference) in pairs {
        if gap_closed_exceeds_ceiling(local, DENSE_CHANCE, reference) {
            exceeded_ceiling += 1;
        }
        match gap_closed_clamped(local, DENSE_CHANCE, reference, MIN_REFERENCE_GAP) {
            Some(g) => values.push(g),
            None => unidentifiable += 1,
        }
    }
    GapSeries {
        values,
        unidentifiable,
        exceeded_ceiling,
    }
}

#[derive(Clone, Debug)]
struct TrackBResult {
    #[allow(dead_code)]
    seed: u64,
    flat_acc: f32,
    graded_acc: f32,
    reinforce_fb_acc: f32,
    rpe_acc: f32,
    learned_fb_acc: f32,
    gradient_acc: f32,
}

fn mean(vals: &[f32]) -> f32 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.iter().sum::<f32>() / vals.len() as f32
}

fn std_error(vals: &[f32]) -> f32 {
    if vals.len() <= 1 {
        return 0.0;
    }
    let m = mean(vals);
    let var = vals.iter().map(|v| (v - m).powi(2)).sum::<f32>() / (vals.len() - 1) as f32;
    (var / vals.len() as f32).sqrt()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--out" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                println!("Usage: cargo run --release -p binn-lab --bin track-b-rescue [-- --quick] [--out PATH]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let n_seeds = if quick { 5 } else { 20 };
    let n_train = if quick { 48 } else { 200 };
    let n_test = if quick { 24 } else { 100 };
    let hidden = if quick { 128 } else { 256 };
    let epochs = if quick { 60 } else { 80 };
    let master_seed = if quick {
        0x0071_AC00_001C_00F1_u64
    } else {
        0x0071_AC00_001C_00F2_u64
    };

    println!("========================================================================");
    println!("Track B — Low-Cost Rescue Experiments Protocol v{PROTOCOL_VERSION}");
    println!(
        "Schedule: {} (n_seeds={}, hidden={}, n_train={}, epochs={})",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        },
        n_seeds,
        hidden,
        n_train,
        epochs
    );
    println!("========================================================================\n");

    let mut base_cfg = Config::c1_default();
    base_cfg.n_seeds = n_seeds;
    base_cfg.n_train = n_train;
    base_cfg.n_test = n_test;
    base_cfg.n_hidden = hidden;
    base_cfg.bptt_epochs = epochs;

    let mut results = Vec::new();
    for s_idx in 0..n_seeds {
        let seed = master_seed ^ ((s_idx as u64) * 0x1000_0005_u64);
        let split = freeze_trials(&base_cfg, seed);
        let train_data = samples_to_gradient_examples(&split.train);
        let test_data = samples_to_gradient_examples(&split.test);

        let mut flat = MatchedRlFlat::new(hidden, 0.05, 0.0, DEFAULT_MATCHED_BETA, seed);
        let r_flat = flat.train_and_evaluate(epochs, &train_data, &test_data);

        let mut graded = MatchedRlGraded::new(hidden, 0.05, 0.0, DEFAULT_MATCHED_BETA, seed);
        let r_graded = graded.train_and_evaluate(epochs, &train_data, &test_data);

        let mut fb = MatchedRlReinforceFb::new(hidden, 0.05, 0.0, DEFAULT_MATCHED_BETA, seed);
        let r_fb = fb.train_and_evaluate(epochs, &train_data, &test_data);

        let mut rpe = MatchedRlRpe::new(hidden, 0.05, 0.0, 0.02, DEFAULT_MATCHED_BETA, seed);
        let r_rpe = rpe.train_and_evaluate(epochs, &train_data, &test_data);

        let mut lfb = MatchedRlLearnedFb::new(hidden, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, seed);
        let r_lfb = lfb.train_and_evaluate(epochs, &train_data, &test_data);

        let mut grad = MatchedGradient::new(hidden, 0.05, DEFAULT_MATCHED_BETA, seed);
        let r_grad = grad.train_and_evaluate(epochs, &train_data, &test_data);

        results.push(TrackBResult {
            seed,
            flat_acc: r_flat.accuracy,
            graded_acc: r_graded.accuracy,
            reinforce_fb_acc: r_fb.accuracy,
            rpe_acc: r_rpe.accuracy,
            learned_fb_acc: r_lfb.accuracy,
            gradient_acc: r_grad.accuracy,
        });

        println!("Seed {:2}/{}: Flat={:.4} | Graded={:.4} | RFB={:.4} | RPE={:.4} | LearnedFB={:.4} | Grad={:.4}",
            s_idx + 1, n_seeds,
            r_flat.accuracy, r_graded.accuracy, r_fb.accuracy,
            r_rpe.accuracy, r_lfb.accuracy, r_grad.accuracy);
    }

    let flat_accs: Vec<f32> = results.iter().map(|r| r.flat_acc).collect();
    let graded_accs: Vec<f32> = results.iter().map(|r| r.graded_acc).collect();
    let fb_accs: Vec<f32> = results.iter().map(|r| r.reinforce_fb_acc).collect();
    let rpe_accs: Vec<f32> = results.iter().map(|r| r.rpe_acc).collect();
    let lfb_accs: Vec<f32> = results.iter().map(|r| r.learned_fb_acc).collect();
    let grad_accs: Vec<f32> = results.iter().map(|r| r.gradient_acc).collect();

    let m_flat = mean(&flat_accs);
    let m_graded = mean(&graded_accs);
    let m_fb = mean(&fb_accs);
    let m_rpe = mean(&rpe_accs);
    let m_lfb = mean(&lfb_accs);
    let m_grad = mean(&grad_accs);

    let se_flat = std_error(&flat_accs);
    let se_graded = std_error(&graded_accs);
    let se_fb = std_error(&fb_accs);
    let se_rpe = std_error(&rpe_accs);
    let se_lfb = std_error(&lfb_accs);
    let se_grad = std_error(&grad_accs);

    let rpe_pairs: Vec<(f32, f32)> = results
        .iter()
        .map(|r| (r.rpe_acc, r.gradient_acc))
        .collect();
    let lfb_pairs: Vec<(f32, f32)> = results
        .iter()
        .map(|r| (r.learned_fb_acc, r.gradient_acc))
        .collect();
    let gap_rpe = gap_series(&rpe_pairs);
    let gap_lfb = gap_series(&lfb_pairs);

    let gap_rpe_m = mean(&gap_rpe.values);
    let gap_rpe_lcb = gap_rpe_m - 1.96 * std_error(&gap_rpe.values);
    let gap_lfb_m = mean(&gap_lfb.values);
    let gap_lfb_lcb = gap_lfb_m - 1.96 * std_error(&gap_lfb.values);

    // A ceiling that loses to its own treatment invalidates the comparison.
    let ceiling_inverted = gap_rpe.exceeded_ceiling > 0 || gap_lfb.exceeded_ceiling > 0;
    let harness_valid = !ceiling_inverted;

    let v_flat = Verdict::evaluate_mean(
        m_flat,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_graded = Verdict::evaluate_mean(
        m_graded,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_fb = Verdict::evaluate_mean(m_fb, ACCURACY_FLOOR, n_seeds, REQUIRED_SEEDS, harness_valid);
    let v_rpe_floor = Verdict::evaluate_mean(
        m_rpe,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_rpe_gap = Verdict::evaluate_mean(
        gap_rpe_lcb,
        GAP_LCB_THRESHOLD + f32::EPSILON,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_lfb_floor = Verdict::evaluate_mean(
        m_lfb,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_lfb_gap = Verdict::evaluate_mean(
        gap_lfb_lcb,
        GAP_LCB_THRESHOLD + f32::EPSILON,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );

    let rpe_overall = if v_rpe_floor.is_citable_as_positive() && v_rpe_gap.is_citable_as_positive()
    {
        Verdict::Pass
    } else if harness_valid {
        Verdict::Fail
    } else {
        Verdict::InvalidHarness
    };
    let lfb_overall = if v_lfb_floor.is_citable_as_positive() && v_lfb_gap.is_citable_as_positive()
    {
        Verdict::Pass
    } else if harness_valid {
        Verdict::Fail
    } else {
        Verdict::InvalidHarness
    };

    let harness_note = if ceiling_inverted {
        format!(
            "**HARNESS WARNING — ceiling inverted.** {} of {n_seeds} RPE seeds and {} of \
             {n_seeds} learned-FB seeds produced a raw gap-closed above 1.0, i.e. the arm \
             beat the gradient reference it is supposed to be bounded by. This indicates a \
             saturated task or an undertrained ceiling, not a credit-assignment result. \
             Gap-closed is clamped to [0, 1] for reporting; no PASS is permitted while this \
             warning is present.",
            gap_rpe.exceeded_ceiling, gap_lfb.exceeded_ceiling
        )
    } else {
        "Ceiling health: no seed exceeded the gradient reference; gap-closed is identifiable."
            .to_string()
    };
    let unidentifiable_note = format!(
        "Seeds excluded from gap-closed for insufficient reference separation \
         (< {MIN_REFERENCE_GAP}): RPE {} / {n_seeds}, learned-FB {} / {n_seeds}.",
        gap_rpe.unidentifiable, gap_lfb.unidentifiable
    );

    let summary = format!(
        "# Track B Rescue Experiment Report\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment ID:** {TRACK_B_EXPERIMENT} (schedule name; not a `c1-*-<hex>` config hash)  \n\
        **Schedule:** {} (n={n_seeds})  \n\
        **Substrate:** matched dense-LIF — G2-numeric thresholds only (not live Engine G2)  \n\n\
        **Gap-closed:** clamped to `[0, 1]` via `binn_lab::guards::gap_closed_clamped`, \
        identical to the C1 runner. Seeds whose reference is within \
        {MIN_REFERENCE_GAP} of chance are excluded rather than divided through.  \n\n\
        ## Accuracy Summary (Mean ± SE)\n\n\
        | Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Floor (≥{ACCURACY_FLOOR}) | Gap LCB (>{GAP_LCB_THRESHOLD}) |\n\
        |---|---:|---:|---:|---:|---|---|\n\
        | Baseline Flat (±1) | {m_flat:.4} | {se_flat:.4} | — | — | {} | — |\n\
        | Graded Broadcast | {m_graded:.4} | {se_graded:.4} | — | — | {} | — |\n\
        | Frozen REINFORCE×B_i | {m_fb:.4} | {se_fb:.4} | — | — | {} | — |\n\
        | **E1.1 Graded RPE Critic** | **{m_rpe:.4}** | {se_rpe:.4} | {gap_rpe_m:.4} | **{gap_rpe_lcb:.4}** | **{}** | **{}** |\n\
        | **E1.3 Online Learned FB** | **{m_lfb:.4}** | {se_lfb:.4} | {gap_lfb_m:.4} | **{gap_lfb_lcb:.4}** | **{}** | **{}** |\n\
        | Gradient Ceiling | {m_grad:.4} | {se_grad:.4} | 1.0000 | 1.0000 | reference | reference |\n\n\
        ## Harness health\n\n\
        {harness_note}\n\n\
        {unidentifiable_note}\n\n\
        ## Scientific Verdict\n\n\
        - E1.1 RPE Critic: **{}**\n\
        - E1.3 Online Learned FB: **{}**\n\
        - Matched dense-LIF schedule only — **not** live Engine G2.\n",
        if quick { "QUICK / PILOT" } else { "FULL SCIENTIFIC" },
        v_flat.label(),
        v_graded.label(),
        v_fb.label(),
        v_rpe_floor.label(),
        v_rpe_gap.label(),
        v_lfb_floor.label(),
        v_lfb_gap.label(),
        rpe_overall.label(),
        lfb_overall.label(),
    );

    println!("\n{summary}");

    if let Some(path) = out {
        if let Err(e) = fs::write(&path, &summary) {
            eprintln!("Failed to write output report to {}: {e}", path.display());
            return ExitCode::from(1);
        }
        println!("Report saved to: {}", path.display());
    }

    ExitCode::SUCCESS
}
