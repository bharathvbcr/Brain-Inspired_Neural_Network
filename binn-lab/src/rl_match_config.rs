//! Matched-architecture in-family RL recipe (protocol v12) — config and hashing.
//!
//! Protocol v11 gated primary `rl_graded` and **FAIL**ed
//! (`c1-rl-ef504db58916720d`). Protocol v12 preregisters the v11 contrast
//! `rl_reinforce_fb` as the **primary** gated arm under a fresh hash — no
//! retune of failed graded knobs. Graded / flat remain contrasts.
//!
//! Isolated from protocol-v4 `c1-match-*`, protocol-v5 `c1-dfa-*`, and
//! protocol-v10 `c1x-dfa-spike-*`. Does **not** reopen `c1-118207fbc3eaba53`.
//! G2 thresholds unchanged.

use crate::Config;

/// Protocol version for the matched-architecture RL reinforce-fb primary recipe.
pub const C1_RL_PROTOCOL_VERSION: u64 = 12;

/// Experiment name hashed into the scientific preset.
pub const C1_RL_EXPERIMENT: &str = "c1-rl-fb";

/// Prefix for all C1-RL config hashes.
pub const C1_RL_HASH_PREFIX: &str = "c1-rl-";

/// Chance baseline used in `gap_closed_rl` (binary coincidence task).
pub const C1_RL_CHANCE_BASELINE: f32 = 0.5;

/// Primary gated arm identity (mixed into the config hash).
pub const C1_RL_PRIMARY_ARM: &str = "rl_reinforce_fb";

/// Public config for the matched-architecture RL control.
#[derive(Clone, Debug, PartialEq)]
pub struct RlMatchConfig {
    /// Substrate / schedule knobs shared with C1 where sensible.
    pub base: Config,
    /// Explicit protocol version (always 12 for this suite).
    pub protocol_version: u64,
    /// Chance accuracy used as the dense/chance floor in the gap metric.
    pub chance_baseline: f32,
    /// Primary gated arm label (`rl_reinforce_fb` for v12).
    pub primary_arm: &'static str,
    /// Minimum scientific seed count.
    pub scientific_n_seeds: usize,
    /// Development-only schedule marker.
    pub quick: bool,
}

impl RlMatchConfig {
    /// Full n=20 scientific schedule. Distinct seed lineage from v2–v11.
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_RL_EXPERIMENT.into();
        // Fresh lineage vs v11 (`0xC1A1_6000_0001`) — primary arm flip, not retune.
        base.master_seed = 0xC1A1_6000_0012;
        base.n_seeds = 20;
        base.quick = false;
        // Same η / λ as v11 / NumPy deep preview / DFA recipe (no graded retune).
        base.eta = 0.05;
        base.lambda = 0.0;
        Self {
            protocol_version: C1_RL_PROTOCOL_VERSION,
            chance_baseline: C1_RL_CHANCE_BASELINE,
            primary_arm: C1_RL_PRIMARY_ARM,
            scientific_n_seeds: 20,
            quick: false,
            base,
        }
    }

    /// Development/PILOT schedule (not a scientific verdict).
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{C1_RL_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1A1_D3ED_0012;
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

    /// Stable hash over protocol 12 + primary arm + all public scientific fields.
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
        for b in self.primary_arm.as_bytes() {
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
        format!("{C1_RL_HASH_PREFIX}{:016x}", self.hash())
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
        Config, CreditArm, CreditConfig, DfaMatchConfig, DfaSpikeConfig, MatchConfig,
        C1_DFA_PROTOCOL_VERSION, C1_ISOLATION_PROTOCOL_VERSION, C1_MATCH_PROTOCOL_VERSION,
        C1_PROTOCOL_VERSION, C1_SENSITIVITY_PROTOCOL_VERSION, CREDIT_HASH_PREFIX,
        DFA_SPIKE_PROTOCOL_VERSION,
    };

    #[test]
    fn gate_thresholds_match_canonical_g2() {
        let canonical = Config::c1_default();
        let rl = RlMatchConfig::scientific();
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert_eq!(rl.protocol_version, C1_RL_PROTOCOL_VERSION);
        assert_eq!(rl.protocol_version, 12);
        assert_eq!(rl.primary_arm, C1_RL_PRIMARY_ARM);
        assert_eq!(rl.chance_baseline, 0.5);
        assert_eq!(rl.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(rl.base.g2_min_accuracy, canonical.g2_min_accuracy);
        assert_eq!(rl.base.g2_confidence_z, canonical.g2_confidence_z);
        assert_eq!(rl.base.g2_min_reference_gap, canonical.g2_min_reference_gap);
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn hash_is_stable_and_distinct_from_prior_families() {
        let sci = RlMatchConfig::scientific();
        let hash = sci.hash_string();
        assert!(hash.starts_with(C1_RL_HASH_PREFIX));
        assert_ne!(hash, "c1-118207fbc3eaba53");
        // Closed v11 graded-primary scientific hash must not collide.
        assert_ne!(hash, "c1-rl-ef504db58916720d");
        assert_ne!(hash, Config::c1_default().hash_string());
        assert_ne!(hash, MatchConfig::scientific().hash_string());
        assert_ne!(hash, DfaMatchConfig::scientific().hash_string());
        let spike = DfaSpikeConfig::scientific();
        for arm in crate::DfaSpikeArm::ALL {
            assert_ne!(hash, spike.hash_string_for_arm(arm));
        }
        assert_eq!(C1_SENSITIVITY_PROTOCOL_VERSION, 3);
        assert_eq!(C1_MATCH_PROTOCOL_VERSION, 4);
        assert_eq!(C1_DFA_PROTOCOL_VERSION, 5);
        assert_eq!(C1_ISOLATION_PROTOCOL_VERSION, 5);
        assert_eq!(DFA_SPIKE_PROTOCOL_VERSION, 10);
        assert_eq!(C1_RL_PROTOCOL_VERSION, 12);
        let credit = CreditConfig::scientific();
        for arm in CreditArm::ALL {
            let ch = credit.hash_string_for_arm(arm);
            assert!(ch.starts_with(CREDIT_HASH_PREFIX));
            assert_ne!(hash, ch);
        }
        assert_eq!(RlMatchConfig::from_hash(&hash).as_ref(), Some(&sci));
        // Paper-cited scientific hash freeze (v12 reinforce_fb primary).
        assert_eq!(hash, "c1-rl-42eddc9c801308e9");
    }

    #[test]
    fn quick_hash_differs_and_seeds_are_disjoint() {
        let full = RlMatchConfig::scientific();
        let quick = RlMatchConfig::quick();
        assert_ne!(full.hash_string(), quick.hash_string());
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        for preset in RlMatchConfig::known_presets() {
            let hash = preset.hash_string();
            assert_eq!(RlMatchConfig::from_hash(&hash).as_ref(), Some(&preset));
        }
    }
}
