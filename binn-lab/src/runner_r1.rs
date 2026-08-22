//! R1 / U16 harness: multi-area composition vs additive baseline.
//!
//! Opt-in only — requires `--enable-r1` / `--override-g2-for r1` (or env).
//!
//! Protocol: for each `n_areas` in `[min, max]`, build a [`Hub`], learn a
//! parity (compositional) task with hub-routed features, and compare against
//! an additive late-fusion baseline that uses the same per-area experts
//! without hub interaction. Composition *compounds* when
//! `acc_composed > acc_additive + margin`.

use binn_areas::Hub;
use binn_core::Rng;

use crate::logging::{TraceArea, TraceProjection, TraceRecorder};
use crate::r1_config::R1Config;
use crate::runner::mean;

/// Per-`n_areas` outcome (mean over seeds).
#[derive(Clone, Debug, PartialEq)]
pub struct AreaSweepPoint {
    pub n_areas: usize,
    pub mean_composed: f32,
    pub mean_additive: f32,
    pub mean_nnz: f32,
    pub mean_locality: f32,
    pub compounds: bool,
    pub seed_composed: Vec<f32>,
    pub seed_additive: Vec<f32>,
}

/// Aggregated R1 report.
#[derive(Clone, Debug, PartialEq)]
pub struct R1Report {
    pub config_hash: String,
    pub protocol_version: u64,
    pub kill_gate_override: bool,
    pub points: Vec<AreaSweepPoint>,
    /// Fraction of sweep points where composition compounds.
    pub compound_fraction: f32,
    pub verdict: R1Verdict,
    /// Disclosed budgets (shared across the sweep).
    pub cells_per_area: usize,
    pub train_per_point: usize,
    pub test_per_point: usize,
}

/// R1 reporting verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R1Verdict {
    Pilot,
    /// Scientific schedule completed; composition compounds on a majority of points.
    Compounds,
    /// Scientific schedule completed; composition does not clearly compound.
    Additive,
}

impl R1Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "PILOT",
            Self::Compounds => "COMPOUNDS",
            Self::Additive => "ADDITIVE",
        }
    }
}

/// R1 experiment runner.
#[derive(Default)]
pub struct R1Runner;

impl R1Runner {
    pub fn new() -> Self {
        Self
    }

    /// Run the R1 composition sweep.
    ///
    /// Panics if `kill_gate_override` is false.
    pub fn run_r1(&mut self, config: &R1Config) -> R1Report {
        assert!(
            config.kill_gate_override,
            "R1Runner::run_r1 requires kill_gate_override (CLI --enable-r1)"
        );
        assert!(config.min_areas >= 2);
        assert!(config.max_areas >= config.min_areas);

        let mut points = Vec::new();
        for n_areas in config.min_areas..=config.max_areas {
            let mut seed_composed = Vec::with_capacity(config.n_seeds);
            let mut seed_additive = Vec::with_capacity(config.n_seeds);
            let mut nnz_acc = 0.0f32;
            let mut loc_acc = 0.0f32;
            for seed in config.seeds() {
                let outcome = run_r1_point(config, seed, n_areas);
                seed_composed.push(outcome.composed);
                seed_additive.push(outcome.additive);
                nnz_acc += outcome.nnz as f32;
                loc_acc += outcome.locality;
            }
            let mean_composed = mean(&seed_composed);
            let mean_additive = mean(&seed_additive);
            let compounds = mean_composed > mean_additive + config.compound_margin;
            points.push(AreaSweepPoint {
                n_areas,
                mean_composed,
                mean_additive,
                mean_nnz: nnz_acc / config.n_seeds.max(1) as f32,
                mean_locality: loc_acc / config.n_seeds.max(1) as f32,
                compounds,
                seed_composed,
                seed_additive,
            });
        }

        let compound_fraction = if points.is_empty() {
            0.0
        } else {
            points.iter().filter(|p| p.compounds).count() as f32 / points.len() as f32
        };

        let verdict = if config.quick || config.n_seeds < config.scientific_n_seeds {
            R1Verdict::Pilot
        } else if compound_fraction >= 0.5 {
            R1Verdict::Compounds
        } else {
            R1Verdict::Additive
        };

        R1Report {
            config_hash: config.hash_string(),
            protocol_version: crate::r1_config::R1_PROTOCOL_VERSION,
            kill_gate_override: config.kill_gate_override,
            points,
            compound_fraction,
            verdict,
            cells_per_area: config.cells_per_area,
            train_per_point: config.n_train,
            test_per_point: config.n_test,
        }
    }

