//! Mechanism diagnostic runner (`c1-mech-*`).
//!
//! Aggregates one-step loss-drop / eligibility-energy across seeds on the
//! matched feed-forward dense-LIF coincidence forward.

use binn_data::Sample;
use binn_learn::{run_mech_diagnostic, GradientExample, MechArmMetrics, REFERENCE_SEQUENCE_LEN};

use crate::mech_config::MechConfig;
use crate::runner::freeze_trials;

/// Per-seed arm metrics.
#[derive(Clone, Debug, PartialEq)]
pub struct MechSeedResult {
    pub seed: u64,
    pub arms: Vec<MechArmMetrics>,
}

/// Aggregated mechanism report.
#[derive(Clone, Debug, PartialEq)]
pub struct MechReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub seeds: Vec<MechSeedResult>,
    pub mean_arms: Vec<MechArmMetrics>,
    pub pilot: bool,
}

#[derive(Default)]
pub struct MechRunner;

impl MechRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, config: &MechConfig) -> MechReport {
        assert!(config.base.n_seeds >= 1);
        assert_eq!(
            config.base.sequence_len, REFERENCE_SEQUENCE_LEN,
            "mech diagnostic requires sequence_len={REFERENCE_SEQUENCE_LEN}"
        );

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        for seed in config.seeds() {
            seeds.push(run_seed(config, seed));
        }
        let mean_arms = mean_over_seeds(&seeds);
        let pilot = config.quick || config.base.n_seeds < config.scientific_n_seeds;
        MechReport {
            config_hash: config.hash_string(),
            protocol_version: config.protocol_version,
            seeds,
            mean_arms,
            pilot,
        }
    }

    pub fn render_markdown(report: &MechReport, config: &MechConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN credit mechanism diagnostic (C1-MECH)\n\n");
        md.push_str(
            "**claim_axis:** Mechanism measurement (recording)\n\
             **object_under_test:** One-step update usefulness on frozen matched dense-LIF\n\
             **may_claim:** Relative loss-drop / eligibility-energy across credit rules\n\
             **must_not_claim:** Gate G2 reopen; neuromorphic SOTA; “local learning impossible”\n\n",
        );
        md.push_str(
            "Does **not** mutate frozen hashes `c1-118207fbc3eaba53`, \
             `c1-match-5dc6822e71229e9e`, `c1-dfa-*`, or `c1-rl-*`.\n\n",
        );
        md.push_str(&format!(
            "- schedule: **{}**\n\
             - config hash: `{}`\n\
             - protocol version: {}\n\
             - seeds: {}\n\
             - probes/seed: {}\n\
             - hidden / β: {} / {:.1}\n\
             - forward: feed-forward matched dense-LIF (wrec=0)\n\
             - warm-start: 30 SuperSpike epochs on probe set, then freeze\n\n",
            if report.pilot {
                "PILOT (development only — not a scientific verdict)"
            } else {
                "SCIENTIFIC"
            },
            report.config_hash,
            report.protocol_version,
            config.base.n_seeds,
            config.n_probe,
            config.base.n_hidden,
            config.base.surrogate_beta,
        ));

        md.push_str("## Primary metrics (means)\n\n");
        md.push_str(
            "| arm | loss_drop | loss_drop_rotate | elig_energy_capture |\n\
             |---|---:|---:|---:|\n",
        );
        for a in &report.mean_arms {
            md.push_str(&format!(
                "| `{}` | {:.6} | {:.6} | {:.4} |\n",
                a.arm, a.loss_drop, a.loss_drop_rotate, a.elig_energy_capture
            ));
        }
        md.push_str(
            "\n`loss_drop` = BCE decrease after a **unit-norm** one-step `win` update. \
             `loss_drop_rotate` shuffles the same-norm vector (direction control). \
             `elig_energy_capture` = fraction of SuperSpike ‖∇L‖² on synapses with |E|>ε \
             (shared E; identical across arms).\n\n",
        );

        md.push_str("## Secondary (not headline)\n\n");
        md.push_str(
            "| arm | cosine(Δw, −∇L_SS) | sign_agree |\n\
             |---|---:|---:|\n",
        );
        for a in &report.mean_arms {
            md.push_str(&format!(
                "| `{}` | {:.4} | {:.4} |\n",
                a.arm, a.cosine_vs_ss, a.sign_agree_vs_ss
            ));
        }

        md.push_str("\n## Per-seed loss_drop\n\n");
        md.push_str("| seed");
        for a in &report.mean_arms {
            md.push_str(&format!(" | {}", a.arm));
        }
        md.push_str(" |\n|---:");
        for _ in &report.mean_arms {
            md.push_str("|---:");
        }
        md.push_str("|\n");
        for s in &report.seeds {
            md.push_str(&format!("| {}", s.seed));
            for a in &s.arms {
                md.push_str(&format!(" | {:.6}", a.loss_drop));
            }
            md.push_str(" |\n");
        }

        md.push_str(
            "\n## Reproduce\n\n\
             ```bash\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-mech --quick \\\n\
               --out results/c1_credit_mech.md\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-mech \\\n\
               --out results/c1_credit_mech.md\n\
             ```\n",
        );
        md
    }
}

fn run_seed(config: &MechConfig, seed: u64) -> MechSeedResult {
    let split = freeze_trials(&config.base, seed);
    let mut train = samples_to_gradient_examples(&split.train);
    if train.len() > config.n_probe {
        train.truncate(config.n_probe);
    }
    let report = run_mech_diagnostic(
        config.base.n_hidden,
        config.base.surrogate_beta,
        seed,
        &train,
    );
    MechSeedResult {
        seed,
        arms: report.arms,
    }
}

fn mean_over_seeds(seeds: &[MechSeedResult]) -> Vec<MechArmMetrics> {
    if seeds.is_empty() {
        return Vec::new();
    }
    let n_arms = seeds[0].arms.len();
    let inv = 1.0 / seeds.len() as f32;
    let mut out = Vec::with_capacity(n_arms);
    for i in 0..n_arms {
        let arm = seeds[0].arms[i].arm;
        let mut m = MechArmMetrics {
            arm,
            loss_drop: 0.0,
            loss_drop_rotate: 0.0,
            elig_energy_capture: 0.0,
            cosine_vs_ss: 0.0,
            sign_agree_vs_ss: 0.0,
        };
        for s in seeds {
            let a = &s.arms[i];
            assert_eq!(a.arm, arm);
            m.loss_drop += a.loss_drop;
            m.loss_drop_rotate += a.loss_drop_rotate;
            m.elig_energy_capture += a.elig_energy_capture;
            m.cosine_vs_ss += a.cosine_vs_ss;
            m.sign_agree_vs_ss += a.sign_agree_vs_ss;
        }
        m.loss_drop *= inv;
        m.loss_drop_rotate *= inv;
        m.elig_energy_capture *= inv;
        m.cosine_vs_ss *= inv;
        m.sign_agree_vs_ss *= inv;
        out.push(m);
    }
    out
}

fn samples_to_gradient_examples(trials: &[(Vec<Sample>, u32)]) -> Vec<GradientExample> {
    trials
        .iter()
        .map(|(sequence, label)| {
            let mut x1 = [0.0f32; REFERENCE_SEQUENCE_LEN];
            let mut x2 = [0.0f32; REFERENCE_SEQUENCE_LEN];
            for (t, sample) in sequence.iter().enumerate().take(REFERENCE_SEQUENCE_LEN) {
                x1[t] = sample.values[0];
                x2[t] = sample.values[1];
            }
            (x1, x2, *label as f32)
        })
        .collect()
}
