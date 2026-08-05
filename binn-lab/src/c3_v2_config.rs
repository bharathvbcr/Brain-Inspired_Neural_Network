//! Production-faithful C3 v2 preregistration.
//!
//! C3 v1 remains the tabular proxy.  This separate protocol runs actual
//! event-engine transition areas and production `ThreeFactor` eligibility.

pub const C3_V2_HASH_PREFIX: &str = "c3v2-";
pub const C3_V2_PROTOCOL_VERSION: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum C3V2Arm {
    Broadcast,
    Rpe,
    Eprop,
    Dfa,
    MatchedGradient,
}

impl C3V2Arm {
    pub const ALL: [Self; 5] = [
        Self::Broadcast,
        Self::Rpe,
        Self::Eprop,
        Self::Dfa,
        Self::MatchedGradient,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Broadcast => "broadcast-three-factor",
            Self::Rpe => "rpe-three-factor",
            Self::Eprop => "eprop-postsynaptic",
            Self::Dfa => "dfa-fixed-feedback",
            Self::MatchedGradient => "matched-forward-oracle-gradient-reference",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3V2Config {
    pub master_seed: u64,
    pub n_seeds: usize,
    pub min_depth: usize,
    pub max_depth: usize,
    pub n_states: usize,
    pub n_operations: usize,
    pub n_train: usize,
    pub n_test: usize,
    pub init_w: f32,
    pub eta: f32,
    pub lambda: f32,
    pub tau_e: f32,
    pub accuracy_floor: f32,
    pub scientific_n_seeds: usize,
    pub quick: bool,
    pub kill_gate_override: bool,
}

impl C3V2Config {
    pub fn scientific() -> Self {
        Self {
            master_seed: 0xC3F2_5C1E_0001,
            n_seeds: 20,
            min_depth: 1,
            max_depth: 8,
            n_states: 4,
            n_operations: 2,
            n_train: 2_000,
            n_test: 500,
            init_w: 0.15,
            eta: 0.08,
            lambda: 0.0,
            tau_e: 40.0,
            accuracy_floor: 0.65,
            scientific_n_seeds: 20,
            quick: false,
            kill_gate_override: false,
        }
    }

    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.master_seed = 0xC3D2_5EED_0001;
        c.n_seeds = 3;
        c.max_depth = 4;
        c.n_train = 120;
        c.n_test = 80;
        c.quick = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|index| self.master_seed ^ GOLDEN.wrapping_mul(index as u64 + 1))
            .collect()
    }

    pub fn hash_for_arm(&self, arm: C3V2Arm) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, C3_V2_PROTOCOL_VERSION);
        for byte in arm.as_str().as_bytes() {
            mix(&mut h, u64::from(*byte));
        }
        for word in [
            self.master_seed,
            self.n_seeds as u64,
            self.min_depth as u64,
            self.max_depth as u64,
            self.n_states as u64,
            self.n_operations as u64,
            self.n_train as u64,
            self.n_test as u64,
            u64::from(self.init_w.to_bits()),
            u64::from(self.eta.to_bits()),
            u64::from(self.lambda.to_bits()),
            u64::from(self.tau_e.to_bits()),
            u64::from(self.accuracy_floor.to_bits()),
            self.scientific_n_seeds as u64,
            u64::from(self.quick),
        ] {
            mix(&mut h, word);
        }
        h
    }

    pub fn hash_string_for_arm(&self, arm: C3V2Arm) -> String {
        format!(
            "{C3_V2_HASH_PREFIX}{}-{:016x}",
            arm.as_str(),
            self.hash_for_arm(arm)
        )
    }

    pub fn from_hash(hash: &str) -> Option<(Self, C3V2Arm)> {
        for preset in Self::known_presets() {
            for arm in C3V2Arm::ALL {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{C3Config, Config};

    #[test]
    fn v2_hashes_are_separate_from_c1_and_c3_v1() {
        let c = C3V2Config::scientific();
        for arm in C3V2Arm::ALL {
            let hash = c.hash_string_for_arm(arm);
            assert!(hash.starts_with(C3_V2_HASH_PREFIX));
            assert_ne!(hash, Config::c1_default().hash_string());
            assert_ne!(hash, C3Config::c3_default().hash_string());
        }
    }

    #[test]
    fn hashes_round_trip_and_override_is_neutral() {
        for preset in C3V2Config::known_presets() {
            for arm in C3V2Arm::ALL {
                let hash = preset.hash_string_for_arm(arm);
                let (decoded, decoded_arm) = C3V2Config::from_hash(&hash).unwrap();
                assert_eq!(decoded, preset);
                assert_eq!(decoded_arm, arm);
                let mut overridden = preset.clone();
                overridden.kill_gate_override = true;
                assert_eq!(overridden.hash_for_arm(arm), preset.hash_for_arm(arm));
            }
        }
    }

    #[test]
    fn full_uses_twenty_fresh_seeds() {
        let full = C3V2Config::scientific();
        let quick = C3V2Config::quick();
        assert_eq!(full.n_seeds, 20);
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
    }
}
