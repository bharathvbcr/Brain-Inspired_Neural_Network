//! U21-U23 exploratory extension harness.
//!
//! Produces separate result notes for consolidation, pruning, and resting-state
//! dynamics while preserving the protocol-v2 G2 FAIL.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_core::{Csr, Rng};
use binn_data::{ClassIncConfig, ClassIncrementalStream, Metrics};
use binn_engine::{
    characterize, matched_null, simulate_resting, Engine, RestingConfig, RestingNull, RestingRaster,
};
use binn_lab::mean;
use binn_learn::{
    prune, replay_schedule, ConsolidationBudget, ConsolidationMode, ExactReplayBuffer,
    GenerativeReplay, PruningStrategy, ReplayItem, ReplaySource,
};

const G2_HASH: &str = "c1-118207fbc3eaba53";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let enabled = args.iter().any(|arg| arg == "--enable-extensions")
        || env::var("BINN_OVERRIDE_G2_FOR")
            .map(|v| {
                v.split(',').any(|part| {
                    matches!(
                        part.trim().to_ascii_lowercase().as_str(),
                        "u21" | "u22" | "u23" | "extensions" | "all"
                    )
                })
            })
            .unwrap_or(false);
    if !enabled {
        eprintln!("U21-U23 require --enable-extensions (post-G2 exploratory override)");
        return ExitCode::from(2);
    }
    let quick = args.iter().any(|arg| arg == "--quick");
    let out_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from("results"));
    if let Err(error) = fs::create_dir_all(&out_dir) {
        eprintln!("failed to create {}: {error}", out_dir.display());
        return ExitCode::from(1);
    }
    let suffix = if quick { "_quick" } else { "" };
    let outputs = [
        (
            out_dir.join(format!("u21_consolidation{suffix}.md")),
            consolidation_note(quick),
        ),
        (
            out_dir.join(format!("u22_pruning{suffix}.md")),
            pruning_note(quick),
        ),
        (
            out_dir.join(format!("u23_resting{suffix}.md")),
            resting_note(quick),
        ),
    ];
    for (path, note) in outputs {
        if let Err(error) = write_note(&path, &note) {
            eprintln!("failed to write {}: {error}", path.display());
            return ExitCode::from(1);
        }
        println!("results note: {}", path.display());
    }
    ExitCode::SUCCESS
}

fn write_note(path: &Path, note: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, note)
}

#[derive(Clone)]
struct LocalLearner {
    n_classes: usize,
    n_features: usize,
    weights: Vec<f32>,
    eta: f32,
}

impl LocalLearner {
    fn new(n_classes: usize, n_features: usize) -> Self {
        Self {
            n_classes,
            n_features,
            weights: vec![0.0; n_classes * n_features],
            eta: 0.2,
        }
    }

    fn predict(&self, features: &[f32]) -> u32 {
        (0..self.n_classes)
            .max_by(|&a, &b| {
                self.score(a, features)
                    .partial_cmp(&self.score(b, features))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.cmp(&a))
            })
            .unwrap_or(0) as u32
    }

    fn score(&self, class: usize, features: &[f32]) -> f32 {
        (0..self.n_features)
            .map(|i| self.weights[class * self.n_features + i] * features[i])
            .sum()
    }

    fn observe(&mut self, item: &ReplayItem, use_label: bool) {
        let class = if use_label {
            item.label as usize
        } else {
            self.predict(&item.features) as usize
        };
        for weight in &mut self.weights {
            *weight *= 0.999;
        }
        for (i, &feature) in item.features.iter().enumerate() {
            let index = class * self.n_features + i;
            self.weights[index] += self.eta * feature;
        }
        // Shared per-feature capacity creates measurable class interference.
        for feature in 0..self.n_features {
            let norm: f32 = (0..self.n_classes)
                .map(|class| self.weights[class * self.n_features + feature].abs())
                .sum();
            if norm > 1.0 {
                for class in 0..self.n_classes {
                    self.weights[class * self.n_features + feature] /= norm;
                }
            }
        }
    }

    fn accuracy(&self, items: &[ReplayItem]) -> f32 {
        if items.is_empty() {
            return 0.0;
        }
        items
            .iter()
            .filter(|item| self.predict(&item.features) == item.label)
            .count() as f32
            / items.len() as f32
    }
}

