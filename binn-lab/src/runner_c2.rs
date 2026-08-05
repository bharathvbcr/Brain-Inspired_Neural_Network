//! C2 / U14 harness: class-incremental continual learning (Gate G3 path).
//!
//! Opt-in only — the CLI refuses to run unless `--enable-c2` /
//! `--override-g2-for c2` (or `BINN_OVERRIDE_G2_FOR=c2`) is set.
//!
//! Production path: online three-factor + sparse assembly (no autodiff, no
//! raw-example replay). Gradient/replay baseline lives in
//! `binn_learn::c2_replay_baseline` (GC1-exempt).

use std::collections::BTreeSet;

use binn_areas::{k_wta, wire, Area, AreaRole, Pos, WiringPrior};
use binn_core::{Csr, Rng, Tick};
use binn_data::{
    ClassIncExample, ClassIncrementalStream, Encoder, LatencyEncoder, Metrics, Sample,
};
use binn_engine::{CellId, Engine};
use binn_learn::{C2ReplayBaseline, Modulators, ThreeFactor, C2_REPLAY_BASELINE_LABEL};

use crate::c2_config::C2Config;

/// Preregistered overlap interventions (U14): mechanistic, not correlation alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlapIntervention {
    /// Natural assemblies (no intervention).
    None,
    /// Shuffle which hidden cells are reserved while holding activity `k` fixed.
    ShuffleOverlap,
    /// Force high overlap with previously seen class assemblies.
    ForceHigh,
    /// Force low (near-disjoint) overlap with previously seen class assemblies.
    ForceLow,
}

impl OverlapIntervention {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ShuffleOverlap => "shuffle-overlap",
            Self::ForceHigh => "force-high",
            Self::ForceLow => "force-low",
        }
    }
}

/// Gate G3 verdict under the C2 protocol (exploratory; requires kill-gate override).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateG3Verdict {
    /// PILOT / quick schedule — not a scientific decision.
    Pilot,
    /// Local forgetting below replay baseline **and** overlap intervention
    /// moves forgetting in the predicted direction (high > low).
    Pass,
    /// Missed one or both preregistered criteria.
    Fail,
}

impl GateG3Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "PILOT",
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
        }
    }
}

/// Per-seed C2 outcomes.
#[derive(Clone, Debug)]
pub struct C2SeedResult {
    pub seed: u64,
    /// Mean forgetting across earlier classes after the final phase (local).
    pub mean_forgetting_local: f32,
    /// Mean forgetting for the capacity/replay-matched gradient baseline.
    pub mean_forgetting_baseline: f32,
    /// Forgetting under force-high overlap intervention.
    pub forgetting_force_high: f32,
    /// Forgetting under force-low overlap intervention.
    pub forgetting_force_low: f32,
    /// Forgetting under shuffle-overlap intervention.
    pub forgetting_shuffle: f32,
    /// Natural inter-class assembly overlap (mean Jaccard across class pairs).
    pub mean_assembly_overlap: f32,
    /// Local path parameter count (nnz).
    pub n_params_local: usize,
    /// Baseline parameter count.
    pub n_params_baseline: usize,
    /// Final mean accuracy on all classes (local).
    pub final_accuracy_local: f32,
    /// Final mean accuracy on all classes (baseline).
    pub final_accuracy_baseline: f32,
}

/// Aggregated C2 report.
#[derive(Clone, Debug)]
pub struct C2Report {
    pub config_hash: String,
    pub protocol_version: u64,
    pub kill_gate_override: bool,
    pub baseline_label: &'static str,
    pub seeds: Vec<C2SeedResult>,
    pub mean_forgetting_local: f32,
    pub mean_forgetting_baseline: f32,
    pub mean_forgetting_high: f32,
    pub mean_forgetting_low: f32,
    pub mean_forgetting_shuffle: f32,
    pub intervention_direction_ok: bool,
    pub below_baseline: bool,
    pub verdict: GateG3Verdict,
}

/// C2 experiment runner.
#[derive(Default)]
pub struct C2Runner;

impl C2Runner {
    pub fn new() -> Self {
        Self
    }

