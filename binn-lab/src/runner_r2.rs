//! R2 / U17 harness: capability-vs-#areas scaling curve (Gate G4 DECISION).
//!
//! Opt-in only — requires `--enable-r2` / `--override-g2-for r2` (or env).
//!
//! Reuses R1 hub composition helpers. Fits a simple log-linear model
//! `capability ≈ a · ln(n) + b` over the disclosed sweep and classifies the
//! empirical shape as healthy / plateau / degrade.
//!
//! **G4 is DECISION, not kill.** A healthy curve justifies exploring the next
//! order of magnitude of areas — it does **not** prove scaling to 10⁴–10⁶.

use binn_areas::Hub;
use binn_core::Rng;

use crate::r2_config::R2Config;
use crate::runner::mean;

/// One point on the scaling curve.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalingPoint {
    pub n_areas: usize,
    pub mean_capability: f32,
    pub mean_nnz: f32,
    pub seed_capabilities: Vec<f32>,
}

/// Empirical curve shape (G4 interpretation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveShape {
    /// Capability rises with #areas over the disclosed range.
    Healthy,
    /// Capability flattens (relative improvement below plateau ε).
    Plateau,
    /// Capability falls as #areas grow.
    Degrade,
}

impl CurveShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Plateau => "plateau",
            Self::Degrade => "degrade",
        }
    }
}

/// Gate G4 decision under the R2 protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateG4Decision {
    /// PILOT / quick — not a scientific decision.
    Pilot,
    /// Healthy non-plateauing curve → justifies next order of magnitude.
    Go,
    /// Plateau or degrade → redirect (no-go for larger scale).
    NoGo,
}

impl GateG4Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "PILOT",
            Self::Go => "GO",
            Self::NoGo => "NO-GO",
        }
    }
}

/// Fitted log-linear coefficients: `cap ≈ slope * ln(n) + intercept`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogLinearFit {
    pub slope: f32,
    pub intercept: f32,
    pub r_squared: f32,
}

/// Aggregated R2 report.
#[derive(Clone, Debug, PartialEq)]
pub struct R2Report {
    pub config_hash: String,
    pub protocol_version: u64,
    pub kill_gate_override: bool,
    pub points: Vec<ScalingPoint>,
    pub fit: LogLinearFit,
    pub shape: CurveShape,
    pub decision: GateG4Decision,
    /// Disclosed sweep ceiling (do not extrapolate beyond this as proof).
    pub disclosed_max_areas: usize,
}

/// R2 experiment runner.
#[derive(Default)]
pub struct R2Runner;

impl R2Runner {
    pub fn new() -> Self {
        Self
    }

    /// Run the R2 scaling sweep.
    ///
    /// Panics if `kill_gate_override` is false.
    pub fn run_r2(&mut self, config: &R2Config) -> R2Report {
        assert!(
            config.kill_gate_override,
            "R2Runner::run_r2 requires kill_gate_override (CLI --enable-r2)"
        );

        let counts = config.area_counts();
        assert!(
            !counts.is_empty(),
            "R2 sweep requires at least one area count"
        );

        let mut points = Vec::with_capacity(counts.len());
        for &n_areas in &counts {
            let mut caps = Vec::with_capacity(config.n_seeds);
            let mut nnz_acc = 0.0f32;
            for seed in config.seeds() {
                let (cap, nnz) = run_scaling_point(config, seed, n_areas);
                caps.push(cap);
                nnz_acc += nnz as f32;
            }
            points.push(ScalingPoint {
                n_areas,
                mean_capability: mean(&caps),
                mean_nnz: nnz_acc / config.n_seeds.max(1) as f32,
                seed_capabilities: caps,
            });
        }

        let fit = fit_log_linear(&points);
        let shape = classify_shape(&points, config);
        let decision = if config.quick || config.n_seeds < config.scientific_n_seeds {
            GateG4Decision::Pilot
        } else if shape == CurveShape::Healthy {
            GateG4Decision::Go
        } else {
            GateG4Decision::NoGo
        };

        R2Report {
            config_hash: config.hash_string(),
            protocol_version: crate::r2_config::R2_PROTOCOL_VERSION,
            kill_gate_override: config.kill_gate_override,
            points,
            fit,
            shape,
            decision,
            disclosed_max_areas: config.max_areas,
        }
    }

