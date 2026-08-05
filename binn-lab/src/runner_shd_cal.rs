//! SHD calibration / full-corpus runner (`c1-shd-cal-*` / `c1-shd-full-*`).
//!
//! Arms: broadcast ±1 three-factor, DFA, REINFORCE×frozen B (RL×B),
//! and either true e-prop (p27) or true SuperSpike BPTT (p29) as ceiling.
//! **Not Gate G2** — calibration bars only (chance = 1/20).

#![allow(clippy::if_same_then_else)]

use binn_data::{default_shd_dir, load_fixture, load_shd_split_capped, ShdSample, SHD_CHANCE};
use binn_learn::{
    ModulatorScale, ShdBroadcastPm1, ShdDfa, ShdEpropCeiling, ShdExample, ShdRlReinforceFb,
    ShdSuperSpikeCeiling, ShdTrainConfig, MODULATOR_PARITY_TOLERANCE, SHD_BROADCAST_PM1_LABEL,
    SHD_DFA_LABEL, SHD_EPROP_CEILING_LABEL, SHD_RL_REINFORCE_FB_LABEL,
    SHD_SUPERSPIKE_CEILING_LABEL,
};

use crate::shd_cal_config::{
    ShdCalConfig, C1_SHD_FULL_PROTOCOL_VERSION, C1_SHD_FULL_SCIENTIFIC_HASH,
};

/// Per-seed arm accuracies.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdCalSeedResult {
    pub seed: u64,
    pub broadcast_pm1: f32,
    pub dfa: f32,
    pub rl_reinforce_fb: f32,
    /// E-prop ceiling when `!include_superspike`; else NaN.
    pub eprop_ceiling: f32,
    /// SuperSpike BPTT ceiling when `include_superspike`; else NaN.
    pub superspike_ceiling: f32,
}

/// Aggregated SHD calibration report.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdCalReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub seeds: Vec<ShdCalSeedResult>,
    pub mean_broadcast_pm1: f32,
    pub mean_dfa: f32,
    pub mean_rl_reinforce_fb: f32,
    pub mean_eprop_ceiling: f32,
    pub mean_superspike_ceiling: f32,
    pub chance_baseline: f32,
    pub n_in: usize,
    pub t: usize,
    pub n_classes: usize,
    pub n_train: usize,
    pub n_test: usize,
    pub fixture: bool,
    pub pilot: bool,
    pub note: String,
    /// RMS of the DFA arm's realised hidden-layer modulator, pooled over seeds.
    pub dfa_modulator_rms: f32,
    /// RMS of the e-prop ceiling's realised hidden-layer modulator, pooled over seeds.
    pub eprop_modulator_rms: f32,
    /// Larger-over-smaller ratio of the two modulator RMS values.
    ///
    /// A ratio above [`MODULATOR_PARITY_TOLERANCE`] means the arms ran at
    /// materially different effective learning rates and the DFA-vs-ceiling
    /// comparison is not interpretable. Before the 2026-07-25 fix this ratio was
    /// ≈ 56 at `hidden = 128`, which is why the "ceiling" scored below the
    /// treatment at every width.
    pub modulator_rms_ratio: f32,
    /// True when a ceiling arm scored below the DFA treatment it is meant to bound.
    pub ceiling_inverted: bool,
}

#[derive(Default)]
pub struct ShdCalRunner;

impl ShdCalRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, config: &ShdCalConfig) -> Result<ShdCalReport, String> {
        let split = if config.use_fixture {
            load_fixture()?
        } else {
            let max_train = if config.max_train > 0 {
                Some(config.max_train)
            } else {
                None
            };
            let max_test = if config.max_test > 0 {
                Some(config.max_test)
            } else {
                None
            };
            load_shd_split_capped(&default_shd_dir(), max_train, max_test).map_err(|e| {
                format!(
                    "{e} (hint: use --shd-cal --quick for the CI fixture, or convert official SHD)"
                )
            })?
        };

        let mut train: Vec<ShdExample> = split.train.iter().map(to_example).collect();
        let mut test: Vec<ShdExample> = split.test.iter().map(to_example).collect();
        if config.max_train > 0 && train.len() > config.max_train {
            train.truncate(config.max_train);
        }
        if config.max_test > 0 && test.len() > config.max_test {
            test.truncate(config.max_test);
        }
        if train.is_empty() || test.is_empty() {
            return Err("SHD split empty after caps".into());
        }

