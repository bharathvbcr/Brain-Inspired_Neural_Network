//! BINN L4 composition: areas, k-WTA, assemblies, projection, wiring.
//!
//! Public surface matches Project Plan v6 §4.3 and Build Spec v8 U06–U08.

pub mod area;
pub mod assembly;
pub mod hub;
pub mod project;
pub mod wiring;
pub mod wta;

pub use area::{ActivityLog, ActivitySample, Area};
pub use assembly::{overlap, overlap_count, Assembly};
pub use project::{associate, project, project_reference};
pub use wiring::{intra_area_event_fraction, wire, AreaRole, Pos, WiringPrior};
pub use wta::k_wta;
