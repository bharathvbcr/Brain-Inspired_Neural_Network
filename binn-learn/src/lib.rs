//! BINN L5 learning: online three-factor plasticity + labeled gradient refs.
//!
//! Production path: [`three_factor::ThreeFactor`] via [`Learner`].
//! Gradient / eligibility references only: `*_baseline.rs` modules (GC1-exempt).

pub mod bptt_baseline;
pub mod eligibility;
pub mod eprop_baseline;
pub mod modulators;
pub mod surrogate_lif_baseline;
pub mod three_factor;

pub use bptt_baseline::{
    BpttBaseline, GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN,
};
pub use eligibility::{decay, stdp, Eligibility};
pub use eprop_baseline::{EpropReference, DEFAULT_EPROP_BETA, EPROP_REFERENCE_LABEL};
pub use modulators::Modulators;
pub use surrogate_lif_baseline::{
    SurrogateLifReference, DEFAULT_SURROGATE_BETA, SURROGATE_LIF_REFERENCE_LABEL,
};
pub use three_factor::{Learner, ThreeFactor};
