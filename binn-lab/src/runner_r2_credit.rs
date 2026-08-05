//! R2 directed-credit mitigation probe harness (`r2-credit-*`).
//!
//! Same hub/majority substrate and #areas grid as frozen R2, but credit is
//! **graded DFA** and/or **REINFORCE × frozen B** (directed). Optional 1-seed
//! broadcast ±1 smoke control.
//!
//! Banner contract: does **not** reopen Gate G4 NO-GO (`r2-afafa0fa6f43e3fc`)
//! or Gate G2 FAIL (`c1-118207fbc3eaba53`).

use binn_areas::Hub;
use binn_core::Rng;
use binn_learn::{reinforce_term, FixedRandomFeedback, ReinforceFeedback};

use crate::r2_credit_config::{R2CreditArm, R2CreditConfig, R2_CREDIT_PROTOCOL_VERSION};
use crate::runner_r2::{
    classify_shape_eps, fit_log_linear, CurveShape, LogLinearFit, ScalingPoint,
};

/// Frozen R2 scientific hash cited in every result banner (do not remassage).
pub const FROZEN_R2_G4_HASH: &str = "r2-afafa0fa6f43e3fc";
/// Frozen G2 kill-gate hash cited in every result banner.
pub const FROZEN_G2_HASH: &str = "c1-118207fbc3eaba53";

/// One arm's fitted scaling curve.
#[derive(Clone, Debug, PartialEq)]
pub struct R2CreditArmCurve {
    pub arm: R2CreditArm,
    pub n_seeds_used: usize,
    pub points: Vec<ScalingPoint>,
    pub fit: LogLinearFit,
    pub shape: CurveShape,
}

/// Aggregated R2-credit mitigation report.
#[derive(Clone, Debug, PartialEq)]
pub struct R2CreditReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub kill_gate_override: bool,
    pub pilot: bool,
    pub directed: Vec<R2CreditArmCurve>,
    /// Optional 1-seed broadcast ±1 smoke control.
    pub pm1_smoke: Option<R2CreditArmCurve>,
    pub disclosed_max_areas: usize,
    /// Human reading: recover / flatten / still-degrade across directed arms.
    pub mitigation_reading: String,
}

/// R2-credit experiment runner.
#[derive(Default)]
pub struct R2CreditRunner;

impl R2CreditRunner {
    pub fn new() -> Self {
        Self
    }

    /// Run the directed-credit mitigation sweep.
    ///
    /// Panics if `kill_gate_override` is false.
    pub fn run(&mut self, config: &R2CreditConfig) -> R2CreditReport {
        assert!(
            config.kill_gate_override,
            "R2CreditRunner::run requires kill_gate_override (CLI --enable-r2 --credit)"
        );
        let counts = config.area_counts();
        assert!(
            !counts.is_empty(),
            "R2-credit sweep requires at least one area count"
        );
        assert!(
            !config.directed_arms().is_empty() || config.include_pm1_smoke,
            "R2-credit requires at least one directed arm or pm1 smoke"
        );

        let mut directed = Vec::new();
        for arm in config.directed_arms() {
            directed.push(run_arm_curve(config, arm, &config.seeds(), &counts));
        }

        let pm1_smoke = if config.include_pm1_smoke {
            Some(run_arm_curve(
                config,
                R2CreditArm::BroadcastPm1,
                &[config.pm1_smoke_seed()],
                &counts,
            ))
        } else {
            None
        };

        let pilot = config.quick || config.n_seeds < config.scientific_n_seeds;
        let mitigation_reading = mitigation_reading(&directed, pilot);

        R2CreditReport {
            config_hash: config.hash_string(),
            protocol_version: R2_CREDIT_PROTOCOL_VERSION,
            kill_gate_override: config.kill_gate_override,
            pilot,
            directed,
            pm1_smoke,
            disclosed_max_areas: config.max_areas,
            mitigation_reading,
        }
    }