    /// Run the full C2 schedule for `config`.
    ///
    /// Panics if `kill_gate_override` is false — the CLI must set it after
    /// parsing `--enable-c2` / `--override-g2-for c2`.
    pub fn run_c2(&mut self, config: &C2Config) -> C2Report {
        assert!(
            config.kill_gate_override,
            "C2Runner::run_c2 requires kill_gate_override (CLI --enable-c2)"
        );

        let mut seeds = Vec::with_capacity(config.n_seeds);
        for seed in config.seeds() {
            seeds.push(run_c2_seed(config, seed));
        }

        let mean_forgetting_local = mean_f32(seeds.iter().map(|s| s.mean_forgetting_local));
        let mean_forgetting_baseline = mean_f32(seeds.iter().map(|s| s.mean_forgetting_baseline));
        let mean_forgetting_high = mean_f32(seeds.iter().map(|s| s.forgetting_force_high));
        let mean_forgetting_low = mean_f32(seeds.iter().map(|s| s.forgetting_force_low));
        let mean_forgetting_shuffle = mean_f32(seeds.iter().map(|s| s.forgetting_shuffle));

        // Predicted direction: higher forced overlap → more forgetting.
        let intervention_direction_ok = mean_forgetting_high > mean_forgetting_low + 1e-6;
        let below_baseline = mean_forgetting_local < mean_forgetting_baseline - 1e-6;

        let verdict = if config.quick || config.n_seeds < config.scientific_n_seeds {
            GateG3Verdict::Pilot
        } else if below_baseline && intervention_direction_ok {
            GateG3Verdict::Pass
        } else {
            GateG3Verdict::Fail
        };

        C2Report {
            config_hash: config.hash_string(),
            protocol_version: crate::c2_config::C2_PROTOCOL_VERSION,
            kill_gate_override: config.kill_gate_override,
            baseline_label: C2_REPLAY_BASELINE_LABEL,
            seeds,
            mean_forgetting_local,
            mean_forgetting_baseline,
            mean_forgetting_high,
            mean_forgetting_low,
            mean_forgetting_shuffle,
            intervention_direction_ok,
            below_baseline,
            verdict,
        }
    }

    /// Render a results markdown note.
    pub fn render_results_markdown(report: &C2Report, config: &C2Config) -> String {
        let mut md = String::new();
        md.push_str("# C2 / Gate G3 — class-incremental continual learning\n\n");
        md.push_str(
            "**Kill-gate override:** this run is an **exploratory post-G2 branch**. \
             Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. C2 does **not** \
             reopen the v8 kill-gate; it requires `--enable-c2` / `--override-g2-for c2`.\n\n",
        );
        md.push_str(&format!("- config hash: `{}`\n", report.config_hash));
        md.push_str(&format!(
            "- protocol version: {}\n",
            report.protocol_version
        ));
        md.push_str(&format!("- quick/PILOT: {}\n", config.quick));
        md.push_str(&format!("- seeds: {}\n", config.n_seeds));
        md.push_str(&format!(
            "- stream: {} classes, {} train/class, {} test/class\n",
            config.stream.n_classes, config.stream.train_per_class, config.stream.test_per_class
        ));
        md.push_str(&format!(
            "- baseline: `{}` (replay_capacity={}, lr={})\n",
            report.baseline_label, config.baseline_replay_capacity, config.baseline_lr
        ));
        md.push_str(&format!(
            "- G3 verdict: **{}**\n\n",
            report.verdict.as_str()
        ));

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "| metric | value |\n|---|---:|\n\
             | mean forgetting (local) | {:.4} |\n\
             | mean forgetting (replay baseline) | {:.4} |\n\
             | local below baseline | {} |\n\
             | forgetting force-high | {:.4} |\n\
             | forgetting force-low | {:.4} |\n\
             | forgetting shuffle | {:.4} |\n\
             | intervention direction (high > low) | {} |\n\n",
            report.mean_forgetting_local,
            report.mean_forgetting_baseline,
            report.below_baseline,
            report.mean_forgetting_high,
            report.mean_forgetting_low,
            report.mean_forgetting_shuffle,
            report.intervention_direction_ok
        ));

        md.push_str("## Per-seed\n\n");
        md.push_str(
            "| seed | forget_local | forget_baseline | high | low | shuffle | overlap | acc_local | acc_base |\n",
        );
        md.push_str("|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
        for s in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                s.seed,
                s.mean_forgetting_local,
                s.mean_forgetting_baseline,
                s.forgetting_force_high,
                s.forgetting_force_low,
                s.forgetting_shuffle,
                s.mean_assembly_overlap,
                s.final_accuracy_local,
                s.final_accuracy_baseline
            ));
        }
        md.push('\n');
        md.push_str(
            "## Preregistered interventions\n\n\
             - `force-high` / `force-low`: bias k-WTA toward / away from earlier class assemblies \
             while holding activity `k` fixed.\n\
             - `shuffle-overlap`: randomly reassign the reserved set at the same cardinality.\n\
             Predicted direction: **force-high forgetting > force-low forgetting**.\n\n",
        );
        md.push_str(
            "## Full scientific schedule\n\n\
             ```bash\n\
             cargo run -p binn-lab --release --bin c2 -- --enable-c2 \\\n\
               --out results/c2_g3.md\n\
             ```\n",
        );
        md
    }
}

