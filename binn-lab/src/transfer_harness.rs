//! Same-specification matched/live transfer harness.
//!
//! Only the two endpoint configurations are scheduled by the current
//! protocol. The four explicit axes remain independently parameterized so a
//! later preregistration can enumerate the 16-rung lattice without rewriting
//! dynamics.

#![allow(clippy::needless_range_loop)]

use binn_data::{
    TemporalOrderExample, TransferBundle, TEMPORAL_ORDER_N_CLASSES, TEMPORAL_ORDER_N_IN,
    TEMPORAL_ORDER_T,
};
use binn_engine::{Event, TimingWheel};

pub const TRANSFER_PROTOCOL_VERSION: u64 = 146;
pub const TRANSFER_HASH_PREFIX: &str = "transfer-v146";
pub const TRANSFER_K: usize = 8;
pub const TRACE_TAU_E: f32 = 40.0;
pub const MICRO_TOLERANCE: f32 = 1e-6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    AllUnits,
    HardKwta { k: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing {
    SynchronousFrames,
    EventQueueDelays,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThresholdReset {
    FixedSoftReset,
    AdaptiveHardReset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceMode {
    HeldUntilReward,
    RealtimeDecay { tau_e_bits: u32 },
}

impl TraceMode {
    pub fn realtime(tau_e: f32) -> Self {
        assert!(tau_e > 0.0 && tau_e.is_finite());
        Self::RealtimeDecay {
            tau_e_bits: tau_e.to_bits(),
        }
    }

    fn tau_e(self) -> Option<f32> {
        match self {
            Self::HeldUntilReward => None,
            Self::RealtimeDecay { tau_e_bits } => Some(f32::from_bits(tau_e_bits)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferPole {
    pub selection: Selection,
    pub timing: Timing,
    pub threshold_reset: ThresholdReset,
    pub trace: TraceMode,
}

impl TransferPole {
    pub const fn matched() -> Self {
        Self {
            selection: Selection::AllUnits,
            timing: Timing::SynchronousFrames,
            threshold_reset: ThresholdReset::FixedSoftReset,
            trace: TraceMode::HeldUntilReward,
        }
    }

    pub fn live() -> Self {
        Self {
            selection: Selection::HardKwta { k: TRANSFER_K },
            timing: Timing::EventQueueDelays,
            threshold_reset: ThresholdReset::AdaptiveHardReset,
            trace: TraceMode::realtime(TRACE_TAU_E),
        }
    }

    pub fn hash_string(self, bundle_fingerprint: u64, epochs: usize) -> String {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for value in [
            TRANSFER_PROTOCOL_VERSION,
            bundle_fingerprint,
            epochs as u64,
            selection_code(self.selection),
            timing_code(self.timing),
            reset_code(self.threshold_reset),
            trace_code(self.trace),
        ] {
            hash ^= value;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        format!("{TRANSFER_HASH_PREFIX}-{hash:016x}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransferModel {
    hidden: usize,
    input_weights: Vec<f32>,
    feedback: Vec<f32>,
    readout: Vec<f32>,
    readout_bias: Vec<f32>,
    delays: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MicroTrace {
    pub event_ticks: Vec<u64>,
    pub recipients: Vec<u32>,
    pub winners_by_tick: Vec<Vec<u32>>,
    pub prediction: u32,
    pub eligibility: Vec<f32>,
    pub weight_delta: Vec<f32>,
    pub final_thresholds: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransferEval {
    pub accuracy: f32,
    pub predictions: Vec<u32>,
    pub n_distinct_predicted: usize,
    pub majority_pred_frac: f32,
    pub no_test_update: bool,
}

struct ForwardProducts {
    rates: Vec<f32>,
    logits: Vec<f32>,
    eligibility: Vec<f32>,
    event_ticks: Vec<u64>,
    recipients: Vec<u32>,
    winners_by_tick: Vec<Vec<u32>>,
    final_thresholds: Vec<f32>,
}

impl TransferModel {
    pub fn from_bundle(bundle: &TransferBundle) -> Self {
        Self {
            hidden: bundle.hidden,
            input_weights: bundle.input_weights.clone(),
            feedback: bundle.feedback.clone(),
            readout: bundle.readout.clone(),
            readout_bias: bundle.readout_bias.clone(),
            delays: bundle.delays.clone(),
        }
    }

    pub fn train(
        &mut self,
        examples: &[TemporalOrderExample],
        pole: TransferPole,
        epochs: usize,
        lr: f32,
    ) {
        assert!(!examples.is_empty() && lr > 0.0);
        for _ in 0..epochs {
            for example in examples {
                self.step(example, pole, lr);
            }
        }
    }

    pub fn evaluate(&self, examples: &[TemporalOrderExample], pole: TransferPole) -> TransferEval {
        assert!(!examples.is_empty());
        let before = self.parameter_fingerprint();
        let mut predictions = Vec::with_capacity(examples.len());
        let mut counts = [0usize; TEMPORAL_ORDER_N_CLASSES];
        let mut correct = 0usize;
        for example in examples {
            let forward = self.forward(example, pole);
            let prediction = argmax(&forward.logits);
            predictions.push(prediction as u32);
            counts[prediction] += 1;
            correct += usize::from(prediction == example.label as usize);
        }
        let after = self.parameter_fingerprint();
        TransferEval {
            accuracy: correct as f32 / examples.len() as f32,
            predictions,
            n_distinct_predicted: counts.iter().filter(|&&count| count > 0).count(),
            majority_pred_frac: counts.iter().copied().max().unwrap_or(0) as f32
                / examples.len() as f32,
            no_test_update: before == after,
        }
    }

    pub fn micro_step(
        &mut self,
        example: &TemporalOrderExample,
        pole: TransferPole,
        lr: f32,
    ) -> MicroTrace {
        let before = self.input_weights.clone();
        let forward = self.forward(example, pole);
        let prediction = argmax(&forward.logits) as u32;
        self.apply_update(example.label as usize, &forward, lr);
        let weight_delta = self
            .input_weights
            .iter()
            .zip(before)
            .map(|(after, before)| after - before)
            .collect();
        MicroTrace {
            event_ticks: forward.event_ticks,
            recipients: forward.recipients,
            winners_by_tick: forward.winners_by_tick,
            prediction,
            eligibility: forward.eligibility,
            weight_delta,
            final_thresholds: forward.final_thresholds,
        }
    }

    pub fn parameter_fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for &value in self
            .input_weights
            .iter()
            .chain(&self.readout)
            .chain(&self.readout_bias)
        {
            hash ^= value.to_bits() as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        hash
    }

    fn step(&mut self, example: &TemporalOrderExample, pole: TransferPole, lr: f32) {
        let forward = self.forward(example, pole);
        self.apply_update(example.label as usize, &forward, lr);
    }

    fn apply_update(&mut self, label: usize, forward: &ForwardProducts, lr: f32) {
        let mut delta = softmax(&forward.logits);
        delta[label] -= 1.0;
        let readout_snapshot = self.readout.clone();
        for class in 0..TEMPORAL_ORDER_N_CLASSES {
            for unit in 0..self.hidden {
                self.readout[class * self.hidden + unit] -= lr * delta[class] * forward.rates[unit];
            }
            self.readout_bias[class] -= lr * delta[class];
        }
        for unit in 0..self.hidden {
            let mut modulator = 0.0f32;
            for class in 0..TEMPORAL_ORDER_N_CLASSES {
                // Frozen feedback is the treatment signal. The readout snapshot
                // is intentionally read to keep the one-step trace capable of
                // checking the counterfactual exact transport without using it.
                let _counterfactual = readout_snapshot[class * self.hidden + unit];
                modulator +=
                    self.feedback[unit * TEMPORAL_ORDER_N_CLASSES + class] * (-delta[class]);
            }
            for input in 0..TEMPORAL_ORDER_N_IN {
                let edge = unit * TEMPORAL_ORDER_N_IN + input;
                self.input_weights[edge] += lr * modulator * forward.eligibility[edge];
            }
        }
    }

    fn forward(&self, example: &TemporalOrderExample, pole: TransferPole) -> ForwardProducts {
        assert_eq!(example.frames.len(), TEMPORAL_ORDER_N_IN * TEMPORAL_ORDER_T);
        if let Selection::HardKwta { k } = pole.selection {
            assert!(k > 0 && k <= self.hidden);
        }
        let max_delay = match pole.timing {
            Timing::SynchronousFrames => 0,
            Timing::EventQueueDelays => self.delays.iter().copied().max().unwrap_or(0) as usize,
        };
        let total_ticks = TEMPORAL_ORDER_T + max_delay;
        let mut wheel = TimingWheel::new();
        if matches!(pole.timing, Timing::EventQueueDelays) {
            for t in 0..TEMPORAL_ORDER_T {
                for input in 0..TEMPORAL_ORDER_N_IN {
                    let count = example.frames[t * TEMPORAL_ORDER_N_IN + input];
                    if count == 0.0 {
                        continue;
                    }
                    let repetitions = count.round().max(0.0) as usize;
                    assert!(
                        (count - repetitions as f32).abs() < 1e-6,
                        "transfer events require integer spike counts"
                    );
                    for unit in 0..self.hidden {
                        let edge = unit * TEMPORAL_ORDER_N_IN + input;
                        let at = t as u64 + self.delays[edge] as u64;
                        for _ in 0..repetitions {
                            wheel.insert(
                                at,
                                Event::with_amount(edge as u64, self.input_weights[edge]),
                            );
                        }
                    }
                }
            }
        }

        let mut membrane = vec![0.0f32; self.hidden];
        let mut thresholds = vec![1.0f32; self.hidden];
        let mut rates = vec![0.0f32; self.hidden];
        let mut eligibility = vec![0.0f32; self.hidden * TEMPORAL_ORDER_N_IN];
        let mut event_ticks = Vec::new();
        let mut recipients = Vec::new();
        let mut winners_by_tick = Vec::with_capacity(total_ticks);
        let alpha = (-1.0f32 / 10.0).exp();
        let threshold_decay = (-1.0f32 / 20.0).exp();

        for tick in 0..total_ticks {
            if let Some(tau_e) = pole.trace.tau_e() {
                let decay = (-1.0 / tau_e).exp();
                for value in &mut eligibility {
                    *value *= decay;
                }
            }
            let mut current = vec![0.0f32; self.hidden];
            let mut pre = vec![0.0f32; self.hidden * TEMPORAL_ORDER_N_IN];
            match pole.timing {
                Timing::SynchronousFrames if tick < TEMPORAL_ORDER_T => {
                    for unit in 0..self.hidden {
                        for input in 0..TEMPORAL_ORDER_N_IN {
                            let count = example.frames[tick * TEMPORAL_ORDER_N_IN + input];
                            if count != 0.0 {
                                let edge = unit * TEMPORAL_ORDER_N_IN + input;
                                current[unit] += self.input_weights[edge] * count;
                                pre[edge] += count;
                            }
                        }
                    }
                }
                Timing::EventQueueDelays => {
                    while wheel.peek_earliest_tick() == Some(tick as u64) {
                        let (at, event) = wheel.pop_earliest().expect("peeked event exists");
                        let edge = event.id as usize;
                        let unit = edge / TEMPORAL_ORDER_N_IN;
                        current[unit] += event.amount();
                        pre[edge] += 1.0;
                        event_ticks.push(at);
                        recipients.push(unit as u32);
                    }
                }
                _ => {}
            }

            for unit in 0..self.hidden {
                if matches!(pole.threshold_reset, ThresholdReset::AdaptiveHardReset) {
                    thresholds[unit] = 1.0 + (thresholds[unit] - 1.0) * threshold_decay;
                }
                membrane[unit] = alpha * membrane[unit] + current[unit];
            }

            let spikes = select_spikes(&membrane, &thresholds, pole.selection);
            let winners: Vec<u32> = spikes
                .iter()
                .enumerate()
                .filter_map(|(index, &spike)| (spike > 0.5).then_some(index as u32))
                .collect();
            for unit in 0..self.hidden {
                rates[unit] += spikes[unit] / total_ticks as f32;
                match pole.threshold_reset {
                    ThresholdReset::FixedSoftReset => {
                        membrane[unit] -= thresholds[unit] * spikes[unit];
                    }
                    ThresholdReset::AdaptiveHardReset if spikes[unit] > 0.5 => {
                        membrane[unit] = 0.0;
                        thresholds[unit] += 0.2;
                    }
                    ThresholdReset::AdaptiveHardReset => {}
                }
                // Local surrogate factor is applied at the postsynaptic unit.
                let derivative = 5.0
                    * sigmoid(5.0 * (membrane[unit] - thresholds[unit]))
                    * (1.0 - sigmoid(5.0 * (membrane[unit] - thresholds[unit])));
                for input in 0..TEMPORAL_ORDER_N_IN {
                    let edge = unit * TEMPORAL_ORDER_N_IN + input;
                    eligibility[edge] += pre[edge] * derivative;
                }
            }
            winners_by_tick.push(winners);
        }

        let mut logits = self.readout_bias.clone();
        for class in 0..TEMPORAL_ORDER_N_CLASSES {
            for unit in 0..self.hidden {
                logits[class] += self.readout[class * self.hidden + unit] * rates[unit];
            }
        }
        ForwardProducts {
            rates,
            logits,
            eligibility,
            event_ticks,
            recipients,
            winners_by_tick,
            final_thresholds: thresholds,
        }
    }
}

fn select_spikes(membrane: &[f32], thresholds: &[f32], selection: Selection) -> Vec<f32> {
    match selection {
        Selection::AllUnits => membrane
            .iter()
            .zip(thresholds)
            .map(|(&value, &threshold)| sigmoid(5.0 * (value - threshold)))
            .collect(),
        Selection::HardKwta { k } => {
            let mut order: Vec<usize> = (0..membrane.len()).collect();
            order.sort_by(|&left, &right| {
                membrane[right]
                    .total_cmp(&membrane[left])
                    .then_with(|| left.cmp(&right))
            });
            let mut spikes = vec![0.0; membrane.len()];
            for &unit in order.iter().take(k) {
                if membrane[unit] >= thresholds[unit] {
                    spikes[unit] = 1.0;
                }
            }
            spikes
        }
    }
}

fn sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values: Vec<f32> = logits.iter().map(|&value| (value - max).exp()).collect();
    let sum = values.iter().sum::<f32>().max(1e-12);
    for value in &mut values {
        *value /= sum;
    }
    values
}

fn argmax(values: &[f32]) -> usize {
    let mut best_index = 0usize;
    let mut best_value = f32::NEG_INFINITY;
    for (index, &value) in values.iter().enumerate() {
        if value > best_value {
            best_value = value;
            best_index = index;
        }
    }
    best_index
}

const fn selection_code(value: Selection) -> u64 {
    match value {
        Selection::AllUnits => 1,
        Selection::HardKwta { k } => 0x100 + k as u64,
    }
}

const fn timing_code(value: Timing) -> u64 {
    match value {
        Timing::SynchronousFrames => 1,
        Timing::EventQueueDelays => 2,
    }
}

const fn reset_code(value: ThresholdReset) -> u64 {
    match value {
        ThresholdReset::FixedSoftReset => 1,
        ThresholdReset::AdaptiveHardReset => 2,
    }
}

const fn trace_code(value: TraceMode) -> u64 {
    match value {
        TraceMode::HeldUntilReward => 1,
        TraceMode::RealtimeDecay { tau_e_bits } => 0x2000_0000_0000_0000 | tau_e_bits as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_data::{TemporalDifficulty, TEMPORAL_DIFFICULTIES};

    fn bundle(hidden: usize) -> TransferBundle {
        TransferBundle::generate(8, 4, TEMPORAL_DIFFICULTIES[0], hidden, 7).unwrap()
    }

    #[test]
    fn endpoint_hashes_are_distinct_and_replay_stable() {
        let bundle = bundle(16);
        let matched = TransferPole::matched().hash_string(bundle.fingerprint(), 20);
        let live = TransferPole::live().hash_string(bundle.fingerprint(), 20);
        assert_ne!(matched, live);
        assert_eq!(
            matched,
            TransferPole::matched().hash_string(bundle.fingerprint(), 20)
        );
    }

    #[test]
    fn delayed_events_report_delivery_ticks_and_recipients() {
        let bundle = bundle(8);
        let example = &bundle.train[0];
        let mut model = TransferModel::from_bundle(&bundle);
        let trace = model.micro_step(example, TransferPole::live(), 0.001);
        assert!(!trace.event_ticks.is_empty());
        assert!(trace
            .event_ticks
            .windows(2)
            .all(|window| window[0] <= window[1]));
        assert!(trace.recipients.iter().all(|&unit| unit < 8));
    }

    #[test]
    fn kwta_ties_choose_lowest_indices() {
        let membrane = vec![2.0; 12];
        let thresholds = vec![1.0; 12];
        let spikes = select_spikes(&membrane, &thresholds, Selection::HardKwta { k: 8 });
        assert_eq!(&spikes[..8], &[1.0; 8]);
        assert_eq!(&spikes[8..], &[0.0; 4]);
    }

    #[test]
    fn readout_ties_choose_lowest_class() {
        assert_eq!(argmax(&[0.0, 0.0, 0.0, 0.0]), 0);
    }

    #[test]
    fn adaptive_hard_reset_raises_thresholds() {
        let mut bundle = bundle(8);
        bundle.input_weights.fill(2.0);
        let mut model = TransferModel::from_bundle(&bundle);
        let trace = model.micro_step(&bundle.train[0], TransferPole::live(), 0.001);
        assert!(trace.final_thresholds.iter().any(|&theta| theta > 1.0));
    }

    #[test]
    fn realtime_trace_decays_relative_to_held_trace() {
        let bundle = bundle(8);
        let example = &bundle.train[0];
        let mut held_model = TransferModel::from_bundle(&bundle);
        let held = held_model.micro_step(example, TransferPole::matched(), 0.001);
        let mut decay_pole = TransferPole::matched();
        decay_pole.trace = TraceMode::realtime(TRACE_TAU_E);
        let mut decay_model = TransferModel::from_bundle(&bundle);
        let decayed = decay_model.micro_step(example, decay_pole, 0.001);
        let held_norm = held.eligibility.iter().map(|v| v.abs()).sum::<f32>();
        let decay_norm = decayed.eligibility.iter().map(|v| v.abs()).sum::<f32>();
        assert!(decay_norm < held_norm);
    }

    #[test]
    fn micro_step_and_evaluation_replay() {
        let bundle = TransferBundle::generate(8, 4, TemporalDifficulty::new(1, 8), 8, 99).unwrap();
        let mut a = TransferModel::from_bundle(&bundle);
        let mut b = TransferModel::from_bundle(&bundle);
        assert_eq!(
            a.micro_step(&bundle.train[0], TransferPole::matched(), 0.001),
            b.micro_step(&bundle.train[0], TransferPole::matched(), 0.001)
        );
        let before = a.parameter_fingerprint();
        let eval = a.evaluate(&bundle.test, TransferPole::matched());
        assert!(eval.no_test_update);
        assert_eq!(before, a.parameter_fingerprint());
    }
}
