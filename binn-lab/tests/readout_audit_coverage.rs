//! Every module that reports an accuracy must audit that accuracy for degeneracy.
//!
//! `binn-lab/src/guards.rs` states in its header that building a [`ReadoutAudit`]
//! is **mandatory** for any experiment reporting an accuracy. Until 2026-08-22
//! nothing enforced that, and the cost was concrete: `runner_match.rs` and
//! `runner_eventprop_match.rs` published `0.5000` with variance `0.000000` across
//! twenty seeds — the exact majority-class rate of a hidden layer that cannot
//! spike — and no check anywhere looked at it.
//! See `results/FINDING_2026-08-22_THE_MATCHED_ARCHITECTURE_CANNOT_SPIKE.md`.
//!
//! `report_verdict_guard.rs` enforces a different rule (no hardcoded verdict
//! literals) and resolves its root as `CARGO_MANIFEST_DIR/experiments`, so
//! `src/runner_*.rs` — where most published numbers are actually assembled — was
//! outside anything it could see.
//!
//! # This is a ratchet, not a clean bill of health
//!
//! 24 modules currently report an accuracy without an audit. Failing the
//! build on all of them would mean either reverting this test or twenty-four
//! unreviewed edits, so they are listed explicitly in [`KNOWN_UNAUDITED`] with
//! the debt visible. The list can only shrink:
//!
//! * a **new** module reporting an accuracy without an audit fails immediately;
//! * a listed module that gains an audit **also** fails, until it is removed from
//!   the list — so the list cannot rot into a permanent exemption.
//!
//! That second half is the important one. An allow-list nobody prunes is how a
//! temporary exception becomes the rule.

use std::fs;
use std::path::{Path, PathBuf};

/// Any of these means the module looked at its own readout.
const AUDIT_MARKERS: &[&str] = &["guards::", "ReadoutAudit", "Degeneracy", "CeilingHealth"];

/// Modules reporting an accuracy with no degeneracy audit, as of 2026-08-22.
///
/// Remove an entry when it gains an audit. Do not add one without saying why in
/// the commit that adds it.
const KNOWN_UNAUDITED: &[&str] = &[
    "experiments/c3.rs",
    "experiments/continual_learning.rs",
    "experiments/credit_assignment.rs",
    "experiments/efficiency.rs",
    "experiments/extensions.rs",
    "experiments/shd_frozen_attention.rs",
    "experiments/shd_input_control.rs",
    "experiments/shd_instrument.rs",
    "experiments/shd_scientific_sweep.rs",
    "experiments/shortcut_accessibility_contrast.rs",
    "experiments/temporal_eligibility_diagnostic.rs",
    "src/runner.rs",
    "src/runner_c2.rs",
    "src/runner_c3.rs",
    "src/runner_c3_bptt.rs",
    "src/runner_c3_v2.rs",
    "src/runner_credit.rs",
    "src/runner_dfa_match.rs",
    "src/runner_dfa_spike.rs",
    "src/runner_eprop_true.rs",
    "src/runner_eventprop_match.rs",
    "src/runner_match.rs",
    "src/runner_rl_match.rs",
    "src/runner_shd_cal.rs",
];

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Files that assemble a reported accuracy: the experiment binaries and the
/// runner modules they delegate to.
fn scanned_sources() -> Vec<(String, String)> {
    let root = manifest_dir();
    let mut out = Vec::new();
    for (dir, prefix) in [
        (root.join("experiments"), "experiments"),
        (root.join("src"), "src"),
    ] {
        let entries =
            fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            // In `src/`, only the runner modules assemble reports; the rest are
            // configuration, logging and plotting.
            if prefix == "src" && !name.starts_with("runner") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read source");
            out.push((format!("{prefix}/{name}"), text));
        }
    }
    out.sort();
    assert!(
        out.len() > 20,
        "found only {} sources; the scan root is wrong and this test would pass \
         by looking at nothing",
        out.len()
    );
    out
}

fn reports_accuracy(text: &str) -> bool {
    text.contains("accuracy")
}

fn has_audit(text: &str) -> bool {
    AUDIT_MARKERS.iter().any(|marker| text.contains(marker))
}

#[test]
fn every_module_reporting_an_accuracy_audits_it_or_is_listed() {
    let mut unlisted: Vec<String> = Vec::new();
    for (name, text) in scanned_sources() {
        if !reports_accuracy(&text) || has_audit(&text) {
            continue;
        }
        if !KNOWN_UNAUDITED.contains(&name.as_str()) {
            unlisted.push(name);
        }
    }
    assert!(
        unlisted.is_empty(),
        "these report an accuracy with no degeneracy audit, and are not on the \
         known-unaudited list: {unlisted:#?}\n\n\
         `guards.rs` requires a ReadoutAudit for any reported accuracy. Build one, \
         or add the file to KNOWN_UNAUDITED and say why in the commit message. \
         A constant predictor and a working learner produce the same-shaped \
         number; the audit is what tells them apart."
    );
}

#[test]
fn the_known_unaudited_list_only_shrinks() {
    let sources = scanned_sources();
    let mut fixed: Vec<&str> = Vec::new();
    let mut missing: Vec<&str> = Vec::new();
    for name in KNOWN_UNAUDITED {
        match sources.iter().find(|(file, _)| file == name) {
            None => missing.push(name),
            Some((_, text)) => {
                if has_audit(text) || !reports_accuracy(text) {
                    fixed.push(name);
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "KNOWN_UNAUDITED names files that no longer exist: {missing:#?}. \
         Remove them; a list of ghosts hides how much debt is real."
    );
    assert!(
        fixed.is_empty(),
        "these now audit their readout and must be removed from KNOWN_UNAUDITED: \
         {fixed:#?}\n\n\
         Leaving a fixed file on the list is how an exemption outlives its reason."
    );
}
