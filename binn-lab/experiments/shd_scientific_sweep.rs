//! Multi-seed local-rule sweep on **synthetic** frames (Suite 4).
//!
//! # This binary does not touch SHD
//!
//! Despite its name, it never loads the Spiking Heidelberg Digits corpus.
//! [`generate_synthetic_frames`] fabricates its own data: 5 classes, 24 input
//! channels, 16 timesteps, 100 train / 50 test examples per seed. Real SHD is
//! 700 channels and 20 classes over thousands of utterances, and the loader for
//! it is `binn-data`, which this file does not import.
//!
//! The synthetic task is also **trivially separable and order-free**. A label
//! `l` fires only in channels `{3l, 3l+1, 3l+2}`, which are disjoint for every
//! class at `n_in = 24`, so per-channel spike counts alone determine the label
//! and the timestep at which each spike lands is irrelevant. Any classifier that
//! can read a rate solves it.
//!
//! Consequently **no accuracy from this binary is evidence about SHD, about
//! temporal credit assignment, or about locality**, and the report says so.
//! Retained as an exploratory smoke test of the five `Shd*` arm constructors.
//! See `DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`.
//!
//! Evaluates local feedback alignment vs broadcast vs e-prop ceiling on that
//! synthetic task.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_core::Rng;
use binn_learn::{
    ShdBroadcastPm1, ShdDfa, ShdEpropCeiling, ShdExample, ShdRlLearnedFb, ShdRlReinforceFb,
    ShdTrainConfig,
};

const PROTOCOL_VERSION: u64 = 135;
const EXPERIMENT_NAME: &str = "shd-scientific-sweep (SYNTHETIC DATA)";

