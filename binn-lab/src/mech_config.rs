//! Matched mechanism diagnostic config (protocol `c1-mech-*`).
//!
//! Recording-only protocol: one-step loss-drop / eligibility-energy on the
//! frozen feed-forward dense-LIF coincidence forward. Does **not** reopen
//! `c1-118207fbc3eaba53`, `c1-match-*`, `c1-dfa-*`, or `c1-rl-*`.

use crate::Config;

/// Protocol version for the mechanism diagnostic family.
pub const C1_MECH_PROTOCOL_VERSION: u64 = 25;

/// Experiment name hashed into presets.
pub const C1_MECH_EXPERIMENT: &str = "c1-mech";

/// Prefix for mechanism diagnostic config hashes.
pub const C1_MECH_HASH_PREFIX: &str = "c1-mech-";

/// Public config for the mechanism diagnostic.
#[derive(Clone, Debug, PartialEq)]
pub struct MechConfig {
    pub base: Config,
    pub protocol_version: u64,
    pub scientific_n_seeds: usize,
    pub quick: bool,
    /// Max train examples used as one-step probes per seed.
    pub n_probe: usize,
}

impl MechConfig {
    /// Scientific recording schedule (n=20 seeds).
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_MECH_EXPERIMENT.into();
        base.master_seed = 0xC1EC_4A00_0001;
        base.n_seeds = 20;
        base.quick = false;
        // Align with DFA/RL feed-forward matched schedule (richness contrast).
        base.eta = 0.05;
        base.lambda = 0.0;
        Self {
            protocol_version: C1_MECH_PROTOCOL_VERSION,
            scientific_n_seeds: 20,
            quick: false,
            n_probe: 64,
            base,
        }
    }

    /// Development / PILOT schedule.
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{C1_MECH_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1EC_D3ED_0001;
        c.base.n_seeds = 3;
        c.base.n_train = 24;
        c.base.n_test = 8;
        c.base.n_hidden = 64;
        c.base.quick = true;
        c.n_probe = 16;
        c.quick = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

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
        mix(&mut h, self.base.surrogate_beta.to_bits() as u64);
        mix(&mut h, self.n_probe as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        // Marker: feed-forward matched forward (wrec=0) + SuperSpike warm-start probes.
        mix(&mut h, 0xFEED_F04D_0000_0001);
        mix(&mut h, 0xA4A0_0030_0000_0001); // warm=30 epochs marker
        h
    }

    pub fn hash_string(&self) -> String {
        format!("{C1_MECH_HASH_PREFIX}{:016x}", self.hash())
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
    use crate::{MatchConfig, C1_MATCH_HASH_PREFIX};

    #[test]
    fn hashes_diverge_from_frozen_match_family() {
        let mech = MechConfig::scientific();
        let h = mech.hash_string();
        assert!(h.starts_with(C1_MECH_HASH_PREFIX));
        assert!(!h.starts_with(C1_MATCH_HASH_PREFIX));
        assert_ne!(h, MatchConfig::scientific().hash_string());
        assert_ne!(h, "c1-118207fbc3eaba53");
        assert_ne!(h, "c1-match-5dc6822e71229e9e");
    }

    #[test]
    fn quick_and_scientific_hashes_differ() {
        assert_ne!(
            MechConfig::scientific().hash_string(),
            MechConfig::quick().hash_string()
        );
    }
}
