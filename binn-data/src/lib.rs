//! BINN L6 data: synthetic streams, fixed encoders/decoders, metrics (U12).
//!
//! Dependency direction: `binn-data` → `binn-core` only (plan architecture).
//! Encoders/decoders are **fixed** through the crux — no `train`/`fit` (GC4).

pub mod class_incremental;
pub mod credit_depth;
pub mod datasets;
pub mod decoder;
pub mod encoder;
pub mod metrics;
pub mod shd;
pub mod shd_contract;
pub mod synth;
pub mod temporal_order;
pub mod transfer_bundle;

pub use class_incremental::{ClassIncConfig, ClassIncExample, ClassIncrementalStream};
pub use credit_depth::{
    compose_target, draw_example, true_transition, CreditDepthConfig, CreditDepthExample,
    CreditDepthTask,
};
pub use datasets::{CoincidenceTask, TemporalClassification};
pub use decoder::{Decoder, LatencyDecoder, PopulationDecoder, Prediction, SpikeLog};
pub use encoder::{CellId, Encoder, LatencyEncoder, PopulationEncoder, Sample, SpikeEvent};
pub use metrics::{ActivityComputeAccount, Metrics, WorkCosts, WorkCounters};
pub use shd::{
    default_shd_dir, load_fixture, load_shd_split, load_shd_split_capped, synthesize_fixture,
    write_split, ShdSample, ShdSplit, SHD_CHANCE, SHD_DEFAULT_T, SHD_N_CLASSES, SHD_N_IN,
};
pub use shd_contract::{
    frame_events, read_event_cache, FramedShdSample, FrequencyGeometry, ShdEvent, ShdEventContract,
    ShdEventSample, SparseFrame, SHD_EVENT_MAGIC, SHD_FIXED_WINDOW_MS, SHD_PHYSICAL_TAU_MS,
};
pub use synth::{SynthConfig, SyntheticStream};
pub use temporal_order::{
    time_shuffle, RateAccessibility, TemporalDifficulty, TemporalOrderExample, TemporalOrderSplit,
    RATE_ACCESSIBLE_MARKER_EVENTS, TEMPORAL_DIFFICULTIES, TEMPORAL_ORDER_CHANCE,
    TEMPORAL_ORDER_N_CLASSES, TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
pub use transfer_bundle::{TransferBundle, BINNTRF1_MAGIC, BINNTRF1_VERSION};