fn mean_f32(iter: impl Iterator<Item = f32>) -> f32 {
    let v: Vec<f32> = iter.collect();
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f32>() / v.len() as f32
    }
}

fn run_c2_seed(config: &C2Config, seed: u64) -> C2SeedResult {
    let natural = run_local_continual(config, seed, OverlapIntervention::None);
    let high = run_local_continual(config, seed, OverlapIntervention::ForceHigh);
    let low = run_local_continual(config, seed, OverlapIntervention::ForceLow);
    let shuffle = run_local_continual(config, seed, OverlapIntervention::ShuffleOverlap);
    let baseline = run_baseline_continual(config, seed);

    C2SeedResult {
        seed,
        mean_forgetting_local: natural.mean_forgetting,
        mean_forgetting_baseline: baseline.mean_forgetting,
        forgetting_force_high: high.mean_forgetting,
        forgetting_force_low: low.mean_forgetting,
        forgetting_shuffle: shuffle.mean_forgetting,
        mean_assembly_overlap: natural.mean_overlap,
        n_params_local: natural.n_params,
        n_params_baseline: baseline.n_params,
        final_accuracy_local: natural.final_accuracy,
        final_accuracy_baseline: baseline.final_accuracy,
    }
}

struct ContinualOutcome {
    mean_forgetting: f32,
    mean_overlap: f32,
    final_accuracy: f32,
    n_params: usize,
}

fn run_baseline_continual(config: &C2Config, seed: u64) -> ContinualOutcome {
    let mut stream_cfg = config.stream.clone();
    stream_cfg.seed ^= seed;
    let mut stream = ClassIncrementalStream::new(stream_cfg);
    let n_in = config.stream.n_features;
    let n_classes = config.stream.n_classes;
    let mut baseline = C2ReplayBaseline::new(
        n_in,
        n_classes,
        config.baseline_lr,
        config.baseline_replay_capacity,
        seed,
    );

    let mut acc_after_learn = vec![0.0f32; n_classes];
    let mut forgettings = Vec::new();

    while !stream.exhausted() {
        let class = stream.phase();
        let batch = stream.drain_phase_train();
        for ex in &batch {
            baseline.observe(&ex.flat_features(), ex.label);
        }
        // Probe accuracy right after learning this class.
        let probe = stream.probe_class(class as u32);
        let probe_pairs: Vec<_> = probe.iter().map(|e| (e.flat_features(), e.label)).collect();
        acc_after_learn[class] = baseline.accuracy(&probe_pairs);

        // Forgetting on earlier classes.
        for (earlier, &initial) in acc_after_learn.iter().enumerate().take(class) {
            let p = stream.probe_class(earlier as u32);
            let pairs: Vec<_> = p.iter().map(|e| (e.flat_features(), e.label)).collect();
            let after = baseline.accuracy(&pairs);
            forgettings.push(Metrics::forgetting(initial as f64, after as f64) as f32);
        }

        if !stream.advance_phase() {
            break;
        }
    }

    let mut final_acc = 0.0f32;
    for c in 0..n_classes {
        let p = stream.probe_class(c as u32);
        let pairs: Vec<_> = p.iter().map(|e| (e.flat_features(), e.label)).collect();
        final_acc += baseline.accuracy(&pairs);
    }
    final_acc /= n_classes.max(1) as f32;

    ContinualOutcome {
        mean_forgetting: mean_f32(forgettings.into_iter()),
        mean_overlap: 0.0,
        final_accuracy: final_acc,
        n_params: baseline.n_params(),
    }
}

