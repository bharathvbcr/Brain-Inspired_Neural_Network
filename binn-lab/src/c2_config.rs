//! C2 / U14 experiment config + stable hashing (Gate G3 path).
//!
//! Separate from C1 protocol-v2 — must **never** alias hash `c1-118207fbc3eaba53`.
//! Running C2 requires an explicit kill-gate override (`--enable-c2` /
//! `--override-g2-for c2`); see `results/C2_OVERRIDE.md`.

use binn_data::ClassIncConfig;

/// Prefix for C2 config hashes (distinct from `c1-`).
pub const C2_HASH_PREFIX: &str = "c2-";

/// Scientific protocol version mixed into every C2 config hash.
pub const C2_PROTOCOL_VERSION: u64 = 1;

/// Experiment name for the default C2 preset.
pub const C2_EXPERIMENT: &str = "c2";

/// Public, hashable C2 configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct C2Config {
    /// Experiment name (`"c2"`, …).
    pub experiment: String,
    /// Master seed; per-seed runs use `master_seed ^ seed_i`.
    pub master_seed: u64,
    /// Number of independent seeds.
    pub n_seeds: usize,
    /// Hidden population size.
    pub n_hidden: usize,
    /// k-WTA cap.
    pub k_wta: usize,
    /// Sparse intra-area edge probability.
    pub p_sparse: f32,
    /// Initial synaptic weight.
    pub init_w: f32,
    /// Three-factor `η`.
    pub eta: f32,
    /// Three-factor `λ`.
    pub lambda: f32,
    /// Eligibility `τ_e`.
    pub tau_e: f32,
    /// Class-incremental stream knobs.
    pub stream: ClassIncConfig,
    /// Replay capacity for the labeled gradient baseline (raw examples).
    pub baseline_replay_capacity: usize,
    /// Baseline learning rate.
    pub baseline_lr: f32,
    /// Minimum scientific seeds for a non-PILOT G3 decision.
    pub scientific_n_seeds: usize,
    /// When true, short PILOT schedule (never a scientific PASS/FAIL alone).
    pub quick: bool,
    /// Explicit acknowledgment that this run overrides the v8 G2 kill-gate.
    pub kill_gate_override: bool,
}

impl C2Config {
    /// Default scientific C2 schedule (still requires CLI override to run).
    pub fn c2_default() -> Self {
        Self {
            experiment: C2_EXPERIMENT.into(),
            master_seed: 0xC200_0000_0001,
            n_seeds: 10,
            n_hidden: 64,
            k_wta: 2,
            p_sparse: 0.35,
            init_w: 0.15,
            eta: 0.30,
            lambda: 0.002,
            tau_e: 40.0,
            stream: ClassIncConfig::scientific(0xC200_57EA),
            baseline_replay_capacity: 64,
            baseline_lr: 0.25,
            scientific_n_seeds: 10,
            quick: false,
            kill_gate_override: false,
        }
    }

    /// Quick/PILOT schedule for CI + smoke.
    pub fn c2_quick() -> Self {
        let mut c = Self::c2_default();
        c.quick = true;
        c.n_seeds = 3;
        c.n_hidden = 32;
        c.k_wta = 1;
        c.stream = ClassIncConfig::quick(0xC200_001C);
        c.baseline_replay_capacity = 24;
        c.baseline_lr = 0.35;
        c
    }

    /// Known presets for hash round-trips.
    pub fn known_presets() -> Vec<Self> {
        vec![Self::c2_default(), Self::c2_quick()]
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(C2_HASH_PREFIX)
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
    /// scientific knob — so enabling the override does not change the hash.
    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, C2_PROTOCOL_VERSION);
        for b in self.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.master_seed);
        mix(&mut h, self.n_seeds as u64);
        mix(&mut h, self.n_hidden as u64);
        mix(&mut h, self.k_wta as u64);
        mix(&mut h, self.p_sparse.to_bits() as u64);
        mix(&mut h, self.init_w.to_bits() as u64);
        mix(&mut h, self.eta.to_bits() as u64);
        mix(&mut h, self.lambda.to_bits() as u64);
        mix(&mut h, self.tau_e.to_bits() as u64);
        mix(&mut h, self.stream.fingerprint());
        mix(&mut h, self.baseline_replay_capacity as u64);
        mix(&mut h, self.baseline_lr.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        h
    }

    /// Hex string form used in logs and results notes.
    #[inline]
    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", C2_HASH_PREFIX, self.hash())
    }

    /// Seed list of length `n_seeds`.
    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|i| self.master_seed ^ GOLDEN.wrapping_mul(i as u64 + 1))
            .collect()
    }

    /// Nominal activity fraction `k_wta / n_hidden`.
    #[inline]
    pub fn nominal_activity_fraction(&self) -> f32 {
        if self.n_hidden == 0 {
            0.0
        } else {
            self.k_wta as f32 / self.n_hidden as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn c2_hash_diverges_from_c1_kill_gate() {
        let c2 = C2Config::c2_default();
        let c1 = Config::c1_default();
        assert_ne!(c2.hash_string(), c1.hash_string());
        assert!(c2.hash_string().starts_with("c2-"));
        assert_eq!(c1.hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn c2_hash_stable_and_sensitive() {
        let a = C2Config::c2_default();
        let b = C2Config::c2_default();
        assert_eq!(a.hash(), b.hash());
        let mut c = C2Config::c2_default();
        c.eta *= 1.01;
        assert_ne!(a.hash(), c.hash());
        let mut d = C2Config::c2_default();
        d.kill_gate_override = true;
        assert_eq!(
            a.hash(),
            d.hash(),
            "override flag must not change scientific hash"
        );
    }

    #[test]
    fn from_hash_round_trips() {
        for preset in C2Config::known_presets() {
            assert_eq!(C2Config::from_hash(&preset.hash_string()).unwrap(), preset);
        }
    }

    #[test]
    fn pinned_c2_hashes() {
        assert_eq!(C2Config::c2_default().hash_string(), "c2-c45f08841f2f9df9");
        assert_eq!(C2Config::c2_quick().hash_string(), "c2-ddc6176952829d90");
    }

    #[test]
    fn print_c2_hashes_for_docs() {
        for p in C2Config::known_presets() {
            eprintln!(
                "C2 HASH {} proto={} quick={}",
                p.hash_string(),
                C2_PROTOCOL_VERSION,
                p.quick
            );
        }
    }
}
