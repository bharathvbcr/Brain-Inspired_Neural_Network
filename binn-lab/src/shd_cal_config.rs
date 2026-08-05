//! SHD calibration protocol config (`c1-shd-cal-*` / `c1-shd-full-*`).
//!
//! Multi-class (20-way) passthrough spike-frame calibration. **Not Gate G2.**
//!
//! - **Protocol 27** (default `--shd-cal`): capped 2000/500, e-prop ceiling,
//!   frozen hash `c1-shd-cal-eb3cb5d93417a638` — do not remassage.
//! - **Protocol 29** (`--shd-full`): official full train/test splits, true
//!   SuperSpike reverse-mode BPTT ceiling (`c1-shd-full-*`). Distinct from p27
//!   and exploratory proto-135 (5-class sweep).
//! - Frozen v26 scientific hash `c1-shd-cal-de44bb52bbd28fbc` remains a known
//!   preset (3 arms; no RL×B).
//!
//! Does not reopen frozen C1 / match / DFA / RL hashes.

use crate::Config;

/// Current SHD calibration protocol (27 = +RL×B arm vs v26).
pub const C1_SHD_CAL_PROTOCOL_VERSION: u64 = 27;
/// Archived 3-arm SHD calibration protocol (pre-RL×B).
pub const C1_SHD_CAL_PROTOCOL_VERSION_V26: u64 = 26;
/// Full-corpus SHD + SuperSpike ceiling (distinct from p27 / proto-135).
pub const C1_SHD_FULL_PROTOCOL_VERSION: u64 = 29;

pub const C1_SHD_CAL_EXPERIMENT: &str = "c1-shd-cal";
pub const C1_SHD_FULL_EXPERIMENT: &str = "c1-shd-full";
pub const C1_SHD_CAL_HASH_PREFIX: &str = "c1-shd-cal-";
pub const C1_SHD_FULL_HASH_PREFIX: &str = "c1-shd-full-";
/// Frozen scientific hash for protocol-26 (broadcast / DFA / e-prop only).
pub const C1_SHD_CAL_V26_SCIENTIFIC_HASH: &str = "c1-shd-cal-de44bb52bbd28fbc";
/// Frozen protocol-27 scientific hash (`shd_hidden=128`). Do not remassage.
pub const C1_SHD_CAL_SCIENTIFIC_HASH: &str = "c1-shd-cal-eb3cb5d93417a638";
/// Protocol-27 scientific with `shd_hidden=256` (same arms; new geometry hash).
pub const C1_SHD_CAL_HIDDEN256_SCIENTIFIC_HASH: &str = "c1-shd-cal-bafa6835d8de7eb8";
/// Frozen protocol-29 full-corpus scientific hash. Do not remassage.
pub const C1_SHD_FULL_SCIENTIFIC_HASH: &str = "c1-shd-full-2c93117075740ed0";
/// Protocol-29 path-proof subset (400/100, 2 seeds, 8 epochs).
pub const C1_SHD_FULL_SMOKE_HASH: &str = "c1-shd-full-a9542a730cb22c74";

/// Chance baseline for 20-way SHD.
pub const C1_SHD_CAL_CHANCE: f32 = 1.0 / 20.0;

/// Public config for SHD calibration / full-corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdCalConfig {
    pub base: Config,
    pub protocol_version: u64,
    pub chance_baseline: f32,
    pub scientific_n_seeds: usize,
    pub quick: bool,
    /// Use on-disk CI fixture (or synthesize) instead of full SHD cache.
    pub use_fixture: bool,
    /// Hidden width for multiclass LIF.
    pub shd_hidden: usize,
    /// Train epochs for all arms.
    pub shd_epochs: usize,
    /// Learning rate shared across arms.
    pub shd_lr: f32,
    /// Cap train/test examples (0 = use all available).
    pub max_train: usize,
    pub max_test: usize,
    /// When true, run REINFORCE×frozen B arm (protocol ≥ 27).
    pub include_rl_fb: bool,
    /// When true, ceiling is true SuperSpike BPTT (protocol 29); else e-prop.
    pub include_superspike: bool,
}