fn run_local_continual(
    config: &C2Config,
    seed: u64,
    intervention: OverlapIntervention,
) -> ContinualOutcome {
    let mut stream_cfg = config.stream.clone();
    stream_cfg.seed ^= seed;
    let mut stream = ClassIncrementalStream::new(stream_cfg);

    let n_in = config.stream.n_features;
    let n_hidden = config.n_hidden;
    let n_classes = config.stream.n_classes;
    let readout0 = (n_in + n_hidden) as CellId;
    let n_cells = n_in + n_hidden + n_classes;

    let mut eng = Engine::with_cells(n_cells);
    let (conn, init_w) = build_multiclass_sparse(config, seed, n_in, n_hidden, n_classes);
    let nnz = conn.nnz();
    eng.set_connectivity(conn, vec![init_w; nnz]);
    let readout_boost = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
    boost_readouts(&mut eng, readout0, n_classes, readout_boost);

    let mut area = Area::new(n_in as CellId..(n_in + n_hidden) as CellId, config.k_wta);
    let mut learner = ThreeFactor::new(config.eta, config.lambda, config.tau_e);
    let enc = LatencyEncoder::new(n_in, (config.stream.sequence_len as Tick).max(1), 0);

    let mut class_assemblies: Vec<Vec<CellId>> = vec![Vec::new(); n_classes];
    let mut acc_after_learn = vec![0.0f32; n_classes];
    let mut forgettings = Vec::new();
    let mut t_cursor: Tick = 0;
    let mut intervention_rng = Rng::new(seed ^ 0x0FE8_1AF0);

    while !stream.exhausted() {
        let class = stream.phase();
        let batch = stream.drain_phase_train();
        let mut assembly_hits: Vec<usize> = vec![0; n_hidden];

        for ex in &batch {
            let reserved = reserved_set(
                intervention,
                &class_assemblies,
                class,
                n_in,
                n_hidden,
                config.k_wta,
                &mut intervention_rng,
            );
            let (active, _) = run_multiclass_trial(
                &mut eng,
                &mut learner,
                &mut area,
                &enc,
                &ex.sequence,
                ex.label,
                readout0,
                n_classes,
                t_cursor,
                true,
                reserved.as_ref(),
            );
            for &cell in &active {
                let idx = cell as usize - n_in;
                if idx < n_hidden {
                    assembly_hits[idx] += 1;
                }
            }
            t_cursor = eng.time() + 20;
        }

        // Record assembly as top-k most frequently active cells.
        let mut ranked: Vec<(usize, usize)> = assembly_hits
            .iter()
            .enumerate()
            .map(|(i, &h)| (h, i))
            .collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let k = config.k_wta.max(1);
        class_assemblies[class] = ranked
            .into_iter()
            .take(k)
            .filter(|(hits, _)| *hits > 0)
            .map(|(_, i)| (n_in + i) as CellId)
            .collect();

        let probe = stream.probe_class(class as u32);
        acc_after_learn[class] = eval_local(
            &mut eng,
            &mut learner,
            &mut area,
            &enc,
            &probe,
            readout0,
            n_classes,
            &mut t_cursor,
        );

        for (earlier, &initial) in acc_after_learn.iter().enumerate().take(class) {
            let p = stream.probe_class(earlier as u32);
            let after = eval_local(
                &mut eng,
                &mut learner,
                &mut area,
                &enc,
                &p,
                readout0,
                n_classes,
                &mut t_cursor,
            );
            forgettings.push(Metrics::forgetting(initial as f64, after as f64) as f32);
        }

        if !stream.advance_phase() {
            break;
        }
    }

    let mut final_acc = 0.0f32;
    for c in 0..n_classes {
        let p = stream.probe_class(c as u32);
        final_acc += eval_local(
            &mut eng,
            &mut learner,
            &mut area,
            &enc,
            &p,
            readout0,
            n_classes,
            &mut t_cursor,
        );
    }
    final_acc /= n_classes.max(1) as f32;

    let mut overlaps = Vec::new();
    for i in 0..n_classes {
        for j in (i + 1)..n_classes {
            if !class_assemblies[i].is_empty() && !class_assemblies[j].is_empty() {
                overlaps.push(Metrics::overlap(&class_assemblies[i], &class_assemblies[j]));
            }
        }
    }

    ContinualOutcome {
        mean_forgetting: mean_f32(forgettings.into_iter()),
        mean_overlap: mean_f32(overlaps.into_iter()),
        final_accuracy: final_acc,
        n_params: nnz,
    }
}

