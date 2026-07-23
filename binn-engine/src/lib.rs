//! BINN L3 event-driven substrate.

pub mod cell;
pub mod engine;
pub mod queue;
pub mod spikelog;
pub mod synapse;

pub use cell::{
    analytic_lif, batch_advance_euler, Cell, CellId, DEFAULT_G_C, DEFAULT_TAU_D, DEFAULT_TAU_M,
    DELTA_THETA, INJECT_WEIGHT, K, TAU_THETA, THETA_REST, V_RESET,
};
pub use engine::{Engine, EngineWorkCounters};
pub use queue::{Event, TimingWheel};
pub use spikelog::{Spike, SpikeLog};
pub use synapse::{Synapse, Synapses};
