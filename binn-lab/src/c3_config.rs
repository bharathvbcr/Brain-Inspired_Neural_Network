//! C3 / U15 experiment config + stable hashing (credit-depth path).
//!
//! Separate from C1 protocol-v2 and C2 — must **never** alias
//! `c1-118207fbc3eaba53` or C2 hashes. Running C3 requires an explicit
//! kill-gate override (`--enable-c3` / `--override-g2-for c3`); see
//! `results/C3_OVERRIDE.md`.

/// Prefix for C3 config hashes (distinct from `c1-` / `c2-`).
pub const C3_HASH_PREFIX: &str = "c3-";

/// Scientific protocol version mixed into every C3 config hash.
pub const C3_PROTOCOL_VERSION: u64 = 1;

/// Experiment name for the default C3 preset.
pub const C3_EXPERIMENT: &str = "c3";

/// Public, hashable C3 configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct C3Config {
    /// Experiment name (`"c3"`, …).
    pub experiment: String,
    /// Master seed; per-seed runs use `master_seed ^ seed_i`.
    pub master_seed: u64,
    /// Number of independent seeds.
    pub n_seeds: usize,
    /// Inclusive minimum compositional depth.
    pub min_depth: usize,
    /// Inclusive maximum compositional depth.
    pub max_depth: usize,
    /// Discrete hidden-state cardinality.
    pub n_states: usize,
    /// Operations available per layer.
    pub n_operations: usize,
    /// Training examples per depth × seed.
    pub n_train: usize,
    /// Test examples per depth × seed.
    pub n_test: usize,
    /// Local three-factor / eligibility step size `η`.
    pub eta: f32,
    /// Per-layer eligibility decay (terminal reward → earlier layers).
    pub eligibility_decay: f32,
    /// ε-greedy exploration during local training.
    pub exploration: f32,
    /// Disclosed gradient-reference learning rate.
    pub gradient_lr: f32,
    /// Accuracy floor used to define `D*`.
    pub accuracy_floor: f32,
    /// Minimum scientific seeds for a non-PILOT depth claim.
    pub scientific_n_seeds: usize,
    /// When true, short PILOT schedule (never a scientific D* claim alone).
    pub quick: bool,
    /// Explicit acknowledgment that this run overrides the v8 G2 kill-gate.
    pub kill_gate_override: bool,
}

impl C3Config {
    /// Default scientific C3 schedule (still requires CLI override to run).
    pub fn c3_default() -> Self {
        Self {
            experiment: C3_EXPERIMENT.into(),
            master_seed: 0xC300_0000_0001,
            n_seeds: 10,
            min_depth: 1,
            max_depth: 8,
            n_states: 4,
            n_operations: 2,
            n_train: 5_000,
            n_test: 1_000,
            eta: 0.08,
            eligibility_decay: 0.72,
            exploration: 0.12,
            gradient_lr: 0.20,
            accuracy_floor: 0.65,
            scientific_n_seeds: 10,
            quick: false,
            kill_gate_override: false,
        }
    }

    /// Quick/PILOT schedule for CI + smoke.
    pub fn c3_quick() -> Self {
        let mut c = Self::c3_default();
        c.quick = true;
        c.n_seeds = 3;
        c.max_depth = 4;
        c.n_train = 600;
        c.n_test = 200;
        c
    }

    /// Known presets for hash round-trips.
    pub fn known_presets() -> Vec<Self> {
        vec![Self::c3_default(), Self::c3_quick()]
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(C3_HASH_PREFIX)
            .to_lowercase();
        for preset in Self::known_presets() {
            if h == format!("{:016x}", preset.hash()) {
                return Some(preset);
            }
        }
        None
    }

    /// Stable FNV-1a 64-bit fingerprint (protocol + fields).
    ///
    /// `kill_gate_override` is **not** mixed in — it is a CLI gate, not a
    /// scientific knob.
    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, C3_PROTOCOL_VERSION);
        for b in self.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.master_seed);
        mix(&mut h, self.n_seeds as u64);
        mix(&mut h, self.min_depth as u64);
        mix(&mut h, self.max_depth as u64);
        mix(&mut h, self.n_states as u64);
        mix(&mut h, self.n_operations as u64);
        mix(&mut h, self.n_train as u64);
        mix(&mut h, self.n_test as u64);
        mix(&mut h, self.eta.to_bits() as u64);
        mix(&mut h, self.eligibility_decay.to_bits() as u64);
        mix(&mut h, self.exploration.to_bits() as u64);
        mix(&mut h, self.gradient_lr.to_bits() as u64);
        mix(&mut h, self.accuracy_floor.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        h
    }

    /// Hex string form used in logs and results notes.
    #[inline]
    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", C3_HASH_PREFIX, self.hash())
    }

    /// Seed list of length `n_seeds`.
    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|i| self.master_seed ^ GOLDEN.wrapping_mul(i as u64 + 1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c2_config::C2Config;
    use crate::config::Config;

    #[test]
    fn c3_hash_diverges_from_c1_and_c2() {
        let c3 = C3Config::c3_default();
        let c1 = Config::c1_default();
        let c2 = C2Config::c2_default();
        assert_ne!(c3.hash_string(), c1.hash_string());
        assert_ne!(c3.hash_string(), c2.hash_string());
        assert!(c3.hash_string().starts_with("c3-"));
        assert_eq!(c1.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(c2.hash_string(), "c2-c45f08841f2f9df9");
    }

    #[test]
    fn c3_hash_stable_and_override_neutral() {
        let a = C3Config::c3_default();
        let b = C3Config::c3_default();
        assert_eq!(a.hash(), b.hash());
        let mut c = C3Config::c3_default();
        c.eta *= 1.01;
        assert_ne!(a.hash(), c.hash());
        let mut d = C3Config::c3_default();
        d.kill_gate_override = true;
        assert_eq!(a.hash(), d.hash());
    }

    #[test]
    fn from_hash_round_trips() {
        for preset in C3Config::known_presets() {
            assert_eq!(C3Config::from_hash(&preset.hash_string()).unwrap(), preset);
        }
    }

    #[test]
    fn pinned_c3_hashes() {
        assert_eq!(C3Config::c3_default().hash_string(), "c3-445aa8de7761d4f4");
        assert_eq!(C3Config::c3_quick().hash_string(), "c3-adf27f8ffc4185ca");
    }

    #[test]
    fn print_c3_hashes_for_docs() {
        for p in C3Config::known_presets() {
            eprintln!(
                "C3 HASH {} proto={} quick={}",
                p.hash_string(),
                C3_PROTOCOL_VERSION,
                p.quick
            );
        }
    }
}