fn consolidation_note(quick: bool) -> String {
    let seeds = if quick { 3 } else { 10 };
    let mut mode_forgetting = Vec::new();
    let mut mode_accuracy = Vec::new();
    let modes = [
        ConsolidationMode::NoSleep,
        ConsolidationMode::ExactReplay,
        ConsolidationMode::GenerativeReplay,
        ConsolidationMode::OfflineLocal,
    ];
    for mode in modes {
        let mut forgettings = Vec::new();
        let mut accuracies = Vec::new();
        for seed_index in 0..seeds {
            let stream_config = if quick {
                ClassIncConfig::quick(0x2100 + seed_index as u64)
            } else {
                ClassIncConfig::scientific(0x2100 + seed_index as u64)
            };
            let n_classes = stream_config.n_classes;
            let n_features = stream_config.n_features;
            let budget = ConsolidationBudget {
                max_items: if quick { 24 } else { 64 },
                offline_updates: if quick { 24 } else { 64 },
            };
            let mut stream = ClassIncrementalStream::new(stream_config);
            let mut learner = LocalLearner::new(n_classes, n_features);
            let mut exact = ExactReplayBuffer::new(budget.max_items);
            let mut generator = GenerativeReplay::new();
            let mut learned_accuracy = vec![0.0; n_classes];
            while !stream.exhausted() {
                let class = stream.phase();
                for example in stream.drain_phase_train() {
                    let item = ReplayItem {
                        features: example.flat_features(),
                        label: example.label,
                        source: ReplaySource::Train,
                    };
                    learner.observe(&item, true);
                    exact.observe(item.clone());
                    generator.observe(&item);
                }
                let probe = probe_items(&stream, class as u32);
                learned_accuracy[class] = learner.accuracy(&probe);
                for replay in
                    replay_schedule(mode, &exact, &generator, budget, 0x51EE_0000 + class as u64)
                {
                    learner.observe(&replay, mode != ConsolidationMode::OfflineLocal);
                }
                if !stream.advance_phase() {
                    break;
                }
            }
            let mut final_acc = Vec::new();
            let mut forgetting = Vec::new();
            for (class, &initial_accuracy) in learned_accuracy.iter().enumerate() {
                let accuracy = learner.accuracy(&probe_items(&stream, class as u32));
                final_acc.push(accuracy);
                forgetting
                    .push(Metrics::forgetting(initial_accuracy as f64, accuracy as f64) as f32);
            }
            accuracies.push(mean(&final_acc));
            forgettings.push(mean(&forgetting));
        }
        mode_accuracy.push((mode, mean(&accuracies)));
        mode_forgetting.push((mode, mean(&forgettings)));
    }
    let mut table = String::new();
    for (mode, forgetting) in &mode_forgetting {
        let accuracy = mode_accuracy
            .iter()
            .find(|(candidate, _)| candidate == mode)
            .map(|(_, value)| *value)
            .unwrap_or(0.0);
        table.push_str(&format!(
            "| {} | {:.4} | {:.4} |\n",
            mode.as_str(),
            forgetting,
            accuracy
        ));
    }
    format!(
        "# U21 — offline consolidation / replay\n\n\
         **Exploratory post-G2 override.** `{G2_HASH}` remains a FAIL.\n\n\
         - schedule: {}\n- seeds: {seeds}\n\
         - exact and generated arms use the same replay/update budget\n\
         - test examples are rejected by replay storage and generation\n\n\
         | arm | mean forgetting | final mean accuracy |\n|---|---:|---:|\n{table}\n\
         `offline-local-consolidation` drops replay labels and reinforces the \
         locally predicted assembly; the other replay arms disclose supervised \
         labels. The comparison identifies whether offline local consolidation \
         adds value beyond matched replay without changing the C1 decision.\n",
        if quick { "PILOT" } else { "scientific" }
    )
}

fn probe_items(stream: &ClassIncrementalStream, class: u32) -> Vec<ReplayItem> {
    stream
        .probe_class(class)
        .into_iter()
        .map(|example| ReplayItem {
            features: example.flat_features(),
            label: example.label,
            source: ReplaySource::Test,
        })
        .collect()
}

