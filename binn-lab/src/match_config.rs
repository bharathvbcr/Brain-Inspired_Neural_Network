//! Matched-architecture C1 control (protocol v4) — config and hashing.
//!
//! Isolated from canonical C1 protocol v2. Reuses G2 gate *thresholds* but
//! mints a fresh `c1-match-*` hash and does **not** reopen
//! `c1-118207fbc3eaba53`. Chance baseline for `gap_closed_matched` is 0.5.

use crate::Config;
use binn_learn::MatchedForward;

/// Protocol version for the matched-architecture confound control.
pub const C1_MATCH_PROTOCOL_VERSION: u64 = 4;

/// Matched three-factor undertraining adversarial (protocol 22).
///
/// Same matched dense-LIF forward and broadcast three-factor rule as v4, but
/// local + gradient arms train for **4×** epochs. Isolates whether the v4 FAIL
/// is an undertraining artifact. Fresh hash; does **not** remassage
/// `c1-match-5dc6822e71229e9e`.
pub const C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION: u64 = 22;

/// Epoch multiplier for protocol 22 vs scientific matched v4.
pub const C1_MATCH_UNDERTRAIN_EPOCH_MULT: usize = 4;

/// Experiment name hashed into the scientific preset.
pub const C1_MATCH_EXPERIMENT: &str = "c1-match";

/// Experiment name for matched undertrain (protocol 22).
pub const C1_MATCH_UNDERTRAIN_EXPERIMENT: &str = "c1-match-ep4";

/// Prefix for all C1-MATCH config hashes.
pub const C1_MATCH_HASH_PREFIX: &str = "c1-match-";

/// Chance baseline used in `gap_closed_matched` (binary coincidence task).
pub const C1_MATCH_CHANCE_BASELINE: f32 = 0.5;

/// Domain separator mixed in beside a non-default forward, so that "ran on
/// the other graph" cannot collide with some other single-bit difference.
pub const FORWARD_HASH_TAG: u64 = 0xF0AD_5EED_0000_0001;

/// The forward graph this suite has always run on, preserved as its default.
pub const MATCHCONFIG_DEFAULT_FORWARD: MatchedForward = MatchedForward::Recurrent;

/// Public config for the matched-architecture control.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchConfig {
    /// Substrate / schedule knobs shared with C1 where sensible.
    pub base: Config,
    /// Explicit protocol version (always 4 for this suite).
    pub protocol_version: u64,
    /// Chance accuracy used as the dense/chance floor in the gap metric.
    pub chance_baseline: f32,
    /// Minimum scientific seed count.
    pub scientific_n_seeds: usize,
    /// Development-only schedule marker.
    pub quick: bool,
    /// The forward graph the arm and its ceiling are both built on.
    ///
    /// Historically this suite's graph was `Recurrent`, chosen by which
    /// constructor the runner happened to call and recorded nowhere. It is now
    /// named so that a number can be read without consulting the source, and
    /// so that the same arm can be run on the other graph.
    pub forward: MatchedForward,
}

