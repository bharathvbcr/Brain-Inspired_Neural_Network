//! Every config family's `known_presets()` must be non-empty, and must not shrink.
//!
//! # The gap this closes
//!
//! Fourteen config families each carry a `from_hash` round-trip test shaped
//! like this:
//!
//! ```ignore
//! for preset in XConfig::known_presets() {
//!     assert_eq!(XConfig::from_hash(&preset.hash_string()).unwrap(), preset);
//! }
//! ```
//!
//! The loop is the whole test. If `known_presets()` ever returned an empty
//! `Vec` — a family deleted, a `cfg` that stopped matching, a refactor that
//! dropped a push — every one of those tests would pass, printing `ok`, having
//! compared nothing. `find_weak_checks.py` names that shape: assertions that
//! only run if something is there.
//!
//! An empty preset list is not a hypothetical failure. It is precisely how
//! `--config-hash` stops resolving, which is the defect that left three
//! unrunnable replay commands in `README.md` for thirteen days.
//!
//! The floors are asserted **here, once**, rather than in twenty round-trip
//! tests, so a family that empties fails in one place and every one of those
//! loops is thereby known to run. Adding presets is fine; losing them is not.

use binn_lab::credit_config::CreditConfig;
use binn_lab::dfa_match_config::DfaMatchConfig;
use binn_lab::dfa_spike_config::DfaSpikeConfig;
use binn_lab::eprop_true_config::EpropTrueConfig;
use binn_lab::eventprop_match_config::EventPropMatchConfig;
use binn_lab::match_config::MatchConfig;
use binn_lab::r2_credit_config::R2CreditConfig;
use binn_lab::rl_match_config::RlMatchConfig;
use binn_lab::{C2Config, C3BpttConfig, C3Config, C3V2Config, Config, R1Config, R2Config};

/// `(family, floor)` — the count on 2026-09-07, so a silent shrink fails.
fn census() -> Vec<(&'static str, usize, usize)> {
    vec![
        ("Config", Config::known_presets().len(), 44),
        ("C2Config", C2Config::known_presets().len(), 2),
        ("C3Config", C3Config::known_presets().len(), 2),
        ("C3V2Config", C3V2Config::known_presets().len(), 2),
        ("C3BpttConfig", C3BpttConfig::known_presets().len(), 2),
        ("R1Config", R1Config::known_presets().len(), 2),
        ("R2Config", R2Config::known_presets().len(), 2),
        ("R2CreditConfig", R2CreditConfig::known_presets().len(), 2),
        ("CreditConfig", CreditConfig::known_presets().len(), 2),
        ("MatchConfig", MatchConfig::known_presets().len(), 1),
        ("DfaMatchConfig", DfaMatchConfig::known_presets().len(), 1),
        ("RlMatchConfig", RlMatchConfig::known_presets().len(), 1),
        ("EventPropMatchConfig", EventPropMatchConfig::known_presets().len(), 1),
        ("DfaSpikeConfig", DfaSpikeConfig::known_presets().len(), 1),
        ("EpropTrueConfig", EpropTrueConfig::known_presets().len(), 1),
    ]
}

#[test]
fn no_config_family_is_empty() {
    for (family, count, _) in census() {
        assert!(
            count > 0,
            "{family}::known_presets() is empty, so every `--config-hash` for \
             that family fails to resolve — and its round-trip test still passes"
        );
    }
}

#[test]
fn no_config_family_has_shrunk() {
    for (family, count, floor) in census() {
        assert!(
            count >= floor,
            "{family}::known_presets() returned {count}, below the recorded \
             floor of {floor}. A preset was removed: every archived hash in it \
             now fails to resolve. If the removal is deliberate, retire the \
             hashes in the record first, then lower the floor here."
        );
    }
}

#[test]
fn the_census_covers_something() {
    // A census over an empty list would satisfy both tests above forever.
    let families = census();
    assert!(families.len() >= 15, "{} families censused", families.len());
    assert!(
        families.iter().map(|(_, n, _)| n).sum::<usize>() >= 56,
        "the census reaches fewer presets than the floors it asserts"
    );
}