    /// Render results markdown.
    pub fn render_results_markdown(report: &R2Report, config: &R2Config) -> String {
        let mut md = String::new();
        md.push_str("# R2 / U17 — scaling curve (Gate G4 DECISION)\n\n");
        md.push_str(
            "**Kill-gate override:** this run is an **exploratory post-G2 branch**. \
             Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. R2 does **not** \
             reopen the v8 kill-gate; it requires `--enable-r2` / `--override-g2-for r2`.\n\n",
        );
        md.push_str(
            "**Gate G4 is DECISION, not kill.** A GO (healthy, non-plateauing) \
             justifies exploring the *next order of magnitude* of areas. It is \
             **not** proof the curve continues to 10⁴–10⁶ areas (v7 F6 / v8 U17).\n\n",
        );
        md.push_str(&format!("- config hash: `{}`\n", report.config_hash));
        md.push_str(&format!(
            "- protocol version: {}\n",
            report.protocol_version
        ));
        md.push_str(&format!("- quick/PILOT: {}\n", config.quick));
        md.push_str(&format!("- seeds: {}\n", config.n_seeds));
        md.push_str(&format!(
            "- disclosed sweep: {}..= {} step {}\n",
            config.min_areas, config.max_areas, config.area_step
        ));
        md.push_str(&format!(
            "- fit: capability ≈ {:.4} · ln(n) + {:.4}  (R²={:.3})\n",
            report.fit.slope, report.fit.intercept, report.fit.r_squared
        ));
        md.push_str(&format!("- curve shape: **{}**\n", report.shape.as_str()));
        md.push_str(&format!(
            "- G4 decision: **{}**\n\n",
            report.decision.as_str()
        ));
        if config.quick {
            md.push_str(
                "> PILOT only: the quick schedule validates the harness and cannot \
                 alone support a scientific G4 GO/NO-GO.\n\n",
            );
        }
        md.push_str("## Capability vs #areas\n\n");
        md.push_str("| n_areas | mean capability | mean nnz |\n|---:|---:|---:|\n");
        for p in &report.points {
            md.push_str(&format!(
                "| {} | {:.4} | {:.0} |\n",
                p.n_areas, p.mean_capability, p.mean_nnz
            ));
        }
        md.push_str(
            "\n## Go / no-go interpretation\n\n\
             | shape | G4 reading |\n|---|---|\n\
             | healthy | GO — justify next OOM of areas (still post-G2 exploratory) |\n\
             | plateau | NO-GO — redirect toward edge / continual-learning product |\n\
             | degrade | NO-GO — composition cost dominates; do not scale further |\n\n",
        );
        md.push_str(
            "## Fuller sweep (still disclosed; not 10⁴)\n\n\
             ```bash\n\
             cargo run -p binn-lab --release --bin r2 -- --enable-r2 \\\n\
               --out results/r2_scaling.md\n\
             ```\n",
        );
        md
    }
}