    /// Static JSONL trace for one representative `(seed, n_areas)` point.
    ///
    /// Emits `meta` + `topology` + `flow_static` from Hub CSR / coupling only.
    /// No Engine, no SpikeLog, no invented spikes.
    pub fn export_static_trace(config: &R1Config, seed: u64, n_areas: usize) -> TraceRecorder {
        assert!(n_areas >= 2, "export_static_trace requires n_areas >= 2");
        let hub = Hub::with_central_hub(n_areas, config.cells_per_area, config.k_wta);
        let csr = hub.compose_csr(seed ^ 0xA0EA_0001, config.p_intra, config.p_inter);
        let coupling = hub_coupling_scores(&hub, &csr);
        let areas = trace_areas_from_hub(&hub);
        let topo_projections = projection_nnz(&hub, &csr);
        let flow_projections: Vec<TraceProjection> = topo_projections
            .iter()
            .map(|p| TraceProjection {
                src: p.src,
                dst: p.dst,
                nnz: p.nnz,
                coupling: Some(vec![coupling[p.src as usize], coupling[p.dst as usize]]),
            })
            .collect();

        let mut tr = TraceRecorder::new();
        tr.emit_meta(
            &config.hash_string(),
            seed,
            "hub-composed",
            "r1",
            2, // binary majority label
            config.k_wta as u32,
            hub.num_cells() as u32,
        );
        tr.emit_topology(&areas, &topo_projections);
        tr.emit_flow_static(&flow_projections);
        tr
    }

    /// Render results markdown.
    pub fn render_results_markdown(report: &R1Report, config: &R1Config) -> String {
        let mut md = String::new();
        md.push_str("# R1 / U16 — multi-area composition\n\n");
        md.push_str(
            "**Kill-gate override:** this run is an **exploratory post-G2 branch**. \
             Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. R1 does **not** \
             reopen the v8 kill-gate; it requires `--enable-r1` / `--override-g2-for r1`.\n\n",
        );
        md.push_str(&format!("- config hash: `{}`\n", report.config_hash));
        md.push_str(&format!(
            "- protocol version: {}\n",
            report.protocol_version
        ));
        md.push_str(&format!("- quick/PILOT: {}\n", config.quick));
        md.push_str(&format!("- seeds: {}\n", config.n_seeds));
        md.push_str(&format!(
            "- area sweep: {}..= {}\n",
            config.min_areas, config.max_areas
        ));
        md.push_str(&format!(
            "- cells/area × k-WTA: {} × {}\n",
            config.cells_per_area, config.k_wta
        ));
        md.push_str(&format!(
            "- train / test per point: {} / {}\n",
            report.train_per_point, report.test_per_point
        ));
        md.push_str(&format!(
            "- compound margin: {:.3}\n",
            config.compound_margin
        ));
        md.push_str(&format!(
            "- compound fraction: {:.3}\n",
            report.compound_fraction
        ));
        md.push_str(&format!("- verdict: **{}**\n\n", report.verdict.as_str()));
        if config.quick {
            md.push_str(
                "> PILOT only: the quick schedule validates the harness and cannot \
                 alone license a scientific composition claim.\n\n",
            );
        }
        md.push_str("## Composition vs additive\n\n");
        md.push_str(
            "| n_areas | composed | additive | compounds | mean nnz | locality |\n\
             |---:|---:|---:|---|---:|---:|\n",
        );
        for p in &report.points {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {} | {:.0} | {:.3} |\n",
                p.n_areas,
                p.mean_composed,
                p.mean_additive,
                p.compounds,
                p.mean_nnz,
                p.mean_locality
            ));
        }
        md.push_str(
            "\n## Protocol\n\n\
             Task: noisy majority of `n_areas` latent bits (compositional pooling). \
             **Composed** path: hub-routed linear readout (coupling from hub CSR). \
             **Additive** path: identical learner with uniform coupling (no hub \
             structure; matched train/test and learning rates).\n\n\
             Budgets disclosed: cells = `n_areas × cells_per_area`, nnz from hub CSR, \
             train/test counts above.\n\n",
        );
        md.push_str(
            "## Full scientific schedule\n\n\
             ```bash\n\
             cargo run -p binn-lab --release --bin r1 -- --enable-r1 \\\n\
               --out results/r1_composition.md\n\
             ```\n",
        );
        md
    }
}

