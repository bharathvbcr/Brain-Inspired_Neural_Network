//! Stimulus-free resting-state dynamics and matched nulls (U23).
//!
//! Background events are unlabeled endogenous noise, never dataset examples.
//! Rate- and activity-matched nulls randomize spike identities/times; the
//! spectrum-matched null circularly shifts each cell train, preserving each
//! cell's temporal spectrum exactly.

use binn_core::Rng;

use crate::{CellId, Engine};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RestingConfig {
    pub seed: u64,
    pub ticks: usize,
    pub background_probability: f32,
    pub background_drive: f32,
    pub reactivation_overlap: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestingRaster {
    pub n_cells: usize,
    pub spikes_by_tick: Vec<Vec<CellId>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestingNull {
    RateMatched,
    ActivityMatched,
    SpectrumMatched,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RestingMetrics {
    pub mean_activity: f32,
    pub metastability: f32,
    pub reactivation_rate: f32,
    pub transition_rate: f32,
    pub lag1_autocorrelation: f32,
}

pub fn simulate_resting(engine: &mut Engine, config: RestingConfig) -> RestingRaster {
    assert!(config.ticks > 1);
    assert!((0.0..=1.0).contains(&config.background_probability));
    assert!(config.background_drive.is_finite());
    let n_cells = engine.num_cells();
    let mut rng = Rng::new(config.seed ^ 0xAE57_1A6E_0000_0001);
    let mut spikes_by_tick = Vec::with_capacity(config.ticks);
    let mut previous_spike_len = engine.spikes().len();
    for _ in 0..config.ticks {
        let tick = engine.time() + 1;
        for cell in 0..n_cells as CellId {
            if rng.next_f32() < config.background_probability {
                engine.inject_weighted(cell, 0, tick, config.background_drive);
            }
        }
        engine.step_until(tick);
        let all = engine.spikes().as_slice();
        let mut tick_spikes: Vec<CellId> = all[previous_spike_len..]
            .iter()
            .map(|spike| spike.cell)
            .collect();
        tick_spikes.sort_unstable();
        tick_spikes.dedup();
        spikes_by_tick.push(tick_spikes);
        previous_spike_len = all.len();
    }
    RestingRaster {
        n_cells,
        spikes_by_tick,
    }
}

pub fn matched_null(raster: &RestingRaster, kind: RestingNull, seed: u64) -> RestingRaster {
    let ticks = raster.spikes_by_tick.len();
    let mut rng = Rng::new(seed ^ 0xA011_5EED_0000_0001 ^ kind as u64);
    let mut out = vec![Vec::new(); ticks];
    match kind {
        RestingNull::RateMatched => {
            // Redraw on a within-tick collision, so the null actually carries the
            // rate it is named for.
            //
            // This used to push `total` unconditional draws and rely on the
            // shared `dedup()` below, which deleted every collision: the null
            // emitted fewer spikes than the raster it matched, by an amount that
            // grew with density. Measured on this module's own fixture, it lost
            // spikes for 52 of 64 seeds -- and seed 7, the one its test
            // hardcoded, was one of the 12 that did not. The shortfall is
            // visible in `results/u23_resting.md`, where RateMatched reads
            // 0.0140 against an observed 0.0141 while both other nulls, which
            // already redrew, read 0.0141.
            //
            // `ActivityMatched` below has always done this. The two arms now
            // differ only in what they hold fixed, which is the point of having
            // both.
            let total: usize = raster.spikes_by_tick.iter().map(Vec::len).sum();
            let capacity = ticks.saturating_mul(raster.n_cells);
            let target = total.min(capacity);
            let mut placed = 0usize;
            while placed < target {
                let tick = rng.gen_index(ticks);
                let cell = rng.gen_index(raster.n_cells) as CellId;
                if !out[tick].contains(&cell) {
                    out[tick].push(cell);
                    placed += 1;
                }
            }
        }
        RestingNull::ActivityMatched => {
            for (tick, spikes) in raster.spikes_by_tick.iter().enumerate() {
                while out[tick].len() < spikes.len().min(raster.n_cells) {
                    let cell = rng.gen_index(raster.n_cells) as CellId;
                    if !out[tick].contains(&cell) {
                        out[tick].push(cell);
                    }
                }
            }
        }
        RestingNull::SpectrumMatched => {
            for cell in 0..raster.n_cells as CellId {
                let shift = rng.gen_index(ticks);
                for (tick, spikes) in raster.spikes_by_tick.iter().enumerate() {
                    if spikes.contains(&cell) {
                        out[(tick + shift) % ticks].push(cell);
                    }
                }
            }
        }
    }
    for spikes in &mut out {
        spikes.sort_unstable();
        spikes.dedup();
    }
    RestingRaster {
        n_cells: raster.n_cells,
        spikes_by_tick: out,
    }
}

pub fn characterize(
    raster: &RestingRaster,
    templates: &[Vec<CellId>],
    threshold: f32,
) -> RestingMetrics {
    assert!((0.0..=1.0).contains(&threshold));
    if raster.spikes_by_tick.is_empty() || raster.n_cells == 0 {
        return RestingMetrics::default();
    }
    let rates: Vec<f32> = raster
        .spikes_by_tick
        .iter()
        .map(|spikes| spikes.len() as f32 / raster.n_cells as f32)
        .collect();
    let mean_activity = rates.iter().sum::<f32>() / rates.len() as f32;
    let variance = rates
        .iter()
        .map(|rate| (rate - mean_activity).powi(2))
        .sum::<f32>()
        / rates.len() as f32;
    let metastability = variance.sqrt();

    let mut states = Vec::with_capacity(raster.spikes_by_tick.len());
    let mut reactivated = 0usize;
    for spikes in &raster.spikes_by_tick {
        let best = templates
            .iter()
            .enumerate()
            .map(|(i, template)| (i, jaccard(spikes, template)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        match best {
            Some((state, overlap)) if overlap >= threshold => {
                reactivated += 1;
                states.push(Some(state));
            }
            _ => states.push(None),
        }
    }
    let transitions = states
        .windows(2)
        .filter(|pair| pair[0].is_some() && pair[1].is_some() && pair[0] != pair[1])
        .count();
    let transition_rate = transitions as f32 / (states.len() - 1).max(1) as f32;
    RestingMetrics {
        mean_activity,
        metastability,
        reactivation_rate: reactivated as f32 / states.len() as f32,
        transition_rate,
        lag1_autocorrelation: lag1(&rates),
    }
}

fn jaccard(a: &[CellId], b: &[CellId]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.iter().filter(|cell| b.contains(cell)).count();
    let union = a.len() + b.len() - intersection;
    intersection as f32 / union.max(1) as f32
}

fn lag1(values: &[f32]) -> f32 {
    if values.len() < 3 {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let numerator = values
        .windows(2)
        .map(|pair| (pair[0] - mean) * (pair[1] - mean))
        .sum::<f32>();
    let denominator = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>();
    if denominator <= f32::EPSILON {
        0.0
    } else {
        (numerator / denominator).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raster() -> RestingRaster {
        RestingRaster {
            n_cells: 6,
            spikes_by_tick: vec![
                vec![0, 1],
                vec![0, 1],
                vec![3, 4],
                vec![3, 4],
                vec![],
                vec![0, 1],
            ],
        }
    }

    #[test]
    fn nulls_preserve_their_registered_marginals() {
        let source = raster();
        let source_total: usize = source.spikes_by_tick.iter().map(Vec::len).sum();
        // Every seed, not one. This assertion used to hold only for seed 7:
        // `RateMatched` dropped spikes on within-tick collisions and lost them
        // for 52 of the 64 seeds below.
        for seed in 0..64u64 {
            for kind in [
                RestingNull::RateMatched,
                RestingNull::ActivityMatched,
                RestingNull::SpectrumMatched,
            ] {
                let null = matched_null(&source, kind, seed);
                let total: usize = null.spikes_by_tick.iter().map(Vec::len).sum();
                assert_eq!(
                    total, source_total,
                    "{kind:?} emitted {total} spikes against {source_total} at seed {seed}"
                );
            }
        }
        for kind in [
            RestingNull::RateMatched,
            RestingNull::ActivityMatched,
            RestingNull::SpectrumMatched,
        ] {
            let null = matched_null(&source, kind, 7);
            assert_eq!(
                null.spikes_by_tick.iter().map(Vec::len).sum::<usize>(),
                source_total
            );
            assert_eq!(null, matched_null(&source, kind, 7));
        }
        let activity = matched_null(&source, RestingNull::ActivityMatched, 7);
        assert_eq!(
            activity
                .spikes_by_tick
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            source
                .spikes_by_tick
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn metrics_detect_template_reactivation_and_transitions() {
        let metrics = characterize(&raster(), &[vec![0, 1], vec![3, 4]], 0.8);
        assert!(metrics.reactivation_rate > 0.7);
        assert!(metrics.transition_rate > 0.0);
        assert!(metrics.metastability > 0.0);
    }

    #[test]
    fn stimulus_free_simulation_is_seeded() {
        let config = RestingConfig {
            seed: 9,
            ticks: 20,
            background_probability: 0.2,
            background_drive: 1.1,
            reactivation_overlap: 0.5,
        };
        let mut a = Engine::with_cells(8);
        let mut b = Engine::with_cells(8);
        assert_eq!(
            simulate_resting(&mut a, config),
            simulate_resting(&mut b, config)
        );
    }
}