fn reserved_set(
    intervention: OverlapIntervention,
    assemblies: &[Vec<CellId>],
    current_class: usize,
    n_in: usize,
    n_hidden: usize,
    k: usize,
    rng: &mut Rng,
) -> Option<ReservedPolicy> {
    if current_class == 0 || intervention == OverlapIntervention::None {
        return None;
    }
    let mut prior: BTreeSet<CellId> = BTreeSet::new();
    for a in assemblies.iter().take(current_class) {
        for &c in a {
            prior.insert(c);
        }
    }
    if prior.is_empty() {
        return None;
    }
    let k = k.max(1).min(n_hidden);
    match intervention {
        OverlapIntervention::None => None,
        OverlapIntervention::ForceHigh => {
            Some(ReservedPolicy::Prefer(prior.into_iter().take(k).collect()))
        }
        OverlapIntervention::ForceLow => Some(ReservedPolicy::Exclude(prior.into_iter().collect())),
        OverlapIntervention::ShuffleOverlap => {
            // Hold activity cardinality fixed: pick a random k-set of same size
            // as the natural prior union (capped at k).
            let card = prior.len().min(k).max(1);
            let mut pool: Vec<CellId> = (0..n_hidden).map(|i| (n_in + i) as CellId).collect();
            // Fisher–Yates with BINN RNG.
            for i in (1..pool.len()).rev() {
                let j = rng.gen_index(i + 1);
                pool.swap(i, j);
            }
            Some(ReservedPolicy::Prefer(
                pool.into_iter().take(card).collect(),
            ))
        }
    }
}

#[derive(Clone, Debug)]
enum ReservedPolicy {
    Prefer(Vec<CellId>),
    Exclude(Vec<CellId>),
}

#[allow(clippy::too_many_arguments)]
fn eval_local(
    eng: &mut Engine,
    learner: &mut ThreeFactor,
    area: &mut Area,
    enc: &LatencyEncoder,
    probe: &[ClassIncExample],
    readout0: CellId,
    n_classes: usize,
    t_cursor: &mut Tick,
) -> f32 {
    if probe.is_empty() {
        return 0.0;
    }
    let mut ok = 0usize;
    for ex in probe {
        let (_active, correct) = run_multiclass_trial(
            eng,
            learner,
            area,
            enc,
            &ex.sequence,
            ex.label,
            readout0,
            n_classes,
            *t_cursor,
            false,
            None,
        );
        if correct {
            ok += 1;
        }
        *t_cursor = eng.time() + 20;
    }
    ok as f32 / probe.len() as f32
}

