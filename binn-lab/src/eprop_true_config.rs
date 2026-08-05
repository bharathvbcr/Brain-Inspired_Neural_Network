//! True surrogate-derivative e-prop on the exact-forward C1 substrate.
//!
//! Isolated from frozen `c1x-eprop-exact-forward-*` (hybrid STDP eligibility ×
//! output-weight-transported M). Uses prefix `c1x-eprop-true-*`, experiment
//! `c1x-eprop-true`, and fresh held-out seeds.

use crate::Config;

pub const EPROP_TRUE_PROTOCOL_VERSION: u64 = 8;
pub const EPROP_TRUE_HASH_PREFIX: &str = "c1x-eprop-true-";
pub const EPROP_TRUE_EXPERIMENT: &str = "c1x-eprop-true";

/// Learning arms in the true e-prop protocol family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EpropTrueArm {
    /// Surrogate eligibility `e ∝ σ'(score − cutoff) · pre` with transported δ.
    TrueSurrogate,
    /// Hybrid STDP eligibility × transported M (contrast; mirrors frozen c1x hybrid).
    HybridStdp,
}

impl EpropTrueArm {
    pub const ALL: [Self; 2] = [Self::TrueSurrogate, Self::HybridStdp];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TrueSurrogate => "true-surrogate-eprop",
            Self::HybridStdp => "hybrid-stdp-eprop",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|arm| arm.as_str() == value)
    }
}

/// Public config for true surrogate e-prop on the exact-forward graph.
#[derive(Clone, Debug, PartialEq)]
pub struct EpropTrueConfig {
    /// C1 substrate / G2 thresholds (unchanged from canonical C1).
    pub base: Config,
    /// Matched exposure count for both arms.
    pub matched_epochs: usize,
    /// SuperSpike surrogate steepness for the true-surrogate arm.
    pub surrogate_beta: f32,
    pub scientific_n_seeds: usize,
    pub quick: bool,
}

impl EpropTrueConfig {
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = EPROP_TRUE_EXPERIMENT.into();
        base.master_seed = 0xC1E7_7E00_0001;
        base.n_seeds = 20;
        base.quick = false;
        base.matched_budget_repeat = false;
        Self {
            matched_epochs: base.bptt_epochs,
            surrogate_beta: base.surrogate_beta,
            scientific_n_seeds: 20,
            quick: false,
            base,
        }
    }

    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{EPROP_TRUE_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1E7_D3ED_0001;
        c.base.n_seeds = 5;
        c.base.n_train = 24;
        c.base.n_test = 16;
        c.base.n_hidden = 64;
        c.base.k_wta = 1;
        c.base.quick = true;
        c.matched_epochs = 4;
        c.quick = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

    pub fn hash_for_arm(&self, arm: EpropTrueArm) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x0100_0000_01b3);
        }
        mix(&mut h, EPROP_TRUE_PROTOCOL_VERSION);
        for b in arm.as_str().as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.base.hash());
        mix(&mut h, self.matched_epochs as u64);
        mix(&mut h, self.surrogate_beta.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        h
    }

    pub fn hash_string_for_arm(&self, arm: EpropTrueArm) -> String {
        format!(
            "{}{}-{:016x}",
            EPROP_TRUE_HASH_PREFIX,
            arm.as_str(),
            self.hash_for_arm(arm)
        )
    }

    pub fn from_hash(hash: &str) -> Option<(Self, EpropTrueArm)> {
        for preset in Self::known_presets() {
            for arm in EpropTrueArm::ALL {
                if hash
                    .trim()
                    .eq_ignore_ascii_case(&preset.hash_string_for_arm(arm))
                {
                    return Some((preset, arm));
                }
            }
        }
        None
    }

    #[inline]
    pub fn seeds(&self) -> Vec<u64> {
        self.base.seeds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CreditArm, CreditConfig, C1_PROTOCOL_VERSION};

    #[test]
    fn gate_thresholds_match_canonical_and_hashes_diverge_from_frozen_hybrid() {
        let canonical = Config::c1_default();
        let cfg = EpropTrueConfig::scientific();
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert_eq!(cfg.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(cfg.base.g2_min_accuracy, canonical.g2_min_accuracy);
        let frozen_hybrid =
            CreditConfig::scientific().hash_string_for_arm(CreditArm::EpropExactForward);
        for arm in EpropTrueArm::ALL {
            let hash = cfg.hash_string_for_arm(arm);
            assert!(hash.starts_with(EPROP_TRUE_HASH_PREFIX));
            assert_ne!(hash, frozen_hybrid);
            assert_ne!(hash, "c1x-eprop-exact-forward-fcedc76a80ff0f0e");
        }
    }

    #[test]
    fn arms_have_distinct_hashes_and_round_trip() {
        let cfg = EpropTrueConfig::scientific();
        let true_hash = cfg.hash_string_for_arm(EpropTrueArm::TrueSurrogate);
        let hybrid_hash = cfg.hash_string_for_arm(EpropTrueArm::HybridStdp);
        assert_ne!(true_hash, hybrid_hash);
        for preset in EpropTrueConfig::known_presets() {
            for arm in EpropTrueArm::ALL {
                let hash = preset.hash_string_for_arm(arm);
                let (decoded, decoded_arm) = EpropTrueConfig::from_hash(&hash).unwrap();
                assert_eq!(decoded, preset);
                assert_eq!(decoded_arm, arm);
            }
        }
    }

    #[test]
    fn quick_seeds_are_disjoint_from_scientific() {
        let full = EpropTrueConfig::scientific();
        let quick = EpropTrueConfig::quick();
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        assert_ne!(
            full.hash_string_for_arm(EpropTrueArm::TrueSurrogate),
            quick.hash_string_for_arm(EpropTrueArm::TrueSurrogate)
        );
    }
}
