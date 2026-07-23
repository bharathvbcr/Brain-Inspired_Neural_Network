//! BINN L6 data: synthetic streams, fixed encoders/decoders, metrics (U12).
//!
//! Dependency direction: `binn-data` → `binn-core` only (plan architecture).
//! Encoders/decoders are **fixed** through the crux — no `train`/`fit` (GC4).

pub mod datasets;
pub mod decoder;
pub mod encoder;
pub mod metrics;
pub mod synth;

pub use datasets::{CoincidenceTask, TemporalClassification};
pub use decoder::{Decoder, LatencyDecoder, PopulationDecoder, Prediction, SpikeLog};
pub use encoder::{CellId, Encoder, LatencyEncoder, PopulationEncoder, Sample, SpikeEvent};
pub use metrics::{Metrics, WorkCosts, WorkCounters};
pub use synth::{SynthConfig, SyntheticStream};
