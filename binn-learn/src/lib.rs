//! BINN L5 learning: online three-factor plasticity + labeled gradient refs.
//!
//! Production path: [`three_factor::ThreeFactor`] via [`Learner`].
//! Gradient / eligibility references only: `*_baseline.rs` modules (GC1-exempt).

pub mod bptt_baseline;
pub mod c2_replay_baseline;
pub mod consolidation;
pub mod contrastive;
pub mod credit;
pub mod eligibility;
pub mod eprop_baseline;
pub mod input_rate_control;
pub mod matched_deep_gradient;
pub mod matched_dfa_baseline;
pub mod matched_eventprop_baseline;
pub mod matched_local_baseline;
pub mod matched_mech_baseline;
pub mod matched_rl_baseline;
pub mod modulators;
pub mod multi_area_learn;
pub mod pruning;
pub mod scan_training;
pub mod shared_bptt;
pub mod shd_alif;
pub mod shd_attention;
pub mod shd_eprop_baseline;
pub mod shd_matched;
pub mod shd_matched_arms;
pub mod shd_temporal;
pub mod surrogate_lif_baseline;
pub mod three_factor;

pub use multi_area_learn::{winners_to_activity, MultiAreaLearner};

pub use bptt_baseline::{
    BpttBaseline, GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN,
};
pub use c2_replay_baseline::{C2ReplayBaseline, ReplayExample, C2_REPLAY_BASELINE_LABEL};
pub use consolidation::{
    replay_schedule, ConsolidationBudget, ConsolidationMode, ExactReplayBuffer, GenerativeReplay,
    ReplayItem, ReplaySource,
};
pub use contrastive::ContrastiveWakeSleepLearner;
pub use credit::{
    reinforce_term, CreditSignal, FixedRandomFeedback, LearnedReinforceFeedback, LearnedRpeCritic,
    MarginScaledCredit, MultiChannelComponents, MultiChannelNeuromodulator, PostSynapticCredit,
    ReinforceFeedback, RunningMeanBaseline,
};
pub use eligibility::{
    decay, stdp, stdp_surrogate, DualEligibility, Eligibility, PlateauGatedEligibility,
    SurrogateEligibility,
};
pub use eprop_baseline::{EpropReference, DEFAULT_EPROP_BETA, EPROP_REFERENCE_LABEL};
pub use input_rate_control::{
    hierarchical_bootstrap, EquivalenceSummary, InputRateClassifier, InputRateConfig,
    InputRateReport, PairedPredictions, INPUT_RATE_CONTROL_LABEL,
};
pub use matched_deep_gradient::{MatchedDeepGradient, ModulatorScale, MATCHED_DEEP_GRADIENT_LABEL};
pub use matched_dfa_baseline::{
    MatchedBroadcastErr, MatchedDfa, MATCHED_BROADCAST_ERR_LABEL, MATCHED_DFA_LABEL,
};
pub use matched_eventprop_baseline::{MatchedEventProp, MATCHED_EVENTPROP_LABEL};
pub use matched_local_baseline::{
    MatchedArch, MatchedGradient, MatchedLocal, DEFAULT_MATCHED_BETA, MATCHED_GRADIENT_LABEL,
    MATCHED_LOCAL_LABEL,
};
pub use matched_mech_baseline::{
    run_mech_diagnostic, MechArmMetrics, MechDiagnosticReport, MECH_ARM_BROADCAST_PM1,
    MECH_ARM_DFA, MECH_ARM_GRADED_BROADCAST, MECH_ARM_RL_FB, MECH_ARM_SUPERSPIKE,
};
pub use matched_rl_baseline::{
    MatchedRl3LayerLearnedFb, MatchedRl4LayerLearnedFb, MatchedRlDeepLearnedFb, MatchedRlFlat,
    MatchedRlGraded, MatchedRlLearnedFb, MatchedRlReinforceFb, MatchedRlRpe, MATCHED_RL_FLAT_LABEL,
    MATCHED_RL_GRADED_LABEL, MATCHED_RL_LEARNED_FB_LABEL, MATCHED_RL_REINFORCE_FB_LABEL,
    MATCHED_RL_RPE_LABEL,
};
pub use modulators::Modulators;
pub use pruning::{prune, PruneReport, PruningStrategy};
pub use scan_training::{forward_scan_training, ScanTrainingTrace};
pub use shared_bptt::{
    mean_step_rms, random_feedback, train_bptt, train_bptt_sgd, train_feedback,
    train_learned_feedback, train_learned_feedback_adam, Adam, DenseTemporalExample, SharedForward,
    SharedGradients, SharedTemporalNet, StepDiagnostics, ADAM_BETA1, ADAM_BETA2, ADAM_EPS, ADAM_LR,
    GRADIENT_CLIP_NORM, SHARED_BPTT_LABEL,
};
pub use shd_alif::{
    shuffle_labels, AlifEval, ShdAlifArch, ShdAlifArm, ShdAlifConfig, ShdAlifRule, ACTIVITY_MAX,
    ACTIVITY_MIN, DEFAULT_BETA_A, DEFAULT_TAU_A, MAJORITY_PRED_MAX, SHD_ALIF_BROADCAST_LABEL,
    SHD_ALIF_DFA_LABEL, SHD_ALIF_EPROP_LABEL,
};
pub use shd_attention::{
    attention_forward, attention_gradient, attention_logits, AttentionBlock, AttentionCache,
    AttentionConfig, AttentionGradient, AttentionParams, ATTENTION_MAX_CYCLES,
    ATTENTION_MIN_CYCLES, DEFAULT_ATTENTION_DIM, DEFAULT_ATTENTION_LAYERS,
};
pub use shd_eprop_baseline::{
    shd_out_scale, ShdArmReport, ShdBroadcastPm1, ShdDfa, ShdEpropCeiling, ShdExample,
    ShdRlLearnedFb, ShdRlReinforceFb, ShdSuperSpikeCeiling, ShdTrainConfig,
    MODULATOR_PARITY_TOLERANCE, SHD_BROADCAST_PM1_LABEL, SHD_DFA_LABEL, SHD_EPROP_CEILING_LABEL,
    SHD_RL_REINFORCE_FB_LABEL, SHD_SUPERSPIKE_CEILING_LABEL,
};
pub use shd_matched::{
    load_epoch_orders, loss_and_gradient as shd_matched_loss_and_gradient, one_cycle_lr,
    save_epoch_orders, surrogate_derivative as shd_matched_surrogate_derivative, MatchedAdam,
    MatchedForward as ShdMatchedForward, MatchedGradient as ShdMatchedGradient, MatchedShdSample,
    MatchedTrainSpec as ShdMatchedTrainSpec, MatchedWeights as ShdMatchedWeights, PortableRng,
    MATCHED_ADAM_BETA1, MATCHED_ADAM_BETA2, MATCHED_ADAM_EPS, MATCHED_ORDER_MAGIC,
    MATCHED_PHYSICAL_TAU_MS, MATCHED_SURROGATE_ALPHA, MATCHED_THRESHOLD, MATCHED_WEIGHTS_MAGIC,
};
pub use shd_matched_arms::{
    loss_and_gradient_arm as shd_matched_loss_and_gradient_arm,
    loss_and_gradient_arm_scaled as shd_matched_loss_and_gradient_arm_scaled,
    loss_and_gradient_arm_scaled_prepared as shd_matched_loss_and_gradient_arm_scaled_prepared,
    ArmAdam as ShdArmAdam, ArmGradient as ShdArmGradient, ArmWeightLayout as ShdArmWeightLayout,
    ArmWeights as ShdArmWeights, MatchedArm, MATCHED_DEFAULT_BETA_A, MATCHED_DEFAULT_TAU_A,
    MATCHED_WEIGHTS_MAGIC_V2, MATCHED_WEIGHTS_MAGIC_V3,
};
pub use shd_temporal::{apply_temporal, TemporalAudit, TemporalCondition};
pub use surrogate_lif_baseline::{
    SurrogateLifReference, DEFAULT_SURROGATE_BETA, SURROGATE_LIF_REFERENCE_LABEL,
};
pub use three_factor::{Learner, ThreeFactor};
