//! Matched online-FB schedule contrast binary (protocol v131).
//!
//! **Honesty:** Despite the historical binary / experiment name
//! `live-transfer-rescue`, this harness trains **matched-only** arms
//! (`MatchedRlFlat`, `MatchedRlReinforceFb`, `MatchedRlLearnedFb`,
//! `MatchedGradient`). It does **not** run the event-driven Engine,
//! muted-θ integrate, or live k-WTA. It is **not** a live-transfer PASS
//! and must **not** be cited as clearing Gate G2 on the live substrate.
//!
//! Live k-WTA transfer remains the v13–v24 package (all FAIL).
//! This schedule only reconfirms matched online learned-`B_i` vs flat /
//! frozen RFB / SuperSpike ceiling under the dense-LIF matched forward.
//!
//! Arms:
//! - Matched RL Flat (±1 broadcast baseline)
//! - Matched Frozen RFB (v12 frozen B_i)
//! - Matched Online Learned FB (v130-style online B_i alignment)
//! - Matched Gradient ceiling (SuperSpike BPTT)

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::guards::{gap_closed_clamped, gap_closed_exceeds_ceiling, CeilingHealth, Verdict};
use binn_lab::{freeze_trials, samples_to_gradient_examples, Config};
use binn_learn::{
    MatchedGradient, MatchedRlFlat, MatchedRlLearnedFb, MatchedRlReinforceFb, DEFAULT_MATCHED_BETA,
};

const PROTOCOL_VERSION: u64 = 132;
const EXPERIMENT_NAME: &str = "live-transfer-rescue";

/// Preregistered accuracy floor.
const ACCURACY_FLOOR: f32 = 0.65;
/// Preregistered gap-closed LCB threshold.
const GAP_LCB_THRESHOLD: f32 = 0.5;
/// Preregistered seed requirement for a scientific verdict.
const REQUIRED_SEEDS: usize = 20;
/// Chance level of the binary matched task; the "dense" pole of gap-closed.
const DENSE_CHANCE: f32 = 0.5;
/// Minimum reference−dense separation for gap-closed to be identifiable.
const MIN_REFERENCE_GAP: f32 = 0.15;

