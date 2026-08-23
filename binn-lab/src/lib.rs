//! BINN L7 experiment harness (U13–U17): seeds, config hashing, logging, plots, C1–R2.

pub mod c2_config;
pub mod c3_bptt_config;
pub mod c3_config;
pub mod c3_v2_config;
pub mod config;
pub mod credit_config;
pub mod credit_depth_dense;
pub mod dfa_match_config;
pub mod dfa_spike_config;
pub mod eprop_true_config;
pub mod eventprop_match_config;
pub mod gradient_clip;
pub mod guards;
#[cfg(feature = "tables")]
pub mod harvest;
pub mod instrument_status;
pub mod logging;
pub mod mac_probe_config;
pub mod match_config;
pub mod mech_config;
#[cfg(feature = "plots")]
pub mod paper_figures;
pub mod plots;
pub mod r1_config;
pub mod r2_config;
pub mod r2_credit_config;
pub mod replay;
pub mod rl_match_config;
pub mod runner;
pub mod runner_c2;
pub mod runner_c3;
pub mod runner_c3_bptt;
pub mod runner_c3_v2;
pub mod runner_credit;
pub mod runner_dfa_match;
pub mod runner_dfa_spike;
pub mod runner_eprop_true;
pub mod runner_eventprop_match;
pub mod runner_match;
pub mod runner_mech;
pub mod runner_r1;
pub mod runner_r2;
pub mod runner_r2_credit;
pub mod runner_rl_match;
pub mod runner_shd_cal;
pub mod shd_cal_config;
pub mod shd_dense;
pub mod shd_sweep_runner;
pub mod transfer_harness;

