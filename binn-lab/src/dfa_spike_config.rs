//! Spiking-path DFA rescue on LatencyEncoder + k-WTA (protocol family 10).
//!
//! Matched dense-LIF DFA (`c1-dfa-*`) PASSES; frozen credit
//! `c1x-dfa-exact-forward-*` / `c1x-iso-s-dfa-*` FAIL under hard k-WTA + hybrid
//! STDP×DFA. This family asks whether **true graded DFA** (σ′ eligibility, no
//! STDP absorb) can clear unchanged G2 thresholds after disclosed substrate
//! knobs: multi-pass, richer burst latency encoder, winner-floor k-WTA, denser
//! assembly, η=0.05.
//!
//! Does **not** reopen `c1-118207fbc3eaba53`, `c1-dfa-*`, or frozen `c1x-*`.

use crate::Config;

pub const DFA_SPIKE_PROTOCOL_VERSION: u64 = 10;
pub const DFA_SPIKE_HASH_PREFIX: &str = "c1x-dfa-spike-";
pub const DFA_SPIKE_EXPERIMENT: &str = "c1x-dfa-spike";
pub const DFA_SPIKE_CHANCE_BASELINE: f32 = 0.5;

/// Learning arms in the spiking DFA rescue family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DfaSpikeArm {
    /// Graded output error × fixed-random DFA × σ′ eligibility (primary).
    TrueDfa,
    /// STDP eligibility × DFA feedback (contrast to frozen credit DFA).
    HybridStdpDfa,
    /// Same-forward surrogate-gradient ceiling (harness / gap reference).
    SurrogateGradient,
}

impl DfaSpikeArm {
    pub const ALL: [Self; 3] = [Self::TrueDfa, Self::HybridStdpDfa, Self::SurrogateGradient];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TrueDfa => "true-dfa",
            Self::HybridStdpDfa => "hybrid-stdp-dfa",
            Self::SurrogateGradient => "surrogate-gradient",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|arm| arm.as_str() == value)
    }
}

/// Public config for the spiking-path DFA rescue.
#[derive(Clone, Debug, PartialEq)]
pub struct DfaSpikeConfig {
    /// C1 substrate / G2 thresholds (thresholds unchanged; knobs disclosed).
    pub base: Config,
    /// Multi-pass exposure count (matched epochs).
    pub matched_epochs: usize,
    /// SuperSpike / σ′ steepness.
    pub surrogate_beta: f32,
    /// Chance floor for `gap_closed_dfa`.
    pub chance_baseline: f32,
    /// Richer encoder: repeats each latency spike this many times.
    pub burst_count: usize,
    /// Tick stride between burst repeats.
    pub burst_stride: u64,
    /// Winner-floor k-WTA (score all finite membranes).
    pub kwta_all_finite: bool,
    /// Learning rate for the surrogate-gradient ceiling (production-scale).
    pub surrogate_eta: f32,
    pub scientific_n_seeds: usize,
    pub quick: bool,
}

