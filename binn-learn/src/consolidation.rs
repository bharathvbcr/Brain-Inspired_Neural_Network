//! Budgeted offline consolidation and replay primitives (U21).
//!
//! Exact replay may retain raw **training** examples under a fixed capacity.
//! Generative replay stores per-class sufficient statistics only. Test examples
//! are rejected, preventing consolidation from generating held-out data.

use std::collections::BTreeMap;

use binn_core::Rng;

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
}

impl ConsolidationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoSleep => "no-sleep",
            Self::ExactReplay => "exact-replay",
            Self::GenerativeReplay => "generative-replay",
            Self::OfflineLocal => "offline-local-consolidation",
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
        ConsolidationMode::GenerativeReplay | ConsolidationMode::OfflineLocal => {
            generator.generate(count, seed)
        }
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
}