struct PointOutcome {
    composed: f32,
    additive: f32,
    nnz: usize,
    locality: f32,
}

fn run_r1_point(config: &R1Config, seed: u64, n_areas: usize) -> PointOutcome {
    let hub = Hub::with_central_hub(n_areas, config.cells_per_area, config.k_wta);
    let csr = hub.compose_csr(seed ^ 0xA0EA_0001, config.p_intra, config.p_inter);
    let locality = hub.event_locality(&csr, seed ^ 0xA0EA_0001, config.p_intra, config.p_inter);
    let nnz = csr.nnz();

    // Hub routing weights: for each spoke→hub edge count a soft coupling score.
    let coupling = hub_coupling_scores(&hub, &csr);

    let mut composed = ComposedMajority::new(n_areas, config.lr, seed ^ 0xC0A9_0001, &coupling);
    // Additive: same architecture but uniform coupling (no hub structure) +
    // matched lr/train — tests whether hub routing compounds over flat fusion.
    let flat = vec![1.0f32; n_areas];
    let mut additive =
        ComposedMajority::new(n_areas, config.additive_lr, seed ^ 0xADD1_0001, &flat);

    let mut train_rng = Rng::new(seed ^ 0x71A1_0001);
    for _ in 0..config.n_train {
        let (bits, label) = draw_majority(&mut train_rng, n_areas);
        composed.observe(&bits, label);
        additive.observe(&bits, label);
    }

    let mut test_rng = Rng::new(seed ^ 0x7E57_0001);
    let mut ok_c = 0usize;
    let mut ok_a = 0usize;
    for _ in 0..config.n_test {
        let (bits, label) = draw_majority(&mut test_rng, n_areas);
        if composed.predict(&bits) == label {
            ok_c += 1;
        }
        if additive.predict(&bits) == label {
            ok_a += 1;
        }
    }
    let n = config.n_test.max(1) as f32;
    PointOutcome {
        composed: ok_c as f32 / n,
        additive: ok_a as f32 / n,
        nnz,
        locality,
    }
}

fn draw_majority(rng: &mut Rng, n_areas: usize) -> (Vec<f32>, u32) {
    let mut bits = Vec::with_capacity(n_areas);
    let mut sum = 0u32;
    for _ in 0..n_areas {
        let b = if rng.next_f32() < 0.5 { 0u32 } else { 1u32 };
        sum += b;
        // Substantial noise so hub coupling can matter vs flat fusion.
        let noise = (rng.next_f32() - 0.5) * 0.55;
        bits.push(b as f32 + noise);
    }
    let label = u32::from(sum * 2 > n_areas as u32);
    (bits, label)
}