        let cfg = ShdTrainConfig {
            hidden: config.shd_hidden,
            n_classes: split.n_classes,
            lr: config.shd_lr,
            beta: if config.base.surrogate_beta > 0.0 {
                config.base.surrogate_beta
            } else {
                5.0
            },
            epochs: config.shd_epochs,
        };

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        let mut dfa_modulator = ModulatorScale::new();
        let mut eprop_modulator = ModulatorScale::new();
        for seed in config.seeds() {
            let mut pm1 = ShdBroadcastPm1::new(&train[0], cfg, seed);
            let mut dfa = ShdDfa::new(&train[0], cfg, seed ^ 0xDFA);
            let r_pm1 = pm1.train_and_evaluate(cfg.epochs, &train, &test);
            let r_dfa = dfa.train_and_evaluate(cfg.epochs, &train, &test);
            dfa_modulator.merge(&dfa.modulator_scale());
            let r_rlb = if config.include_rl_fb {
                let mut rlb = ShdRlReinforceFb::new(&train[0], cfg, seed ^ 0xF1B0);
                rlb.train_and_evaluate(cfg.epochs, &train, &test).accuracy
            } else {
                f32::NAN
            };
            let (r_ep, r_ss) = if config.include_superspike {
                let mut ss = ShdSuperSpikeCeiling::new(&train[0], cfg, seed ^ 0x5055);
                (
                    f32::NAN,
                    ss.train_and_evaluate(cfg.epochs, &train, &test).accuracy,
                )
            } else {
                let mut eprop = ShdEpropCeiling::new(&train[0], cfg, seed ^ 0xE940);
                let acc = eprop.train_and_evaluate(cfg.epochs, &train, &test).accuracy;
                eprop_modulator.merge(&eprop.modulator_scale());
                (acc, f32::NAN)
            };
            seeds.push(ShdCalSeedResult {
                seed,
                broadcast_pm1: r_pm1.accuracy,
                dfa: r_dfa.accuracy,
                rl_reinforce_fb: r_rlb,
                eprop_ceiling: r_ep,
                superspike_ceiling: r_ss,
            });
        }

        let mean_broadcast_pm1 = mean(seeds.iter().map(|s| s.broadcast_pm1));
        let mean_dfa = mean(seeds.iter().map(|s| s.dfa));
        let mean_rl_reinforce_fb = if config.include_rl_fb {
            mean(seeds.iter().map(|s| s.rl_reinforce_fb))
        } else {
            f32::NAN
        };
        let mean_eprop_ceiling = if config.include_superspike {
            f32::NAN
        } else {
            mean(seeds.iter().map(|s| s.eprop_ceiling))
        };
        let mean_superspike_ceiling = if config.include_superspike {
            mean(seeds.iter().map(|s| s.superspike_ceiling))
        } else {
            f32::NAN
        };
        let pilot =
            config.quick || config.base.n_seeds < config.scientific_n_seeds || split.fixture;
        let note = if split.fixture {
            "FIXTURE / smoke data — not a full-SHD scientific calibration. \
             Fetch official SHD and convert offline (see data/shd/README.md)."
                .into()
        } else if config.protocol_version == C1_SHD_FULL_PROTOCOL_VERSION
            && config.max_train == 0
            && config.max_test == 0
        {
            format!(
                "Full official SHD splits (n_train={}, n_test={}; uncapped). \
                 Ceiling = true SuperSpike reverse-mode BPTT on feed-forward hard-reset LIF. \
                 Calibration / software-harness only — not Gate G2, not neuromorphic SOTA.",
                train.len(),
                test.len(),
            )
        } else if config.include_superspike {
            format!(
                "SHD cache loaded; evaluation uses capped subsets \
                 (n_train={}, n_test={}; caps max_train={}, max_test={}). \
                 Protocol-29 SuperSpike path (subset). Ceiling = true SuperSpike BPTT; \
                 not full-corpus SOTA under caps. Not Gate G2.",
                train.len(),
                test.len(),
                config.max_train,
                config.max_test,
            )
        } else {
            format!(
                "Full SHD cache loaded; evaluation uses capped subsets \
                 (n_train={}, n_test={}; caps max_train={}, max_test={}). \
                 Calibration only — not full-corpus SOTA. Ceiling = true e-prop; \
                 see protocol-29 `c1-shd-full-*` for SuperSpike BPTT on full splits.",
                train.len(),
                test.len(),
                config.max_train,
                config.max_test,
            )
        };

