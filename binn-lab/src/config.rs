//! Experiment config + stable config hashing (U13).

/// Prefix used when printing the default C1 config hash in docs / notes.
pub const C1_DEFAULT_HASH_PREFIX: &str = "c1-";

/// Public, hashable C1 / harness configuration.
///
/// Identical field values ⇒ identical [`Config::hash`] (GC3). Changing any
/// scientific knob changes the hash so a results note can cite it.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Experiment name (`"c1"`, …).
    pub experiment: String,
    /// Master seed; per-seed runs use `master_seed ^ seed_i`.
    pub master_seed: u64,
    /// Number of independent seeds (five is pilot-only; default uses twenty).
    pub n_seeds: usize,
    /// Coincidence sequence length (frames).
    pub sequence_len: usize,
    /// Max lag counted as coincident.
    pub max_lag: usize,
    /// Hidden population size (assembly / dense share this N).
    pub n_hidden: usize,
    /// k-WTA cap for the local-assembly hidden area.
    pub k_wta: usize,
    /// Sparse intra-area edge probability (local-assembly).
    pub p_sparse: f32,
    /// Initial synaptic weight.
    pub init_w: f32,
    /// Three-factor `η`.
    pub eta: f32,
    /// Three-factor `λ`.
    pub lambda: f32,
    /// Eligibility `τ_e`.
    pub tau_e: f32,
    /// Training trials per seed.
    pub n_train: usize,
    /// Held-out test trials per seed.
    pub n_test: usize,
    /// BPTT / surrogate-LIF epochs (labeled gradient reference).
    pub bptt_epochs: usize,
    /// BPTT / surrogate-LIF learning rate.
    pub bptt_lr: f32,
    /// Required lower confidence bound on the fraction of gradient gap closed.
    pub g2_min_gap_closed: f32,
    /// Absolute local-assembly accuracy floor for a positive verdict.
    pub g2_min_accuracy: f32,
    /// Critical value used for the preregistered lower confidence bound.
    pub g2_confidence_z: f32,
    /// Minimum mean positive-control accuracy for a valid harness (else INVALID_HARNESS).
    pub g2_min_positive_control: f32,
    /// Inclusive lower bound on mean activity sparsity for a valid harness.
    pub activity_sparsity_min: f32,
    /// Inclusive upper bound on mean activity sparsity for a valid harness.
    pub activity_sparsity_max: f32,
    /// Minimum seeds required for a scientific (non-Pilot) G2 decision.
    pub scientific_n_seeds: usize,
    /// Preregistered prior σ of `gap_closed` for power analysis (hashed).
    pub power_sigma_prior: f32,
    /// Preregistered detectable effect size for power analysis (hashed).
    pub power_effect_size: f32,
    /// Use same-architecture surrogate-LIF as primary gradient reference.
    pub use_surrogate_lif_reference: bool,
    /// Surrogate steepness β (ignored when surrogate-LIF is off).
    pub surrogate_beta: f32,
    /// When true, re-run dense-local under a parameter-matched edge budget.
    pub matched_budget_repeat: bool,
    /// When true, use the short deterministic smoke schedule (CI / unit tests).
    pub quick: bool,
}

impl Config {
    /// Default C1 scientific config (≥20 seeds, ~1.56% activity, full trial counts).
    pub fn c1_default() -> Self {
        Self {
            experiment: "c1".into(),
            master_seed: 0xC160_0000_0001,
            n_seeds: 20,
            sequence_len: 8,
            max_lag: 1,
            n_hidden: 128,
            k_wta: 2,
            p_sparse: 0.35,
            init_w: 0.15,
            eta: 0.35,
            lambda: 0.002,
            tau_e: 40.0,
            n_train: 80,
            n_test: 40,
            bptt_epochs: 80,
            bptt_lr: 0.05,
            g2_min_gap_closed: 0.5,
            g2_min_accuracy: 0.65,
            g2_confidence_z: 1.96,
            g2_min_positive_control: 0.90,
            activity_sparsity_min: 0.005,
            activity_sparsity_max: 0.03,
            scientific_n_seeds: 20,
            power_sigma_prior: 0.15,
            power_effect_size: 0.10,
            use_surrogate_lif_reference: true,
            surrogate_beta: 5.0,
            matched_budget_repeat: true,
            quick: false,
        }
    }

    /// Short deterministic schedule for unit tests / CI (still ≥5 seeds).
    ///
    /// Keeps `k/N ≈ 0.015` (N=64, k=1) so sparsity stays in the scientific band.
    pub fn c1_quick() -> Self {
        let mut c = Self::c1_default();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(C1_DEFAULT_HASH_PREFIX)
            .to_lowercase();
        let def = Self::c1_default();
        let quick = Self::c1_quick();
        if h == format!("{:016x}", def.hash()) {
            return Some(def);
        }
        if h == format!("{:016x}", quick.hash()) {
            return Some(quick);
        }
        None
    }

