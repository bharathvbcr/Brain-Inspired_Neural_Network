//! R2 directed-credit mitigation probe (`r2-credit-*`).
//!
//! Same disclosed #areas sweep grid as frozen R2 / Gate G4, but replaces
//! broadcast ±1 three-factor credit with **graded DFA** and/or
//! **REINFORCE × frozen B** (directed). Optional 1-seed broadcast ±1 smoke
//! control for harness sanity.
//!
//! **Does not reopen** frozen G4 NO-GO (`r2-afafa0fa6f43e3fc`) or G2 FAIL
//! (`c1-118207fbc3eaba53`). Distinct hash family; kill-gate override still
//! required (`--enable-r2`).

use crate::r2_config::R2Config;

/// Prefix for R2-credit config hashes (distinct from frozen `r2-`).
pub const R2_CREDIT_HASH_PREFIX: &str = "r2-credit-";

/// Scientific protocol version mixed into every R2-credit config hash.
pub const R2_CREDIT_PROTOCOL_VERSION: u64 = 1;

/// Experiment name for the default R2-credit preset.
pub const R2_CREDIT_EXPERIMENT: &str = "r2-credit";

/// Credit arm identities mixed into the hash and reported in notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum R2CreditArm {
    /// Broadcast scalar error (frozen-R2-faithful ±1 / logistic err).
    BroadcastPm1,
    /// Graded supervised error × fixed-random DFA feedback (directed).
    GradedDfa,
    /// REINFORCE `r·(a−p)` × frozen per-area feedback `B` (directed).
    ReinforceFb,
}

impl R2CreditArm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BroadcastPm1 => "broadcast-pm1",
            Self::GradedDfa => "graded-dfa",
            Self::ReinforceFb => "reinforce-fb",
        }
    }
}

/// Public, hashable R2-credit configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct R2CreditConfig {
    /// Experiment name (`"r2-credit"`, …).
    pub experiment: String,
    /// Master seed (distinct lineage from frozen R2).
    pub master_seed: u64,
    /// Number of independent seeds per area-count (directed arms).
    pub n_seeds: usize,
    /// Inclusive minimum number of areas (matches R2 grid).
    pub min_areas: usize,
    /// Inclusive maximum number of areas.
    pub max_areas: usize,
    /// Step between area counts.
    pub area_step: usize,
    /// Cells per area.
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
    /// Plateau detection ε (shared with R2 classifier).
    pub plateau_rel_eps: f32,
    /// Degrade detection ε.
    pub degrade_rel_eps: f32,
    /// Minimum scientific seeds for a non-PILOT reading.
    pub scientific_n_seeds: usize,
    /// When true, short PILOT schedule.
    pub quick: bool,
    /// Run graded-DFA directed arm.
    pub run_graded_dfa: bool,
    /// Run REINFORCE×frozen-B directed arm.
    pub run_reinforce_fb: bool,
    /// When true, also run a 1-seed broadcast ±1 smoke control.
    pub include_pm1_smoke: bool,
    /// Explicit kill-gate override acknowledgment (not mixed into hash).
    pub kill_gate_override: bool,
}

impl R2CreditConfig {
    /// Scientific R2-credit schedule — same area grid as [`R2Config::r2_default`].
    pub fn scientific() -> Self {
        let r2 = R2Config::r2_default();
        Self {
            experiment: R2_CREDIT_EXPERIMENT.into(),
            // Fresh lineage vs frozen R2 (`0xA200_0000_0001`).
            master_seed: 0xA2C1_ED17_0001,
            n_seeds: r2.n_seeds,
            min_areas: r2.min_areas,
            max_areas: r2.max_areas,
            area_step: r2.area_step,
            cells_per_area: r2.cells_per_area,
            k_wta: r2.k_wta,
            p_intra: r2.p_intra,
            p_inter: r2.p_inter,
            n_train: r2.n_train,
            n_test: r2.n_test,
            lr: r2.lr,
            plateau_rel_eps: r2.plateau_rel_eps,
            degrade_rel_eps: r2.degrade_rel_eps,
            scientific_n_seeds: r2.scientific_n_seeds,
            quick: false,
            run_graded_dfa: true,
            run_reinforce_fb: true,
            include_pm1_smoke: true,
            kill_gate_override: false,
        }
    }

