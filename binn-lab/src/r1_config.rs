//! R1 / U16 experiment config + stable hashing (multi-area composition).
//!
//! Separate from C1/C2/C3 hashes. Running R1 requires an explicit kill-gate
//! override (`--enable-r1` / `--override-g2-for r1`); see `results/R1_OVERRIDE.md`.

/// Prefix for R1 config hashes.
pub const R1_HASH_PREFIX: &str = "r1-";

/// Scientific protocol version mixed into every R1 config hash.
pub const R1_PROTOCOL_VERSION: u64 = 1;

/// Experiment name for the default R1 preset.
pub const R1_EXPERIMENT: &str = "r1";

/// Public, hashable R1 configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct R1Config {
    /// Experiment name (`"r1"`, …).
    pub experiment: String,
    /// Master seed.
    pub master_seed: u64,
    /// Number of independent seeds.
    pub n_seeds: usize,
    /// Inclusive minimum number of areas (composition sweep start).
    pub min_areas: usize,
    /// Inclusive maximum number of areas (composition sweep end).
    pub max_areas: usize,
    /// Cells per area.
    pub cells_per_area: usize,
    /// k-WTA cap per area.
    pub k_wta: usize,
    /// Intra-area edge probability.
    pub p_intra: f32,
    /// Inter-area edge probability (base; hub role modulates).
    pub p_inter: f32,
    /// Training examples per (n_areas × seed).
    pub n_train: usize,
    /// Test examples per (n_areas × seed).
    pub n_test: usize,
    /// Local / composed learning rate.
    pub lr: f32,
    /// Additive late-fusion learning rate (matched budget disclosure).
    pub additive_lr: f32,
    /// Margin by which composed accuracy must exceed additive to count as compounding.
    pub compound_margin: f32,
    /// Minimum scientific seeds for a non-PILOT claim.
    pub scientific_n_seeds: usize,
    /// When true, short PILOT schedule.
    pub quick: bool,
    /// Explicit kill-gate override acknowledgment.
    pub kill_gate_override: bool,
}

impl R1Config {
    /// Default scientific R1 schedule (3→10 areas).
    pub fn r1_default() -> Self {
        Self {
            experiment: R1_EXPERIMENT.into(),
            master_seed: 0xA100_0000_0001,
            n_seeds: 8,
            min_areas: 3,
            max_areas: 10,
            cells_per_area: 16,
            k_wta: 2,
            p_intra: 0.35,
            p_inter: 0.05,
            n_train: 2_000,
            n_test: 400,
            lr: 0.15,
            additive_lr: 0.15,
            compound_margin: 0.05,
            scientific_n_seeds: 8,
            quick: false,
            kill_gate_override: false,
        }
    }

    /// Quick/PILOT schedule (3→5 areas).
    pub fn r1_quick() -> Self {
        let mut c = Self::r1_default();
        c.quick = true;
        c.n_seeds = 3;
        c.max_areas = 5;
        c.cells_per_area = 8;
        c.k_wta = 1;
        c.n_train = 400;
        c.n_test = 100;
        c
    }

    /// Known presets for hash round-trips.
    pub fn known_presets() -> Vec<Self> {
        vec![Self::r1_default(), Self::r1_quick()]
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(R1_HASH_PREFIX)
            .to_lowercase();
        for preset in Self::known_presets() {
            if h == format!("{:016x}", preset.hash()) {
                return Some(preset);
            }
        }
        None
    }

    /// Stable FNV-1a fingerprint. `kill_gate_override` is not mixed in.
    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, R1_PROTOCOL_VERSION);
        for b in self.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.master_seed);
        mix(&mut h, self.n_seeds as u64);
        mix(&mut h, self.min_areas as u64);
        mix(&mut h, self.max_areas as u64);
        mix(&mut h, self.cells_per_area as u64);
        mix(&mut h, self.k_wta as u64);
        mix(&mut h, self.p_intra.to_bits() as u64);
        mix(&mut h, self.p_inter.to_bits() as u64);
        mix(&mut h, self.n_train as u64);
        mix(&mut h, self.n_test as u64);
        mix(&mut h, self.lr.to_bits() as u64);
        mix(&mut h, self.additive_lr.to_bits() as u64);
        mix(&mut h, self.compound_margin.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        h
    }

    /// Hex string form.
    #[inline]
    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", R1_HASH_PREFIX, self.hash())
    }

    /// Seed list.
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
    use crate::c3_config::C3Config;
    use crate::config::Config;

    #[test]
    fn r1_hash_diverges_from_c1_c2_c3() {
        let r1 = R1Config::r1_default();
        assert_ne!(r1.hash_string(), Config::c1_default().hash_string());
        assert_ne!(r1.hash_string(), C2Config::c2_default().hash_string());
        assert_ne!(r1.hash_string(), C3Config::c3_default().hash_string());
        assert!(r1.hash_string().starts_with("r1-"));
        assert_eq!(Config::c1_default().hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn r1_hash_override_neutral() {
        let a = R1Config::r1_default();
        let mut b = a.clone();
        b.kill_gate_override = true;
        assert_eq!(a.hash(), b.hash());
    }

    #[test]
    fn from_hash_round_trips() {
        for preset in R1Config::known_presets() {
            assert_eq!(R1Config::from_hash(&preset.hash_string()).unwrap(), preset);
        }
    }

    #[test]
    fn pinned_r1_hashes() {
        assert_eq!(R1Config::r1_default().hash_string(), "r1-5d30383e334b9cbe");
        assert_eq!(R1Config::r1_quick().hash_string(), "r1-ab69e1b6eb9b98e6");
    }

    #[test]
    fn print_r1_hashes_for_docs() {
        for p in R1Config::known_presets() {
            eprintln!(
                "R1 HASH {} proto={} quick={}",
                p.hash_string(),
                R1_PROTOCOL_VERSION,
                p.quick
            );
        }
    }
}