impl DfaSpikeConfig {
    /// Scientific n=20 schedule with disclosed rescue knobs.
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = DFA_SPIKE_EXPERIMENT.into();
        base.master_seed = 0xC1DF_A501_0001;
        base.n_seeds = 20;
        base.quick = false;
        base.matched_budget_repeat = false;
        // Graded DFA recipe η (production 0.35 destabilizes).
        base.eta = 0.05;
        base.lambda = 0.0;
        // Denser assembly so per-neuron credit can express under k-WTA.
        base.p_sparse = 0.70;
        Self {
            matched_epochs: base.bptt_epochs,
            surrogate_beta: base.surrogate_beta,
            chance_baseline: DFA_SPIKE_CHANCE_BASELINE,
            burst_count: 3,
            burst_stride: 2,
            kwta_all_finite: true,
            surrogate_eta: 0.35,
            scientific_n_seeds: 20,
            quick: false,
            base,
        }
    }

    /// Development/PILOT schedule (disjoint seeds).
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{DFA_SPIKE_EXPERIMENT}-quick");
        c.base.master_seed = 0xC1DF_A5D3_0001;
        c.base.n_seeds = 5;
        c.base.n_train = 24;
        c.base.n_test = 16;
        c.base.n_hidden = 64;
        c.base.k_wta = 1;
        c.base.quick = true;
        // Need enough passes for graded DFA / surrogate to move off chance.
        c.matched_epochs = 20;
        c.quick = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

    pub fn hash_for_arm(&self, arm: DfaSpikeArm) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x0100_0000_01b3);
        }
        mix(&mut h, DFA_SPIKE_PROTOCOL_VERSION);
        for b in arm.as_str().as_bytes() {
            mix(&mut h, *b as u64);
        }
        for b in self.base.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.base.hash());
        mix(&mut h, self.matched_epochs as u64);
        mix(&mut h, self.surrogate_beta.to_bits() as u64);
        mix(&mut h, self.chance_baseline.to_bits() as u64);
        mix(&mut h, self.burst_count as u64);
        mix(&mut h, self.burst_stride);
        mix(&mut h, u64::from(self.kwta_all_finite));
        mix(&mut h, self.surrogate_eta.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        // Rescue knobs also live in base.hash (eta/lambda/p_sparse); mix
        // explicitly so arm hashes diverge if those fields ever leave hash().
        mix(&mut h, self.base.eta.to_bits() as u64);
        mix(&mut h, self.base.lambda.to_bits() as u64);
        mix(&mut h, self.base.p_sparse.to_bits() as u64);
        h
    }

    pub fn hash_string_for_arm(&self, arm: DfaSpikeArm) -> String {
        format!(
            "{}{}-{:016x}",
            DFA_SPIKE_HASH_PREFIX,
            arm.as_str(),
            self.hash_for_arm(arm)
        )
    }

    pub fn from_hash(hash: &str) -> Option<(Self, DfaSpikeArm)> {
        for preset in Self::known_presets() {
            for arm in DfaSpikeArm::ALL {
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
    fn gate_thresholds_match_canonical_and_hashes_diverge() {
        let canonical = Config::c1_default();
        let cfg = DfaSpikeConfig::scientific();
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert_eq!(cfg.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(cfg.base.g2_min_accuracy, canonical.g2_min_accuracy);
        let frozen = CreditConfig::scientific().hash_string_for_arm(CreditArm::DfaExactForward);
        for arm in DfaSpikeArm::ALL {
            let hash = cfg.hash_string_for_arm(arm);
            assert!(hash.starts_with(DFA_SPIKE_HASH_PREFIX));
            assert_ne!(hash, frozen);
            assert_ne!(hash, "c1x-dfa-exact-forward-4a1601e725edbc80");
            assert_ne!(hash, "c1x-iso-s-dfa-exact-forward-d2c8d3c929a68bd2");
        }
    }

    #[test]
    fn arms_have_distinct_hashes_and_round_trip() {
        let cfg = DfaSpikeConfig::scientific();
        let hashes: Vec<_> = DfaSpikeArm::ALL
            .into_iter()
            .map(|arm| cfg.hash_string_for_arm(arm))
            .collect();
        assert_eq!(hashes.len(), 3);
        assert_ne!(hashes[0], hashes[1]);
        assert_ne!(hashes[0], hashes[2]);
        assert_ne!(hashes[1], hashes[2]);
        for preset in DfaSpikeConfig::known_presets() {
            for arm in DfaSpikeArm::ALL {
                let hash = preset.hash_string_for_arm(arm);
                let (decoded, decoded_arm) = DfaSpikeConfig::from_hash(&hash).unwrap();
                assert_eq!(decoded, preset);
                assert_eq!(decoded_arm, arm);
            }
        }
    }

    #[test]
    fn rescue_knobs_are_disclosed_on_scientific() {
        let cfg = DfaSpikeConfig::scientific();
        assert!((cfg.base.eta - 0.05).abs() < 1e-6);
        assert_eq!(cfg.base.lambda, 0.0);
        assert!((cfg.base.p_sparse - 0.70).abs() < 1e-6);
        assert_eq!(cfg.burst_count, 3);
        assert_eq!(cfg.burst_stride, 2);
        assert!(cfg.kwta_all_finite);
        assert!((cfg.surrogate_eta - 0.35).abs() < 1e-6);
        assert_eq!(cfg.matched_epochs, cfg.base.bptt_epochs);
    }

    #[test]
    fn quick_seeds_are_disjoint_from_scientific() {
        let full = DfaSpikeConfig::scientific();
        let quick = DfaSpikeConfig::quick();
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        assert_ne!(
            full.hash_string_for_arm(DfaSpikeArm::TrueDfa),
            quick.hash_string_for_arm(DfaSpikeArm::TrueDfa)
        );
    }
}