    /// Stable FNV-1a 64-bit fingerprint of every public field.
    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        for b in self.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.master_seed);
        mix(&mut h, self.n_seeds as u64);
        mix(&mut h, self.sequence_len as u64);
        mix(&mut h, self.max_lag as u64);
        mix(&mut h, self.n_hidden as u64);
        mix(&mut h, self.k_wta as u64);
        mix(&mut h, self.p_sparse.to_bits() as u64);
        mix(&mut h, self.init_w.to_bits() as u64);
        mix(&mut h, self.eta.to_bits() as u64);
        mix(&mut h, self.lambda.to_bits() as u64);
        mix(&mut h, self.tau_e.to_bits() as u64);
        mix(&mut h, self.n_train as u64);
        mix(&mut h, self.n_test as u64);
        mix(&mut h, self.bptt_epochs as u64);
        mix(&mut h, self.bptt_lr.to_bits() as u64);
        mix(&mut h, self.g2_min_gap_closed.to_bits() as u64);
        mix(&mut h, self.g2_min_accuracy.to_bits() as u64);
        mix(&mut h, self.g2_confidence_z.to_bits() as u64);
        mix(&mut h, self.g2_min_positive_control.to_bits() as u64);
        mix(&mut h, self.activity_sparsity_min.to_bits() as u64);
        mix(&mut h, self.activity_sparsity_max.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, self.power_sigma_prior.to_bits() as u64);
        mix(&mut h, self.power_effect_size.to_bits() as u64);
        mix(&mut h, u64::from(self.use_surrogate_lif_reference));
        mix(&mut h, self.surrogate_beta.to_bits() as u64);
        mix(&mut h, u64::from(self.matched_budget_repeat));
        mix(&mut h, u64::from(self.quick));
        h
    }

    /// Hex string form used in logs and results notes.
    #[inline]
    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", C1_DEFAULT_HASH_PREFIX, self.hash())
    }

    /// Seed list of length `n_seeds` derived from `master_seed`.
    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|i| self.master_seed ^ GOLDEN.wrapping_mul(i as u64 + 1))
            .collect()
    }

    /// Preregistered one-sample normal-approximation seed count for 80% power
    /// at two-sided α≈0.05 against `power_effect_size` with prior σ.
    ///
    /// Formula (hashed via the config fields it depends on):
    /// `n = ceil((z_{1-α/2} + z_{1-β})² · σ² / δ²)` with
    /// `z_{1-α/2}=1.96`, `z_{0.80}=0.8416`.
    pub fn n_seeds_for_80_power(&self) -> usize {
        n_for_80_percent_power(self.power_sigma_prior, self.power_effect_size)
    }

    /// Required scientific seed count: `max(scientific_n_seeds, n_for_80%_power)`.
    pub fn required_scientific_n_seeds(&self) -> usize {
        self.scientific_n_seeds
            .max(20)
            .max(self.n_seeds_for_80_power())
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

/// `n = ceil((1.96 + 0.8416)² · σ² / δ²)` for a one-sample mean test.
pub fn n_for_80_percent_power(sigma: f32, effect: f32) -> usize {
    assert!(sigma.is_finite() && sigma > 0.0, "sigma must be positive");
    assert!(
        effect.is_finite() && effect > 0.0,
        "effect must be positive"
    );
    let z = 1.96f64 + 0.8416;
    let n = (z * z) * (sigma as f64).powi(2) / (effect as f64).powi(2);
    n.ceil().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_stable_and_sensitive() {
        let a = Config::c1_default();
        let b = Config::c1_default();
        assert_eq!(a.hash(), b.hash());
        let mut c = Config::c1_default();
        c.eta *= 1.01;
        assert_ne!(a.hash(), c.hash());
        let mut d = Config::c1_default();
        d.g2_min_positive_control = 0.91;
        assert_ne!(a.hash(), d.hash());
    }

    #[test]
    fn from_hash_round_trips_presets() {
        let d = Config::c1_default();
        let q = Config::c1_quick();
        assert_eq!(Config::from_hash(&d.hash_string()).unwrap(), d);
        assert_eq!(Config::from_hash(&format!("{:016x}", q.hash())).unwrap(), q);
    }

    #[test]
    fn scientific_defaults_target_one_to_two_percent_activity() {
        let d = Config::c1_default();
        let frac = d.nominal_activity_fraction();
        assert!((frac - 2.0 / 128.0).abs() < 1e-6);
        assert!((0.005..=0.03).contains(&frac));
        let q = Config::c1_quick();
        let qf = q.nominal_activity_fraction();
        assert!((qf - 1.0 / 64.0).abs() < 1e-6);
        assert!((qf - 0.015).abs() < 0.002);
    }

    #[test]
    fn power_analysis_respects_floor() {
        let c = Config::c1_default();
        assert!(c.required_scientific_n_seeds() >= 20);
        assert_eq!(n_for_80_percent_power(0.15, 0.10), c.n_seeds_for_80_power());
    }
}
