//! Matched one-hidden-layer SHD BPTT reference.
//!
//! This module is deliberately independent of the legacy SHD ceiling. It uses
//! physical-time leak scaling, detached reset, a masked mean-rate readout, and
//! exposes exact gradients so a second backend can be checked before accuracy
//! is interpreted.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub const MATCHED_WEIGHTS_MAGIC: &[u8; 8] = b"SHDWGT1\0";
pub const MATCHED_ORDER_MAGIC: &[u8; 8] = b"SHDORD1\0";
pub const MATCHED_THRESHOLD: f32 = 1.0;
pub const MATCHED_SURROGATE_ALPHA: f32 = 5.0;
pub const MATCHED_PHYSICAL_TAU_MS: f32 = 10.05;
pub const MATCHED_ADAM_BETA1: f32 = 0.9;
pub const MATCHED_ADAM_BETA2: f32 = 0.999;
pub const MATCHED_ADAM_EPS: f32 = 1e-8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchedTrainSpec {
    pub hidden: usize,
    pub n_classes: usize,
    pub batch_size: usize,
    pub epochs: usize,
    pub base_lr: f32,
    pub max_lr: f32,
    pub weight_decay: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedShdSample {
    pub label: u32,
    pub frames: Vec<Vec<(usize, f32)>>,
    pub n_inputs: usize,
    pub dt_ms: f32,
}

impl MatchedTrainSpec {
    pub fn calibration(hidden: usize, epochs: usize) -> Self {
        Self {
            hidden,
            n_classes: 20,
            batch_size: 256,
            epochs,
            base_lr: 1e-3,
            max_lr: 5e-3,
            weight_decay: 1e-5,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedWeights {
    pub n_inputs: usize,
    pub hidden: usize,
    pub n_classes: usize,
    /// `[hidden, n_inputs]`.
    pub w_in: Vec<f32>,
    /// `[n_classes, hidden]`.
    pub w_out: Vec<f32>,
    pub b_out: Vec<f32>,
}

impl MatchedWeights {
    pub fn deterministic(n_inputs: usize, hidden: usize, n_classes: usize, seed: u64) -> Self {
        let mut rng = PortableRng::new(seed);
        let in_limit = (6.0_f32 / (n_inputs + hidden) as f32).sqrt();
        let out_limit = (6.0_f32 / (hidden + n_classes) as f32).sqrt();
        let w_in = (0..hidden * n_inputs)
            .map(|_| rng.uniform(-in_limit, in_limit))
            .collect();
        let w_out = (0..n_classes * hidden)
            .map(|_| rng.uniform(-out_limit, out_limit))
            .collect();
        Self {
            n_inputs,
            hidden,
            n_classes,
            w_in,
            w_out,
            b_out: vec![0.0; n_classes],
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let file = File::create(path).map_err(|error| error.to_string())?;
        let mut writer = BufWriter::new(file);
        writer
            .write_all(MATCHED_WEIGHTS_MAGIC)
            .map_err(|error| error.to_string())?;
        for value in [self.n_inputs, self.hidden, self.n_classes] {
            write_u32(&mut writer, value as u32)?;
        }
        for &value in self
            .w_in
            .iter()
            .chain(self.w_out.iter())
            .chain(self.b_out.iter())
        {
            writer
                .write_all(&value.to_bits().to_le_bytes())
                .map_err(|error| error.to_string())?;
        }
        writer.flush().map_err(|error| error.to_string())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let mut reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
        let mut magic = [0_u8; 8];
        reader.read_exact(&mut magic).map_err(|error| error.to_string())?;
        if &magic != MATCHED_WEIGHTS_MAGIC {
            return Err(format!("bad matched-weight magic in {}", path.display()));
        }
        let n_inputs = read_u32(&mut reader)? as usize;
        let hidden = read_u32(&mut reader)? as usize;
        let n_classes = read_u32(&mut reader)? as usize;
        let w_in = read_f32_vec(&mut reader, n_inputs * hidden)?;
        let w_out = read_f32_vec(&mut reader, hidden * n_classes)?;
        let b_out = read_f32_vec(&mut reader, n_classes)?;
        Ok(Self {
            n_inputs,
            hidden,
            n_classes,
            w_in,
            w_out,
            b_out,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedGradient {
    pub w_in: Vec<f32>,
    pub w_out: Vec<f32>,
    pub b_out: Vec<f32>,
}

impl MatchedGradient {
    pub fn zeros_like(weights: &MatchedWeights) -> Self {
        Self {
            w_in: vec![0.0; weights.w_in.len()],
            w_out: vec![0.0; weights.w_out.len()],
            b_out: vec![0.0; weights.b_out.len()],
        }
    }

    pub fn add_assign(&mut self, other: &Self) {
        add_vec(&mut self.w_in, &other.w_in);
        add_vec(&mut self.w_out, &other.w_out);
        add_vec(&mut self.b_out, &other.b_out);
    }

    pub fn scale(&mut self, factor: f32) {
        for value in self
            .w_in
            .iter_mut()
            .chain(self.w_out.iter_mut())
            .chain(self.b_out.iter_mut())
        {
            *value *= factor;
        }
    }

    /// L2 norm, with an f64 fallback used **only** when the f32 accumulation
    /// overflows.
    ///
    /// # Why the fallback is conditional
    ///
    /// The f32 sum of squares overflows well before the norm itself does: a
    /// gradient whose true norm is `1e20` — perfectly representable in f32,
    /// whose max is ~3.4e38 — has a sum of squares of `1e40`, which is not. So
    /// the naive version returned `inf` for norms that f32 can hold, and the
    /// per-entry `all_finite` check never caught it because every individual
    /// entry was finite. `rec+alif` at h512 hit this on 420 of 640 optimizer
    /// steps; see `MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md` §3.6.
    ///
    /// Accumulating in f64 unconditionally would be the tidier fix and is
    /// **deliberately not what this does**. Different summation order and
    /// precision would move `mean_gradient_norm` in the last ulp for the 216
    /// recorded `ff+fixed` cells, breaking Gate F — a change to a registered
    /// result, not a bug fix. Gating the fallback on `!is_finite()` keeps every
    /// value that f32 could already represent **bit-identical**, and only
    /// replaces the ones that were `inf`, which were wrong anyway.
    ///
    /// A norm genuinely above `f32::MAX` still returns infinity, correctly.
    pub fn l2_norm(&self) -> f32 {
        let sum = self
            .w_in
            .iter()
            .chain(self.w_out.iter())
            .chain(self.b_out.iter())
            .map(|value| value * value)
            .sum::<f32>();
        if sum.is_finite() {
            return sum.sqrt();
        }
        let wide: f64 = self
            .w_in
            .iter()
            .chain(self.w_out.iter())
            .chain(self.b_out.iter())
            .map(|value| {
                let value = f64::from(*value);
                value * value
            })
            .sum();
        wide.sqrt() as f32
    }

    pub fn all_finite(&self) -> bool {
        self.w_in
            .iter()
            .chain(self.w_out.iter())
            .chain(self.b_out.iter())
            .all(|value| value.is_finite())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedForward {
    pub membrane: Vec<f32>,
    pub spikes: Vec<f32>,
    pub rates: Vec<f32>,
    pub logits: Vec<f32>,
    pub loss: f32,
    pub prediction: usize,
}

pub fn surrogate_derivative(u_minus_threshold: f32) -> f32 {
    let scaled = std::f32::consts::FRAC_PI_2 * MATCHED_SURROGATE_ALPHA * u_minus_threshold;
    (MATCHED_SURROGATE_ALPHA * 0.5) / (1.0 + scaled * scaled)
}

pub fn loss_and_gradient(
    weights: &MatchedWeights,
    sample: &MatchedShdSample,
) -> Result<(MatchedForward, MatchedGradient), String> {
    if sample.n_inputs != weights.n_inputs {
        return Err("sample/model input mismatch".into());
    }
    if sample.label as usize >= weights.n_classes || sample.frames.is_empty() {
        return Err("invalid label or empty framed sample".into());
    }
    let t_steps = sample.frames.len();
    let hidden = weights.hidden;
    let alpha = (-sample.dt_ms / MATCHED_PHYSICAL_TAU_MS).exp();
    let mut membrane = vec![0.0_f32; t_steps * hidden];
    let mut spikes = vec![0.0_f32; t_steps * hidden];
    let mut previous_u = vec![0.0_f32; hidden];
    let mut previous_s = vec![0.0_f32; hidden];

    for (t, frame) in sample.frames.iter().enumerate() {
        for h in 0..hidden {
            let mut current = alpha * previous_u[h] * (1.0 - previous_s[h]);
            let row = h * weights.n_inputs;
            for &(channel, count) in frame {
                current += weights.w_in[row + channel] * count;
            }
            let spike = f32::from(current >= MATCHED_THRESHOLD);
            membrane[t * hidden + h] = current;
            spikes[t * hidden + h] = spike;
            previous_u[h] = current;
            previous_s[h] = spike;
        }
    }

    let inv_t = 1.0 / t_steps as f32;
    let mut rates = vec![0.0_f32; hidden];
    for t in 0..t_steps {
        for h in 0..hidden {
            rates[h] += spikes[t * hidden + h] * inv_t;
        }
    }
    let mut logits = weights.b_out.clone();
    for (class, logit) in logits.iter_mut().enumerate() {
        let row = class * hidden;
        for (h, rate) in rates.iter().enumerate() {
            *logit += weights.w_out[row + h] * rate;
        }
    }
    let prediction = argmax(&logits);
    let mut probabilities = softmax(&logits);
    let loss = -probabilities[sample.label as usize].max(1e-30).ln();
    probabilities[sample.label as usize] -= 1.0;

    let mut gradient = MatchedGradient::zeros_like(weights);
    gradient.b_out.copy_from_slice(&probabilities);
    let mut direct_spike = vec![0.0_f32; hidden];
    for (class, probability) in probabilities.iter().copied().enumerate() {
        let row = class * hidden;
        for h in 0..hidden {
            gradient.w_out[row + h] = probability * rates[h];
            direct_spike[h] += weights.w_out[row + h] * probability * inv_t;
        }
    }

    let mut du_next = vec![0.0_f32; hidden];
    for t in (0..t_steps).rev() {
        let frame = &sample.frames[t];
        for h in 0..hidden {
            let index = t * hidden + h;
            let du = direct_spike[h]
                * surrogate_derivative(membrane[index] - MATCHED_THRESHOLD)
                + alpha * (1.0 - spikes[index]) * du_next[h];
            let row = h * weights.n_inputs;
            for &(channel, count) in frame {
                gradient.w_in[row + channel] += du * count;
            }
            du_next[h] = du;
        }
    }

    Ok((
        MatchedForward {
            membrane,
            spikes,
            rates,
            logits,
            loss,
            prediction,
        },
        gradient,
    ))
}

#[derive(Clone, Debug)]
pub struct MatchedAdam {
    mw_in: Vec<f32>,
    vw_in: Vec<f32>,
    mw_out: Vec<f32>,
    vw_out: Vec<f32>,
    mb_out: Vec<f32>,
    vb_out: Vec<f32>,
    step: usize,
}

impl MatchedAdam {
    pub fn new(weights: &MatchedWeights) -> Self {
        Self {
            mw_in: vec![0.0; weights.w_in.len()],
            vw_in: vec![0.0; weights.w_in.len()],
            mw_out: vec![0.0; weights.w_out.len()],
            vw_out: vec![0.0; weights.w_out.len()],
            mb_out: vec![0.0; weights.b_out.len()],
            vb_out: vec![0.0; weights.b_out.len()],
            step: 0,
        }
    }

    pub fn update(
        &mut self,
        weights: &mut MatchedWeights,
        gradient: &MatchedGradient,
        lr: f32,
        weight_decay: f32,
    ) -> f32 {
        self.step += 1;
        let correction1 = 1.0 - MATCHED_ADAM_BETA1.powi(self.step as i32);
        let correction2 = 1.0 - MATCHED_ADAM_BETA2.powi(self.step as i32);
        let mut squared_step = 0.0_f64;
        let mut n_step = 0usize;
        update_slice(
            &mut weights.w_in,
            &gradient.w_in,
            &mut self.mw_in,
            &mut self.vw_in,
            lr,
            weight_decay,
            correction1,
            correction2,
            &mut squared_step,
            &mut n_step,
        );
        update_slice(
            &mut weights.w_out,
            &gradient.w_out,
            &mut self.mw_out,
            &mut self.vw_out,
            lr,
            weight_decay,
            correction1,
            correction2,
            &mut squared_step,
            &mut n_step,
        );
        update_slice(
            &mut weights.b_out,
            &gradient.b_out,
            &mut self.mb_out,
            &mut self.vb_out,
            lr,
            0.0,
            correction1,
            correction2,
            &mut squared_step,
            &mut n_step,
        );
        (squared_step / n_step.max(1) as f64).sqrt() as f32
    }
}

#[allow(clippy::too_many_arguments)]
fn update_slice(
    weights: &mut [f32],
    gradients: &[f32],
    first: &mut [f32],
    second: &mut [f32],
    lr: f32,
    weight_decay: f32,
    correction1: f32,
    correction2: f32,
    squared_step: &mut f64,
    n_step: &mut usize,
) {
    for index in 0..weights.len() {
        let gradient = gradients[index] + weight_decay * weights[index];
        first[index] =
            MATCHED_ADAM_BETA1 * first[index] + (1.0 - MATCHED_ADAM_BETA1) * gradient;
        second[index] =
            MATCHED_ADAM_BETA2 * second[index] + (1.0 - MATCHED_ADAM_BETA2) * gradient * gradient;
        let update = lr * (first[index] / correction1)
            / ((second[index] / correction2).sqrt() + MATCHED_ADAM_EPS);
        weights[index] -= update;
        *squared_step += f64::from(update) * f64::from(update);
        *n_step += 1;
    }
}

pub fn one_cycle_lr(step: usize, total_steps: usize, base_lr: f32, max_lr: f32) -> f32 {
    if total_steps <= 1 {
        return base_lr;
    }
    let progress = step as f32 / (total_steps - 1) as f32;
    if progress <= 0.3 {
        base_lr + (max_lr - base_lr) * (progress / 0.3)
    } else {
        let final_lr = base_lr / 100.0;
        max_lr - (max_lr - final_lr) * ((progress - 0.3) / 0.7)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortableRng(u64);

impl PortableRng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.0 = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn uniform(&mut self, low: f32, high: f32) -> f32 {
        let unit = (self.next_u64() >> 40) as f32 / (1_u32 << 24) as f32;
        low + (high - low) * unit
    }

    pub fn shuffle(&mut self, values: &mut [usize]) {
        for index in (1..values.len()).rev() {
            let other = self.next_u64() as usize % (index + 1);
            values.swap(index, other);
        }
    }
}

pub fn save_epoch_orders(path: &Path, orders: &[Vec<usize>]) -> Result<(), String> {
    let n_items = orders.first().map_or(0, Vec::len);
    if orders.iter().any(|order| order.len() != n_items) {
        return Err("inconsistent epoch order lengths".into());
    }
    let mut writer = BufWriter::new(File::create(path).map_err(|error| error.to_string())?);
    writer
        .write_all(MATCHED_ORDER_MAGIC)
        .map_err(|error| error.to_string())?;
    write_u32(&mut writer, orders.len() as u32)?;
    write_u32(&mut writer, n_items as u32)?;
    for order in orders {
        for &index in order {
            write_u32(&mut writer, index as u32)?;
        }
    }
    writer.flush().map_err(|error| error.to_string())
}

pub fn load_epoch_orders(path: &Path) -> Result<Vec<Vec<usize>>, String> {
    let mut reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
    let mut magic = [0_u8; 8];
    reader.read_exact(&mut magic).map_err(|error| error.to_string())?;
    if &magic != MATCHED_ORDER_MAGIC {
        return Err(format!("bad epoch-order magic in {}", path.display()));
    }
    let n_epochs = read_u32(&mut reader)? as usize;
    let n_items = read_u32(&mut reader)? as usize;
    let mut orders = Vec::with_capacity(n_epochs);
    for _ in 0..n_epochs {
        let mut order = Vec::with_capacity(n_items);
        for _ in 0..n_items {
            order.push(read_u32(&mut reader)? as usize);
        }
        orders.push(order);
    }
    Ok(orders)
}

fn add_vec(target: &mut [f32], source: &[f32]) {
    for (target, source) in target.iter_mut().zip(source) {
        *target += *source;
    }
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values: Vec<f32> = logits.iter().map(|value| (value - max).exp()).collect();
    let sum: f32 = values.iter().sum();
    for value in &mut values {
        *value /= sum;
    }
    values
}

fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map_or(0, |(index, _)| index)
}

fn write_u32(writer: &mut impl Write, value: u32) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn read_u32(reader: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_f32_vec(reader: &mut impl Read, len: usize) -> Result<Vec<f32>, String> {
    (0..len)
        .map(|_| {
            let mut bytes = [0_u8; 4];
            reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
            Ok(f32::from_bits(u32::from_le_bytes(bytes)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> MatchedShdSample {
        MatchedShdSample {
            label: 1,
            frames: vec![vec![(0, 1.0)], vec![(1, 1.0)]],
            n_inputs: 700,
            dt_ms: 10.0,
        }
    }

    #[test]
    fn rate_readout_is_normalized_by_valid_steps() {
        let mut weights = MatchedWeights::deterministic(700, 3, 20, 7);
        weights.w_in.fill(0.0);
        weights.w_in[0] = 2.0;
        weights.w_out.fill(0.0);
        weights.w_out[0] = 1.0;
        let (forward, _) = loss_and_gradient(&weights, &sample()).unwrap();
        assert_eq!(forward.rates[0], 0.5);
    }

    #[test]
    fn gradients_and_adam_update_are_finite() {
        let mut weights = MatchedWeights::deterministic(700, 4, 20, 11);
        let (_, gradient) = loss_and_gradient(&weights, &sample()).unwrap();
        assert!(gradient.all_finite());
        let mut adam = MatchedAdam::new(&weights);
        let step = adam.update(&mut weights, &gradient, 1e-3, 1e-5);
        assert!(step.is_finite() && step > 0.0);
    }

    #[test]
    fn readout_gradient_matches_finite_difference() {
        let weights = MatchedWeights::deterministic(700, 3, 20, 13);
        let (_, gradient) = loss_and_gradient(&weights, &sample()).unwrap();
        let index = 2;
        let epsilon = 1e-3;
        let mut plus = weights.clone();
        let mut minus = weights.clone();
        plus.w_out[index] += epsilon;
        minus.w_out[index] -= epsilon;
        let plus_loss = loss_and_gradient(&plus, &sample()).unwrap().0.loss;
        let minus_loss = loss_and_gradient(&minus, &sample()).unwrap().0.loss;
        let numerical = (plus_loss - minus_loss) / (2.0 * epsilon);
        assert!((gradient.w_out[index] - numerical).abs() < 1e-4);
    }

    #[test]
    fn weight_and_order_serialization_roundtrip() {
        let root = std::env::temp_dir().join(format!("shd_matched_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let weights = MatchedWeights::deterministic(7, 3, 2, 17);
        let weights_path = root.join("init.bin");
        weights.save(&weights_path).unwrap();
        assert_eq!(weights, MatchedWeights::load(&weights_path).unwrap());
        let orders = vec![vec![2, 0, 1], vec![1, 2, 0]];
        let order_path = root.join("orders.bin");
        save_epoch_orders(&order_path, &orders).unwrap();
        assert_eq!(orders, load_epoch_orders(&order_path).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }
}