    /// Quick/PILOT schedule — same area grid as [`R2Config::r2_quick`].
    pub fn quick() -> Self {
        let r2 = R2Config::r2_quick();
        let mut c = Self::scientific();
        c.experiment = format!("{R2_CREDIT_EXPERIMENT}-quick");
        c.master_seed = 0xA2C1_D3ED_0001;
        c.quick = true;
        c.n_seeds = r2.n_seeds;
        c.min_areas = r2.min_areas;
        c.max_areas = r2.max_areas;
        c.area_step = r2.area_step;
        c.cells_per_area = r2.cells_per_area;
        c.k_wta = r2.k_wta;
        c.n_train = r2.n_train;
        c.n_test = r2.n_test;
        c.include_pm1_smoke = true;
        c
    }

    /// Known presets for hash round-trips.
    pub fn known_presets() -> Vec<Self> {
        vec![Self::scientific(), Self::quick()]
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(R2_CREDIT_HASH_PREFIX)
            .to_lowercase();
        for preset in Self::known_presets() {
            if h == format!("{:016x}", preset.hash()) {
                return Some(preset);
            }
        }
        None
    }

    /// Directed arms scheduled for this preset (excludes optional ±1 smoke).
    pub fn directed_arms(&self) -> Vec<R2CreditArm> {
        let mut arms = Vec::new();
        if self.run_graded_dfa {
            arms.push(R2CreditArm::GradedDfa);
        }
        if self.run_reinforce_fb {
            arms.push(R2CreditArm::ReinforceFb);
        }
        arms
    }

    /// Stable FNV-1a fingerprint. `kill_gate_override` is not mixed in.
    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, R2_CREDIT_PROTOCOL_VERSION);
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
        mix(&mut h, u64::from(self.run_graded_dfa));
        mix(&mut h, u64::from(self.run_reinforce_fb));
        mix(&mut h, u64::from(self.include_pm1_smoke));
        h
    }

    /// Hex string form.
    #[inline]
    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", R2_CREDIT_HASH_PREFIX, self.hash())
    }

    /// Seed list for directed arms.
    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|i| self.master_seed ^ GOLDEN.wrapping_mul(i as u64 + 1))
            .collect()
    }

    /// Single seed used by the optional broadcast ±1 smoke control.
    pub fn pm1_smoke_seed(&self) -> u64 {
        self.master_seed ^ 0x00B1_0001_5A0E_u64
    }

    /// Area counts in the disclosed sweep (identical grid construction to R2).
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
    use crate::r2_config::R2Config;

    #[test]
    fn credit_hash_diverges_from_frozen_r2() {
        let credit = R2CreditConfig::scientific();
        let r2 = R2Config::r2_default();
        assert_ne!(credit.hash_string(), r2.hash_string());
        assert!(credit.hash_string().starts_with("r2-credit-"));
        assert_eq!(r2.hash_string(), "r2-afafa0fa6f43e3fc");
    }

    #[test]
    fn area_grid_matches_r2() {
        assert_eq!(
            R2CreditConfig::scientific().area_counts(),
            R2Config::r2_default().area_counts()
        );
        assert_eq!(
            R2CreditConfig::quick().area_counts(),
            R2Config::r2_quick().area_counts()
        );
    }

    #[test]
    fn from_hash_round_trips() {
        for preset in R2CreditConfig::known_presets() {
            assert_eq!(
                R2CreditConfig::from_hash(&preset.hash_string()).unwrap(),
                preset
            );
        }
    }

    #[test]
    fn pinned_r2_credit_hashes() {
        assert_eq!(
            R2CreditConfig::scientific().hash_string(),
            "r2-credit-2f5647981724c62b"
        );
        assert_eq!(
            R2CreditConfig::quick().hash_string(),
            "r2-credit-eaa83da10229dd22"
        );
    }

    #[test]
    fn print_r2_credit_hashes_for_docs() {
        for p in R2CreditConfig::known_presets() {
            eprintln!(
                "R2-CREDIT HASH {} proto={} quick={} arms={:?}",
                p.hash_string(),
                R2_CREDIT_PROTOCOL_VERSION,
                p.quick,
                p.directed_arms()
            );
        }
    }
}