        // A ceiling must bound the arm it is compared against. When it does not,
        // the usual cause is a modulator-scale mismatch, so both are reported.
        let ceiling_arm = if config.include_superspike {
            mean_superspike_ceiling
        } else {
            mean_eprop_ceiling
        };
        let ceiling_inverted = ceiling_arm.is_finite() && ceiling_arm < mean_dfa;

        Ok(ShdCalReport {
            config_hash: config.hash_string(),
            protocol_version: config.protocol_version,
            seeds,
            mean_broadcast_pm1,
            mean_dfa,
            mean_rl_reinforce_fb,
            mean_eprop_ceiling,
            mean_superspike_ceiling,
            chance_baseline: config.chance_baseline,
            n_in: split.n_in,
            t: split.t,
            n_classes: split.n_classes,
            n_train: train.len(),
            n_test: test.len(),
            fixture: split.fixture,
            pilot,
            note,
            dfa_modulator_rms: dfa_modulator.rms(),
            eprop_modulator_rms: eprop_modulator.rms(),
            modulator_rms_ratio: ModulatorScale::ratio(&dfa_modulator, &eprop_modulator),
            ceiling_inverted,
        })
    }

    pub fn render_markdown(report: &ShdCalReport, config: &ShdCalConfig) -> String {
        let mut md = String::new();
        let title = if config.protocol_version == C1_SHD_FULL_PROTOCOL_VERSION {
            "# BINN SHD full-corpus + SuperSpike ceiling (C1-SHD-FULL)\n\n"
        } else {
            "# BINN SHD calibration (C1-SHD-CAL)\n\n"
        };
        md.push_str(title);
        md.push_str(
            "**claim_axis:** Standard-benchmark calibration\n\
             **object_under_test:** Multiclass passthrough-spike LIF under local credit rules\n\
             **may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling\n\
             **must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; \
             “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; \
             proto-135 5-class sweep mix-in\n\n",
        );
        md.push_str(&format!(
            "- schedule: **{}**\n\
             - config hash: `{}`\n\
             - protocol version: {}\n\
             - seeds: {}\n\
             - dims: N_IN={}, T={}, n_classes={} (chance={:.4})\n\
             - subset: n_train={}, n_test={} (caps max_train={}, max_test={}; 0=uncapped)\n\
             - hidden / epochs / lr: {} / {} / {:.4}\n\
             - fixture: {}\n\
             - note: {}\n\n",
            if report.pilot {
                "PILOT (development / fixture — not a scientific SHD verdict)"
            } else {
                "SCIENTIFIC"
            },
            report.config_hash,
            report.protocol_version,
            config.base.n_seeds,
            report.n_in,
            report.t,
            report.n_classes,
            report.chance_baseline,
            report.n_train,
            report.n_test,
            config.max_train,
            config.max_test,
            config.shd_hidden,
            config.shd_epochs,
            config.shd_lr,
            report.fixture,
            report.note,
        ));

        md.push_str("## Results\n\n");
        if config.include_superspike {
            if config.include_rl_fb {
                md.push_str(&format!(
                    "| arm | mean accuracy |\n\
                     |---|---:|\n\
                     | `{SHD_BROADCAST_PM1_LABEL}` | {:.4} |\n\
                     | `{SHD_DFA_LABEL}` | {:.4} |\n\
                     | `{SHD_RL_REINFORCE_FB_LABEL}` (REINFORCE×B) | {:.4} |\n\
                     | `{SHD_SUPERSPIKE_CEILING_LABEL}` (ceiling) | {:.4} |\n\
                     | chance (1/20) | {:.4} |\n\n",
                    report.mean_broadcast_pm1,
                    report.mean_dfa,
                    report.mean_rl_reinforce_fb,
                    report.mean_superspike_ceiling,
                    SHD_CHANCE,
                ));
            } else {
                md.push_str(&format!(
                    "| arm | mean accuracy |\n\
                     |---|---:|\n\
                     | `{SHD_BROADCAST_PM1_LABEL}` | {:.4} |\n\
                     | `{SHD_DFA_LABEL}` | {:.4} |\n\
                     | `{SHD_SUPERSPIKE_CEILING_LABEL}` (ceiling) | {:.4} |\n\
                     | chance (1/20) | {:.4} |\n\n",
                    report.mean_broadcast_pm1,
                    report.mean_dfa,
                    report.mean_superspike_ceiling,
                    SHD_CHANCE,
                ));
            }
            md.push_str(
                "**Ceiling disclosure:** true SuperSpike reverse-mode BPTT on the same \
                 feed-forward hard-reset LIF used by the local arms (no `W_rec`). \
                 Surrogate `σ'(u)=1/(1+β|u|)²`; hard reset cuts the membrane adjoint. \
                 This is the nearest feasible BPTT ceiling at SHD scale in this crate — \
                 **not** a Zenke SuperSpike drop-in on a recurrent net, and **not** the \
                 overnight capped e-prop ceiling (~0.09–0.10 under p27 2000/500).\n\n",
            );
        } else if config.include_rl_fb {
            md.push_str(&format!(
                "| arm | mean accuracy |\n\
                 |---|---:|\n\
                 | `{SHD_BROADCAST_PM1_LABEL}` | {:.4} |\n\
                 | `{SHD_DFA_LABEL}` | {:.4} |\n\
                 | `{SHD_RL_REINFORCE_FB_LABEL}` (REINFORCE×B) | {:.4} |\n\
                 | `{SHD_EPROP_CEILING_LABEL}` (ceiling) | {:.4} |\n\
                 | chance (1/20) | {:.4} |\n\n",
                report.mean_broadcast_pm1,
                report.mean_dfa,
                report.mean_rl_reinforce_fb,
                report.mean_eprop_ceiling,
                SHD_CHANCE,
            ));
            md.push_str(
                "**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. \
                 Full SuperSpike BPTT is available under protocol-29 `c1-shd-full-*` \
                 (feed-forward reverse-mode). Do not read the p27 e-prop ceiling as matched SuperSpike.\n\n",
            );
        } else {
            md.push_str(&format!(
                "| arm | mean accuracy |\n\
                 |---|---:|\n\
                 | `{SHD_BROADCAST_PM1_LABEL}` | {:.4} |\n\
                 | `{SHD_DFA_LABEL}` | {:.4} |\n\
                 | `{SHD_EPROP_CEILING_LABEL}` (ceiling) | {:.4} |\n\
                 | chance (1/20) | {:.4} |\n\n",
                report.mean_broadcast_pm1, report.mean_dfa, report.mean_eprop_ceiling, SHD_CHANCE,
            ));
            md.push_str(
                "_Protocol 26 archive: RL×B arm not included. Use default `--shd-cal` \
                 (protocol 27) for REINFORCE×B parity with matched mech._\n\n",
            );
            md.push_str(
                "**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. \
                 See protocol-29 for SuperSpike BPTT on full splits.\n\n",
            );
        }

        md.push_str("## Per-seed\n\n");
        if config.include_superspike {
            if config.include_rl_fb {
                md.push_str(
                    "| seed | broadcast_pm1 | dfa | rl_reinforce_fb | superspike_ceiling |\n\
                     |---:|---:|---:|---:|---:|\n",
                );
                for s in &report.seeds {
                    md.push_str(&format!(
                        "| {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                        s.seed, s.broadcast_pm1, s.dfa, s.rl_reinforce_fb, s.superspike_ceiling
                    ));
                }
            } else {
                md.push_str(
                    "| seed | broadcast_pm1 | dfa | superspike_ceiling |\n\
                     |---:|---:|---:|---:|\n",
                );
                for s in &report.seeds {
                    md.push_str(&format!(
                        "| {} | {:.4} | {:.4} | {:.4} |\n",
                        s.seed, s.broadcast_pm1, s.dfa, s.superspike_ceiling
                    ));
                }
            }
        } else if config.include_rl_fb {
            md.push_str(
                "| seed | broadcast_pm1 | dfa | rl_reinforce_fb | eprop_ceiling |\n\
                 |---:|---:|---:|---:|---:|\n",
            );
            for s in &report.seeds {
                md.push_str(&format!(
                    "| {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                    s.seed, s.broadcast_pm1, s.dfa, s.rl_reinforce_fb, s.eprop_ceiling
                ));
            }
        } else {
            md.push_str(
                "| seed | broadcast_pm1 | dfa | eprop_ceiling |\n\
                 |---:|---:|---:|---:|\n",
            );
            for s in &report.seeds {
                md.push_str(&format!(
                    "| {} | {:.4} | {:.4} | {:.4} |\n",
                    s.seed, s.broadcast_pm1, s.dfa, s.eprop_ceiling
                ));
            }
        }

        let repro = if config.protocol_version == C1_SHD_FULL_PROTOCOL_VERSION {
            if config.quick {
                "cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --quick \\\n\
                   --out results/c1_shd_full_quick.md"
                    .to_string()
            } else if config.max_train > 0 {
                format!(
                    "cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --smoke \\\n\
                       --out results/c1_shd_full_smoke.md\n\
                     # hash: {}",
                    config.hash_string()
                )
            } else {
                format!(
                    "cargo run --locked --release -p binn-lab --bin c1 -- --shd-full \\\n\
                       --out results/c1_shd_full.md\n\
                     # hash: {} (frozen scientific: {C1_SHD_FULL_SCIENTIFIC_HASH})",
                    config.hash_string()
                )
            }
        } else {
            // 2026-07-25 fix: this branch previously hardcoded the h128 command
            // for every non-256 width, so `c1_shd_h512.md` shipped a Reproduce
            // block that reproduces h128. The width flag is now always emitted
            // and the output path always matches the width that actually ran.
            let hidden = config.shd_hidden;
            format!(
                "cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \\\n\
                   --shd-hidden {hidden} \\\n\
                   --out results/c1_shd_h{hidden}.md\n\
                 # equivalent: --config-hash {}",
                config.hash_string()
            )
        };
        md.push_str(&format!(
            "\n## Reproduce\n\n\
             ```bash\n\
             # Rust SHD convert (no Python / h5py):\n\
             PKG_CONFIG_PATH=\"$(brew --prefix hdf5)/lib/pkgconfig:${{PKG_CONFIG_PATH:-}}\" \\\n\
               cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \\\n\
                 --cache-dir data/shd\n\
             {repro}\n\
             ```\n"
        ));

        // ---- Ceiling health (2026-07-25) ----
        md.push_str("\n## Ceiling health\n\n");
        md.push_str(&format!(
            "| Quantity | Value |\n|---|---:|\n\
             | DFA hidden-modulator RMS | {:.4e} |\n\
             | E-prop hidden-modulator RMS | {:.4e} |\n\
             | RMS ratio (larger / smaller) | {:.2} |\n\
             | Parity tolerance | {MODULATOR_PARITY_TOLERANCE:.2} |\n\n",
            report.dfa_modulator_rms, report.eprop_modulator_rms, report.modulator_rms_ratio,
        ));
        if report.modulator_rms_ratio > MODULATOR_PARITY_TOLERANCE {
            md.push_str(&format!(
                "> **MODULATOR-SCALE MISMATCH.** The DFA arm and the ceiling apply hidden-layer \
                 updates differing by {:.1}× in magnitude at a shared learning rate \
                 (lr = {:.4}). The comparison measures effective step size, not \
                 credit-assignment quality, and must not be reported as a ceiling result.\n\n",
                report.modulator_rms_ratio, config.shd_lr,
            ));
        }
        if report.ceiling_inverted {
            md.push_str(&format!(
                "> **CEILING INVERTED.** The ceiling arm ({:.4}) scored below the DFA \
                 treatment ({:.4}) it is meant to bound. A ceiling below its own treatment \
                 invalidates the comparison; do not cite a DFA-vs-ceiling conclusion from \
                 this run.\n\n",
                if config.include_superspike {
                    report.mean_superspike_ceiling
                } else {
                    report.mean_eprop_ceiling
                },
                report.mean_dfa,
            ));
        }

        md.push_str(
            "\n## Non-claims\n\n\
             - **Not Gate G2** and does not reopen `c1-118207fbc3eaba53`.\n\
             - **Not** overnight capped p27 (`c1-shd-cal-eb3cb5d93417a638` / h256) remassage.\n\
             - **Not** proto-135 5-class exploratory sweep.\n\
             - **Not** neuromorphic hardware SOTA without compute / substrate disclosure.\n\
             - SuperSpike here = feed-forward reverse-mode BPTT with SuperSpike surrogate — \
               disclose wall time and feasibility; do not claim biology.\n",
        );
        md
    }
}

fn to_example(s: &ShdSample) -> ShdExample {
    ShdExample {
        frames: s.frames.clone(),
        t: s.t,
        n_in: s.n_in,
        label: s.label,
    }
}

fn mean(iter: impl Iterator<Item = f32>) -> f32 {
    let v: Vec<f32> = iter.collect();
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f32>() / v.len() as f32
    }
}