#[allow(clippy::too_many_arguments)]
fn run_multiclass_trial(
    eng: &mut Engine,
    learner: &mut ThreeFactor,
    area: &mut Area,
    enc: &LatencyEncoder,
    seq: &[Sample],
    label: u32,
    readout0: CellId,
    n_classes: usize,
    t0: Tick,
    train: bool,
    reserved: Option<&ReservedPolicy>,
) -> (Vec<CellId>, bool) {
    let frame_stride = enc.max_delay().saturating_add(1);
    let hidden_cells: Vec<CellId> = area.cells.clone().collect();
    let saved_thresholds: Vec<f32> = hidden_cells
        .iter()
        .map(|&cell| eng.cell(cell).theta)
        .collect();
    for &cell in &hidden_cells {
        eng.cell_mut(cell).theta = f32::INFINITY;
    }

    let mut latest_input_at = t0;
    for (frame_i, sample) in seq.iter().enumerate() {
        let encoded = enc.encode(sample);
        for ev in &encoded {
            let at = t0
                + (frame_i as Tick)
                    .saturating_mul(frame_stride)
                    .saturating_add(ev.t);
            latest_input_at = latest_input_at.max(at);
            eng.force_spike(ev.cell, at);
        }
    }

    let selection_until = latest_input_at
        .checked_add(eng.max_synaptic_delay().max(1))
        .expect("selection window overflow");
    let _ = eng.step_until(selection_until);

    let mut scores: Vec<(CellId, f32)> = hidden_cells
        .iter()
        .map(|&cell| {
            eng.cell_mut(cell).advance_to(selection_until);
            (cell, eng.cell(cell).v)
        })
        .filter(|(_, v)| v.is_finite() && *v > 0.0)
        .collect();

    // Mechanistic overlap intervention: bias scores before k-WTA.
    if let Some(policy) = reserved {
        match policy {
            ReservedPolicy::Prefer(pref) => {
                let set: BTreeSet<_> = pref.iter().copied().collect();
                for (cell, v) in &mut scores {
                    if set.contains(cell) {
                        *v += 10.0;
                    }
                }
            }
            ReservedPolicy::Exclude(excl) => {
                let set: BTreeSet<_> = excl.iter().copied().collect();
                scores.retain(|(cell, _)| !set.contains(cell));
            }
        }
    }

    let active_cells = k_wta(&scores, area.effective_k());
    let active = active_cells.len();
    area.log_activity(active);

    for &cell in &hidden_cells {
        eng.cell_mut(cell).v = 0.0;
    }
    let winner_at = selection_until
        .checked_add(1)
        .expect("winner time overflow");
    for &cell in &active_cells {
        eng.force_spike(cell, winner_at);
    }
    let readout_until = winner_at
        .checked_add(eng.max_synaptic_delay().max(1) + 4)
        .expect("readout horizon overflow");
    let produced = eng.step_until(readout_until);

    let mut charges: Vec<f32> = (0..n_classes)
        .map(|c| eng.last_step_charge(readout0 + c as CellId))
        .collect();
    for (c, ch) in charges.iter_mut().enumerate() {
        let cell = readout0 + c as CellId;
        if produced.as_slice().iter().any(|sp| sp.cell == cell) {
            *ch += 1.0;
        }
    }
    let pred = charges
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i as u32)
        .unwrap_or(0);

    let selected = readout0 + pred;
    let target = readout0 + label;
    let action_at = readout_until.checked_add(1).expect("action time overflow");
    let delay = eng.max_synaptic_delay().max(1) + 4;

    if train {
        let correct = pred == label;
        let reward = if correct { 1.0 } else { -1.0 };
        eng.force_spike(selected, action_at);
        let until_sel = action_at
            .checked_add(delay)
            .expect("selected horizon overflow");
        let _ = eng.step_until(until_sel);
        let _ = learner.update_counted(eng, Modulators::reward(reward));
        clear_eligibility(eng);
        if !correct && target != selected {
            let target_at = until_sel
                .checked_add(2)
                .expect("target action time overflow");
            eng.force_spike(target, target_at);
            let until_tgt = target_at
                .checked_add(delay)
                .expect("target horizon overflow");
            let _ = eng.step_until(until_tgt);
            let _ = learner.update_counted(eng, Modulators::reward(1.0));
            clear_eligibility(eng);
        }
    } else {
        eng.force_spike(selected, action_at);
        let until = action_at
            .checked_add(delay)
            .expect("trial horizon overflow");
        let _ = eng.step_until(until);
    }

    for (&cell, &theta) in hidden_cells.iter().zip(saved_thresholds.iter()) {
        let hidden = eng.cell_mut(cell);
        hidden.theta = theta;
        hidden.v = 0.0;
    }
    eng.close_inhibited_cycle();

    (active_cells, pred == label)
}

fn clear_eligibility(eng: &mut Engine) {
    for syn in eng.syn.as_mut_slice() {
        syn.eligibility = 0.0;
    }
}

fn boost_readouts(eng: &mut Engine, readout0: CellId, n_classes: usize, boost: f32) {
    if (boost - 1.0).abs() < 1e-6 {
        return;
    }
    let conn = eng.conn.clone();
    let readout_end = readout0 + n_classes as CellId;
    for pre in 0..conn.nrows() {
        let start = conn.row_ptr[pre] as usize;
        let end = conn.row_ptr[pre + 1] as usize;
        for (i, &post) in conn.col[start..end].iter().enumerate() {
            if (readout0..readout_end).contains(&post) {
                let idx = start + i;
                eng.edge_w[idx] *= boost;
                eng.syn.as_mut_slice()[idx].weight = eng.edge_w[idx];
            }
        }
    }
}

