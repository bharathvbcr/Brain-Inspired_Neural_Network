//! Real SuperSpike BPTT on the production C3 credit-depth graph.
//!
//! Separate from frozen `c3v2-*` (oracle-pulse matched gradient is **not** BPTT).
//! Uses prefix `c3-bptt-*` and fresh held-out seeds.

pub const C3_BPTT_HASH_PREFIX: &str = "c3-bptt-";
pub const C3_BPTT_PROTOCOL_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum C3BpttArm {
    /// Surrogate BPTT through layer transitions (no oracle correction pulses).
    SuperSpikeBptt,
    /// Oracle target pulses + STDP credit (contrast; labeled not-BPTT).
    OraclePulses,
}

impl C3BpttArm {
    pub const ALL: [Self; 2] = [Self::SuperSpikeBptt, Self::OraclePulses];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SuperSpikeBptt => "superspike-bptt",
            Self::OraclePulses => "oracle-pulses",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|arm| arm.as_str() == value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3BpttConfig {
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
    pub surrogate_beta: f32,
    pub accuracy_floor: f32,
    pub scientific_n_seeds: usize,
    pub quick: bool,
    pub kill_gate_override: bool,
}

impl C3BpttConfig {
    pub fn scientific() -> Self {
        Self {
            master_seed: 0xC3B7_5C1E_0001,
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
            surrogate_beta: 5.0,
            accuracy_floor: 0.65,
            scientific_n_seeds: 20,
            quick: false,
            kill_gate_override: false,
        }
    }

    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.master_seed = 0xC3B7_D3ED_0001;
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

    pub fn hash_for_arm(&self, arm: C3BpttArm) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, C3_BPTT_PROTOCOL_VERSION);
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
            u64::from(self.surrogate_beta.to_bits()),
            u64::from(self.accuracy_floor.to_bits()),
            self.scientific_n_seeds as u64,
            u64::from(self.quick),
        ] {
            mix(&mut h, word);
        }
        h
    }

    pub fn hash_string_for_arm(&self, arm: C3BpttArm) -> String {
        format!(
            "{C3_BPTT_HASH_PREFIX}{}-{:016x}",
            arm.as_str(),
            self.hash_for_arm(arm)
        )
    }

    pub fn from_hash(hash: &str) -> Option<(Self, C3BpttArm)> {
        for preset in Self::known_presets() {
            for arm in C3BpttArm::ALL {
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
    use crate::{C3V2Arm, C3V2Config, Config};

    #[test]
    fn hashes_are_separate_from_c3v2_and_c1() {
        let cfg = C3BpttConfig::scientific();
        let v2 = C3V2Config::scientific();
        for arm in C3BpttArm::ALL {
            let hash = cfg.hash_string_for_arm(arm);
            assert!(hash.starts_with(C3_BPTT_HASH_PREFIX));
            assert_ne!(hash, Config::c1_default().hash_string());
            for v2_arm in C3V2Arm::ALL {
                assert_ne!(hash, v2.hash_string_for_arm(v2_arm));
            }
        }
    }

    #[test]
    fn arms_differ_and_round_trip() {
        let cfg = C3BpttConfig::scientific();
        assert_ne!(
            cfg.hash_string_for_arm(C3BpttArm::SuperSpikeBptt),
            cfg.hash_string_for_arm(C3BpttArm::OraclePulses)
        );
        for preset in C3BpttConfig::known_presets() {
            for arm in C3BpttArm::ALL {
                let hash = preset.hash_string_for_arm(arm);
                let (decoded, decoded_arm) = C3BpttConfig::from_hash(&hash).unwrap();
                assert_eq!(decoded, preset);
                assert_eq!(decoded_arm, arm);
            }
        }
    }

    #[test]
    fn kill_gate_override_is_neutral_to_hash() {
        let cfg = C3BpttConfig::scientific();
        let mut overridden = cfg.clone();
        overridden.kill_gate_override = true;
        for arm in C3BpttArm::ALL {
            assert_eq!(
                overridden.hash_for_arm(arm),
                cfg.hash_for_arm(arm),
                "override must not shift preregistered hash"
            );
        }
    }
}