impl MatchConfig {
    /// Full n=20 scientific schedule. Distinct seed lineage from protocol v2/v3.
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_MATCH_EXPERIMENT.into();
        base.master_seed = 0xC1A4_C400_0001;
        base.n_seeds = 20;
        base.quick = false;
        // Dense-LIF matched arms do not use k-WTA / sparse topology; keep
        // C1-aligned n_hidden / epochs / lr / eta / lambda / beta.
        Self {
            protocol_version: C1_MATCH_PROTOCOL_VERSION,
            chance_baseline: C1_MATCH_CHANCE_BASELINE,
            scientific_n_seeds: 20,
            quick: false,
            forward: MATCHCONFIG_DEFAULT_FORWARD,
            base,
        }
    }

    /// Development/PILOT schedule (not a scientific verdict).
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{C1_MATCH_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1A4_D3ED_0001;
        c.base.n_seeds = 5;
        c.base.n_train = 24;
        c.base.n_test = 16;
        c.base.n_hidden = 64;
        c.base.bptt_epochs = 20;
        c.base.quick = true;
        c.quick = true;
        c
    }

    /// Protocol 22: matched three-factor under 4× epochs (scientific).
    pub fn undertrain_epochs() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = C1_MATCH_UNDERTRAIN_EXPERIMENT.into();
        c.protocol_version = C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION;
        c.base.bptt_epochs = c
            .base
            .bptt_epochs
            .saturating_mul(C1_MATCH_UNDERTRAIN_EPOCH_MULT);
        c
    }

    /// Quick/PILOT for protocol 22.
    pub fn undertrain_epochs_quick() -> Self {
        let mut c = Self::quick();
        c.base.experiment = format!("{C1_MATCH_UNDERTRAIN_EXPERIMENT}-quick");
        c.protocol_version = C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION;
        c.base.bptt_epochs = c
            .base
            .bptt_epochs
            .saturating_mul(C1_MATCH_UNDERTRAIN_EPOCH_MULT);
        c
    }

    /// True when this config is the matched undertrain adversarial (protocol 22).
    #[inline]
    pub fn is_undertrain_protocol(&self) -> bool {
        self.protocol_version == C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION
            || self
                .base
                .experiment
                .starts_with(C1_MATCH_UNDERTRAIN_EXPERIMENT)
    }

    pub fn known_presets() -> Vec<Self> {
        vec![
            Self::scientific(),
            Self::quick(),
            Self::undertrain_epochs(),
            Self::undertrain_epochs_quick(),
        ]
    }

    /// Stable hash over protocol 4 + all public scientific fields.
    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x0100_0000_01b3);
        }
        mix(&mut h, self.protocol_version);
        for b in self.base.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.base.master_seed);
        mix(&mut h, self.base.n_seeds as u64);
        mix(&mut h, self.base.sequence_len as u64);
        mix(&mut h, self.base.max_lag as u64);
        mix(&mut h, self.base.n_hidden as u64);
        mix(&mut h, self.base.n_train as u64);
        mix(&mut h, self.base.n_test as u64);
        mix(&mut h, self.base.bptt_epochs as u64);
        mix(&mut h, self.base.bptt_lr.to_bits() as u64);
        mix(&mut h, self.base.eta.to_bits() as u64);
        mix(&mut h, self.base.lambda.to_bits() as u64);
        mix(&mut h, self.base.surrogate_beta.to_bits() as u64);
        mix(&mut h, self.base.g2_min_gap_closed.to_bits() as u64);
        mix(&mut h, self.base.g2_min_accuracy.to_bits() as u64);
        mix(&mut h, self.base.g2_confidence_z.to_bits() as u64);
        mix(&mut h, self.base.g2_min_reference_gap.to_bits() as u64);
        mix(&mut h, self.chance_baseline.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        // The forward graph, and the input scale the whole matched family is
        // built at. Both are mixed unconditionally.
        //
        // `MATCHED_INPUT_SCALE` was NOT in this hash, and on 2026-08-25 that
        // stopped being survivable. The constant went 0.5 -> 2.0 in the silent-
        // initialisation repair, and re-running the archived config afterwards
        // produced `c1-eventprop-5bb083d5e88d0ad2` reporting **0.8900** where
        // the July record has the same hash at **0.5000**. A hash that
        // identifies an experiment must cover everything that changes its
        // result, or it silently names two.
        //
        // This does mean the archived hashes no longer resolve through
        // `from_hash`. That is the correct outcome and not a regression: this
        // binary genuinely cannot reproduce those numbers, and being told
        // "unknown hash" is strictly better than being handed different ones
        // under the name you asked for.
        mix(&mut h, FORWARD_HASH_TAG);
        mix(&mut h, u64::from(self.forward.is_recurrent()));
        mix(&mut h, binn_learn::MATCHED_INPUT_SCALE.to_bits() as u64);
        h
    }

    pub fn hash_string(&self) -> String {
        format!("{C1_MATCH_HASH_PREFIX}{:016x}", self.hash())
    }

    pub fn from_hash(hash: &str) -> Option<Self> {
        let trimmed = hash.trim();
        Self::known_presets()
            .into_iter()
            .find(|preset| trimmed.eq_ignore_ascii_case(&preset.hash_string()))
    }

    #[inline]
    pub fn seeds(&self) -> Vec<u64> {
        self.base.seeds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config, CreditArm, CreditConfig, C1_PROTOCOL_VERSION, C1_SENSITIVITY_PROTOCOL_VERSION,
        CREDIT_HASH_PREFIX,
    };

    #[test]
    fn gate_thresholds_match_canonical_g2() {
        let canonical = Config::c1_default();
        let matched = MatchConfig::scientific();
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert_eq!(matched.protocol_version, C1_MATCH_PROTOCOL_VERSION);
        assert_eq!(matched.protocol_version, 4);
        assert_eq!(matched.chance_baseline, 0.5);
        assert_eq!(matched.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(matched.base.g2_min_accuracy, canonical.g2_min_accuracy);
        assert_eq!(matched.base.g2_confidence_z, canonical.g2_confidence_z);
        assert_eq!(
            matched.base.g2_min_reference_gap,
            canonical.g2_min_reference_gap
        );
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn hash_is_stable_and_distinct_from_v2_v3_and_credit() {
        let sci = MatchConfig::scientific();
        let hash = sci.hash_string();
        assert!(hash.starts_with(C1_MATCH_HASH_PREFIX));
        assert_ne!(hash, "c1-118207fbc3eaba53");
        assert_ne!(hash, Config::c1_default().hash_string());
        assert_ne!(hash, Config::c1_capacity_sensitivity().hash_string());
        assert_ne!(hash, Config::c1_temporal_pc_sensitivity().hash_string());
        assert_eq!(C1_SENSITIVITY_PROTOCOL_VERSION, 3);
        let credit = CreditConfig::scientific();
        for arm in CreditArm::ALL {
            let ch = credit.hash_string_for_arm(arm);
            assert!(ch.starts_with(CREDIT_HASH_PREFIX));
            assert_ne!(hash, ch);
        }
        // Stability: known_presets round-trip.
        assert_eq!(MatchConfig::from_hash(&hash).as_ref(), Some(&sci));
        // The frozen scientific hash. It MOVED on 2026-08-25, deliberately, and
        // the retired value is recorded so the break is visible rather than
        // inferred:
        //
        //     retired: c1-match-5dc6822e71229e9e   (MATCHED_INPUT_SCALE = 0.5, and the scale absent
        //                       from the hash entirely)
        //     current: c1-match-6f6000f148f7d30c   (MATCHED_INPUT_SCALE = 2.0, mixed in)
        //
        // The silent-initialisation repair moved the input scale 0.5 -> 2.0
        // while the constant was not part of the hash, so the retired value
        // named two different experiments -- `c1-match-5dc6822e71229e9e` appears in the
        // July record and, re-run afterwards, produced materially different
        // numbers under the same name. Mixing the scale in is what stops that;
        // this freeze is what stops the hash drifting again by accident.
        assert_eq!(hash, "c1-match-6f6000f148f7d30c");
        let under = MatchConfig::undertrain_epochs();
        assert!(under.is_undertrain_protocol());
        assert_eq!(under.protocol_version, C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION);
        assert_eq!(
            under.base.bptt_epochs,
            MatchConfig::scientific().base.bptt_epochs * C1_MATCH_UNDERTRAIN_EPOCH_MULT
        );
        assert_ne!(under.hash_string(), hash);
        assert_ne!(under.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(
            MatchConfig::from_hash(&under.hash_string())
                .unwrap()
                .hash_string(),
            under.hash_string()
        );
    }

    #[test]
    fn quick_hash_differs_and_seeds_are_disjoint() {
        let full = MatchConfig::scientific();
        let quick = MatchConfig::quick();
        assert_ne!(full.hash_string(), quick.hash_string());
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        for preset in MatchConfig::known_presets() {
            let hash = preset.hash_string();
            assert_eq!(MatchConfig::from_hash(&hash).as_ref(), Some(&preset));
        }
    }
}