fn run_scaling_point(config: &R2Config, seed: u64, n_areas: usize) -> (f32, usize) {
    let hub = Hub::with_central_hub(n_areas, config.cells_per_area, config.k_wta);
    let csr = hub.compose_csr(seed ^ 0xA2EA_0001, config.p_intra, config.p_inter);
    let nnz = csr.nnz();
    let coupling = hub_coupling_scores(&hub, &csr);

    let mut model = ComposedMajority::new(n_areas, config.lr, seed ^ 0xC0A9_0002, &coupling);
    let mut train_rng = Rng::new(seed ^ 0x71A2_0001);
    for _ in 0..config.n_train {
        let (bits, label) = draw_majority(&mut train_rng, n_areas);
        model.observe(&bits, label);
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

struct ComposedMajority {
    w: Vec<f32>,
    bias: f32,
    lr: f32,
    coupling: Vec<f32>,
}

impl ComposedMajority {
    fn new(n: usize, lr: f32, seed: u64, coupling: &[f32]) -> Self {
        let mut rng = Rng::new(seed);
        let w = (0..n).map(|_| (rng.next_f32() - 0.5) * 0.05).collect();
        Self {
            w,
            bias: 0.0,
            lr,
            coupling: coupling.to_vec(),
        }
    }

    fn logit(&self, bits: &[f32]) -> f32 {
        let mut s = self.bias;
        for ((w, c), b) in self.w.iter().zip(self.coupling.iter()).zip(bits.iter()) {
            s += *w * *b * *c;
        }
        s
    }

    fn observe(&mut self, bits: &[f32], label: u32) {
        let y = label as f32;
        let p = sigmoid(self.logit(bits));
        let err = y - p;
        for ((w, c), b) in self.w.iter_mut().zip(self.coupling.iter()).zip(bits.iter()) {
            *w += self.lr * err * *b * *c;
        }
        self.bias += self.lr * err;
    }

    fn predict(&self, bits: &[f32]) -> u32 {
        u32::from(sigmoid(self.logit(bits)) >= 0.5)
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x.clamp(-20.0, 20.0)).exp())
}

pub(crate) fn fit_log_linear(points: &[ScalingPoint]) -> LogLinearFit {
    let n = points.len() as f32;
    if points.len() < 2 {
        return LogLinearFit {
            slope: 0.0,
            intercept: points.first().map(|p| p.mean_capability).unwrap_or(0.0),
            r_squared: 0.0,
        };
    }
    let xs: Vec<f32> = points
        .iter()
        .map(|p| (p.n_areas as f32).max(1.0).ln())
        .collect();
    let ys: Vec<f32> = points.iter().map(|p| p.mean_capability).collect();
    let mean_x = xs.iter().sum::<f32>() / n;
    let mean_y = ys.iter().sum::<f32>() / n;
    let mut num = 0.0f32;
    let mut den = 0.0f32;
    for i in 0..points.len() {
        let dx = xs[i] - mean_x;
        num += dx * (ys[i] - mean_y);
        den += dx * dx;
    }
    let slope = if den.abs() < 1e-12 { 0.0 } else { num / den };
    let intercept = mean_y - slope * mean_x;
    let mut ss_tot = 0.0f32;
    let mut ss_res = 0.0f32;
    for i in 0..points.len() {
        let pred = slope * xs[i] + intercept;
        ss_tot += (ys[i] - mean_y) * (ys[i] - mean_y);
        ss_res += (ys[i] - pred) * (ys[i] - pred);
    }
    let r_squared = if ss_tot < 1e-12 {
        0.0
    } else {
        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    };
    LogLinearFit {
        slope,
        intercept,
        r_squared,
    }
}

/// Classify empirical curve shape from relative first/last/peak capability.
pub(crate) fn classify_shape_eps(
    points: &[ScalingPoint],
    plateau_rel_eps: f32,
    degrade_rel_eps: f32,
) -> CurveShape {
    if points.len() < 2 {
        return CurveShape::Plateau;
    }
    let first = points.first().unwrap().mean_capability;
    let last = points.last().unwrap().mean_capability;
    let peak = points
        .iter()
        .map(|p| p.mean_capability)
        .fold(f32::NEG_INFINITY, f32::max);

    if last + degrade_rel_eps < peak && last + degrade_rel_eps < first {
        return CurveShape::Degrade;
    }
    let gain = last - first;
    if gain < plateau_rel_eps {
        CurveShape::Plateau
    } else {
        CurveShape::Healthy
    }
}

fn classify_shape(points: &[ScalingPoint], config: &R2Config) -> CurveShape {
    classify_shape_eps(points, config.plateau_rel_eps, config.degrade_rel_eps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_quick_pilot_runs_with_override() {
        let mut cfg = R2Config::r2_quick();
        cfg.kill_gate_override = true;
        cfg.n_seeds = 1;
        cfg.n_train = 40;
        cfg.n_test = 20;
        let mut runner = R2Runner::new();
        let report = runner.run_r2(&cfg);
        assert_eq!(report.decision, GateG4Decision::Pilot);
        assert!(report.kill_gate_override);
        assert!(report.config_hash.starts_with("r2-"));
        assert_eq!(report.points.len(), 3);
        assert!(report.fit.slope.is_finite());
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn r2_refuses_without_override() {
        let cfg = R2Config::r2_quick();
        let mut runner = R2Runner::new();
        let _ = runner.run_r2(&cfg);
    }

    #[test]
    fn classify_healthy_vs_plateau() {
        let cfg = R2Config::r2_quick();
        let healthy = vec![
            ScalingPoint {
                n_areas: 3,
                mean_capability: 0.55,
                mean_nnz: 1.0,
                seed_capabilities: vec![],
            },
            ScalingPoint {
                n_areas: 9,
                mean_capability: 0.75,
                mean_nnz: 1.0,
                seed_capabilities: vec![],
            },
        ];
        assert_eq!(classify_shape(&healthy, &cfg), CurveShape::Healthy);
        let flat = vec![
            ScalingPoint {
                n_areas: 3,
                mean_capability: 0.60,
                mean_nnz: 1.0,
                seed_capabilities: vec![],
            },
            ScalingPoint {
                n_areas: 9,
                mean_capability: 0.61,
                mean_nnz: 1.0,
                seed_capabilities: vec![],
            },
        ];
        assert_eq!(classify_shape(&flat, &cfg), CurveShape::Plateau);
    }
}
