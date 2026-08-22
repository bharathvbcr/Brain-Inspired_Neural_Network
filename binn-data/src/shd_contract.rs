//! Canonical, count-preserving SHD event contracts used for instrument calibration.
//!
//! The legacy `BINNSHD1` cache is intentionally not reused here: it stores dense
//! binary occupancy and therefore cannot recover collisions. `SHDEVT1` stores the
//! original event time/channel pairs once; every temporal/frequency contract is
//! derived deterministically at runtime by both calibration backends.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub const SHD_EVENT_MAGIC: &[u8; 8] = b"SHDEVT1\0";
pub const SHD_FIXED_WINDOW_MS: f32 = 1_400.0;
pub const SHD_PHYSICAL_TAU_MS: f32 = 10.05;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShdEventContract {
    PublishedDuration { frame_ms: u32 },
    FixedWindow { frames: usize, duration_ms: f32 },
}

impl ShdEventContract {
    pub fn published(frame_ms: u32) -> Result<Self, String> {
        if !matches!(frame_ms, 2 | 4 | 10) {
            return Err("published frame_ms must be one of 2, 4, 10".into());
        }
        Ok(Self::PublishedDuration { frame_ms })
    }

    pub fn fixed(frames: usize) -> Result<Self, String> {
        if !matches!(frames, 100 | 250 | 500) {
            return Err("fixed frames must be one of 100, 250, 500".into());
        }
        Ok(Self::FixedWindow {
            frames,
            duration_ms: SHD_FIXED_WINDOW_MS,
        })
    }

    pub fn dt_ms(self) -> f32 {
        match self {
            Self::PublishedDuration { frame_ms } => frame_ms as f32,
            Self::FixedWindow {
                frames,
                duration_ms,
            } => duration_ms / frames as f32,
        }
    }