fn pruning_note(quick: bool) -> String {
    let strategies = [
        PruningStrategy::Magnitude,
        PruningStrategy::Age,
        PruningStrategy::Eligibility,
        PruningStrategy::Random,
    ];
    let mut rows = String::new();
    for strategy in strategies {
        let mut engine = classification_engine(0x2200);
        let before = classifier_accuracy(&engine, 200, 0x2201);
        let report = prune(&mut engine, strategy, 0.5, 0x2202);
        let after = classifier_accuracy(&engine, 200, 0x2201);
        recover_classifier(&mut engine, if quick { 80 } else { 400 }, 0x2203);
        let recovered = classifier_accuracy(&engine, 200, 0x2201);
        let old_after_recovery = classifier_accuracy_classes(&engine, 200, 0x2204, &[0, 1]);
        rows.push_str(&format!(
            "| {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.3} |\n",
            strategy.as_str(),
            before,
            after,
            recovered,
            old_after_recovery,
            report.realized_sparsity
        ));
    }
    format!(
        "# U22 — active forgetting / synaptic pruning\n\n\
         **Exploratory post-G2 override.** `{G2_HASH}` remains a FAIL.\n\n\
         - schedule: {}\n- matched target sparsity: 0.50\n\
         - recovery uses local reward-modulated regrowth on the unchanged CSR topology\n\n\
         | rule | accuracy before | after prune | after recovery | old-class retention | realized sparsity |\n\
         |---|---:|---:|---:|---:|---:|\n{rows}\n\
         Magnitude, age, eligibility, and random pruning receive the exact same \
         edge budget. The table exposes recovery, interference on old classes, \
         and retained active capacity; no rule is selected post hoc.\n",
        if quick { "PILOT" } else { "scientific" }
    )
}

fn classification_engine(seed: u64) -> Engine {
    let n_features = 16usize;
    let n_classes = 4usize;
    let n_cells = n_features + n_classes;
    let rows: Vec<Vec<u32>> = (0..n_cells)
        .map(|pre| {
            if pre < n_features {
                (n_features..n_cells).map(|post| post as u32).collect()
            } else {
                Vec::new()
            }
        })
        .collect();
    let conn = Csr::from_adjacency(&rows);
    let mut rng = Rng::new(seed);
    let mut weights = Vec::with_capacity(conn.nnz());
    for feature in 0..n_features {
        for class in 0..n_classes {
            let relevant = feature % n_classes == class;
            weights.push(if relevant { 0.8 } else { 0.08 } + rng.next_f32() * 0.02);
        }
    }
    let mut engine = Engine::with_cells(n_cells);
    engine.set_connectivity(conn, weights);
    for (i, synapse) in engine.syn.as_mut_slice().iter_mut().enumerate() {
        synapse.eligibility = engine.edge_w[i];
        synapse.last_elig_update = i as u64;
    }
    engine
}

fn draw_classification(rng: &mut Rng, label: usize) -> Vec<f32> {
    (0..16)
        .map(|feature| {
            let prototype = if feature % 4 == label { 0.9 } else { 0.1 };
            (0.9 * prototype + 0.1 * rng.next_f32()).clamp(0.0, 1.0)
        })
        .collect()
}