impl ShdCalConfig {
    /// Scientific schedule with RL×B (protocol 27; expects `data/shd/` bins).
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_SHD_CAL_EXPERIMENT.into();
        base.master_seed = 0xC154_DCA1_0001;
        base.n_seeds = 5;
        base.quick = false;
        Self {
            protocol_version: C1_SHD_CAL_PROTOCOL_VERSION,
            chance_baseline: C1_SHD_CAL_CHANCE,
            scientific_n_seeds: 5,
            quick: false,
            use_fixture: false,
            shd_hidden: 128,
            shd_epochs: 20,
            shd_lr: 0.02,
            max_train: 2000,
            max_test: 500,
            include_rl_fb: true,
            include_superspike: false,
            base,
        }
    }

    /// Frozen protocol-26 scientific schedule (3 arms; hash `de44bb52bbd28fbc`).
    pub fn scientific_v26() -> Self {
        let mut c = Self::scientific();
        c.protocol_version = C1_SHD_CAL_PROTOCOL_VERSION_V26;
        c.include_rl_fb = false;
        c
    }

    /// Protocol-27 scientific schedule with `shd_hidden=256` (same arms as
    /// [`Self::scientific` philosophy]; distinct hashed geometry — do not remassage p27).
    pub fn scientific_hidden256() -> Self {
        let mut c = Self::scientific();
        c.shd_hidden = 256;
        c
    }

    /// Protocol-27 scientific schedule with `shd_hidden=512` (width scaling on real audio).
    pub fn scientific_hidden512() -> Self {
        let mut c = Self::scientific();
        c.shd_hidden = 512;
        c
    }

    /// Protocol-29 full official train/test + SuperSpike BPTT ceiling.
    ///
    /// Uses `max_train=0` / `max_test=0` (all samples in `data/shd/{train,test}.bin`).
    /// Distinct hash family `c1-shd-full-*` — does not remassage p27.
    pub fn scientific_full() -> Self {
        let mut base = Config::c1_default();
        base.experiment = C1_SHD_FULL_EXPERIMENT.into();
        base.master_seed = 0xC154_F011_0001;
        base.n_seeds = 5;
        base.quick = false;
        Self {
            protocol_version: C1_SHD_FULL_PROTOCOL_VERSION,
            chance_baseline: C1_SHD_CAL_CHANCE,
            scientific_n_seeds: 5,
            quick: false,
            use_fixture: false,
            shd_hidden: 128,
            shd_epochs: 20,
            shd_lr: 0.02,
            max_train: 0,
            max_test: 0,
            include_rl_fb: true,
            include_superspike: true,
            base,
        }
    }

    /// Protocol-29 path-proof subset (not full-corpus; proves SuperSpike path).
    ///
    /// Caps 400/100, 2 seeds, 8 epochs — scientific enough to exercise all arms
    /// on real BINNSHD1 bins without overnight wall time.
    pub fn scientific_full_smoke() -> Self {
        let mut c = Self::scientific_full();
        c.base.experiment = format!("{C1_SHD_FULL_EXPERIMENT}-smoke");
        c.base.master_seed = 0xC154_F011_5001;
        c.base.n_seeds = 2;
        c.scientific_n_seeds = 2;
        c.shd_epochs = 8;
        c.max_train = 400;
        c.max_test = 100;
        c
    }

    /// PILOT / CI smoke on the tiny fixture (not a scientific SHD verdict).
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{C1_SHD_CAL_EXPERIMENT}-quick");
        c.base.master_seed = 0xC154_D3ED_0001;
        c.base.n_seeds = 2;
        c.base.quick = true;
        c.quick = true;
        c.use_fixture = true;
        c.shd_hidden = 32;
        c.shd_epochs = 4;
        c.shd_lr = 0.05;
        c.max_train = 24;
        c.max_test = 8;
        c
    }

    /// Fixture smoke that still exercises SuperSpike (protocol-29 path on CI data).
    pub fn quick_full() -> Self {
        let mut c = Self::scientific_full();
        c.base.experiment = format!("{C1_SHD_FULL_EXPERIMENT}-quick");
        c.base.master_seed = 0xC154_F011_D3ED;
        c.base.n_seeds = 2;
        c.base.quick = true;
        c.quick = true;
        c.use_fixture = true;
        c.scientific_n_seeds = 2;
        c.shd_hidden = 32;
        c.shd_epochs = 4;
        c.shd_lr = 0.05;
        c.max_train = 24;
        c.max_test = 8;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![
            Self::scientific(),
            Self::scientific_hidden256(),
            Self::scientific_hidden512(),
            Self::scientific_v26(),
            Self::scientific_full(),
            Self::scientific_full_smoke(),
            Self::quick(),
            Self::quick_full(),
        ]
    }

    pub fn hash_prefix(&self) -> &'static str {
        if self.protocol_version == C1_SHD_FULL_PROTOCOL_VERSION
            || self.base.experiment.starts_with(C1_SHD_FULL_EXPERIMENT)
        {
            C1_SHD_FULL_HASH_PREFIX
        } else {
            C1_SHD_CAL_HASH_PREFIX
        }
    }

    pub fn hash(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x0100_0000_01b3);
        }
        mix(&mut h, self.protocol_version);
        for b in self.base.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.base.master_seed);
        mix(&mut h, self.base.n_seeds as u64);
        mix(&mut h, self.chance_baseline.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        mix(&mut h, u64::from(self.use_fixture));
        mix(&mut h, self.shd_hidden as u64);
        mix(&mut h, self.shd_epochs as u64);
        mix(&mut h, self.shd_lr.to_bits() as u64);
        mix(&mut h, self.max_train as u64);
        mix(&mut h, self.max_test as u64);
        // Passthrough encoder + e-prop ceiling markers (v26 fingerprint).
        mix(&mut h, 0xFA57_5444_0000_0001);
        mix(&mut h, 0xE940_CE17_0000_0001);
        // RL×B arm marker only when included (keeps v26 hash stable).
        if self.include_rl_fb {
            mix(&mut h, 0xF1B0_54D0_0000_0001);
        }
        // SuperSpike ceiling marker (protocol 29); must not touch p27 mix path
        // when include_superspike=false.
        if self.include_superspike {
            mix(&mut h, 0x5055_B177_0000_0001);
        }
        h
    }

    pub fn hash_string(&self) -> String {
        format!("{}{:016x}", self.hash_prefix(), self.hash())
    }

    pub fn from_hash(hash: &str) -> Option<Self> {
        let trimmed = hash.trim();
        Self::known_presets()
            .into_iter()
            .find(|preset| trimmed.eq_ignore_ascii_case(&preset.hash_string()))
    }

    #[inline]
    pub fn seeds(&self) -> Vec<u64> {
        self.base.seeds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_fresh_family() {
        let h = ShdCalConfig::scientific().hash_string();
        assert!(h.starts_with(C1_SHD_CAL_HASH_PREFIX));
        assert_ne!(h, "c1-118207fbc3eaba53");
        assert_ne!(h, "c1-match-5dc6822e71229e9e");
        assert_ne!(
            ShdCalConfig::scientific().hash_string(),
            ShdCalConfig::quick().hash_string()
        );
        assert_ne!(
            ShdCalConfig::scientific().hash_string(),
            ShdCalConfig::scientific_v26().hash_string()
        );
    }

    #[test]
    fn p27_scientific_hash_is_frozen() {
        assert_eq!(
            ShdCalConfig::scientific().hash_string(),
            C1_SHD_CAL_SCIENTIFIC_HASH
        );
        assert!(ShdCalConfig::from_hash(C1_SHD_CAL_SCIENTIFIC_HASH).is_some());
    }

    #[test]
    fn v26_scientific_hash_is_frozen() {
        assert_eq!(
            ShdCalConfig::scientific_v26().hash_string(),
            C1_SHD_CAL_V26_SCIENTIFIC_HASH
        );
        assert!(ShdCalConfig::from_hash(C1_SHD_CAL_V26_SCIENTIFIC_HASH).is_some());
    }

    #[test]
    fn scientific_hidden256_hash_differs_and_roundtrips() {
        let h128 = ShdCalConfig::scientific().hash_string();
        let h256 = ShdCalConfig::scientific_hidden256().hash_string();
        assert_eq!(h128, C1_SHD_CAL_SCIENTIFIC_HASH);
        assert_ne!(h256, h128);
        assert_ne!(h256, C1_SHD_CAL_V26_SCIENTIFIC_HASH);
        assert_eq!(h256, C1_SHD_CAL_HIDDEN256_SCIENTIFIC_HASH);
        assert_eq!(ShdCalConfig::scientific_hidden256().shd_hidden, 256);
        assert_eq!(
            ShdCalConfig::scientific_hidden256().protocol_version,
            C1_SHD_CAL_PROTOCOL_VERSION
        );
        assert!(ShdCalConfig::scientific_hidden256().include_rl_fb);
        let again = ShdCalConfig::from_hash(&h256).expect("known preset");
        assert_eq!(again, ShdCalConfig::scientific_hidden256());
        assert!(ShdCalConfig::known_presets()
            .iter()
            .any(|p| p.hash_string() == h256));
    }

    #[test]
    fn scientific_hidden512_hash_differs_and_roundtrips() {
        let h128 = ShdCalConfig::scientific().hash_string();
        let h256 = ShdCalConfig::scientific_hidden256().hash_string();
        let h512 = ShdCalConfig::scientific_hidden512().hash_string();
        assert_ne!(h512, h128);
        assert_ne!(h512, h256);
        assert_eq!(ShdCalConfig::scientific_hidden512().shd_hidden, 512);
        let again = ShdCalConfig::from_hash(&h512).expect("known preset");
        assert_eq!(again, ShdCalConfig::scientific_hidden512());
    }

    #[test]
    fn full_corpus_hashes_are_distinct_from_p27() {
        let full = ShdCalConfig::scientific_full();
        let smoke = ShdCalConfig::scientific_full_smoke();
        let p27 = ShdCalConfig::scientific();
        assert_eq!(full.protocol_version, C1_SHD_FULL_PROTOCOL_VERSION);
        assert!(full.include_superspike);
        assert_eq!(full.max_train, 0);
        assert_eq!(full.max_test, 0);
        assert!(full.hash_string().starts_with(C1_SHD_FULL_HASH_PREFIX));
        assert!(smoke.hash_string().starts_with(C1_SHD_FULL_HASH_PREFIX));
        assert_ne!(full.hash_string(), p27.hash_string());
        assert_ne!(smoke.hash_string(), full.hash_string());
        assert_ne!(full.hash_string(), C1_SHD_CAL_SCIENTIFIC_HASH);
        assert_ne!(full.hash_string(), C1_SHD_CAL_HIDDEN256_SCIENTIFIC_HASH);
        assert_eq!(full.hash_string(), C1_SHD_FULL_SCIENTIFIC_HASH);
        assert_eq!(smoke.hash_string(), C1_SHD_FULL_SMOKE_HASH);
        assert!(ShdCalConfig::from_hash(&full.hash_string()).is_some());
        assert!(ShdCalConfig::from_hash(&smoke.hash_string()).is_some());
    }
}