fn build_multiclass_sparse(
    config: &C2Config,
    seed: u64,
    n_in: usize,
    n_hidden: usize,
    n_classes: usize,
) -> (Csr, f32) {
    let n_cells = n_in + n_hidden + n_classes;
    let hidden = n_in as CellId..(n_in + n_hidden) as CellId;
    let readout0 = (n_in + n_hidden) as CellId;
    let areas = vec![
        0..n_in as CellId,
        hidden.clone(),
        readout0..readout0 + n_classes as CellId,
    ];
    let prior = WiringPrior::new(
        seed ^ 0x00A5_5EC2,
        areas,
        config.p_sparse,
        config.p_sparse * 0.15,
    );
    let csr0 = wire(AreaRole::Association, Pos::new(1), &prior);

    let mut rows: Vec<Vec<u32>> = (0..n_cells)
        .map(|pre| {
            let start = csr0.row_ptr[pre] as usize;
            let end = csr0.row_ptr[pre + 1] as usize;
            csr0.col[start..end]
                .iter()
                .copied()
                .filter(|&post| {
                    let post = post as usize;
                    if pre < n_in {
                        (n_in..n_in + n_hidden).contains(&post)
                    } else if pre < n_in + n_hidden {
                        (n_in..n_in + n_hidden).contains(&post)
                            || (n_in + n_hidden..n_cells).contains(&post)
                    } else {
                        false
                    }
                })
                .collect()
        })
        .collect();

    let mut rng = Rng::new(seed ^ 0x0051_A5C2);
    for row in rows.iter_mut().take(n_in) {
        let fan = config.k_wta.max(1) * 2;
        for _ in 0..fan {
            let post = n_in + rng.gen_index(n_hidden);
            if !row.contains(&(post as u32)) {
                row.push(post as u32);
            }
        }
    }
    for h in hidden {
        for c in 0..n_classes {
            let ro = readout0 + c as CellId;
            if !rows[h as usize].contains(&ro) && rng.next_f32() < 0.55 {
                rows[h as usize].push(ro);
            }
        }
    }
    for c in 0..n_classes {
        let ro = readout0 + c as CellId;
        if !(0..n_cells).any(|pre| rows[pre].contains(&ro)) {
            let h = n_in + rng.gen_index(n_hidden.max(1));
            rows[h].push(ro);
        }
    }
    for row in &mut rows {
        row.sort_unstable();
        row.dedup();
    }
    (Csr::from_adjacency(&rows), config.init_w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_data::ClassIncConfig;

    #[test]
    fn c2_quick_pilot_runs_with_override() {
        let mut cfg = C2Config::c2_quick();
        cfg.kill_gate_override = true;
        // Shrink further for unit-test wall time.
        cfg.stream = ClassIncConfig {
            seed: 1,
            n_classes: 3,
            n_features: 3,
            train_per_class: 8,
            test_per_class: 4,
            sequence_len: 2,
            difficulty: 0.02,
        };
        cfg.n_seeds = 1;
        cfg.n_hidden = 16;
        cfg.k_wta = 1;
        cfg.baseline_replay_capacity = 8;
        let mut runner = C2Runner::new();
        let report = runner.run_c2(&cfg);
        assert_eq!(report.verdict, GateG3Verdict::Pilot);
        assert!(report.kill_gate_override);
        assert!(report.config_hash.starts_with("c2-"));
        assert_eq!(report.seeds.len(), 1);
        let s = &report.seeds[0];
        assert!(s.mean_forgetting_local.is_finite());
        assert!(s.mean_forgetting_baseline.is_finite());
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn c2_refuses_without_override() {
        let cfg = C2Config::c2_quick();
        let mut runner = C2Runner::new();
        let _ = runner.run_c2(&cfg);
    }

    #[test]
    fn intervention_direction_smoke() {
        // Mechanistic prediction: force-high forgetting ≥ force-low on a tiny schedule.
        let mut cfg = C2Config::c2_quick();
        cfg.kill_gate_override = true;
        cfg.stream = ClassIncConfig {
            seed: 2,
            n_classes: 3,
            n_features: 3,
            train_per_class: 10,
            test_per_class: 5,
            sequence_len: 2,
            difficulty: 0.02,
        };
        cfg.n_seeds = 2;
        cfg.n_hidden = 16;
        cfg.k_wta = 1;
        cfg.baseline_replay_capacity = 8;
        let mut runner = C2Runner::new();
        let report = runner.run_c2(&cfg);
        assert!(
            report.mean_forgetting_high + 1e-6 >= report.mean_forgetting_low,
            "high={:.4} low={:.4}",
            report.mean_forgetting_high,
            report.mean_forgetting_low
        );
    }

    #[test]
    fn intervention_enum_labels_are_stable() {
        assert_eq!(OverlapIntervention::ForceHigh.as_str(), "force-high");
        assert_eq!(OverlapIntervention::ForceLow.as_str(), "force-low");
        assert_eq!(
            OverlapIntervention::ShuffleOverlap.as_str(),
            "shuffle-overlap"
        );
    }
}
