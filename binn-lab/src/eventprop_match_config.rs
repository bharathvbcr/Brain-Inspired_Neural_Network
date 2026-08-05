//! Matched-architecture EventProp H2H (protocol v28) — config and hashing.
//!
//! Rule-only contrast: discrete EventProp-style spike-triggered adjoint vs
//! SuperSpike BPTT on the identical recurrent dense-LIF matched forward.
//! Fresh `c1-eventprop-*` hash family. Does **not** reopen
//! `c1-118207fbc3eaba53`, mutate `c1-match-5dc6822e71229e9e`, or remassage
//! DFA/RL scientific hashes. G2 thresholds unchanged.

use crate::Config;

/// Protocol version for the matched EventProp head-to-head.
pub const C1_EVENTPROP_PROTOCOL_VERSION: u64 = 28;

/// Experiment name hashed into the scientific preset.
pub const C1_EVENTPROP_EXPERIMENT: &str = "c1-eventprop";

/// Prefix for all C1-EVENTPROP config hashes.
pub const C1_EVENTPROP_HASH_PREFIX: &str = "c1-eventprop-";

/// Chance baseline used in `gap_closed_eventprop` (binary coincidence task).
pub const C1_EVENTPROP_CHANCE_BASELINE: f32 = 0.5;

/// Public config for the matched EventProp control.
#[derive(Clone, Debug, PartialEq)]
pub struct EventPropMatchConfig {
    /// Substrate / schedule knobs shared with C1 where sensible.
    pub base: Config,
    /// Explicit protocol version (always 28 for this suite).
    pub protocol_version: u64,
    /// Chance accuracy used as the dense/chance floor in the gap metric.
    pub chance_baseline: f32,
    /// Minimum scientific seed count.
    pub scientific_n_seeds: usize,
    /// Development-only schedule marker.
    pub quick: bool,
}

impl EventPropMatchConfig {
    /// Full n=20 scientific schedule. Distinct seed lineage from v2/v4/v5/v12.
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_EVENTPROP_EXPERIMENT.into();
        base.master_seed = 0xC1E7_E700_0001;
        base.n_seeds = 20;
        base.quick = false;
        // Match matched-arch SuperSpike schedule (rule-only H2H).
        Self {
            protocol_version: C1_EVENTPROP_PROTOCOL_VERSION,
            chance_baseline: C1_EVENTPROP_CHANCE_BASELINE,
            scientific_n_seeds: 20,
            quick: false,
            base,
        }
    }

    /// Development/PILOT schedule (not a scientific verdict).
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{C1_EVENTPROP_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1E7_D3ED_0001;
        c.base.n_seeds = 5;
        c.base.n_train = 24;
        c.base.n_test = 16;
        c.base.n_hidden = 64;
        c.base.bptt_epochs = 20;
        c.base.quick = true;
        c.quick = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

    /// Stable hash over protocol 28 + all public scientific fields.
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
        h
    }

    pub fn hash_string(&self) -> String {
        format!("{C1_EVENTPROP_HASH_PREFIX}{:016x}", self.hash())
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
        Config, DfaMatchConfig, MatchConfig, RlMatchConfig, C1_MATCH_PROTOCOL_VERSION,
        C1_PROTOCOL_VERSION,
    };

    #[test]
    fn gate_thresholds_match_canonical_g2() {
        let canonical = Config::c1_default();
        let ep = EventPropMatchConfig::scientific();
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert_eq!(ep.protocol_version, C1_EVENTPROP_PROTOCOL_VERSION);
        assert_eq!(ep.protocol_version, 28);
        assert_eq!(ep.chance_baseline, 0.5);
        assert_eq!(ep.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(ep.base.g2_min_accuracy, canonical.g2_min_accuracy);
        assert_eq!(ep.base.g2_confidence_z, canonical.g2_confidence_z);
        assert_eq!(ep.base.g2_min_reference_gap, canonical.g2_min_reference_gap);
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn hash_is_stable_and_distinct_from_frozen_families() {
        let sci = EventPropMatchConfig::scientific();
        let hash = sci.hash_string();
        assert!(hash.starts_with(C1_EVENTPROP_HASH_PREFIX));
        assert_ne!(hash, "c1-118207fbc3eaba53");
        assert_ne!(hash, "c1-match-5dc6822e71229e9e");
        assert_ne!(hash, MatchConfig::scientific().hash_string());
        assert_ne!(hash, DfaMatchConfig::scientific().hash_string());
        assert_ne!(hash, RlMatchConfig::scientific().hash_string());
        assert_eq!(C1_MATCH_PROTOCOL_VERSION, 4);
        assert_eq!(EventPropMatchConfig::from_hash(&hash).as_ref(), Some(&sci));
        // Paper-cited scientific hash freeze.
        assert_eq!(hash, "c1-eventprop-5bb083d5e88d0ad2");
    }

    #[test]
    fn quick_hash_differs_and_seeds_are_disjoint() {
        let full = EventPropMatchConfig::scientific();
        let quick = EventPropMatchConfig::quick();
        assert_ne!(full.hash_string(), quick.hash_string());
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        for preset in EventPropMatchConfig::known_presets() {
            let hash = preset.hash_string();
            assert_eq!(
                EventPropMatchConfig::from_hash(&hash).as_ref(),
                Some(&preset)
            );
        }
    }
}
