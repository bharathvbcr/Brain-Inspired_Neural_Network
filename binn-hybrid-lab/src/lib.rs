//! Training-only BINN-Hybrid teachers, distillation, and feasibility gates.
//!
//! Nothing in this crate belongs in a teacher-free deployment binary.

pub mod benchmark;
pub mod diagnostics;
pub mod distill;
pub mod factorization;
pub mod production_diagnostics;
pub mod protocol;
pub mod teacher;
pub mod temperature_ablation;
pub mod temperature_ladder;

pub use benchmark::{
    run_feasibility, ArmSummary, BenchmarkSummary, FeasibilityReport, H0Decision, UpperBoundArm,
};
pub use diagnostics::{
    run_diagnostics, DiagnosticArm, DiagnosticConfig, DiagnosticReport, DIAGNOSTIC_PROTOCOL_VERSION,
};
pub use distill::{distill_linear_head, DistillationConfig, DistillationExample};
pub use factorization::{factorization_audit, FactorizationAudit};
pub use production_diagnostics::{
    run_production_diagnostics, ProductionDiagnosticArm, ProductionDiagnosticConfig,
    ProductionDiagnosticReport, PRODUCTION_DIAGNOSTIC_PROTOCOL_VERSION,
};
pub use protocol::{HybridProtocol, HYBRID_PROTOCOL_VERSION};
pub use teacher::{
    SparseTerminalModel, TeacherTargets, TerminalTeacher, TerminalTrace, TerminalTraceTeacher,
};
pub use temperature_ablation::{
    run_temperature_ablation, AblationVariant, ConnectivityPattern, TemperatureAblationConfig,
    TemperatureAblationReport, TEMPERATURE_ABLATION_PROTOCOL_VERSION,
};
pub use temperature_ladder::{
    run_temperature_ladder, LadderArm, TemperatureLadderConfig, TemperatureLadderReport,
    WinnerTemperature, TEMPERATURE_LADDER_PROTOCOL_VERSION,
};