    /// Render results markdown (banner: does not reopen G4 / G2).
    pub fn render_results_markdown(report: &R2CreditReport, config: &R2CreditConfig) -> String {
        let mut md = String::new();
        md.push_str("# R2-credit — directed-credit mitigation probe\n\n");
        md.push_str(&format!(
            "**Does not reopen Gate G4 NO-GO** under frozen `{FROZEN_R2_G4_HASH}`, \
             nor Gate G2 FAIL under `{FROZEN_G2_HASH}`. This is a **separate** \
             directed-credit hypothesis (`r2-credit-*`): same disclosed #areas \
             grid as R2, credit arms = graded DFA / REINFORCE×frozen B \
             (optional 1-seed ±1 smoke). Frozen R2 is **not** remassaged.\n\n",
        ));
        md.push_str(
            "**Kill-gate override:** exploratory post-G2 branch; requires \
             `--enable-r2 --credit` (or `--override-g2-for r2`).\n\n",
        );
        md.push_str(&format!("- config hash: `{}`\n", report.config_hash));
        md.push_str(&format!(
            "- protocol version: {}\n",
            report.protocol_version
        ));
        md.push_str(&format!("- quick/PILOT: {}\n", config.quick));
        md.push_str(&format!("- directed seeds: {}\n", config.n_seeds));
        md.push_str(&format!(
            "- disclosed sweep: {}..= {} step {} (matches frozen R2 grid)\n",
            config.min_areas, config.max_areas, config.area_step
        ));
        md.push_str(&format!(
            "- mitigation reading: **{}**\n\n",
            report.mitigation_reading
        ));
        if report.pilot {
            md.push_str(
                "> PILOT only: the quick schedule validates the harness and cannot \
                 alone support a scientific mitigation claim.\n\n",
            );
        }

        for curve in &report.directed {
            append_arm_section(&mut md, curve);
        }
        if let Some(smoke) = &report.pm1_smoke {
            md.push_str("## Optional ±1 smoke control (1 seed)\n\n");
            md.push_str(
                "_Harness sanity only — not a remassage of frozen G4. Expected \
                 to still degrade under broadcast credit._\n\n",
            );
            append_arm_section(&mut md, smoke);
        }

        md.push_str(
            "## Interpretation\n\n\
             | directed shape | reading |\n|---|---|\n\
             | healthy | directed credit recovers capability as #areas grow |\n\
             | plateau | directed credit flattens the degrade curve |\n\
             | degrade | directed credit still degrades (no mitigation) |\n\n\
             Either outcome is informative; neither reopens frozen G4 NO-GO.\n\n",
        );
        md.push_str(
            "## How to run\n\n\
             ```bash\n\
             cargo run --locked --release -p binn-lab --bin r2 -- \\\n\
               --enable-r2 --credit --quick --out results/r2_credit_scaling_quick.md\n\
             cargo run --locked --release -p binn-lab --bin r2 -- \\\n\
               --enable-r2 --credit --out results/r2_credit_scaling.md\n\
             ```\n",
        );
        md
    }
}

fn append_arm_section(md: &mut String, curve: &R2CreditArmCurve) {
    md.push_str(&format!(
        "## Arm `{}` (n_seeds={})\n\n",
        curve.arm.as_str(),
        curve.n_seeds_used
    ));
    md.push_str(&format!(
        "- fit: capability ≈ {:.4} · ln(n) + {:.4}  (R²={:.3})\n",
        curve.fit.slope, curve.fit.intercept, curve.fit.r_squared
    ));
    md.push_str(&format!("- curve shape: **{}**\n\n", curve.shape.as_str()));
    md.push_str("| n_areas | mean capability | mean nnz |\n|---:|---:|---:|\n");
    for p in &curve.points {
        md.push_str(&format!(
            "| {} | {:.4} | {:.0} |\n",
            p.n_areas, p.mean_capability, p.mean_nnz
        ));
    }
    md.push('\n');
}

fn mitigation_reading(directed: &[R2CreditArmCurve], pilot: bool) -> String {
    if directed.is_empty() {
        return if pilot {
            "PILOT — no directed arms".into()
        } else {
            "no directed arms".into()
        };
    }
    let any_healthy = directed.iter().any(|c| c.shape == CurveShape::Healthy);
    let any_plateau = directed.iter().any(|c| c.shape == CurveShape::Plateau);
    let all_degrade = directed.iter().all(|c| c.shape == CurveShape::Degrade);
    let body = if any_healthy {
        "directed credit recovers (healthy) on at least one arm"
    } else if any_plateau {
        "directed credit flattens (plateau) on at least one arm; no healthy recovery"
    } else if all_degrade {
        "directed credit still degrades on all arms (no mitigation)"
    } else {
        "mixed directed shapes"
    };
    if pilot {
        format!("PILOT — {body}")
    } else {
        body.into()
    }
}

