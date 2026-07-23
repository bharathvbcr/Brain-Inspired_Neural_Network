//! BINN L7 experiment harness (U13): seeds, config hashing, logging, plots, C1.

pub mod config;
pub mod logging;
pub mod plots;
pub mod runner;

pub use config::{n_for_80_percent_power, Config, C1_DEFAULT_HASH_PREFIX};
pub use logging::{EmitError, RunLog, StructuredLogger};
pub use plots::{PlotKind, PlotRequest, PlotResult, Plots};
pub use runner::{
    BudgetDisclosure, C1Report, ConditionLabel, GateG2Verdict, PairedSummary, RunRecord, Runner,
    SeedResult,
};