pub use c2_config::{C2Config, C2_EXPERIMENT, C2_HASH_PREFIX, C2_PROTOCOL_VERSION};
pub use c3_bptt_config::{C3BpttArm, C3BpttConfig, C3_BPTT_HASH_PREFIX, C3_BPTT_PROTOCOL_VERSION};
pub use c3_config::{C3Config, C3_EXPERIMENT, C3_HASH_PREFIX, C3_PROTOCOL_VERSION};
pub use c3_v2_config::{C3V2Arm, C3V2Config, C3_V2_HASH_PREFIX, C3_V2_PROTOCOL_VERSION};
pub use config::{
    n_for_80_percent_power, Config, C1_DEFAULT_HASH_PREFIX, C1_DFA_LIVE_EXPERIMENT_PREFIX,
    C1_DFA_LIVE_PROTOCOL_VERSION, C1_ELIG_RFB_EXPERIMENT_PREFIX, C1_ELIG_RFB_PROTOCOL_VERSION,
    C1_ELIG_RFB_TAU_E, C1_ISOLATION_EXPERIMENT_PREFIX, C1_ISOLATION_PROTOCOL_VERSION,
    C1_PROJECT_EXPERIMENT_PREFIX, C1_PROJECT_PROTOCOL_VERSION, C1_PROTOCOL_VERSION,
    C1_REINFORCE_FB_EXPERIMENT_PREFIX, C1_REINFORCE_FB_PROTOCOL_VERSION,
    C1_RFB_EPOCH_EXPERIMENT_PREFIX, C1_RFB_EPOCH_LOCAL_EPOCHS_QUICK,
    C1_RFB_EPOCH_LOCAL_EPOCHS_SCIENTIFIC, C1_RFB_EPOCH_PROTOCOL_VERSION,
    C1_SENSITIVITY_EXPERIMENT_PREFIX, C1_SENSITIVITY_PROTOCOL_VERSION, C1_SFB_SOFT_TEMPERATURE,
    C1_SPIKE_EXPERIMENT_PREFIX, C1_SPIKE_PROTOCOL_VERSION, C1_SPIKE_S_EXPERIMENT_PREFIX,
    C1_SPIKE_S_PROTOCOL_VERSION, C1_STRUCTURED_FB_CAPACITY_EXPERIMENT_PREFIX,
    C1_STRUCTURED_FB_CAPACITY_PROTOCOL_VERSION, C1_STRUCTURED_FB_CONT_EXPERIMENT_PREFIX,
    C1_STRUCTURED_FB_CONT_PROTOCOL_VERSION, C1_STRUCTURED_FB_EPOCH_EXPERIMENT_PREFIX,
    C1_STRUCTURED_FB_EPOCH_PROTOCOL_VERSION, C1_STRUCTURED_FB_EXPERIMENT_PREFIX,
    C1_STRUCTURED_FB_FINTH_EXPERIMENT_PREFIX, C1_STRUCTURED_FB_FINTH_PROTOCOL_VERSION,
    C1_STRUCTURED_FB_PROTOCOL_VERSION, C1_STRUCTURED_FB_SOFT_EXPERIMENT_PREFIX,
    C1_STRUCTURED_FB_SOFT_PROTOCOL_VERSION, C1_STRUCTURED_FB_TEACH_EXPERIMENT_PREFIX,
    C1_STRUCTURED_FB_TEACH_PROTOCOL_VERSION,
};
pub use credit_config::{
    CreditArm, CreditConfig, CREDIT_DFA_PROTOCOL_VERSION, CREDIT_EPROP_PROTOCOL_VERSION,
    CREDIT_HASH_PREFIX, CREDIT_ISOLATION_CALIBRATED_EXPERIMENT_PREFIX,
    CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX, CREDIT_ISOLATION_CALIBRATED_PROTOCOL_OFFSET,
    CREDIT_ISOLATION_EXPERIMENT_PREFIX, CREDIT_ISOLATION_HASH_PREFIX,
    CREDIT_ISOLATION_PROTOCOL_OFFSET, CREDIT_MATCHED_PROTOCOL_VERSION, CREDIT_RPE_PROTOCOL_VERSION,
};
pub use credit_depth_dense::{
    credit_depth_chance, credit_depth_examples, credit_depth_input_width,
};
pub use dfa_match_config::{
    DfaMatchConfig, C1_DFA_CHANCE_BASELINE, C1_DFA_EXPERIMENT, C1_DFA_HASH_PREFIX,
    C1_DFA_PROTOCOL_VERSION,
};
pub use dfa_spike_config::{
    DfaSpikeArm, DfaSpikeConfig, DFA_SPIKE_CHANCE_BASELINE, DFA_SPIKE_EXPERIMENT,
    DFA_SPIKE_HASH_PREFIX, DFA_SPIKE_PROTOCOL_VERSION,
};
pub use eprop_true_config::{
    EpropTrueArm, EpropTrueConfig, EPROP_TRUE_EXPERIMENT, EPROP_TRUE_HASH_PREFIX,
    EPROP_TRUE_PROTOCOL_VERSION,
};
pub use eventprop_match_config::{
    EventPropMatchConfig, C1_EVENTPROP_CHANCE_BASELINE, C1_EVENTPROP_EXPERIMENT,
    C1_EVENTPROP_HASH_PREFIX, C1_EVENTPROP_PROTOCOL_VERSION,
};
pub use guards::{
    gap_closed_clamped, gap_closed_exceeds_ceiling, wilson_interval, Degeneracy, ReadoutAudit,
    StimulusProbe, Verdict, CONSTANT_PREDICTOR_EPS, MIN_EVAL_SAMPLES, Z_95,
};
pub use instrument_status::{
    authorize_campaign, CampaignKind, InstrumentState, SHD_INSTRUMENT_STATE,
};
pub use logging::{
    json_escape, trace_export_seed, trace_out_path, write_report, EmitError, RunLog,
    StructuredLogger, TraceArea, TraceEligEdge, TraceProjection, TraceRecorder, TraceScore,
    TraceWeightEdge, TRACE_OUT_ENV, TRACE_SEED_ENV,
};
pub use mac_probe_config::{
    effective_init_w, readout_boost_and_gain, scaled_k_wta, syn_matched_fan_out, MacProbeConfig,
    MacProbeMode, WiringRegime, C1_MAC_PROBE_EXPERIMENT, C1_MAC_PROBE_HASH_PREFIX,
    C1_MAC_PROBE_PROTOCOL_VERSION, C1_MAC_PROBE_SIZE_PROTOCOL_VERSION, C1_MICRO_EXPERIMENT,
    C1_MICRO_FOUNDATION_EXPERIMENT, C1_MICRO_FOUNDATION_PROTOCOL_VERSION, C1_MICRO_HASH_PREFIX,
    C1_MICRO_PROTOCOL_VERSION, DFA_LIVE_SIZE_ACC_FLOOR, DFA_LIVE_SIZE_GAP_LCB_CLEAR,
    DFA_LIVE_SIZE_N_HIDDEN, DFA_LIVE_SIZE_N_SEEDS, FOUNDATION_MICRO_FAN_OUT,
    FOUNDATION_MICRO_NNZ_HI, FOUNDATION_MICRO_NNZ_LO, FOUNDATION_MICRO_N_HIDDEN,
    FOUNDATION_MICRO_RSS_BUDGET_BYTES, FOUNDATION_MICRO_TARGET_NNZ,
    FOUNDATION_MICRO_WALL_SECS_PER_SEED, MAC_PROBE_FULL_C1_REFUSE_N, MAC_PROBE_K_WTA,
    MAC_PROBE_REF_MEAN_FAN_IN, MAC_PROBE_REF_MEAN_READOUT_FAN_IN, MAC_PROBE_TARGET_NNZ,
    MICRO_ACTIVITY_MAX, MICRO_ACTIVITY_MIN, MICRO_MAX_FAN_OUT, MICRO_TARGET_ACTIVITY,
};
pub use match_config::{
    MatchConfig, C1_MATCH_CHANCE_BASELINE, C1_MATCH_EXPERIMENT, C1_MATCH_HASH_PREFIX,
    C1_MATCH_PROTOCOL_VERSION, C1_MATCH_UNDERTRAIN_EPOCH_MULT, C1_MATCH_UNDERTRAIN_EXPERIMENT,
    C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION,
};
pub use mech_config::{
    MechConfig, C1_MECH_EXPERIMENT, C1_MECH_HASH_PREFIX, C1_MECH_PROTOCOL_VERSION,
};
pub use plots::{PlotKind, PlotRequest, PlotResult, Plots};
pub use r1_config::{R1Config, R1_EXPERIMENT, R1_HASH_PREFIX, R1_PROTOCOL_VERSION};
pub use r2_config::{R2Config, R2_EXPERIMENT, R2_HASH_PREFIX, R2_PROTOCOL_VERSION};
pub use r2_credit_config::{
    R2CreditArm, R2CreditConfig, R2_CREDIT_EXPERIMENT, R2_CREDIT_HASH_PREFIX,
    R2_CREDIT_PROTOCOL_VERSION,
};
pub use replay::{ReplayExport, ReplayGroup, ReplayTrial, REPLAY_FORMAT, REPLAY_OUT_ENV};
pub use rl_match_config::{
    RlMatchConfig, C1_RL_CHANCE_BASELINE, C1_RL_EXPERIMENT, C1_RL_HASH_PREFIX, C1_RL_PRIMARY_ARM,
    C1_RL_PROTOCOL_VERSION,
};
pub use runner::{
    freeze_trials, mean, mean_or_nan, mean_var, samples_to_dense_temporal_examples,
    samples_to_gradient_examples, std_error, temporal_order_to_dense_examples,
    temporal_order_to_shd_examples, BudgetDisclosure, C1Report, ConditionLabel, FrozenSplit,
    GateG2Verdict, MacProbeDiagnostics, PairedSummary, RunRecord, Runner, SeedResult,
};
pub use runner_c2::{C2Report, C2Runner, C2SeedResult, GateG3Verdict, OverlapIntervention};
pub use runner_c3::{
    C3Report, C3Runner, C3Verdict, DepthResult, C3_GRADIENT_CREDIT_REFERENCE,
    C3_ORACLE_TEACHER_FORCED_REFERENCE,
};
pub use runner_c3_bptt::{
    C3BpttArmResult, C3BpttDepthResult, C3BpttReport, C3BpttRunner, C3BpttVerdict,
    C3_BPTT_ORACLE_PULSES_CONTRAST, C3_BPTT_SUPERSPIKE_REFERENCE,
};
pub use runner_c3_v2::{
    C3V2ArmResult, C3V2DepthResult, C3V2ParityEvidence, C3V2Report, C3V2Runner, C3V2Verdict,
    C3_V2_MATCHED_GRADIENT_REFERENCE,
};
pub use runner_credit::{
    CreditArmSummary, CreditConditionOutcome, CreditReport, CreditRunner, CreditSeedResult,
    ForwardParityEvidence, EXACT_FORWARD_SURROGATE_GRADIENT_REFERENCE,
};
pub use runner_dfa_match::{DfaMatchReport, DfaMatchRunner, DfaMatchSeedResult};
pub use runner_dfa_spike::{
    DfaSpikeArmSummary, DfaSpikeReport, DfaSpikeRunner, DfaSpikeSeedResult,
    HYBRID_STDP_DFA_CONTRAST, SURROGATE_GRADIENT_SPIKE_CEILING, TRUE_DFA_SPIKE_REFERENCE,
};
pub use runner_eprop_true::{
    EpropTrueArmSummary, EpropTrueReport, EpropTrueRunner, EpropTrueSeedResult,
    HYBRID_STDP_EPROP_CONTRAST, TRUE_SURROGATE_EPROP_REFERENCE,
};
pub use runner_eventprop_match::{
    EventPropMatchReport, EventPropMatchRunner, EventPropMatchSeedResult,
};
pub use runner_match::{gap_closed_matched, MatchReport, MatchRunner, MatchSeedResult};
pub use runner_mech::{MechReport, MechRunner, MechSeedResult};
pub use runner_r1::{AreaSweepPoint, R1Report, R1Runner, R1Verdict};
pub use runner_r2::{CurveShape, GateG4Decision, LogLinearFit, R2Report, R2Runner, ScalingPoint};
pub use runner_r2_credit::{
    R2CreditArmCurve, R2CreditReport, R2CreditRunner, FROZEN_G2_HASH, FROZEN_R2_G4_HASH,
};
pub use runner_rl_match::{RlMatchReport, RlMatchRunner, RlMatchSeedResult};
pub use runner_shd_cal::{ShdCalReport, ShdCalRunner, ShdCalSeedResult};
pub use shd_cal_config::{
    ShdCalConfig, C1_SHD_CAL_CHANCE, C1_SHD_CAL_EXPERIMENT, C1_SHD_CAL_HASH_PREFIX,
    C1_SHD_CAL_HIDDEN256_SCIENTIFIC_HASH, C1_SHD_CAL_PROTOCOL_VERSION,
    C1_SHD_CAL_PROTOCOL_VERSION_V26, C1_SHD_CAL_SCIENTIFIC_HASH, C1_SHD_CAL_V26_SCIENTIFIC_HASH,
    C1_SHD_FULL_EXPERIMENT, C1_SHD_FULL_HASH_PREFIX, C1_SHD_FULL_PROTOCOL_VERSION,
    C1_SHD_FULL_SCIENTIFIC_HASH, C1_SHD_FULL_SMOKE_HASH,
};
pub use shd_dense::{
    class_histogram, contract_alpha, contract_timesteps, framed_to_dense_temporal_example,
    load_shd_dense_examples, majority_class_rate, shd_sample_to_example,
};
pub use shd_sweep_runner::{ShdSweepReport, ShdSweepResult};
pub use transfer_harness::{
    MicroTrace, Selection, ThresholdReset, Timing, TraceMode, TransferEval, TransferModel,
    TransferPole, MICRO_TOLERANCE, TRACE_TAU_E, TRANSFER_HASH_PREFIX, TRANSFER_K,
    TRANSFER_PROTOCOL_VERSION,
};