/// Fabricate the synthetic frames this binary trains on.
///
/// Not SHD, and not a stand-in for it. Each example fires `t/3` spikes in the
/// three channels reserved for its label, so the classes are linearly separable
/// from channel counts and carry no temporal structure.
fn generate_synthetic_frames(
    n: usize,
    n_in: usize,
    t: usize,
    n_classes: usize,
    seed: u64,
) -> Vec<ShdExample> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|_| {
            let label = rng.gen_index(n_classes) as u32;
            let mut frames = vec![0.0f32; t * n_in];
            for _ in 0..(t / 3).max(1) {
                let tt = rng.gen_index(t);
                let ch = (label as usize * 3 + rng.gen_index(3)) % n_in;
                frames[tt * n_in + ch] = 0.95;
            }
            ShdExample {
                frames,
                t,
                n_in,
                label,
            }
        })
        .collect()
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
    if let Err(error) = binn_lab::authorize_campaign(binn_lab::CampaignKind::LocalLearning) {
        eprintln!("shd-scientific-sweep: {error}");
        return ExitCode::from(3);
    }
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
                println!("Usage: cargo run --release -p binn-lab --bin shd-scientific-sweep [-- --quick] [--out PATH]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let n_seeds = if quick { 5 } else { 10 };
    let n_classes = 5;
    let n_in = 24;
    let t_frames = 16;
    let hidden = 64;
    let epochs = if quick { 15 } else { 30 };
    let master_seed = if quick {
        0x0071_AC00_001C_00F3_u64
    } else {
        0x0071_AC00_001C_00F4_u64
    };

    println!("========================================================================");
    println!("SHD Multi-Seed Scientific Sweep Protocol v{PROTOCOL_VERSION}");
    println!(
        "Schedule: {} (n_seeds={}, n_classes={}, hidden={}, epochs={})",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        },
        n_seeds,
        n_classes,
        hidden,
        epochs
    );
    println!("========================================================================\n");

    let cfg = ShdTrainConfig {
        hidden,
        n_classes,
        lr: 0.05,
        beta: 5.0,
        epochs,
    };

    let mut bcast_accs = Vec::new();
    let mut dfa_accs = Vec::new();
    let mut rfb_accs = Vec::new();
    let mut learned_accs = Vec::new();
    let mut eprop_accs = Vec::new();

    for s_idx in 0..n_seeds {
        let seed = master_seed ^ ((s_idx as u64) * 0x1000_0005_u64);
        let train = generate_synthetic_frames(100, n_in, t_frames, n_classes, seed);
        let test = generate_synthetic_frames(50, n_in, t_frames, n_classes, seed ^ 0x9999);

        let mut bcast = ShdBroadcastPm1::new(&train[0], cfg, seed);
        let r_bcast = bcast.train_and_evaluate(epochs, &train, &test);

        let mut dfa = ShdDfa::new(&train[0], cfg, seed);
        let r_dfa = dfa.train_and_evaluate(epochs, &train, &test);

        let mut rfb = ShdRlReinforceFb::new(&train[0], cfg, seed);
        let r_rfb = rfb.train_and_evaluate(epochs, &train, &test);

        let mut learned = ShdRlLearnedFb::new(&train[0], cfg, seed);
        let r_learned = learned.train_and_evaluate(epochs, &train, &test);

        let mut eprop = ShdEpropCeiling::new(&train[0], cfg, seed);
        let r_eprop = eprop.train_and_evaluate(epochs, &train, &test);

        bcast_accs.push(r_bcast.accuracy);
        dfa_accs.push(r_dfa.accuracy);
        rfb_accs.push(r_rfb.accuracy);
        learned_accs.push(r_learned.accuracy);
        eprop_accs.push(r_eprop.accuracy);

        println!("Seed {:2}/{}: Bcast={:.4} | DFA={:.4} | FrozenRFB={:.4} | LearnedFB={:.4} | Eprop={:.4}",
            s_idx + 1, n_seeds,
            r_bcast.accuracy, r_dfa.accuracy, r_rfb.accuracy, r_learned.accuracy, r_eprop.accuracy);
    }

    let m_bcast = mean(&bcast_accs);
    let m_dfa = mean(&dfa_accs);
    let m_rfb = mean(&rfb_accs);
    let m_learned = mean(&learned_accs);
    let m_eprop = mean(&eprop_accs);

    let se_bcast = std_error(&bcast_accs);
    let se_dfa = std_error(&dfa_accs);
    let se_rfb = std_error(&rfb_accs);
    let se_learned = std_error(&learned_accs);
    let se_eprop = std_error(&eprop_accs);

    let chance = 1.0 / n_classes as f32;

    let summary = format!(
        "# Multi-Seed Local-Rule Sweep Report (SYNTHETIC DATA)\n\n\
        > **This report is not about SHD.** No Spiking Heidelberg Digits sample \
        was loaded. The data are fabricated by `generate_synthetic_frames`: \
        {n_classes} classes, {n_in} channels, {t_frames} timesteps, 100 train / \
        50 test per seed, with each label firing only in its own three reserved \
        channels. The task is linearly separable from per-channel spike counts \
        and contains no temporal structure, so nothing below is evidence about \
        SHD, temporal credit assignment, or locality.\n\n\
        **Protocol Version:** {PROTOCOL_VERSION}  \n\
        **Experiment:** {EXPERIMENT_NAME}  \n\
        **Schedule:** {} (n={n_seeds}, classes={n_classes})  \n\n\
        ## Accuracy Summary (Mean ± SE vs Chance={chance:.4})\n\n\
        | Arm | Mean Accuracy | SE | Beats Chance ({chance:.2})? |\n\
        |---|---:|---:|---|\n\
        | Broadcast ±1 Three-Factor | {m_bcast:.4} | {se_bcast:.4} | {} |\n\
        | Graded DFA | {m_dfa:.4} | {se_dfa:.4} | {} |\n\
        | Frozen REINFORCE×B_i | {m_rfb:.4} | {se_rfb:.4} | {} |\n\
        | **Online Learned FB Alignment** | **{m_learned:.4}** | **{se_learned:.4}** | **{}** |\n\
        | True E-prop Ceiling | {m_eprop:.4} | {se_eprop:.4} | ✓ |\n\n\
        ## Verdict\n\n\
        - Online Learned FB Alignment: {}\n",
        if quick {
            "QUICK / PILOT"
        } else {
            "FULL SCIENTIFIC"
        },
        if m_bcast > chance + 0.05 {
            "✓"
        } else {
            "FAIL"
        },
        if m_dfa > chance + 0.05 { "✓" } else { "FAIL" },
        if m_rfb > chance + 0.05 { "✓" } else { "FAIL" },
        if m_learned > chance + 0.05 {
            "✓"
        } else {
            "FAIL"
        },
        if m_learned > chance + 0.05 {
            "above chance on the synthetic task — not an SHD result"
        } else {
            "at or below chance on the synthetic task"
        },
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
