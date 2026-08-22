//! Every campaign binary the SHD instrument gate blocks must actually be
//! blocked, and must say so.
//!
//! # The bug this prevents
//!
//! `binn_lab::instrument_status` tests [`authorize_campaign`] as a function.
//! That is not the same claim as "the binary is refused": a binary can call it
//! with the wrong [`CampaignKind`], call it after doing work, or not call it at
//! all, and the gate's own unit test stays green. Nothing checked the binaries
//! until this file.
//!
//! Each entry below spawns the real binary and requires a nonzero exit **and**
//! the gate's own message on stderr, so a binary that exits nonzero for an
//! unrelated reason — a missing `--out`, an absent data file — cannot pass this
//! test by accident.
//!
//! When `SHD_INSTRUMENT_STATE` is eventually moved to `Calibrated`, these
//! expectations become wrong by design; the test reads the constant and skips
//! rather than pinning the project to the uncalibrated state.

use std::process::{Command, Output};

use binn_lab::{CampaignKind, InstrumentState, SHD_INSTRUMENT_STATE};

/// The binaries whose entry point requests a campaign the gate refuses while
/// the instrument is uncalibrated, with the arguments needed to reach it.
const BLOCKED: &[(&str, CampaignKind, &[&str])] = &[
    ("shd-depth-scaling", CampaignKind::LocalLearning, &["run"]),
    ("shd-frozen-attention", CampaignKind::LocalLearning, &[]),
    ("shd-arch-ablation", CampaignKind::LocalLearning, &[]),
    ("shd-scientific-sweep", CampaignKind::LocalLearning, &[]),
    (
        "shortcut-accessibility-contrast",
        CampaignKind::LocalLearning,
        &[],
    ),
    ("temporal-deep-campaign", CampaignKind::LocalLearning, &[]),
    (
        "temporal-eligibility-diagnostic",
        CampaignKind::LocalLearning,
        &[],
    ),
    ("transfer-falsifier", CampaignKind::Transfer, &[]),
    ("temporal-optimizer-control", CampaignKind::Optimizer, &[]),
];

fn spawn(name: &str, args: &[&str]) -> Output {
    let key = format!("CARGO_BIN_EXE_{name}");
    let mut command = match std::env::var(&key) {
        Ok(path) => Command::new(path),
        Err(_) => {
            let mut cargo = Command::new("cargo");
            cargo.args(["run", "-q", "-p", "binn-lab", "--bin", name, "--"]);
            cargo
        }
    };
    command.args(args);
    command
        .output()
        .unwrap_or_else(|e| panic!("spawn {name}: {e}"))
}

#[test]
fn every_blocked_campaign_binary_is_refused_and_says_why() {
    if SHD_INSTRUMENT_STATE != InstrumentState::Uncalibrated {
        return;
    }
    for (name, kind, args) in BLOCKED {
        assert!(
            binn_lab::authorize_campaign(*kind).is_err(),
            "{name} is listed as blocked but {} is authorized",
            kind.as_str()
        );
        let out = spawn(name, args);
        assert_ne!(
            out.status.code(),
            Some(0),
            "{name} must exit nonzero while the instrument is uncalibrated"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("UNCALIBRATED") && stderr.contains(kind.as_str()),
            "{name} exited nonzero but not because of the gate; stderr was:\n{stderr}"
        );
    }
}

/// The one command of `shd-depth-scaling` that trains nothing and reports no
/// accuracy is *not* blocked, so the refusal above is the gate discriminating
/// between campaign classes rather than the binary being broken.
#[test]
fn the_harness_validation_command_is_not_refused_by_the_gate() {
    let out = spawn(
        "shd-depth-scaling",
        &["activity-probe", "--events", "/nonexistent"],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("UNCALIBRATED"),
        "the probe must not be refused by the instrument gate; stderr was:\n{stderr}"
    );
}

/// Asking for a command that does not exist must not be reported as a gate
/// refusal, or a typo would read as a scientific block.
#[test]
fn an_unknown_command_is_not_reported_as_a_gate_refusal() {
    let out = spawn("shd-depth-scaling", &["not-a-command"]);
    assert_ne!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown command"), "stderr was:\n{stderr}");
    assert!(!stderr.contains("UNCALIBRATED"), "stderr was:\n{stderr}");
}
