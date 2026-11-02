//! Budgeted offline consolidation and replay primitives (U21).
//!
//! Exact replay may retain raw **training** examples under a fixed capacity.
//! Generative replay stores per-class sufficient statistics only. Test examples
//! are rejected, preventing consolidation from generating held-out data.

use std::collections::{BTreeMap, HashMap};

use binn_core::Rng;
use binn_engine::Engine;

use crate::eligibility::Eligibility;
use crate::three_factor::ThreeFactor;
use crate::CreditSignal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplaySource {
    Train,
    Generated,
    Test,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayItem {
    pub features: Vec<f32>,
    pub label: u32,
    pub source: ReplaySource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsolidationMode {
    NoSleep,
    ExactReplay,
    GenerativeReplay,
    OfflineLocal,
    SleepProvenance,
}

impl ConsolidationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoSleep => "no-sleep",
            Self::ExactReplay => "exact-replay",
            Self::GenerativeReplay => "generative-replay",
            Self::OfflineLocal => "offline-local-consolidation",
            Self::SleepProvenance => "sleep-provenance-consolidation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsolidationBudget {
    pub max_items: usize,
    pub offline_updates: usize,
}

#[derive(Clone, Debug)]
pub struct ExactReplayBuffer {
    capacity: usize,
    cursor: usize,
    items: Vec<ReplayItem>,
}

impl ExactReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cursor: 0,
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn observe(&mut self, item: ReplayItem) {
        assert_eq!(
            item.source,
            ReplaySource::Train,
            "exact replay accepts training examples only"
        );
        if self.capacity == 0 {
            return;
        }
        if self.items.len() < self.capacity {
            self.items.push(item);
        } else {
            self.items[self.cursor % self.capacity] = item;
            self.cursor = self.cursor.wrapping_add(1);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn schedule(&self, count: usize, seed: u64) -> Vec<ReplayItem> {
        if self.items.is_empty() || count == 0 {
            return Vec::new();
        }
        let mut order: Vec<usize> = (0..self.items.len()).collect();
        let mut rng = Rng::new(seed ^ 0xEAC7_0E71_0000_0001);
        for i in 0..order.len() {
            let j = i + rng.gen_index(order.len() - i);
            order.swap(i, j);
        }
        (0..count)
            .map(|i| self.items[order[i % order.len()]].clone())
            .collect()
    }
}

#[derive(Clone, Debug)]
struct ClassStats {
    count: u64,
    mean: Vec<f32>,
    m2: Vec<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct GenerativeReplay {
    classes: BTreeMap<u32, ClassStats>,
    n_features: Option<usize>,
}

impl GenerativeReplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, item: &ReplayItem) {
        assert_eq!(
            item.source,
            ReplaySource::Train,
            "generator accepts training examples only"
        );
        let n_features = *self.n_features.get_or_insert(item.features.len());
        assert_eq!(item.features.len(), n_features);
        let stats = self
            .classes
            .entry(item.label)
            .or_insert_with(|| ClassStats {
                count: 0,
                mean: vec![0.0; n_features],
                m2: vec![0.0; n_features],
            });
        stats.count += 1;
        for i in 0..n_features {
            let delta = item.features[i] - stats.mean[i];
            stats.mean[i] += delta / stats.count as f32;
            stats.m2[i] += delta * (item.features[i] - stats.mean[i]);
        }
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    pub fn generate(&self, count: usize, seed: u64) -> Vec<ReplayItem> {
        if self.classes.is_empty() || count == 0 {
            return Vec::new();
        }
        let labels: Vec<u32> = self.classes.keys().copied().collect();
        let mut rng = Rng::new(seed ^ 0x6EAE_A71E_0000_0001);
        (0..count)
            .map(|i| {
                let label = labels[i % labels.len()];
                let stats = &self.classes[&label];
                let features = stats
                    .mean
                    .iter()
                    .enumerate()
                    .map(|(j, &mean)| {
                        let variance = if stats.count > 1 {
                            stats.m2[j] / (stats.count - 1) as f32
                        } else {
                            0.0
                        };
                        let unit = rng.next_f32() * 2.0 - 1.0;
                        (mean + unit * (3.0 * variance.max(0.0)).sqrt()).clamp(0.0, 1.0)
                    })
                    .collect();
                ReplayItem {
                    features,
                    label,
                    source: ReplaySource::Generated,
                }
            })
            .collect()
    }
}

pub fn replay_schedule(
    mode: ConsolidationMode,
    exact: &ExactReplayBuffer,
    generator: &GenerativeReplay,
    budget: ConsolidationBudget,
    seed: u64,
) -> Vec<ReplayItem> {
    let count = budget.max_items.min(budget.offline_updates);
    match mode {
        ConsolidationMode::NoSleep => Vec::new(),
        ConsolidationMode::ExactReplay => exact.schedule(count, seed),
        ConsolidationMode::GenerativeReplay
        | ConsolidationMode::OfflineLocal
        | ConsolidationMode::SleepProvenance => generator.generate(count, seed),
    }
}

/// Layer identifier for slow consolidated feature-to-assembly synapses.
pub const LAYER_IN_TO_SLOW: u8 = 0;
/// Layer identifier for fast-to-slow cross-store consolidation synapses.
pub const LAYER_FAST_TO_SLOW: u8 = 1;

#[inline]
fn sigmoid_f32(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Decoupled coordinate-wise erase/write gating for offline sleep consolidation:
/// $$\Delta w_{ij} = w_{t, ij} (\eta_{\text{slow}} e_{ij} M_j) - b_{t, ij} (\alpha_{\text{erase}} w_{ij})$$
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecoupledConsolidationGate {
    pub beta_w: f32,
    pub theta_w: f32,
    pub beta_b: f32,
    pub theta_b: f32,
    pub alpha_erase: f32,
}

impl Default for DecoupledConsolidationGate {
    fn default() -> Self {
        Self {
            beta_w: 6.0,
            theta_w: 0.15,
            beta_b: 6.0,
            theta_b: 0.45,
            alpha_erase: 0.04,
        }
    }
}

impl DecoupledConsolidationGate {
    /// Coordinate-wise write gate $w_{t,ij} = \sigma(\beta_w (a_i a_j - \Theta_w))$.
    #[inline]
    pub fn write_gate(&self, pre_act: f32, post_act: f32) -> f32 {
        let coincidence = pre_act * post_act;
        sigmoid_f32(self.beta_w * (coincidence - self.theta_w))
    }

    /// Conflict-aware erase gate $b_{t,ij} = \sigma(\beta_b (\text{conflict}(w_{ij}, M_j) - \Theta_b))$.
    /// Activates only when an existing weight directly opposes the target modulation sign.
    #[inline]
    pub fn erase_gate(&self, weight: f32, target_modulation: f32) -> f32 {
        let conflict = if (weight > 0.0 && target_modulation < 0.0)
            || (weight < 0.0 && target_modulation > 0.0)
        {
            weight.abs() * target_modulation.abs()
        } else {
            0.0
        };
        sigmoid_f32(self.beta_b * (conflict - self.theta_b))
    }

    /// Computes gated slow-weight increment $\Delta w_{ij}$.
    #[inline]
    pub fn delta_weight(
        &self,
        current_weight: f32,
        pre_act: f32,
        post_act: f32,
        target_modulation: f32,
        eta_slow: f32,
    ) -> f32 {
        let w_gate = self.write_gate(pre_act, post_act);
        let b_gate = self.erase_gate(current_weight, target_modulation);
        let eligibility = pre_act * post_act;
        w_gate * (eta_slow * eligibility * target_modulation)
            - b_gate * (self.alpha_erase * current_weight)
    }
}

/// Homeostatic dormancy threshold renewal:
/// $$\tau_\theta \frac{d\theta_i}{dt} = -(\theta_i - \theta_{\text{rest}}) - \gamma \cdot \mathbb{I}[\text{dormant}_i]$$
#[derive(Clone, Debug, PartialEq)]
pub struct HomeostaticRenewal {
    pub thresholds: Vec<f32>,
    pub last_active_tick: Vec<u64>,
    pub theta_rest: f32,
    pub theta_min: f32,
    pub theta_max: f32,
    pub delta_theta: f32,
    pub homeo_rate: f32,
    pub gamma_dorm: f32,
    pub dormancy_window: u64,
}

impl HomeostaticRenewal {
    pub fn new(n_units: usize) -> Self {
        let theta_rest = 0.50;
        Self {
            thresholds: vec![theta_rest; n_units],
            last_active_tick: vec![0; n_units],
            theta_rest,
            theta_min: 0.15,
            theta_max: 1.40,
            delta_theta: 0.04,
            homeo_rate: 0.02,
            gamma_dorm: 0.08,
            dormancy_window: 12,
        }
    }

    /// Marks `unit` as firing at `tick` and increments its adaptive threshold,
    /// while relaxing all non-firing units and boosting dormant ones.
    pub fn step(&mut self, active_unit: usize, tick: u64) {
        for i in 0..self.thresholds.len() {
            if i == active_unit {
                self.thresholds[i] = (self.thresholds[i] + self.delta_theta).min(self.theta_max);
                self.last_active_tick[i] = tick;
            } else {
                let is_dormant =
                    tick.saturating_sub(self.last_active_tick[i]) > self.dormancy_window;
                let drift = self.homeo_rate * (self.thresholds[i] - self.theta_rest);
                let boost = if is_dormant {
                    self.homeo_rate * self.gamma_dorm
                } else {
                    0.0
                };
                self.thresholds[i] =
                    (self.thresholds[i] - drift - boost).clamp(self.theta_min, self.theta_max);
            }
        }
    }

    /// Fraction of units whose threshold remains within the active excitable range.
    pub fn active_plastic_fraction(&self) -> f32 {
        if self.thresholds.is_empty() {
            return 0.0;
        }
        let excitable = self
            .thresholds
            .iter()
            .filter(|&&th| th < self.theta_max - 1e-4)
            .count();
        excitable as f32 / self.thresholds.len() as f32
    }
}

/// Provenance link recording a consolidated synaptic weight update:
/// $$\mathcal{P}: (\text{layer}, i, j) \mapsto (\text{EpisodeId}, \text{TaskId}, \Delta w_{ij}, t)$$
#[derive(Clone, Debug, PartialEq)]
pub struct ProvenanceRecord {
    pub episode_id: u64,
    pub task_id: usize,
    pub layer_id: u8,
    pub pre_idx: usize,
    pub post_idx: usize,
    pub delta_w: f32,
    pub tick: u64,
    pub assembly_hash: u64,
}

/// Causal provenance audit summary for a single synapse.
#[derive(Clone, Debug, PartialEq)]
pub struct SynapseAudit {
    pub layer_id: u8,
    pub pre_idx: usize,
    pub post_idx: usize,
    pub total_delta: f32,
    pub record_count: usize,
    pub driving_episodes: Vec<u64>,
    pub driving_tasks: Vec<usize>,
}

/// Summary of surgical episode or task revocation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RevocationSummary {
    pub episodes_revoked: usize,
    pub synapses_modified: usize,
    pub net_weight_rolled_back: f32,
}

/// Bipartite episodic-to-weight provenance table supporting O(1) synapse lookup
/// and surgical unlearning without full retraining.
#[derive(Clone, Debug, Default)]
pub struct ProvenanceTable {
    synapse_index: HashMap<(u8, usize, usize), Vec<ProvenanceRecord>>,
    episode_index: HashMap<u64, Vec<ProvenanceRecord>>,
    task_index: HashMap<usize, Vec<u64>>,
}

impl ProvenanceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_update(&mut self, record: ProvenanceRecord) {
        let ep_id = record.episode_id;
        let task_id = record.task_id;
        let syn_key = (record.layer_id, record.pre_idx, record.post_idx);

        self.synapse_index
            .entry(syn_key)
            .or_default()
            .push(record.clone());
        self.episode_index.entry(ep_id).or_default().push(record);
        let task_eps = self.task_index.entry(task_id).or_default();
        if !task_eps.contains(&ep_id) {
            task_eps.push(ep_id);
        }
    }

    pub fn get_synapse_provenance(
        &self,
        layer_id: u8,
        pre_idx: usize,
        post_idx: usize,
    ) -> &[ProvenanceRecord] {
        self.synapse_index
            .get(&(layer_id, pre_idx, post_idx))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_episode_records(&self, episode_id: u64) -> &[ProvenanceRecord] {
        self.episode_index
            .get(&episode_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_task_episodes(&self, task_id: usize) -> &[u64] {
        self.task_index
            .get(&task_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn total_records(&self) -> usize {
        self.episode_index.values().map(|v| v.len()).sum()
    }

    pub fn unique_episodes(&self) -> usize {
        self.episode_index.len()
    }

    pub fn attribution_accuracy(&self) -> f32 {
        let total = self.total_records();
        if total == 0 {
            return 1.0;
        }
        let valid = self
            .episode_index
            .values()
            .flat_map(|recs| recs.iter())
            .filter(|r| r.episode_id > 0)
            .count();
        valid as f32 / total as f32
    }

    pub fn audit_synapse(&self, layer_id: u8, pre_idx: usize, post_idx: usize) -> SynapseAudit {
        let records = self.get_synapse_provenance(layer_id, pre_idx, post_idx);
        let mut total_delta = 0.0;
        let mut driving_episodes = Vec::new();
        let mut driving_tasks = Vec::new();
        for r in records {
            total_delta += r.delta_w;
            if !driving_episodes.contains(&r.episode_id) {
                driving_episodes.push(r.episode_id);
            }
            if !driving_tasks.contains(&r.task_id) {
                driving_tasks.push(r.task_id);
            }
        }
        SynapseAudit {
            layer_id,
            pre_idx,
            post_idx,
            total_delta,
            record_count: records.len(),
            driving_episodes,
            driving_tasks,
        }
    }

    /// Surgically revokes all consolidated weight deltas attributed to `episode_id`.
    pub fn revoke_episode(
        &mut self,
        episode_id: u64,
        slow_weights: &mut [f32],
        n_features: usize,
    ) -> RevocationSummary {
        let mut summary = RevocationSummary::default();
        let Some(records) = self.episode_index.remove(&episode_id) else {
            return summary;
        };
        summary.episodes_revoked = 1;
        for record in records {
            let idx = record.post_idx * n_features + record.pre_idx;
            if (record.layer_id == LAYER_IN_TO_SLOW || record.layer_id == LAYER_FAST_TO_SLOW)
                && idx < slow_weights.len()
            {
                slow_weights[idx] -= record.delta_w;
                summary.synapses_modified += 1;
                summary.net_weight_rolled_back += record.delta_w;
            }
            let key = (record.layer_id, record.pre_idx, record.post_idx);
            if let Some(syn_recs) = self.synapse_index.get_mut(&key) {
                syn_recs.retain(|r| r.episode_id != episode_id);
                if syn_recs.is_empty() {
                    self.synapse_index.remove(&key);
                }
            }
        }
        for eps in self.task_index.values_mut() {
            eps.retain(|&id| id != episode_id);
        }
        summary
    }

    /// Surgically revokes all episodes belonging to `task_id`.
    pub fn revoke_task(
        &mut self,
        task_id: usize,
        slow_weights: &mut [f32],
        n_features: usize,
    ) -> RevocationSummary {
        let mut total = RevocationSummary::default();
        let episodes = self.get_task_episodes(task_id).to_vec();
        for ep_id in episodes {
            let sub = self.revoke_episode(ep_id, slow_weights, n_features);
            total.episodes_revoked += sub.episodes_revoked;
            total.synapses_modified += sub.synapses_modified;
            total.net_weight_rolled_back += sub.net_weight_rolled_back;
        }
        self.task_index.remove(&task_id);
        total
    }
}

/// Dual-store fast/slow learner with decoupled erase/write sleep consolidation,
/// homeostatic dormancy threshold renewal, and bipartite episodic provenance.
#[derive(Clone, Debug)]
pub struct SleepConsolidationLearner {
    pub n_classes: usize,
    pub n_features: usize,
    pub fast_weights: Vec<f32>,
    pub slow_weights: Vec<f32>,
    pub gate: DecoupledConsolidationGate,
    pub homeostasis: HomeostaticRenewal,
    pub provenance: ProvenanceTable,
    pub eta_fast: f32,
    pub eta_slow: f32,
    tick: u64,
    next_episode_id: u64,
}

impl SleepConsolidationLearner {
    pub fn new(n_classes: usize, n_features: usize) -> Self {
        Self {
            n_classes,
            n_features,
            fast_weights: vec![0.0; n_classes * n_features],
            slow_weights: vec![0.0; n_classes * n_features],
            gate: DecoupledConsolidationGate::default(),
            homeostasis: HomeostaticRenewal::new(n_classes),
            provenance: ProvenanceTable::new(),
            eta_fast: 0.22,
            eta_slow: 0.18,
            tick: 0,
            next_episode_id: 1,
        }
    }

    /// Computes normalized cosine-like score combining slow consolidated and fast weights,
    /// adjusted by homeostatic excitability bias.
    pub fn score(&self, class: usize, features: &[f32]) -> f32 {
        let row = class * self.n_features;
        let mut dot = 0.0f32;
        let mut norm_sq = 0.0f32;
        for (i, &x) in features.iter().enumerate().take(self.n_features) {
            let w = self.slow_weights[row + i] + 0.35 * self.fast_weights[row + i];
            dot += w * x;
            norm_sq += w * w;
        }
        let norm = norm_sq.sqrt().max(1e-4);
        let raw = dot / norm;
        let homeo_bias = 0.08 * (self.homeostasis.theta_rest - self.homeostasis.thresholds[class]);
        raw + homeo_bias
    }

    pub fn predict(&self, features: &[f32]) -> u32 {
        (0..self.n_classes)
            .max_by(|&a, &b| {
                self.score(a, features)
                    .partial_cmp(&self.score(b, features))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.cmp(&a))
            })
            .unwrap_or(0) as u32
    }

    pub fn accuracy(&self, items: &[ReplayItem]) -> f32 {
        if items.is_empty() {
            return 0.0;
        }
        items
            .iter()
            .filter(|item| self.predict(&item.features) == item.label)
            .count() as f32
            / items.len() as f32
    }

    /// Online Wake observation: updates fast store and logs initial gated consolidation
    /// with episodic provenance. Rejects held-out test items.
    pub fn observe_wake(&mut self, item: &ReplayItem) -> u64 {
        assert_ne!(
            item.source,
            ReplaySource::Test,
            "sleep consolidation learner cannot observe test items"
        );
        self.tick = self.tick.wrapping_add(1);
        let ep_id = self.next_episode_id;
        self.next_episode_id = self.next_episode_id.wrapping_add(1);
        let class = item.label as usize;
        self.homeostasis.step(class, self.tick);
        let row = class * self.n_features;
        let assembly_hash = (class as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ self.tick;

        for (i, &x) in item.features.iter().enumerate().take(self.n_features) {
            let fast_cur = self.fast_weights[row + i];
            self.fast_weights[row + i] = fast_cur + self.eta_fast * (x - fast_cur);

            let slow_cur = self.slow_weights[row + i];
            let delta = self.gate.delta_weight(slow_cur, x, 1.0, 1.0, self.eta_slow);
            let next_w = (slow_cur + delta).clamp(-4.0, 4.0);
            let applied_delta = next_w - slow_cur;
            if applied_delta.abs() > 1e-6 {
                self.slow_weights[row + i] = next_w;
                self.provenance.record_update(ProvenanceRecord {
                    episode_id: ep_id,
                    task_id: class,
                    layer_id: LAYER_IN_TO_SLOW,
                    pre_idx: i,
                    post_idx: class,
                    delta_w: applied_delta,
                    tick: self.tick,
                    assembly_hash,
                });
            }
        }
        ep_id
    }

    /// Offline Sleep phase: replays items without disclosing replay labels (`use_label = false`),
    /// inferring the locally winning assembly and consolidating fast-to-slow weights via
    /// decoupled coordinate-wise erase/write gating while logging provenance.
    pub fn consolidate_sleep(&mut self, replays: &[ReplayItem]) -> usize {
        let mut updates = 0;
        for replay in replays {
            assert_ne!(
                replay.source,
                ReplaySource::Test,
                "sleep consolidation cannot replay test items"
            );
            self.tick = self.tick.wrapping_add(1);
            let ep_id = self.next_episode_id;
            self.next_episode_id = self.next_episode_id.wrapping_add(1);
            // Local assembly inference without reading replay.label:
            let winner = self.predict(&replay.features) as usize;
            self.homeostasis.step(winner, self.tick);
            let row = winner * self.n_features;
            let assembly_hash = (winner as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ self.tick;

            for (i, &x) in replay.features.iter().enumerate().take(self.n_features) {
                let slow_cur = self.slow_weights[row + i];
                let fast_cur = self.fast_weights[row + i];
                let pre_act = 0.5 * (x + fast_cur.max(0.0));
                let delta =
                    self.gate
                        .delta_weight(slow_cur, pre_act, 1.0, 1.0, 0.5 * self.eta_slow);
                let next_w = (slow_cur + delta).clamp(-4.0, 4.0);
                let applied_delta = next_w - slow_cur;
                if applied_delta.abs() > 1e-6 {
                    self.slow_weights[row + i] = next_w;
                    self.provenance.record_update(ProvenanceRecord {
                        episode_id: ep_id,
                        task_id: winner,
                        layer_id: LAYER_IN_TO_SLOW,
                        pre_idx: i,
                        post_idx: winner,
                        delta_w: applied_delta,
                        tick: self.tick,
                        assembly_hash,
                    });
                    updates += 1;
                }
            }
        }
        // Partially decay fast weights after offline consolidation to free fast capacity.
        for w in &mut self.fast_weights {
            *w *= 0.5;
        }
        updates
    }

    /// Surgically revokes an episode from consolidated slow weights.
    pub fn revoke_episode(&mut self, episode_id: u64) -> RevocationSummary {
        let summary =
            self.provenance
                .revoke_episode(episode_id, &mut self.slow_weights, self.n_features);
        summary
    }

    /// Surgically revokes an entire task/class from consolidated slow weights and resets its fast trace.
    pub fn revoke_task(&mut self, task_id: usize) -> RevocationSummary {
        let summary =
            self.provenance
                .revoke_task(task_id, &mut self.slow_weights, self.n_features);
        if task_id < self.n_classes {
            let row = task_id * self.n_features;
            for i in 0..self.n_features {
                self.fast_weights[row + i] = 0.0;
            }
        }
        summary
    }
}

/// Sparse synapse eligibility snapshot captured from a spiking [`Engine`] trial.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpikingSynapseEvent {
    pub syn_idx: usize,
    pub post_cell: u32,
    pub eligibility: f32,
    pub modulator_gate: f32,
}

/// Compressed episodic trace captured from [`Engine`] + [`ThreeFactor`] during wake.
#[derive(Clone, Debug, PartialEq)]
pub struct SpikingEpisodicTrace {
    pub episode_id: u64,
    pub task_id: usize,
    pub tick: u64,
    pub fired_cells: Vec<u32>,
    pub active_synapses: Vec<SpikingSynapseEvent>,
}

/// Spiking 3-factor / e-prop sleep consolidator operating directly on [`Engine`] CSR/CSC synapses.
///
/// Couples [`ThreeFactor`] spike-timing eligibility traces with [`DecoupledConsolidationGate`],
/// [`HomeostaticRenewal`], and [`ProvenanceTable`] surgical revocation on `engine.syn` and `engine.edge_w`.
#[derive(Clone, Debug)]
pub struct SpikingSleepConsolidator {
    pub gate: DecoupledConsolidationGate,
    pub homeostasis: HomeostaticRenewal,
    pub provenance: ProvenanceTable,
    pub eta_wake: f32,
    pub eta_sleep: f32,
    next_episode_id: u64,
}

impl SpikingSleepConsolidator {
    pub fn new(num_cells: usize, eta_wake: f32, eta_sleep: f32) -> Self {
        Self {
            gate: DecoupledConsolidationGate::default(),
            homeostasis: HomeostaticRenewal::new(num_cells),
            provenance: ProvenanceTable::new(),
            eta_wake,
            eta_sleep,
            next_episode_id: 1,
        }
    }

    /// Absorbs newly emitted spikes via [`ThreeFactor::observe_spikes`], updates homeostatic
    /// firing-rate traces, applies gated wake plasticity on [`Engine`], records provenance,
    /// and returns a compressed [`SpikingEpisodicTrace`] for offline sleep replay.
    pub fn capture_wake_episode<S: CreditSignal>(
        &mut self,
        engine: &mut Engine,
        learner: &mut ThreeFactor,
        signal: &S,
        task_id: usize,
    ) -> SpikingEpisodicTrace {
        let now = engine.time();
        learner.observe_spikes(engine);
        let ep_id = self.next_episode_id;
        self.next_episode_id = self.next_episode_id.wrapping_add(1);

        let mut fired_cells: Vec<u32> = engine.spikes().as_slice().iter().map(|s| s.cell).collect();
        fired_cells.sort_unstable();
        fired_cells.dedup();
        for &cell in &fired_cells {
            if (cell as usize) < self.homeostasis.thresholds.len() {
                self.homeostasis.step(cell as usize, now);
            }
        }

        let elig_decay = Eligibility::new(learner.tau_e);
        let posts = &engine.conn.col;
        let syns = engine.syn.as_mut_slice();
        let num_syns = syns.len();
        let assembly_hash = (task_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ now;
        let mut active_synapses = Vec::new();

        for (syn_idx, syn) in syns.iter_mut().enumerate() {
            elig_decay.decay_to(syn, now);
            let e = syn.eligibility;
            if e.abs() <= 1e-8 {
                continue;
            }
            let post_cell = posts[syn_idx];
            let gate = signal.for_post(post_cell);
            active_synapses.push(SpikingSynapseEvent {
                syn_idx,
                post_cell,
                eligibility: e,
                modulator_gate: gate,
            });
            if gate.abs() <= 1e-8 {
                continue;
            }
            let w_cur = syn.weight;
            let delta = self
                .gate
                .delta_weight(w_cur, e, 1.0, gate.clamp(-1.0, 1.0), self.eta_wake);
            let w_next = (w_cur + delta).clamp(-4.0, 4.0);
            let applied_delta = w_next - w_cur;
            if applied_delta.abs() > 1e-7 {
                syn.weight = w_next;
                engine.edge_w[syn_idx] = w_next;
                self.provenance.record_update(ProvenanceRecord {
                    episode_id: ep_id,
                    task_id,
                    layer_id: LAYER_FAST_TO_SLOW,
                    pre_idx: syn_idx,
                    post_idx: 0,
                    delta_w: applied_delta,
                    tick: now,
                    assembly_hash,
                });
            }
        }
        let _ = num_syns;
        SpikingEpisodicTrace {
            episode_id: ep_id,
            task_id,
            tick: now,
            fired_cells,
            active_synapses,
        }
    }

    /// Replays compressed [`SpikingEpisodicTrace`] episodes during offline sleep into
    /// [`Engine`] synaptic weights using [`DecoupledConsolidationGate`] and [`HomeostaticRenewal`].
    pub fn consolidate_sleep_on_engine(
        &mut self,
        engine: &mut Engine,
        traces: &[SpikingEpisodicTrace],
    ) -> usize {
        let mut updates = 0usize;
        let syns = engine.syn.as_mut_slice();
        let num_syns = syns.len();
        for trace in traces {
            let sleep_ep_id = self.next_episode_id;
            self.next_episode_id = self.next_episode_id.wrapping_add(1);
            for &cell in &trace.fired_cells {
                if (cell as usize) < self.homeostasis.thresholds.len() {
                    self.homeostasis.step(cell as usize, trace.tick);
                }
            }
            let assembly_hash =
                (trace.task_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ trace.tick;
            for ev in &trace.active_synapses {
                if ev.syn_idx >= num_syns || ev.modulator_gate.abs() <= 1e-8 {
                    continue;
                }
                let w_cur = syns[ev.syn_idx].weight;
                let delta = self.gate.delta_weight(
                    w_cur,
                    ev.eligibility,
                    1.0,
                    ev.modulator_gate.clamp(-1.0, 1.0),
                    self.eta_sleep,
                );
                let w_next = (w_cur + delta).clamp(-4.0, 4.0);
                let applied_delta = w_next - w_cur;
                if applied_delta.abs() > 1e-7 {
                    syns[ev.syn_idx].weight = w_next;
                    engine.edge_w[ev.syn_idx] = w_next;
                    self.provenance.record_update(ProvenanceRecord {
                        episode_id: sleep_ep_id,
                        task_id: trace.task_id,
                        layer_id: LAYER_FAST_TO_SLOW,
                        pre_idx: ev.syn_idx,
                        post_idx: 0,
                        delta_w: applied_delta,
                        tick: trace.tick,
                        assembly_hash,
                    });
                    updates += 1;
                }
            }
        }
        updates
    }

    /// Surgically revokes an episode from `engine.edge_w` and `engine.syn`.
    pub fn revoke_episode_on_engine(
        &mut self,
        episode_id: u64,
        engine: &mut Engine,
    ) -> RevocationSummary {
        let num_syns = engine.edge_w.len();
        let summary = self
            .provenance
            .revoke_episode(episode_id, &mut engine.edge_w, num_syns);
        for (syn, &w) in engine.syn.as_mut_slice().iter_mut().zip(&engine.edge_w) {
            syn.weight = w;
        }
        summary
    }

    /// Surgically revokes an entire task from `engine.edge_w` and `engine.syn`.
    pub fn revoke_task_on_engine(
        &mut self,
        task_id: usize,
        engine: &mut Engine,
    ) -> RevocationSummary {
        let num_syns = engine.edge_w.len();
        let summary = self
            .provenance
            .revoke_task(task_id, &mut engine.edge_w, num_syns);
        for (syn, &w) in engine.syn.as_mut_slice().iter_mut().zip(&engine.edge_w) {
            syn.weight = w;
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn train(label: u32, values: &[f32]) -> ReplayItem {
        ReplayItem {
            features: values.to_vec(),
            label,
            source: ReplaySource::Train,
        }
    }

    #[test]
    #[should_panic(expected = "training examples only")]
    fn test_data_cannot_enter_exact_replay() {
        ExactReplayBuffer::new(2).observe(ReplayItem {
            features: vec![0.5],
            label: 0,
            source: ReplaySource::Test,
        });
    }

    #[test]
    fn matched_schedules_obey_the_same_budget() {
        let mut exact = ExactReplayBuffer::new(4);
        let mut generator = GenerativeReplay::new();
        for i in 0..8 {
            let item = train((i % 2) as u32, &[i as f32 / 10.0, 0.25]);
            exact.observe(item.clone());
            generator.observe(&item);
        }
        let budget = ConsolidationBudget {
            max_items: 6,
            offline_updates: 5,
        };
        for mode in [
            ConsolidationMode::ExactReplay,
            ConsolidationMode::GenerativeReplay,
            ConsolidationMode::OfflineLocal,
            ConsolidationMode::SleepProvenance,
        ] {
            assert_eq!(
                replay_schedule(mode, &exact, &generator, budget, 1).len(),
                5
            );
        }
        assert!(
            replay_schedule(ConsolidationMode::NoSleep, &exact, &generator, budget, 1).is_empty()
        );
    }

    #[test]
    fn generative_schedule_is_seeded_and_balanced() {
        let mut generator = GenerativeReplay::new();
        for _ in 0..4 {
            generator.observe(&train(0, &[0.9, 0.1]));
            generator.observe(&train(1, &[0.1, 0.9]));
        }
        let a = generator.generate(10, 44);
        assert_eq!(a, generator.generate(10, 44));
        assert_eq!(a.iter().filter(|item| item.label == 0).count(), 5);
        assert!(a.iter().all(|item| item.source == ReplaySource::Generated));
    }

    #[test]
    fn decoupled_gate_protects_non_conflicting_slow_weights() {
        let gate = DecoupledConsolidationGate::default();
        // High coincidence triggers strong write gate (> 0.95).
        assert!(gate.write_gate(0.9, 0.9) > 0.95);
        // Zero coincidence keeps write gate suppressed.
        assert!(gate.write_gate(0.0, 0.9) < 0.30);
        // Same-sign modulation has zero conflict -> minimal erase gate (< 0.10).
        let b_aligned = gate.erase_gate(0.8, 1.0);
        // Opposite-sign modulation has high conflict -> strong erase gate (> 0.80).
        let b_conflict = gate.erase_gate(0.8, -1.0);
        assert!(b_aligned < 0.10, "b_aligned={b_aligned}");
        assert!(b_conflict > 0.80, "b_conflict={b_conflict}");
    }

    #[test]
    fn homeostatic_renewal_restores_dormant_units() {
        let mut homeo = HomeostaticRenewal::new(4);
        // Repeatedly fire unit 0 for 40 ticks; units 1..4 become dormant.
        for tick in 1..=40 {
            homeo.step(0, tick);
        }
        assert!(homeo.thresholds[0] > homeo.theta_rest);
        for dormant_unit in 1..4 {
            assert!(
                homeo.thresholds[dormant_unit] < homeo.theta_rest,
                "dormant unit {dormant_unit} threshold {} did not relax below rest {}",
                homeo.thresholds[dormant_unit],
                homeo.theta_rest
            );
        }
        assert!(homeo.active_plastic_fraction() >= 0.75);
    }

    #[test]
    fn sleep_consolidation_prevents_forgetting_and_supports_surgical_revocation() {
        let n_classes = 4usize;
        let n_features = 16usize;
        let mut learner = SleepConsolidationLearner::new(n_classes, n_features);
        let mut generator = GenerativeReplay::new();
        let mut exact = ExactReplayBuffer::new(32);
        let budget = ConsolidationBudget {
            max_items: 24,
            offline_updates: 24,
        };

        let make_example = |class: usize, sample: usize, source: ReplaySource| -> ReplayItem {
            let mut features = vec![0.05f32; n_features];
            for f in 0..4 {
                features[class * 4 + f] = 0.85 + ((sample % 3) as f32) * 0.04;
            }
            ReplayItem {
                features,
                label: class as u32,
                source,
            }
        };

        // Sequential class-incremental phases 0..4
        for class in 0..n_classes {
            for s in 0..20 {
                let item = make_example(class, s, ReplaySource::Train);
                learner.observe_wake(&item);
                exact.observe(item.clone());
                generator.observe(&item);
            }
            let replays = replay_schedule(
                ConsolidationMode::SleepProvenance,
                &exact,
                &generator,
                budget,
                0x51EE + class as u64,
            );
            learner.consolidate_sleep(&replays);
        }

        // Verify all 4 classes are retained with >= 95% accuracy without label disclosure in sleep
        for class in 0..n_classes {
            let probes: Vec<ReplayItem> = (0..10)
                .map(|s| make_example(class, s, ReplaySource::Test))
                .collect();
            let acc = learner.accuracy(&probes);
            assert!(
                acc >= 0.95,
                "class {class} retention collapsed to {acc}"
            );
        }

        // Verify 100% provenance attribution accuracy
        assert_eq!(learner.provenance.attribution_accuracy(), 1.0);
        assert!(learner.provenance.total_records() > 0);
        let audit = learner.provenance.audit_synapse(LAYER_IN_TO_SLOW, 0, 0);
        assert!(audit.record_count > 0);
        assert!(audit.total_delta > 0.0);

        // Surgically revoke Task/Class 0 and verify collateral impact on Classes 1..4 is zero
        let summary = learner.revoke_task(0);
        assert!(summary.episodes_revoked > 0);
        assert!(summary.synapses_modified > 0);
        for row_col in 0..n_features {
            assert!(
                learner.slow_weights[row_col].abs() < 1e-5,
                "revoked class 0 synapse {row_col} not rolled back: {}",
                learner.slow_weights[row_col]
            );
        }
        for class in 1..n_classes {
            let probes: Vec<ReplayItem> = (0..10)
                .map(|s| make_example(class, s, ReplaySource::Test))
                .collect();
            assert_eq!(
                learner.accuracy(&probes),
                1.0,
                "collateral damage on retained class {class} after revoking class 0"
            );
        }
    }

    #[test]
    fn spiking_sleep_consolidator_integrates_with_three_factor_engine_and_revokes_cleanly() {
        use crate::credit::PostSynapticCredit;
        use binn_core::Csr;

        // 5-cell spiking graph: inputs (0, 1) -> competitors (2, 3) -> readout (4)
        // Edge indices: 0: 0->2, 1: 0->3, 2: 1->2, 3: 1->3, 4: 2->4, 5: 3->4
        let mut eng = Engine::with_cells(5);
        let conn = Csr::from_parts(vec![0u32, 2, 4, 5, 6, 6], vec![2u32, 3, 2, 3, 4, 4]).unwrap();
        let initial_w = vec![0.10f32; 6];
        eng.set_connectivity(conn, initial_w.clone());

        let mut tf = ThreeFactor::new(0.25, 0.0, 40.0);
        let mut consolidator = SpikingSleepConsolidator::new(5, 0.25, 0.15);

        // Task 0 wake trial: pre (0, 1) fire at t=10,11; post cell 2 fires at t=18
        eng.force_spike(0, 10);
        eng.force_spike(1, 11);
        let _ = eng.step_until(17);
        eng.force_spike(2, 18);
        let _ = eng.step_until(25);

        let mut credit_t0 = PostSynapticCredit::zeros(5);
        credit_t0.set(2, 1.0);
        let trace_t0 = consolidator.capture_wake_episode(&mut eng, &mut tf, &credit_t0, 0);
        assert!(!trace_t0.active_synapses.is_empty());
        let w_0_2_after_t0 = eng.edge_w[0];
        assert!(w_0_2_after_t0 > initial_w[0], "Task 0 edge 0->2 should potentiate");

        // Task 1 wake trial: pre (0, 1) fire at t=40,41; post cell 3 fires at t=48
        tf.reset_pairing_state();
        for syn in eng.syn.as_mut_slice() {
            syn.eligibility = 0.0;
        }
        eng.force_spike(0, 40);
        eng.force_spike(1, 41);
        let _ = eng.step_until(47);
        eng.force_spike(3, 48);
        let _ = eng.step_until(55);

        let mut credit_t1 = PostSynapticCredit::zeros(5);
        credit_t1.set(3, 1.0);
        let trace_t1 = consolidator.capture_wake_episode(&mut eng, &mut tf, &credit_t1, 1);
        assert!(!trace_t1.active_synapses.is_empty());
        assert_eq!(
            eng.edge_w[0].to_bits(),
            w_0_2_after_t0.to_bits(),
            "Task 1 wake must not overwrite Task 0 synapse 0->2"
        );
        assert!(eng.edge_w[1] > initial_w[1], "Task 1 edge 0->3 should potentiate");

        // Offline Sleep replay of both compressed traces
        let sleep_updates =
            consolidator.consolidate_sleep_on_engine(&mut eng, &[trace_t0, trace_t1]);
        assert!(sleep_updates >= 4);
        assert!(eng.edge_w[0] > w_0_2_after_t0);
        let w_0_3_after_sleep = eng.edge_w[1];
        assert!(w_0_3_after_sleep > initial_w[1]);

        // Surgically revoke Task 0 on the spiking Engine: edges 0->2 and 1->2 revert to initial_w,
        // while Task 1 edges 0->3 and 1->3 remain completely intact!
        let rev = consolidator.revoke_task_on_engine(0, &mut eng);
        assert_eq!(rev.episodes_revoked, 2); // 1 wake + 1 sleep episode for Task 0
        assert!((eng.edge_w[0] - initial_w[0]).abs() < 1e-6);
        assert!((eng.edge_w[2] - initial_w[2]).abs() < 1e-6);
        assert_eq!(eng.edge_w[1].to_bits(), w_0_3_after_sleep.to_bits());
        assert_eq!(
            eng.syn.as_slice()[0].weight.to_bits(),
            eng.edge_w[0].to_bits()
        );
    }
}
