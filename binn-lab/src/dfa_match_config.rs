//! Matched-architecture DFA recipe (protocol v5) — config and hashing.
//!
//! Ports the directional graded-error + DFA feedback recipe from
//! `MATCHED_ARCH_DEEP_FINDINGS.md` onto the dense-LIF matched forward.
//! Isolated from protocol-v4 `c1-match-*` and from trial-isolation `c1-iso*`
//! (also numbered "5" in that family — distinct experiment / hash prefix).
//! Does **not** reopen `c1-118207fbc3eaba53`. G2 thresholds unchanged.

use crate::match_config::FORWARD_HASH_TAG;
use crate::Config;
use binn_learn::MatchedForward;

/// Protocol version for the matched-architecture DFA recipe.
pub const C1_DFA_PROTOCOL_VERSION: u64 = 5;

/// Experiment name hashed into the scientific preset.
pub const C1_DFA_EXPERIMENT: &str = "c1-dfa";

/// Prefix for all C1-DFA config hashes.
pub const C1_DFA_HASH_PREFIX: &str = "c1-dfa-";

/// Chance baseline used in `gap_closed_dfa` (binary coincidence task).
pub const C1_DFA_CHANCE_BASELINE: f32 = 0.5;

/// The forward graph this suite has always run on, preserved as its default.
pub const DFAMATCHCONFIG_DEFAULT_FORWARD: MatchedForward = MatchedForward::FeedForward;

/// Public config for the matched-architecture DFA control.
#[derive(Clone, Debug, PartialEq)]
pub struct DfaMatchConfig {
    /// Substrate / schedule knobs shared with C1 where sensible.
    pub base: Config,
    /// Explicit protocol version (always 5 for this suite).
    pub protocol_version: u64,
    /// Chance accuracy used as the dense/chance floor in the gap metric.
    pub chance_baseline: f32,
    /// Minimum scientific seed count.
    pub scientific_n_seeds: usize,
    /// Development-only schedule marker.
    pub quick: bool,
    /// The forward graph the arm and its ceiling are both built on.
    ///
    /// Historically this suite's graph was `FeedForward`, chosen by which
    /// constructor the runner happened to call and recorded nowhere. It is now
    /// named so that a number can be read without consulting the source, and
    /// so that the same arm can be run on the other graph.
    pub forward: MatchedForward,
}

impl DfaMatchConfig {
    /// Full n=20 scientific schedule. Distinct seed lineage from v2/v3/v4.
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_DFA_EXPERIMENT.into();
        base.master_seed = 0xC1D5_A400_0001;
        base.n_seeds = 20;
        base.quick = false;
        // Graded supervised error is denser than ±1 reward; use the NumPy
        // preview η=0.05 (production three-factor η=0.35 destabilizes this rule).
        base.eta = 0.05;
        // NumPy preview used lam=0; small λ can erase weak DFA updates.
        base.lambda = 0.0;
        Self {
            protocol_version: C1_DFA_PROTOCOL_VERSION,
            chance_baseline: C1_DFA_CHANCE_BASELINE,
            scientific_n_seeds: 20,
            quick: false,
            forward: DFAMATCHCONFIG_DEFAULT_FORWARD,
            base,
        }
    }

    /// Development/PILOT schedule (not a scientific verdict).
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{C1_DFA_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1D5_D3ED_0001;
        c.base.n_seeds = 5;
        c.base.n_train = 48;
        c.base.n_test = 24;
        c.base.n_hidden = 128;
        c.base.bptt_epochs = 60;
        c.base.quick = true;
        c.quick = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

    /// Stable hash over protocol 5 + all public scientific fields.
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
        // Mixed ONLY when it differs from the historical default, so that the
        // archived `DfaMatchConfig` hashes still resolve through `from_hash` and every
        // citation of them still replays. A run on the other graph is a
        // different experiment and gets a different hash, which is the point;
        // a run on the same graph must keep the identity it was published under.
        if self.forward != DFAMATCHCONFIG_DEFAULT_FORWARD {
            mix(&mut h, FORWARD_HASH_TAG);
            mix(&mut h, u64::from(self.forward.is_recurrent()));
        }
        h
    }

    pub fn hash_string(&self) -> String {
        format!("{C1_DFA_HASH_PREFIX}{:016x}", self.hash())
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
        Config, CreditArm, CreditConfig, MatchConfig, C1_ISOLATION_PROTOCOL_VERSION,
        C1_MATCH_PROTOCOL_VERSION, C1_PROTOCOL_VERSION, C1_SENSITIVITY_PROTOCOL_VERSION,
        CREDIT_HASH_PREFIX,
    };

    #[test]
    fn gate_thresholds_match_canonical_g2() {
        let canonical = Config::c1_default();
        let dfa = DfaMatchConfig::scientific();
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert_eq!(dfa.protocol_version, C1_DFA_PROTOCOL_VERSION);
        assert_eq!(dfa.protocol_version, 5);
        assert_eq!(dfa.chance_baseline, 0.5);
        assert_eq!(dfa.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(dfa.base.g2_min_accuracy, canonical.g2_min_accuracy);
        assert_eq!(dfa.base.g2_confidence_z, canonical.g2_confidence_z);
        assert_eq!(
            dfa.base.g2_min_reference_gap,
            canonical.g2_min_reference_gap
        );
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn hash_is_stable_and_distinct_from_v2_v3_v4_iso_and_credit() {
        let sci = DfaMatchConfig::scientific();
        let hash = sci.hash_string();
        assert!(hash.starts_with(C1_DFA_HASH_PREFIX));
        assert_ne!(hash, "c1-118207fbc3eaba53");
        assert_ne!(hash, Config::c1_default().hash_string());
        assert_ne!(hash, Config::c1_capacity_sensitivity().hash_string());
        assert_ne!(hash, Config::c1_temporal_pc_sensitivity().hash_string());
        assert_ne!(hash, MatchConfig::scientific().hash_string());
        assert_eq!(C1_SENSITIVITY_PROTOCOL_VERSION, 3);
        assert_eq!(C1_MATCH_PROTOCOL_VERSION, 4);
        assert_eq!(C1_ISOLATION_PROTOCOL_VERSION, 5); // same integer, different family
        assert_ne!(hash, Config::c1_isolation().hash_string());
        let credit = CreditConfig::scientific();
        for arm in CreditArm::ALL {
            let ch = credit.hash_string_for_arm(arm);
            assert!(ch.starts_with(CREDIT_HASH_PREFIX));
            assert_ne!(hash, ch);
        }
        assert_eq!(DfaMatchConfig::from_hash(&hash).as_ref(), Some(&sci));
        // Paper-cited scientific hash freeze.
        assert_eq!(hash, "c1-dfa-c8c4fe0899908b84");
    }

    #[test]
    fn quick_hash_differs_and_seeds_are_disjoint() {
        let full = DfaMatchConfig::scientific();
        let quick = DfaMatchConfig::quick();
        assert_ne!(full.hash_string(), quick.hash_string());
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        for preset in DfaMatchConfig::known_presets() {
            let hash = preset.hash_string();
            assert_eq!(DfaMatchConfig::from_hash(&hash).as_ref(), Some(&preset));
        }
    }
}
