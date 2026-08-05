//! R2 / U17 experiment config + stable hashing (scaling curve / Gate G4).
//!
//! Separate from C1–R1 hashes. Running R2 requires an explicit kill-gate
//! override (`--enable-r2` / `--override-g2-for r2`); see `results/R2_OVERRIDE.md`.
//!
//! Gate G4 is a **DECISION** gate (not a kill): a healthy non-plateauing curve
//! *justifies the next order of magnitude* — it is **not** proof the curve
//! continues to 10⁴–10⁶ areas.

/// Prefix for R2 config hashes.
pub const R2_HASH_PREFIX: &str = "r2-";

/// Scientific protocol version mixed into every R2 config hash.
pub const R2_PROTOCOL_VERSION: u64 = 1;

/// Experiment name for the default R2 preset.
pub const R2_EXPERIMENT: &str = "r2";

/// Public, hashable R2 configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct R2Config {
    /// Experiment name (`"r2"`, …).
    pub experiment: String,
    /// Master seed.
    pub master_seed: u64,
    /// Number of independent seeds per area-count.
    pub n_seeds: usize,
    /// Inclusive minimum number of areas.
    pub min_areas: usize,
    /// Inclusive maximum number of areas (disclosed sweep ceiling).
    pub max_areas: usize,
    /// Step between area counts.
    pub area_step: usize,
    /// Cells per area (shared with R1 defaults).
    pub cells_per_area: usize,
    /// k-WTA cap.
    pub k_wta: usize,
    /// Intra-area edge probability.
    pub p_intra: f32,
    /// Inter-area edge probability.
    pub p_inter: f32,
    /// Training examples per point.
    pub n_train: usize,
    /// Test examples per point.
    pub n_test: usize,
    /// Learning rate (composed path).
    pub lr: f32,
    /// Plateau detection: relative improvement below this ⇒ plateau.
    pub plateau_rel_eps: f32,
    /// Degrade detection: relative drop beyond this ⇒ degrade.
    pub degrade_rel_eps: f32,
    /// Minimum scientific seeds for a non-PILOT G4 decision.
    pub scientific_n_seeds: usize,
    /// When true, short PILOT schedule.
    pub quick: bool,
    /// Explicit kill-gate override acknowledgment.
    pub kill_gate_override: bool,
}

impl R2Config {
    /// Default scientific R2 schedule (disclosed small range — not 10⁴).
    pub fn r2_default() -> Self {
        Self {
            experiment: R2_EXPERIMENT.into(),
            master_seed: 0xA200_0000_0001,
            n_seeds: 8,
            min_areas: 3,
            max_areas: 24,
            area_step: 3,
            cells_per_area: 16,
            k_wta: 2,
            p_intra: 0.35,
            p_inter: 0.05,
            n_train: 1_500,
            n_test: 300,
            lr: 0.15,
            plateau_rel_eps: 0.02,
            degrade_rel_eps: 0.05,
            scientific_n_seeds: 8,
            quick: false,
            kill_gate_override: false,
        }
    }

    /// Quick/PILOT schedule (tiny disclosed sweep).
    pub fn r2_quick() -> Self {
        let mut c = Self::r2_default();
        c.quick = true;
        c.n_seeds = 2;
        c.min_areas = 3;
        c.max_areas = 9;
        c.area_step = 3;
        c.cells_per_area = 8;
        c.k_wta = 1;
        c.n_train = 200;
        c.n_test = 60;
        c
    }

    /// Known presets for hash round-trips.
    pub fn known_presets() -> Vec<Self> {
        vec![Self::r2_default(), Self::r2_quick()]
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(R2_HASH_PREFIX)
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
        mix(&mut h, R2_PROTOCOL_VERSION);
        for b in self.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.master_seed);
        mix(&mut h, self.n_seeds as u64);
        mix(&mut h, self.min_areas as u64);
        mix(&mut h, self.max_areas as u64);
        mix(&mut h, self.area_step as u64);
        mix(&mut h, self.cells_per_area as u64);
        mix(&mut h, self.k_wta as u64);
        mix(&mut h, self.p_intra.to_bits() as u64);
        mix(&mut h, self.p_inter.to_bits() as u64);
        mix(&mut h, self.n_train as u64);
        mix(&mut h, self.n_test as u64);
        mix(&mut h, self.lr.to_bits() as u64);
        mix(&mut h, self.plateau_rel_eps.to_bits() as u64);
        mix(&mut h, self.degrade_rel_eps.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        h
    }

    /// Hex string form.
    #[inline]
    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", R2_HASH_PREFIX, self.hash())
    }

    /// Seed list.
    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|i| self.master_seed ^ GOLDEN.wrapping_mul(i as u64 + 1))
            .collect()
    }

    /// Area counts in the disclosed sweep.
    pub fn area_counts(&self) -> Vec<usize> {
        let step = self.area_step.max(1);
        let mut v = Vec::new();
        let mut n = self.min_areas;
        while n <= self.max_areas {
            v.push(n);
            n = n.saturating_add(step);
            if n == 0 {
                break;
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::r1_config::R1Config;

    #[test]
    fn r2_hash_diverges_from_c1_and_r1() {
        let r2 = R2Config::r2_default();
        assert_ne!(r2.hash_string(), Config::c1_default().hash_string());
        assert_ne!(r2.hash_string(), R1Config::r1_default().hash_string());
        assert!(r2.hash_string().starts_with("r2-"));
        assert_eq!(Config::c1_default().hash_string(), "c1-118207fbc3eaba53");
    }

    #[test]
    fn area_counts_respect_step() {
        let c = R2Config::r2_quick();
        assert_eq!(c.area_counts(), vec![3, 6, 9]);
    }

    #[test]
    fn from_hash_round_trips() {
        for preset in R2Config::known_presets() {
            assert_eq!(R2Config::from_hash(&preset.hash_string()).unwrap(), preset);
        }
    }

    #[test]
    fn pinned_r2_hashes() {
        assert_eq!(R2Config::r2_default().hash_string(), "r2-afafa0fa6f43e3fc");
        assert_eq!(R2Config::r2_quick().hash_string(), "r2-a35e33f9937b57bd");
    }

    #[test]
    fn print_r2_hashes_for_docs() {
        for p in R2Config::known_presets() {
            eprintln!(
                "R2 HASH {} proto={} quick={}",
                p.hash_string(),
                R2_PROTOCOL_VERSION,
                p.quick
            );
        }
    }
}