fn predict_engine(engine: &Engine, features: &[f32]) -> usize {
    (0..4)
        .max_by(|&a, &b| {
            let score = |class: usize| {
                (0..16)
                    .map(|feature| engine.edge_w[feature * 4 + class] * features[feature])
                    .sum::<f32>()
            };
            score(a)
                .partial_cmp(&score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0)
}

fn classifier_accuracy(engine: &Engine, n: usize, seed: u64) -> f32 {
    classifier_accuracy_classes(engine, n, seed, &[0, 1, 2, 3])
}

fn classifier_accuracy_classes(engine: &Engine, n: usize, seed: u64, classes: &[usize]) -> f32 {
    let mut rng = Rng::new(seed);
    let correct = (0..n)
        .filter(|i| {
            let label = classes[*i % classes.len()];
            predict_engine(engine, &draw_classification(&mut rng, label)) == label
        })
        .count();
    correct as f32 / n as f32
}

fn recover_classifier(engine: &mut Engine, n: usize, seed: u64) {
    let mut rng = Rng::new(seed);
    for i in 0..n {
        let label = i % 4;
        let features = draw_classification(&mut rng, label);
        let prediction = predict_engine(engine, &features);
        for (feature, &value) in features.iter().enumerate() {
            let reward = 0.04 * value;
            let correct_edge = feature * 4 + label;
            engine.edge_w[correct_edge] += reward;
            engine.syn.get_mut(correct_edge).expect("edge").weight = engine.edge_w[correct_edge];
            if prediction != label {
                let wrong_edge = feature * 4 + prediction;
                engine.edge_w[wrong_edge] -= reward;
                engine.syn.get_mut(wrong_edge).expect("edge").weight = engine.edge_w[wrong_edge];
            }
        }
    }
}

fn resting_note(quick: bool) -> String {
    let ticks = if quick { 300 } else { 3_000 };
    let (mut engine, templates) = resting_engine(0x2300);
    let config = RestingConfig {
        seed: 0x2301,
        ticks,
        background_probability: 0.015,
        background_drive: 1.05,
        reactivation_overlap: 0.25,
    };
    let raster = simulate_resting(&mut engine, config);
    let actual = characterize(&raster, &templates, config.reactivation_overlap);
    let nulls = [
        RestingNull::RateMatched,
        RestingNull::ActivityMatched,
        RestingNull::SpectrumMatched,
    ];
    let mut table = format!(
        "| observed | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
        actual.mean_activity,
        actual.metastability,
        actual.reactivation_rate,
        actual.transition_rate,
        actual.lag1_autocorrelation
    );
    let mut spectrum_raster = None;
    for null in nulls {
        let null_raster = matched_null(&raster, null, 0x2302);
        let metrics = characterize(&null_raster, &templates, config.reactivation_overlap);
        table.push_str(&format!(
            "| {:?} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
            null,
            metrics.mean_activity,
            metrics.metastability,
            metrics.reactivation_rate,
            metrics.transition_rate,
            metrics.lag1_autocorrelation
        ));
        if null == RestingNull::SpectrumMatched {
            spectrum_raster = Some(null_raster);
        }
    }
    let observed_gain = replay_structure_score(&raster, &templates);
    let ablated_gain = replay_structure_score(&spectrum_raster.expect("spectrum null"), &templates);
    format!(
        "# U23 — resting-state dynamics\n\n\
         **Exploratory post-G2 override.** `{G2_HASH}` remains a FAIL.\n\n\
         - schedule: {}\n- ticks: {ticks}\n\
         - stimulus-free background is unlabeled endogenous noise\n\
         - causal consolidation proxy, structured rest: {:.4}\n\
         - causal ablation, spectrum-matched rest: {:.4}\n\n\
         | condition | mean activity | metastability | reactivation | transitions | lag-1 autocorrelation |\n\
         |---|---:|---:|---:|---:|---:|\n{table}\n\
         The spectrum null circularly shifts each cell train, preserving its \
         temporal spectrum while ablating coordinated assembly timing. This is \
         characterized as resting-state dynamics, not a biological Default Mode Network.\n",
        if quick { "PILOT" } else { "scientific" },
        observed_gain,
        ablated_gain
    )
}

fn resting_engine(seed: u64) -> (Engine, Vec<Vec<u32>>) {
    let n_cells = 64usize;
    let templates: Vec<Vec<u32>> = (0..4)
        .map(|class| (0..8).map(|i| (class * 16 + i) as u32).collect())
        .collect();
    let mut rng = Rng::new(seed);
    let rows: Vec<Vec<u32>> = (0..n_cells)
        .map(|pre| {
            let group = pre / 16;
            let mut row = Vec::new();
            for _ in 0..6 {
                let post = group * 16 + rng.gen_index(16);
                if post != pre && !row.contains(&(post as u32)) {
                    row.push(post as u32);
                }
            }
            row
        })
        .collect();
    let conn = Csr::from_adjacency(&rows);
    let mut engine = Engine::with_cells(n_cells);
    engine.set_connectivity(conn.clone(), vec![0.22; conn.nnz()]);
    (engine, templates)
}

fn replay_structure_score(raster: &RestingRaster, templates: &[Vec<u32>]) -> f32 {
    if raster.spikes_by_tick.is_empty() {
        return 0.0;
    }
    let structured = raster
        .spikes_by_tick
        .iter()
        .map(|spikes| {
            templates
                .iter()
                .map(|template| spikes.iter().filter(|cell| template.contains(cell)).count())
                .max()
                .unwrap_or(0) as f32
        })
        .sum::<f32>();
    structured / raster.spikes_by_tick.len() as f32
}