fn trace_areas_from_hub(hub: &Hub) -> Vec<TraceArea> {
    hub.areas
        .iter()
        .enumerate()
        .map(|(i, a)| TraceArea {
            id: i as u32,
            name: if i == hub.hub_index {
                "hub".into()
            } else {
                format!("area{i}")
            },
            start: a.cells.start,
            end: a.cells.end,
        })
        .collect()
}

/// Per directed `(src_area, dst_area)` CSR edge counts (nnz > 0 only).
fn projection_nnz(hub: &Hub, csr: &binn_core::Csr) -> Vec<TraceProjection> {
    let n = hub.n_areas();
    let mut counts = vec![0u64; n * n];
    for (pre, post) in csr.edges() {
        let Some(pa) = hub.areas.iter().position(|a| a.contains(pre)) else {
            continue;
        };
        let Some(qa) = hub.areas.iter().position(|a| a.contains(post)) else {
            continue;
        };
        counts[pa * n + qa] += 1;
    }
    let mut out = Vec::new();
    for src in 0..n {
        for dst in 0..n {
            let nnz = counts[src * n + dst];
            if nnz > 0 {
                out.push(TraceProjection {
                    src: src as u32,
                    dst: dst as u32,
                    nnz,
                    coupling: None,
                });
            }
        }
    }
    out
}

/// Soft hub-coupling score per area from CSR fan-in/out to the hub range.
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
    // Hub area itself gets full coupling.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r1_quick_pilot_runs_with_override() {
        let mut cfg = R1Config::r1_quick();
        cfg.kill_gate_override = true;
        cfg.n_seeds = 1;
        cfg.min_areas = 3;
        cfg.max_areas = 4;
        cfg.n_train = 80;
        cfg.n_test = 40;
        let mut runner = R1Runner::new();
        let report = runner.run_r1(&cfg);
        assert_eq!(report.verdict, R1Verdict::Pilot);
        assert!(report.kill_gate_override);
        assert!(report.config_hash.starts_with("r1-"));
        assert_eq!(report.points.len(), 2);
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn r1_refuses_without_override() {
        let cfg = R1Config::r1_quick();
        let mut runner = R1Runner::new();
        let _ = runner.run_r1(&cfg);
    }

    #[test]
    fn hub_coupling_scores_nonzero() {
        let hub = Hub::with_central_hub(4, 8, 1);
        let csr = hub.compose_csr(7, 0.4, 0.08);
        let scores = hub_coupling_scores(&hub, &csr);
        assert_eq!(scores.len(), 4);
        assert!(scores.iter().all(|s| *s > 0.0));
    }

    #[test]
    fn export_static_trace_meta_topology_flow_no_spikes() {
        let mut cfg = R1Config::r1_quick();
        cfg.kill_gate_override = true;
        let seed = cfg.seeds()[0];
        let n_areas = cfg.max_areas;
        let tr = R1Runner::export_static_trace(&cfg, seed, n_areas);
        let lines = tr.lines();
        assert_eq!(lines.len(), 3, "meta + topology + flow_static only");
        assert!(lines[0].contains(r#""type":"meta""#));
        assert!(lines[0].contains(r#""experiment":"r1""#));
        assert!(lines[1].contains(r#""type":"topology""#));
        assert!(lines[1].contains(r#""name":"hub""#));
        assert!(lines[2].contains(r#""type":"flow_static""#));
        assert!(lines[2].contains(r#""coupling":["#));
        for line in lines {
            assert!(!line.contains(r#""type":"spike""#));
            assert!(!line.contains(r#""type":"kwta""#));
            assert!(!line.contains(r#""type":"stimulus""#));
        }
        let hub = Hub::with_central_hub(n_areas, cfg.cells_per_area, cfg.k_wta);
        let csr = hub.compose_csr(seed ^ 0xA0EA_0001, cfg.p_intra, cfg.p_inter);
        let expected_nnz: u64 = projection_nnz(&hub, &csr).iter().map(|p| p.nnz).sum();
        assert_eq!(expected_nnz, csr.nnz() as u64);
    }
}