    pub fn id(self) -> String {
        match self {
            Self::PublishedDuration { frame_ms } => format!("published-{frame_ms}ms"),
            Self::FixedWindow { frames, .. } => format!("fixed-t{frames}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrequencyGeometry {
    Channels700,
    AdjacentSum5,
}

impl FrequencyGeometry {
    pub const fn n_inputs(self) -> usize {
        match self {
            Self::Channels700 => 700,
            Self::AdjacentSum5 => 140,
        }
    }

    pub const fn map_channel(self, channel: u16) -> Option<usize> {
        let channel = channel as usize;
        if channel >= 700 {
            return None;
        }
        Some(match self {
            Self::Channels700 => channel,
            Self::AdjacentSum5 => channel / 5,
        })
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Channels700 => "channels-700",
            Self::AdjacentSum5 => "adjacent-sum-5",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShdEvent {
    pub time_s: f32,
    pub channel: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShdEventSample {
    pub label: u32,
    pub events: Vec<ShdEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SparseFrame {
    pub values: Vec<(usize, f32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FramedShdSample {
    pub label: u32,
    pub frames: Vec<SparseFrame>,
    pub n_inputs: usize,
    pub dt_ms: f32,
    pub original_events: usize,
    pub retained_events: usize,
    pub clipped_events: usize,
    pub first_time_s: f32,
    pub last_time_s: f32,
}

impl FramedShdSample {
    pub fn valid_steps(&self) -> usize {
        self.frames.len()
    }

    pub fn retained_fraction(&self) -> f64 {
        if self.original_events == 0 {
            1.0
        } else {
            self.retained_events as f64 / self.original_events as f64
        }
    }

    /// Stable FNV-1a digest over dimensions, label, and exact sparse f32 values.
    pub fn fingerprint(&self) -> u64 {
        fn mix(hash: &mut u64, bytes: &[u8]) {
            for byte in bytes {
                *hash ^= *byte as u64;
                *hash = hash.wrapping_mul(0x100_0000_01b3);
            }
        }
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        mix(&mut hash, &self.label.to_le_bytes());
        mix(&mut hash, &(self.frames.len() as u64).to_le_bytes());
        mix(&mut hash, &(self.n_inputs as u64).to_le_bytes());
        mix(&mut hash, &self.dt_ms.to_bits().to_le_bytes());
        for (t, frame) in self.frames.iter().enumerate() {
            mix(&mut hash, &(t as u64).to_le_bytes());
            for &(channel, value) in &frame.values {
                mix(&mut hash, &(channel as u64).to_le_bytes());
                mix(&mut hash, &value.to_bits().to_le_bytes());
            }
        }
        hash
    }
}

pub fn frame_events(
    sample: &ShdEventSample,
    contract: ShdEventContract,
    geometry: FrequencyGeometry,
) -> FramedShdSample {
    let first_time_s = sample.events.first().map_or(0.0, |event| event.time_s);
    let last_time_s = sample.events.last().map_or(0.0, |event| event.time_s);
    let (n_steps, origin_s, duration_s) = match contract {
        ShdEventContract::PublishedDuration { frame_ms } => {
            let duration_s = frame_ms as f32 / 1_000.0;
            let shifted_last = (last_time_s - first_time_s).max(0.0);
            let n_steps = if sample.events.is_empty() {
                1
            } else {
                (shifted_last / duration_s).floor() as usize + 1
            };
            (n_steps.max(1), first_time_s, duration_s)
        }
        ShdEventContract::FixedWindow {
            frames,
            duration_ms,
        } => (frames, 0.0, duration_ms / 1_000.0 / frames as f32),
    };

    let mut sparse: Vec<BTreeMap<usize, f32>> = (0..n_steps).map(|_| BTreeMap::new()).collect();
    let mut retained_events = 0usize;
    let mut clipped_events = 0usize;
    for event in &sample.events {
        let Some(channel) = geometry.map_channel(event.channel) else {
            clipped_events += 1;
            continue;
        };
        let shifted = event.time_s - origin_s;
        if !shifted.is_finite() || shifted < 0.0 {
            clipped_events += 1;
            continue;
        }
        let frame = (shifted / duration_s).floor() as usize;
        if frame >= n_steps {
            clipped_events += 1;
            continue;
        }
        *sparse[frame].entry(channel).or_insert(0.0) += 1.0;
        retained_events += 1;
    }
    let frames = sparse
        .into_iter()
        .map(|values| SparseFrame {
            values: values.into_iter().collect(),
        })
        .collect();
    FramedShdSample {
        label: sample.label,
        frames,
        n_inputs: geometry.n_inputs(),
        dt_ms: contract.dt_ms(),
        original_events: sample.events.len(),
        retained_events,
        clipped_events,
        first_time_s,
        last_time_s,
    }
}

/// Read the count-preserving event cache produced by
/// `scripts/shd_calibration/data.py`.
pub fn read_event_cache(
    path: &Path,
    max_samples: Option<usize>,
) -> Result<Vec<ShdEventSample>, String> {
    let mut reader = BufReader::new(
        File::open(path)
            .map_err(|error| format!("open SHD event cache {}: {error}", path.display()))?,
    );
    let mut magic = [0_u8; 8];
    reader
        .read_exact(&mut magic)
        .map_err(|error| error.to_string())?;
    if &magic != SHD_EVENT_MAGIC {
        return Err(format!("bad SHD event magic in {}", path.display()));
    }
    let n_file = read_u32(&mut reader)? as usize;
    let n = max_samples.unwrap_or(n_file).min(n_file);
    let mut samples = Vec::with_capacity(n);
    for index in 0..n_file {
        let label = read_u32(&mut reader)?;
        let n_events = read_u32(&mut reader)? as usize;
        if index < n {
            let mut events = Vec::with_capacity(n_events);
            for _ in 0..n_events {
                let time_s = read_f32(&mut reader)?;
                let channel = read_u16(&mut reader)?;
                let _reserved = read_u16(&mut reader)?;
                events.push(ShdEvent { time_s, channel });
            }
            samples.push(ShdEventSample { label, events });
        } else {
            let bytes = n_events
                .checked_mul(8)
                .ok_or_else(|| "event skip overflow".to_string())?;
            std::io::copy(
                &mut reader.by_ref().take(bytes as u64),
                &mut std::io::sink(),
            )
            .map_err(|error| error.to_string())?;
        }
    }
    Ok(samples)
}

fn read_u32(reader: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u16(reader: &mut impl Read) -> Result<u16, String> {
    let mut bytes = [0_u8; 2];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_f32(reader: &mut impl Read) -> Result<f32, String> {
    Ok(f32::from_bits(read_u32(reader)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boundary_fixture(label: u32) -> ShdEventSample {
        ShdEventSample {
            label,
            events: vec![
                ShdEvent {
                    time_s: 0.100,
                    channel: 0,
                },
                ShdEvent {
                    time_s: 0.100,
                    channel: 0,
                },
                ShdEvent {
                    time_s: 0.111,
                    channel: 4,
                },
                ShdEvent {
                    time_s: 0.121,
                    channel: 5,
                },
                ShdEvent {
                    time_s: 1.399,
                    channel: 699,
                },
                ShdEvent {
                    time_s: 1.401,
                    channel: 699,
                },
            ],
        }
    }

    #[test]
    fn published_contract_aligns_and_preserves_counts() {
        let framed = frame_events(
            &boundary_fixture(3),
            ShdEventContract::published(10).unwrap(),
            FrequencyGeometry::Channels700,
        );
        assert_eq!(framed.frames[0].values, vec![(0, 2.0)]);
        assert_eq!(framed.retained_events, 6);
        assert_eq!(framed.clipped_events, 0);
        assert_eq!(
            framed.frames.iter().map(|f| f.values.len()).sum::<usize>(),
            5
        );
    }

    #[test]
    fn fixed_contract_clips_after_global_horizon_without_saturating() {
        let framed = frame_events(
            &boundary_fixture(4),
            ShdEventContract::fixed(100).unwrap(),
            FrequencyGeometry::Channels700,
        );
        assert_eq!(framed.valid_steps(), 100);
        assert_eq!(framed.retained_events, 5);
        assert_eq!(framed.clipped_events, 1);
    }

    #[test]
    fn adjacent_sum_five_aggregates_counts() {
        let framed = frame_events(
            &boundary_fixture(5),
            ShdEventContract::published(10).unwrap(),
            FrequencyGeometry::AdjacentSum5,
        );
        assert_eq!(framed.n_inputs, 140);
        assert_eq!(framed.frames[0].values, vec![(0, 2.0)]);
        assert_eq!(framed.frames[1].values, vec![(0, 1.0)]);
        assert_eq!(framed.frames[2].values, vec![(1, 1.0)]);
    }

    #[test]
    fn physical_leak_changes_with_dt_not_arbitrary_frame_count() {
        let alpha_10 = (-10.0_f32 / SHD_PHYSICAL_TAU_MS).exp();
        let alpha_4 = (-4.0_f32 / SHD_PHYSICAL_TAU_MS).exp();
        assert!(alpha_4 > alpha_10);
        assert!((alpha_10.powf(0.4) - alpha_4).abs() < 1e-6);
    }
}
