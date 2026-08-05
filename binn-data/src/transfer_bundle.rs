//! `BINNTRF1` cross-language transfer bundle.
//!
//! The format is intentionally stdlib-only: fixed little-endian scalars and
//! implicit array lengths derived from the header. Rust and NumPy consume the
//! exact same examples, connectivity, delays, weights, readout, and feedback.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use binn_core::Rng;

use crate::temporal_order::{
    TemporalDifficulty, TemporalOrderExample, TemporalOrderSplit, TEMPORAL_ORDER_N_CLASSES,
    TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};

pub const BINNTRF1_MAGIC: &[u8; 8] = b"BINNTRF1";
pub const BINNTRF1_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct TransferBundle {
    pub seed: u64,
    pub hidden: usize,
    pub train: Vec<TemporalOrderExample>,
    pub test: Vec<TemporalOrderExample>,
    /// Input-to-hidden delays, `[hidden × N_IN]`, in ticks.
    pub delays: Vec<u32>,
    /// Input-to-hidden weights, `[hidden × N_IN]`.
    pub input_weights: Vec<f32>,
    /// Class-to-hidden random feedback, `[hidden × N_CLASSES]`.
    pub feedback: Vec<f32>,
    /// Hidden-to-class readout, `[N_CLASSES × hidden]`.
    pub readout: Vec<f32>,
    pub readout_bias: Vec<f32>,
}

impl TransferBundle {
    pub fn generate(
        n_train: usize,
        n_test: usize,
        difficulty: TemporalDifficulty,
        hidden: usize,
        seed: u64,
    ) -> Result<Self, String> {
        if hidden == 0 {
            return Err("transfer bundle hidden width must be positive".into());
        }
        let split = TemporalOrderSplit::generate(n_train, n_test, difficulty, seed)?;
        let mut rng = Rng::new(seed ^ 0xB1A0_7AF1_0000_0001);
        let in_scale = (2.0 / (TEMPORAL_ORDER_N_IN + hidden) as f32).sqrt();
        let out_scale = (2.0 / (hidden + TEMPORAL_ORDER_N_CLASSES) as f32).sqrt();
        let input_weights = (0..hidden * TEMPORAL_ORDER_N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let delays = (0..hidden * TEMPORAL_ORDER_N_IN)
            .map(|_| (1 + rng.gen_index(4)) as u32)
            .collect();
        let feedback = (0..hidden * TEMPORAL_ORDER_N_CLASSES)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();
        let readout = (0..TEMPORAL_ORDER_N_CLASSES * hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();
        Ok(Self {
            seed,
            hidden,
            train: split.train,
            test: split.test,
            delays,
            input_weights,
            feedback,
            readout,
            readout_bias: vec![0.0; TEMPORAL_ORDER_N_CLASSES],
        })
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let mut writer = BufWriter::new(File::create(path).map_err(|error| error.to_string())?);
        writer
            .write_all(BINNTRF1_MAGIC)
            .map_err(|error| error.to_string())?;
        write_u32(&mut writer, BINNTRF1_VERSION)?;
        write_u64(&mut writer, self.seed)?;
        for value in [
            TEMPORAL_ORDER_N_IN,
            TEMPORAL_ORDER_T,
            TEMPORAL_ORDER_N_CLASSES,
            self.hidden,
            self.train.len(),
            self.test.len(),
        ] {
            write_u32(&mut writer, value as u32)?;
        }
        write_examples(&mut writer, &self.train)?;
        write_examples(&mut writer, &self.test)?;
        write_u32_slice(&mut writer, &self.delays)?;
        write_f32_slice(&mut writer, &self.input_weights)?;
        write_f32_slice(&mut writer, &self.feedback)?;
        write_f32_slice(&mut writer, &self.readout)?;
        write_f32_slice(&mut writer, &self.readout_bias)?;
        writer.flush().map_err(|error| error.to_string())
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let mut reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|error| error.to_string())?;
        if &magic != BINNTRF1_MAGIC {
            return Err("invalid BINNTRF1 magic".into());
        }
        let version = read_u32(&mut reader)?;
        if version != BINNTRF1_VERSION {
            return Err(format!("unsupported BINNTRF1 version {version}"));
        }
        let seed = read_u64(&mut reader)?;
        let n_in = read_u32(&mut reader)? as usize;
        let timesteps = read_u32(&mut reader)? as usize;
        let n_classes = read_u32(&mut reader)? as usize;
        let hidden = read_u32(&mut reader)? as usize;
        let n_train = read_u32(&mut reader)? as usize;
        let n_test = read_u32(&mut reader)? as usize;
        if (n_in, timesteps, n_classes)
            != (
                TEMPORAL_ORDER_N_IN,
                TEMPORAL_ORDER_T,
                TEMPORAL_ORDER_N_CLASSES,
            )
        {
            return Err("BINNTRF1 task dimensions do not match protocol".into());
        }
        let train = read_examples(&mut reader, n_train)?;
        let test = read_examples(&mut reader, n_test)?;
        let delays = read_u32_vec(&mut reader, hidden * n_in)?;
        let input_weights = read_f32_vec(&mut reader, hidden * n_in)?;
        let feedback = read_f32_vec(&mut reader, hidden * n_classes)?;
        let readout = read_f32_vec(&mut reader, n_classes * hidden)?;
        let readout_bias = read_f32_vec(&mut reader, n_classes)?;
        let mut trailing = [0u8; 1];
        if reader
            .read(&mut trailing)
            .map_err(|error| error.to_string())?
            != 0
        {
            return Err("BINNTRF1 bundle has trailing bytes".into());
        }
        let bundle = Self {
            seed,
            hidden,
            train,
            test,
            delays,
            input_weights,
            feedback,
            readout,
            readout_bias,
        };
        bundle.validate()?;
        Ok(bundle)
    }

    pub fn fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        mix(&mut hash, self.seed);
        mix(&mut hash, self.hidden as u64);
        for example in self.train.iter().chain(&self.test) {
            mix(&mut hash, example.label as u64);
            for &value in &example.frames {
                mix(&mut hash, value.to_bits() as u64);
            }
        }
        for &value in &self.delays {
            mix(&mut hash, value as u64);
        }
        for &value in self
            .input_weights
            .iter()
            .chain(&self.feedback)
            .chain(&self.readout)
            .chain(&self.readout_bias)
        {
            mix(&mut hash, value.to_bits() as u64);
        }
        hash
    }