fn run_arm_curve(
    config: &R2CreditConfig,
    arm: R2CreditArm,
    seeds: &[u64],
    counts: &[usize],
) -> R2CreditArmCurve {
    let mut points = Vec::with_capacity(counts.len());
    for &n_areas in counts {
        let mut caps = Vec::with_capacity(seeds.len());
        let mut nnz_acc = 0.0f32;
        for &seed in seeds {
            let (cap, nnz) = run_scaling_point(config, arm, seed, n_areas);
            caps.push(cap);
            nnz_acc += nnz as f32;
        }
        points.push(ScalingPoint {
            n_areas,
            mean_capability: mean(&caps),
            mean_nnz: nnz_acc / seeds.len().max(1) as f32,
            seed_capabilities: caps,
        });
    }
    let fit = fit_log_linear(&points);
    let shape = classify_shape_eps(&points, config.plateau_rel_eps, config.degrade_rel_eps);
    R2CreditArmCurve {
        arm,
        n_seeds_used: seeds.len(),
        points,
        fit,
        shape,
    }
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn run_scaling_point(
    config: &R2CreditConfig,
    arm: R2CreditArm,
    seed: u64,
    n_areas: usize,
) -> (f32, usize) {
    let hub = Hub::with_central_hub(n_areas, config.cells_per_area, config.k_wta);
    let csr = hub.compose_csr(seed ^ 0xA2EA_0001, config.p_intra, config.p_inter);
    let nnz = csr.nnz();
    let coupling = hub_coupling_scores(&hub, &csr);

    let mut model =
        DirectedComposedMajority::new(arm, n_areas, config.lr, seed ^ 0xC0A9_0002, &coupling);
    let mut train_rng = Rng::new(seed ^ 0x71A2_0001);
    for _ in 0..config.n_train {
        let (bits, label) = draw_majority(&mut train_rng, n_areas);
        model.observe(&bits, label, &mut train_rng);
    }
    let mut test_rng = Rng::new(seed ^ 0x7E52_0001);
    let mut ok = 0usize;
    for _ in 0..config.n_test {
        let (bits, label) = draw_majority(&mut test_rng, n_areas);
        if model.predict(&bits) == label {
            ok += 1;
        }
    }
    (ok as f32 / config.n_test.max(1) as f32, nnz)
}

fn draw_majority(rng: &mut Rng, n_areas: usize) -> (Vec<f32>, u32) {
    let mut bits = Vec::with_capacity(n_areas);
    let mut sum = 0u32;
    for _ in 0..n_areas {
        let b = if rng.next_f32() < 0.5 { 0u32 } else { 1u32 };
        sum += b;
        let noise = (rng.next_f32() - 0.5) * 0.55;
        bits.push(b as f32 + noise);
    }
    let label = u32::from(sum * 2 > n_areas as u32);
    (bits, label)
}

fn hub_coupling_scores(hub: &Hub, csr: &binn_core::Csr) -> Vec<f32> {
    let hub_range = hub.hub_area().cells.clone();
    let mut scores = vec![0.0f32; hub.n_areas()];
    for (pre, post) in csr.edges() {
        let pre_area = hub.areas.iter().position(|a| a.contains(pre));
        let post_area = hub.areas.iter().position(|a| a.contains(post));
        if let (Some(pa), Some(qa)) = (pre_area, post_area) {
            if pa != qa && (hub_range.contains(&pre) || hub_range.contains(&post)) {
                scores[pa] += 1.0;
                scores[qa] += 1.0;
            }
        }
    }
    let max = scores.iter().cloned().fold(1.0f32, f32::max).max(1.0);
    for s in &mut scores {
        *s = (*s / max).clamp(0.05, 1.0);
    }
    scores[hub.hub_index] = 1.0;
    scores
}

struct DirectedComposedMajority {
    arm: R2CreditArm,
    w: Vec<f32>,
    bias: f32,
    lr: f32,
    coupling: Vec<f32>,
    dfa: Option<FixedRandomFeedback>,
    rfb: Option<ReinforceFeedback>,
}

impl DirectedComposedMajority {
    fn new(arm: R2CreditArm, n: usize, lr: f32, seed: u64, coupling: &[f32]) -> Self {
        let mut rng = Rng::new(seed);
        let w = (0..n).map(|_| (rng.next_f32() - 0.5) * 0.05).collect();
        let (dfa, rfb) = match arm {
            R2CreditArm::BroadcastPm1 => (None, None),
            R2CreditArm::GradedDfa => (
                Some(FixedRandomFeedback::new(n, 1, seed ^ 0xDFA0_0001)),
                None,
            ),
            R2CreditArm::ReinforceFb => (None, Some(ReinforceFeedback::new(n, seed ^ 0xFB00_0001))),
        };
        Self {
            arm,
            w,
            bias: 0.0,
            lr,
            coupling: coupling.to_vec(),
            dfa,
            rfb,
        }
    }

    fn logit(&self, bits: &[f32]) -> f32 {
        let mut s = self.bias;
        for ((w, c), b) in self.w.iter().zip(self.coupling.iter()).zip(bits.iter()) {
            s += *w * *b * *c;
        }
        s
    }

    fn observe(&mut self, bits: &[f32], label: u32, rng: &mut Rng) {
        let y = label as f32;
        let p = sigmoid(self.logit(bits));
        let err = y - p;

        let (scalar_for_bias, per_area): (f32, Vec<f32>) = match self.arm {
            R2CreditArm::BroadcastPm1 => {
                // Frozen-R2-faithful: broadcast logistic error to every area.
                (err, vec![err; self.w.len()])
            }
            R2CreditArm::GradedDfa => {
                let credit = self
                    .dfa
                    .as_ref()
                    .expect("graded-dfa requires FixedRandomFeedback")
                    .project(&[err]);
                (err, credit.values().to_vec())
            }
            R2CreditArm::ReinforceFb => {
                let a = if rng.next_f32() < p { 1.0f32 } else { 0.0 };
                let r = if (a - y).abs() < 0.5 { 1.0f32 } else { -1.0 };
                let reinforce = reinforce_term(r, a, p);
                let credit = self
                    .rfb
                    .as_ref()
                    .expect("reinforce-fb requires ReinforceFeedback")
                    .credit(reinforce);
                (reinforce, credit.values().to_vec())
            }
        };

        assert_eq!(per_area.len(), self.w.len());
        for i in 0..self.w.len() {
            self.w[i] += self.lr * per_area[i] * bits[i] * self.coupling[i];
        }
        self.bias += self.lr * scalar_for_bias;
    }

    fn predict(&self, bits: &[f32]) -> u32 {
        u32::from(sigmoid(self.logit(bits)) >= 0.5)
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x.clamp(-20.0, 20.0)).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_quick_pilot_runs_with_override() {
        let mut cfg = R2CreditConfig::quick();
        cfg.kill_gate_override = true;
        cfg.n_seeds = 1;
        cfg.n_train = 40;
        cfg.n_test = 20;
        let mut runner = R2CreditRunner::new();
        let report = runner.run(&cfg);
        assert!(report.pilot);
        assert!(report.kill_gate_override);
        assert!(report.config_hash.starts_with("r2-credit-"));
        assert_eq!(report.directed.len(), 2);
        assert!(report.pm1_smoke.is_some());
        assert_eq!(report.pm1_smoke.as_ref().unwrap().n_seeds_used, 1);
        for curve in report.directed.iter().chain(report.pm1_smoke.iter()) {
            assert_eq!(curve.points.len(), 3);
            assert!(curve.fit.slope.is_finite());
        }
        let md = R2CreditRunner::render_results_markdown(&report, &cfg);
        assert!(md.contains("Does not reopen Gate G4 NO-GO"));
        assert!(md.contains(FROZEN_R2_G4_HASH));
        assert!(md.contains(FROZEN_G2_HASH));
        // Frozen R2 hash must remain the cited NO-GO; this suite uses a new prefix.
        assert_ne!(report.config_hash, FROZEN_R2_G4_HASH);
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn credit_refuses_without_override() {
        let cfg = R2CreditConfig::quick();
        let mut runner = R2CreditRunner::new();
        let _ = runner.run(&cfg);
    }

    #[test]
    fn frozen_r2_hash_still_pinned() {
        use crate::r2_config::R2Config;
        assert_eq!(R2Config::r2_default().hash_string(), FROZEN_R2_G4_HASH);
    }
}
