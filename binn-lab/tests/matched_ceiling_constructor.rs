//! The matched-arch ceiling must share its architecture with the arms it bounds.
//!
//! Every `MatchedRl*` and `MatchedLocal` arm is built on `MatchedArch::feedforward`
//! (`wrec = 0`). Three experiment binaries built their ceiling with
//! `MatchedGradient::new` instead, which carries an extra `hidden x hidden`
//! recurrent matrix no arm has — against a preregistration
//! (`results/MATCHED_ARCH_RL_CONTROL.md:37`) that names `new_feedforward`, and
//! against two sibling runners that use it.
//!
//! It was not cosmetic. `track_b_results_v132.md` carries a headline "ceiling
//! inverted ... no PASS is permitted while this warning is present", triggered by
//! 3 of 20 seeds; against an architecture-matched ceiling that falls to 1 of 20,
//! and all three inverting seeds are ones where the matched ceiling reaches
//! 1.0000 while the recurrent one does not.
//!
//! `a6_ceiling_health.rs` — the binary built to answer "was the ceiling
//! undertrained?" — uses the correct constructor, so the diagnostic for this
//! class of defect could never have caught this instance of it. That is why the
//! check lives here, across all the call sites, rather than inside any one of
//! them.
//!
//! See `results/FINDING_2026-08-22_THE_MATCHED_ARCHITECTURE_CANNOT_SPIKE.md`.

use std::fs;
use std::path::Path;

/// Binaries whose treatment arms are feedforward, so whose ceiling must be too.
const FEEDFORWARD_ARM_BINARIES: &[&str] = &[
    "track_b_rescue.rs",
    "live_transfer_rescue.rs",
    "continual_learning.rs",
    "a6_ceiling_health.rs",
];

/// Runners that build both an arm and its ceiling, wherever they live.
///
/// This test scanned `experiments/` only, so `src/runner_match.rs` and
/// `src/runner_eventprop_match.rs` — which build `MatchedGradient::new`, the
/// recurrent constructor — were never examined. There is no inversion in
/// either, because `MatchedLocal` and `MatchedEventProp` are recurrent too and
/// arm and ceiling agree. What the omission hid is a different problem, one
/// directory up: those two pairs run on the **recurrent** graph while the DFA
/// and REINFORCE pairs run on the **feed-forward** one, and the paper's central
/// contrast is a comparison across them. See `MatchedForward`.
const RUNNER_SOURCES: &[&str] = &[
    "runner_match.rs",
    "runner_dfa_match.rs",
    "runner_rl_match.rs",
    "runner_eventprop_match.rs",
];

#[test]
fn the_ceiling_is_feedforward_wherever_the_arms_are() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments");
    let mut violations = Vec::new();
    let mut checked = 0usize;

    for name in FEEDFORWARD_ARM_BINARIES {
        let path = dir.join(name);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        for (idx, line) in text.lines().enumerate() {
            if !line.contains("MatchedGradient::new") {
                continue;
            }
            checked += 1;
            if !line.contains("MatchedGradient::new_feedforward") {
                violations.push(format!("{name}:{}: {}", idx + 1, line.trim()));
            }
        }
    }

    // Refuse a vacuous pass: if the constructor is renamed or the call sites move,
    // finding nothing must not read as finding nothing wrong.
    assert!(
        checked >= FEEDFORWARD_ARM_BINARIES.len(),
        "found only {checked} MatchedGradient constructions across {} binaries; \
         the call sites moved and this test is no longer looking at anything",
        FEEDFORWARD_ARM_BINARIES.len()
    );
    assert!(
        violations.is_empty(),
        "these build a recurrent ceiling for feedforward arms: {violations:#?}\n\n\
         The ceiling would carry a hidden x hidden matrix no arm has, and lose to \
         it on some seeds — which reads as the arm beating its reference."
    );
}

/// Every matched runner must name the graph it runs on, not inherit one.
///
/// The four runners chose a forward by which constructor they happened to call,
/// and none of them recorded the choice. Two chose recurrent and two chose
/// feed-forward, which is invisible in any report and load-bearing in the
/// comparison across them. `MatchedForward` makes it an argument; this pins
/// that every runner passes one, so a future arm cannot quietly inherit a graph
/// again.
#[test]
fn every_matched_runner_names_its_forward_graph() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut missing = Vec::new();
    let mut checked = 0usize;

    for name in RUNNER_SOURCES {
        let path = dir.join(name);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        checked += 1;
        // Two conditions, because either alone is satisfiable without the
        // other: a runner could take the graph and not report it, or print a
        // label it never passed to a constructor.
        if !text.contains("config.forward,") {
            missing.push(format!(
                "{name}: builds its arm or ceiling without passing config.forward"
            ));
        }
        if !text.contains("forward.label()") {
            missing.push(format!(
                "{name}: does not record the forward graph in its report"
            ));
        }
    }

    // Refuse a vacuous pass, the same way the test above does.
    assert_eq!(
        checked,
        RUNNER_SOURCES.len(),
        "the runner sources moved and this test is no longer looking at them"
    );
    assert!(
        missing.is_empty(),
        "matched runners not naming their forward graph: {missing:#?}\n\n\
         A runner that inherits its graph from whichever constructor it calls \
         produces numbers that cannot be compared with another runner's without \
         checking the source to find out what they ran on."
    );
}