    fn validate(&self) -> Result<(), String> {
        if self.hidden == 0
            || self.delays.len() != self.hidden * TEMPORAL_ORDER_N_IN
            || self.input_weights.len() != self.hidden * TEMPORAL_ORDER_N_IN
            || self.feedback.len() != self.hidden * TEMPORAL_ORDER_N_CLASSES
            || self.readout.len() != self.hidden * TEMPORAL_ORDER_N_CLASSES
            || self.readout_bias.len() != TEMPORAL_ORDER_N_CLASSES
        {
            return Err("BINNTRF1 array dimensions are inconsistent".into());
        }
        if self.train.is_empty() || self.test.is_empty() {
            return Err("BINNTRF1 splits must be non-empty".into());
        }
        for example in self.train.iter().chain(&self.test) {
            if example.frames.len() != TEMPORAL_ORDER_N_IN * TEMPORAL_ORDER_T
                || example.label as usize >= TEMPORAL_ORDER_N_CLASSES
            {
                return Err("BINNTRF1 example is malformed".into());
            }
        }
        if self
            .input_weights
            .iter()
            .chain(&self.feedback)
            .chain(&self.readout)
            .chain(&self.readout_bias)
            .any(|value| !value.is_finite())
        {
            return Err("BINNTRF1 contains non-finite weights".into());
        }
        Ok(())
    }
}

fn write_examples(
    writer: &mut impl Write,
    examples: &[TemporalOrderExample],
) -> Result<(), String> {
    for example in examples {
        write_u32(writer, example.label)?;
        write_f32_slice(writer, &example.frames)?;
    }
    Ok(())
}

fn read_examples(
    reader: &mut impl Read,
    count: usize,
) -> Result<Vec<TemporalOrderExample>, String> {
    let mut examples = Vec::with_capacity(count);
    for _ in 0..count {
        examples.push(TemporalOrderExample {
            label: read_u32(reader)?,
            frames: read_f32_vec(reader, TEMPORAL_ORDER_N_IN * TEMPORAL_ORDER_T)?,
        });
    }
    Ok(examples)
}

fn write_u32(writer: &mut impl Write, value: u32) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn write_u64(writer: &mut impl Write, value: u64) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn write_u32_slice(writer: &mut impl Write, values: &[u32]) -> Result<(), String> {
    for &value in values {
        write_u32(writer, value)?;
    }
    Ok(())
}

fn write_f32_slice(writer: &mut impl Write, values: &[f32]) -> Result<(), String> {
    for &value in values {
        writer
            .write_all(&value.to_le_bytes())
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn read_u32(reader: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> Result<u64, String> {
    let mut bytes = [0u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_u32_vec(reader: &mut impl Read, count: usize) -> Result<Vec<u32>, String> {
    (0..count).map(|_| read_u32(reader)).collect()
}

fn read_f32_vec(reader: &mut impl Read, count: usize) -> Result<Vec<f32>, String> {
    let mut values = Vec::with_capacity(count);
    let mut bytes = [0u8; 4];
    for _ in 0..count {
        reader
            .read_exact(&mut bytes)
            .map_err(|error| error.to_string())?;
        values.push(f32::from_le_bytes(bytes));
    }
    Ok(values)
}

fn mix(hash: &mut u64, value: u64) {
    *hash ^= value;
    *hash = hash.wrapping_mul(0x100_0000_01b3);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal_order::TEMPORAL_DIFFICULTIES;

    #[test]
    fn bundle_round_trip_is_byte_stable() {
        let bundle = TransferBundle::generate(20, 8, TEMPORAL_DIFFICULTIES[1], 16, 91).unwrap();
        let path = std::env::temp_dir().join(format!(
            "binn-transfer-bundle-{}-{}.bin",
            std::process::id(),
            bundle.fingerprint()
        ));
        bundle.write(&path).unwrap();
        let read = TransferBundle::read(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(read, bundle);
        assert_eq!(read.fingerprint(), bundle.fingerprint());
    }
}