#[derive(Clone, Debug)]
struct TransferSeedResult {
    #[allow(dead_code)]
    seed: u64,
    flat_acc: f32,
    frozen_rfb_acc: f32,
    learned_rfb_acc: f32,
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
                println!("Usage: cargo run --release -p binn-lab --bin live-transfer-rescue [-- --quick] [--out PATH]");
                println!("Note: matched-only schedule (not live Engine / k-WTA).");
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
        0x0071_AC00_001C_00F3_u64
    } else {
        0x0071_AC00_001C_00F4_u64
    };

    println!("========================================================================");
    println!("Matched Online-FB Schedule Contrast Protocol v{PROTOCOL_VERSION}");
    println!("Experiment ID: {EXPERIMENT_NAME} (name is historical — matched-only)");
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
    println!("Substrate: matched dense-LIF ONLY — not live Engine / muted-θ / k-WTA");
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

        let mut frozen = MatchedRlReinforceFb::new(hidden, 0.05, 0.0, DEFAULT_MATCHED_BETA, seed);
        let r_frozen = frozen.train_and_evaluate(epochs, &train_data, &test_data);

        let mut learned =
            MatchedRlLearnedFb::new(hidden, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, seed);
        let r_learned = learned.train_and_evaluate(epochs, &train_data, &test_data);

        let mut grad = MatchedGradient::new(hidden, 0.05, DEFAULT_MATCHED_BETA, seed);
        let r_grad = grad.train_and_evaluate(epochs, &train_data, &test_data);

        results.push(TransferSeedResult {
            seed,
            flat_acc: r_flat.accuracy,
            frozen_rfb_acc: r_frozen.accuracy,
            learned_rfb_acc: r_learned.accuracy,
            gradient_acc: r_grad.accuracy,
        });

        println!(
            "Seed {:2}/{}: Flat={:.4} | FrozenRFB={:.4} | LearnedRFB={:.4} | Grad={:.4}",
            s_idx + 1,
            n_seeds,
            r_flat.accuracy,
            r_frozen.accuracy,
            r_learned.accuracy,
            r_grad.accuracy
        );
    }

    let flat_accs: Vec<f32> = results.iter().map(|r| r.flat_acc).collect();
    let frozen_accs: Vec<f32> = results.iter().map(|r| r.frozen_rfb_acc).collect();
    let learned_accs: Vec<f32> = results.iter().map(|r| r.learned_rfb_acc).collect();
    let grad_accs: Vec<f32> = results.iter().map(|r| r.gradient_acc).collect();

    let m_flat = mean(&flat_accs);
    let m_frozen = mean(&frozen_accs);
    let m_learned = mean(&learned_accs);
    let m_grad = mean(&grad_accs);

    let se_flat = std_error(&flat_accs);
    let se_frozen = std_error(&frozen_accs);
    let se_learned = std_error(&learned_accs);
    let se_grad = std_error(&grad_accs);

    // Gap-closed is clamped to [0, 1] and gated on reference separation, matching
    // the C1 runner. The unclamped `(acc - 0.5) / (grad - 0.5).max(1e-4)` this
    // replaces is how a gap-closed of 1.0244 — "closed 102% of the gap to the
    // ceiling" — reached a shipped report.
    let mut gap_learned: Vec<f32> = Vec::with_capacity(results.len());
    let mut gap_unidentifiable = 0usize;
    let mut gap_exceeded_ceiling = 0usize;
    for r in &results {
        if gap_closed_exceeds_ceiling(r.learned_rfb_acc, DENSE_CHANCE, r.gradient_acc) {
            gap_exceeded_ceiling += 1;
        }
        match gap_closed_clamped(
            r.learned_rfb_acc,
            DENSE_CHANCE,
            r.gradient_acc,
            MIN_REFERENCE_GAP,
        ) {
            Some(g) => gap_learned.push(g),
            None => gap_unidentifiable += 1,
        }
    }
    let gap_learned_m = mean(&gap_learned);
    let gap_learned_lcb = gap_learned_m - 1.96 * std_error(&gap_learned);

    // A treatment that beats its own ceiling means the ceiling is not a ceiling.
    // 2026-08-21: `gap_exceeded_ceiling == 0` is blind to a reference that never
    // learned — no seed can exceed a ceiling the arm is also below. The reference
    // is now tested against chance before it is allowed to bound anything.
    let ceiling_health = CeilingHealth::evaluate(m_grad, m_learned, DENSE_CHANCE);
    let harness_valid = gap_exceeded_ceiling == 0 && ceiling_health.is_usable();

    let v_flat = Verdict::evaluate_mean(
        m_flat,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_frozen = Verdict::evaluate_mean(
        m_frozen,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_learned_floor = Verdict::evaluate_mean(
        m_learned,
        ACCURACY_FLOOR,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let v_learned_gap = Verdict::evaluate_mean(
        gap_learned_lcb,
        GAP_LCB_THRESHOLD + f32::EPSILON,
        n_seeds,
        REQUIRED_SEEDS,
        harness_valid,
    );
    let learned_overall =
        if v_learned_floor.is_citable_as_positive() && v_learned_gap.is_citable_as_positive() {
            Verdict::Pass
        } else if harness_valid {
            Verdict::Fail
        } else {
            Verdict::InvalidHarness
        };

    let matched_threshold_note = match learned_overall {
        Verdict::Pass => {
            "MATCHED thresholds cleared (dense-LIF schedule only — NOT live G2 / NOT live transfer)"
        }
        Verdict::InvalidHarness => {
            "INVALID_HARNESS — the arm exceeded its own gradient ceiling; no PASS/FAIL claim permitted"
        }
        _ => "Did not clear matched accuracy/gap thresholds",
    };

    let harness_note = if harness_valid {
        format!(
            "Ceiling health: no seed exceeded the gradient reference. \
             Seeds excluded from gap-closed for insufficient reference separation \
             (< {MIN_REFERENCE_GAP}): {gap_unidentifiable} / {n_seeds}."
        )
    } else {
        format!(
            "**HARNESS WARNING — ceiling inverted.** {gap_exceeded_ceiling} of {n_seeds} seeds \
             produced a raw gap-closed above 1.0, i.e. the learned-FB arm beat the gradient \
             reference it is supposed to be bounded by. On this matched task \
             (`CoincidenceTask`, N_IN = 2, difficulty 0.05) that indicates task saturation \
             rather than a credit-assignment result: an arm scoring 1.0000 ± 0.0000 across \
             every seed while the BPTT reference scores below it means the task can no longer \
             separate the arms. Gap-closed is clamped to [0, 1]; no PASS is permitted while \
             this warning is present. Seeds excluded for insufficient reference separation \
             (< {MIN_REFERENCE_GAP}): {gap_unidentifiable} / {n_seeds}."
        )
    };

    let summary = format!(
        "# Matched Online-FB Schedule Contrast Report\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment ID:** {EXPERIMENT_NAME} (historical name; **matched-only**)  \n\
        **Schedule:** {} (n={n_seeds})  \n\
        **Substrate:** matched dense-LIF — **not** live Engine / muted-θ / k-WTA  \n\
        **claim_axis:** exploratory matched schedule (not MUST live-transfer)  \n\
        **must_not_claim:** live k-WTA PASS; Gate G2 cleared on live C1; breaks transfer barrier  \n\n\
        **Gap-closed:** clamped to `[0, 1]` via `binn_lab::guards::gap_closed_clamped`, \
        identical to the C1 runner.  \n\n\
        ## Matched Accuracy Summary (Mean ± SE)\n\n\
        | Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Floor (≥{ACCURACY_FLOOR}) | Verdict |\n\
        |---|---:|---:|---:|---:|---|---|\n\
        | Baseline Flat (±1 Broadcast) | {m_flat:.4} | {se_flat:.4} | — | — | {} | {} |\n\
        | Frozen REINFORCE×B_i (v12) | {m_frozen:.4} | {se_frozen:.4} | — | — | {} | {} |\n\
        | **Online Learned B_i Alignment** | **{m_learned:.4}** | **{se_learned:.4}** | **{gap_learned_m:.4}** | **{gap_learned_lcb:.4}** | **{}** | **{}** |\n\
        | Gradient Reference Ceiling | {m_grad:.4} | {se_grad:.4} | 1.0000 | 1.0000 | reference | reference |\n\n\
        ## Harness health\n\n\
        {harness_note}\n\n\
        ## Scientific Verdict\n\n\
        - Accuracy floor (≥{ACCURACY_FLOOR}): **{}**\n\
        - Gap-closed LCB (>{GAP_LCB_THRESHOLD}): **{}**\n\
        - Online Learned B_i Alignment (matched-only): {}\n\
        - Live transfer package remains v13–v24 **FAIL** (see PUBLISHABLE_CLAIMS §2b–2c).\n",
        if quick { "QUICK / PILOT" } else { "FULL SCIENTIFIC" },
        v_flat.label(),
        v_flat.label(),
        v_frozen.label(),
        v_frozen.label(),
        v_learned_floor.label(),
        learned_overall.label(),
        v_learned_floor.label(),
        v_learned_gap.label(),
        matched_threshold_note,
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
