//! BINN L4 composition: areas, k-WTA, assemblies, projection, wiring.
//!
//! Public surface matches Project Plan v6 §4.3 and Build Spec v8 U06–U08.

pub mod area;
pub mod assembly;
pub mod hub;
pub mod inhibitory;
pub mod multi_area;
pub mod predictive;
pub mod project;
pub mod wiring;
pub mod wta;

pub use area::{ActivityLog, ActivitySample, Area, Phase};
pub use assembly::{overlap, overlap_count, Assembly};
pub use hub::Hub;
pub use inhibitory::InhibitoryInterneuronArea;
pub use multi_area::{InterAreaProjection, InterAreaStepOpts, MultiAreaNetwork};
pub use predictive::PredictiveAreaProjection;
pub use project::{associate, project, project_reference};
pub use wiring::{intra_area_event_fraction, wire, AreaRole, Pos, WiringPrior};
pub use wta::{k_wta, k_wta_straight_through, k_wta_with_margin, soft_k_wta, WtaAnnealer};
