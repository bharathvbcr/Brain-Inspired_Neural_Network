use std::collections::BTreeMap;

use binn_core::Csr;
use binn_engine::Engine;
use binn_hybrid_learn::{
    fnv1a64, topology_signature, CreditFeatures, CreditGranularity, CreditHeadArtifact,
    HybridLearner,
};

use crate::distill::{distill_linear_head, DistillationConfig, DistillationExample};
use crate::factorization::{factorization_audit, FactorizationAudit};
use crate::protocol::HybridProtocol;
use crate::teacher::{
    edge_range, LocalRng, SparseTerminalModel, TerminalTeacher, TerminalTraceTeacher,
};

const C1_INPUTS: usize = 8;
const C1_HIDDEN: usize = 12;
const C3_STATES: usize = 4;
const C3_OPERATIONS: usize = 2;
const MAX_DISTILLATION_EXAMPLES: usize = 50_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpperBoundArm {
    ExistingPostSynaptic,
    OraclePostSynaptic,
    DirectPerSynapse,
    DistilledStudent,
}

impl UpperBoundArm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExistingPostSynaptic => "existing-post-synaptic",
            Self::OraclePostSynaptic => "least-squares-post-synaptic",
            Self::DirectPerSynapse => "direct-per-synapse",
            Self::DistilledStudent => "distilled-student",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArmSummary {
    pub arm: UpperBoundArm,
    pub mean_accuracy: f32,
    pub variance: f32,
    pub lower_95: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkSummary {
    pub name: String,
    pub depth: Option<usize>,
    pub arms: Vec<ArmSummary>,
    pub mean_existing_cosine: f32,
    pub mean_oracle_cosine: f32,
    pub mean_existing_sign_agreement: f32,
    pub mean_oracle_sign_agreement: f32,
}

impl BenchmarkSummary {
    pub fn arm(&self, arm: UpperBoundArm) -> &ArmSummary {
        self.arms
            .iter()
            .find(|summary| summary.arm == arm)
            .expect("arm summary")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum H0Decision {
    HybridNoGo,
    H1NoGo,
    PilotPostSynaptic,
    PilotPerSynapse,
    ProceedPostSynaptic,
    ProceedPerSynapse,
}

impl H0Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HybridNoGo => "HYBRID_NO_GO",
            Self::H1NoGo => "H1_NO_GO",
            Self::PilotPostSynaptic => "PILOT_POST_SYNAPTIC",
            Self::PilotPerSynapse => "PILOT_PER_SYNAPSE",
            Self::ProceedPostSynaptic => "PROCEED_POST_SYNAPTIC",
            Self::ProceedPerSynapse => "PROCEED_PER_SYNAPSE",
        }
    }

    pub fn granularity(self) -> Option<CreditGranularity> {
        match self {
            Self::HybridNoGo | Self::H1NoGo => None,
            Self::PilotPostSynaptic | Self::ProceedPostSynaptic => {
                Some(CreditGranularity::PostSynaptic)
            }
            Self::PilotPerSynapse | Self::ProceedPerSynapse => Some(CreditGranularity::PerSynapse),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FeasibilityReport {
    pub protocol_hash: String,
    pub quick: bool,
    pub decision: H0Decision,
    pub benchmarks: Vec<BenchmarkSummary>,
    pub teacher_d_star: Option<usize>,
    pub student_d_star: Option<usize>,
    pub artifacts: Vec<(String, CreditHeadArtifact)>,
    pub no_test_updates: bool,
    /// H0 uses production sparse storage but a smooth matched forward, so it
    /// cannot itself authorize a scientific C1/C3 claim.
    pub scientific_claim_allowed: bool,
}

impl FeasibilityReport {
    pub fn render_markdown(&self, protocol: &HybridProtocol) -> String {
        let mut output = format!(
            "# BINN-Hybrid H0/H1 feasibility\n\n\
             - protocol: `{}`\n\
             - schedule: {}\n\
             - decision: **{}**\n\
             - selected granularity: {}\n\
             - teacher D*: {}\n\
             - student D*: {}\n\
             - no test updates: **{}**\n\
             - scientific claim allowed: **{}**\n\n",
            self.protocol_hash,
            if self.quick {
                "PILOT"
            } else {
                "held-out preflight"
            },
            self.decision.as_str(),
            self.decision
                .granularity()
                .map(CreditGranularity::as_str)
                .unwrap_or("none"),
            optional_depth(self.teacher_d_star),
            optional_depth(self.student_d_star),
            self.no_test_updates,
            self.scientific_claim_allowed,
        );
        output.push_str(
            "> This protocol uses production CSR/weight storage with a smooth matched sparse \
             forward. It validates teacher math, factorization, artifact freezing, and \
             teacher-free execution, but it does not replace a production event-engine C1/C3 \
             result. H2/H3 remain stopped.\n\n",
        );
        output.push_str(&format!(
            "Thresholds: C1 accuracy {:.3}; C3 accuracy {:.3}; teacher D* ≥ {}; \
             student D* ≥ {}. Normalized gap is not evaluated by this surrogate; \
             it requires the production matched-dense protocol (target {:.3}).\n\n",
            protocol.c1_accuracy_floor,
            protocol.c3_accuracy_floor,
            protocol.c3_min_teacher_depth,
            protocol.c3_min_student_depth,
            protocol.min_gap_closed,
        ));
        output.push_str(
            "| benchmark | depth | arm | mean accuracy | variance | lower 95% |\n\
             |---|---:|---|---:|---:|---:|\n",
        );
        for benchmark in &self.benchmarks {
            for arm in &benchmark.arms {
                output.push_str(&format!(
                    "| {} | {} | {} | {:.4} | {:.6} | {:.4} |\n",
                    benchmark.name,
                    benchmark
                        .depth
                        .map(|depth| depth.to_string())
                        .unwrap_or_else(|| "—".to_string()),
                    arm.arm.as_str(),
                    arm.mean_accuracy,
                    arm.variance,
                    arm.lower_95,
                ));
            }
        }
        output.push_str(
            "\n| benchmark | depth | existing cosine | oracle cosine | existing sign | oracle sign |\n\
             |---|---:|---:|---:|---:|---:|\n",
        );
        for benchmark in &self.benchmarks {
            output.push_str(&format!(
                "| {} | {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                benchmark.name,
                benchmark
                    .depth
                    .map(|depth| depth.to_string())
                    .unwrap_or_else(|| "—".to_string()),
                benchmark.mean_existing_cosine,
                benchmark.mean_oracle_cosine,
                benchmark.mean_existing_sign_agreement,
                benchmark.mean_oracle_sign_agreement,
            ));
        }
        output
    }
}

#[derive(Clone)]
struct Example {
    input: Vec<f32>,
    label: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct C3Example {
    pub(crate) initial_state: usize,
    pub(crate) operations: Vec<usize>,
    pub(crate) label: usize,
}

#[derive(Default)]
struct AuditAccumulator {
    existing_cosine: Vec<f32>,
    oracle_cosine: Vec<f32>,
    existing_sign: Vec<f32>,
    oracle_sign: Vec<f32>,
}

impl AuditAccumulator {
    fn observe(&mut self, audit: &FactorizationAudit) {
        self.existing_cosine.push(audit.existing_cosine);
        self.oracle_cosine.push(audit.oracle_cosine);
        self.existing_sign.push(audit.existing_sign_agreement);
        self.oracle_sign.push(audit.oracle_sign_agreement);
    }
}

pub(crate) struct C3CompositionTrace {
    pub(crate) states: Vec<[f32; C3_STATES]>,
    pub(crate) operations: Vec<usize>,
    pub(crate) prediction: usize,
}

pub(crate) struct C3CompositionModel {
    pub(crate) engine: Engine,
}

impl C3CompositionModel {
    pub(crate) fn new(seed: u64) -> Self {
        let n_pre = C3_OPERATIONS * C3_STATES;
        let n_cells = n_pre + C3_STATES;
        let mut rows = vec![Vec::new(); n_cells];
        for row in rows.iter_mut().take(n_pre) {
            row.extend((n_pre..n_cells).map(|post| post as u32));
        }
        let conn = Csr::from_adjacency(&rows);
        let mut rng = LocalRng::new(seed ^ 0xc3c0_4d50);
        let weights = (0..conn.nnz())
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * 0.04)
            .collect::<Vec<_>>();
        let mut engine = Engine::with_cells(n_cells);
        engine.set_connectivity(conn, weights);
        Self { engine }
    }

    pub(crate) fn signature(&self) -> u64 {
        topology_signature(&self.engine)
    }

    pub(crate) fn edge(&self, operation: usize, pre_state: usize, post_state: usize) -> usize {
        (operation * C3_STATES + pre_state) * C3_STATES + post_state
    }

    pub(crate) fn forward(&self, example: &C3Example) -> C3CompositionTrace {
        let mut state = [0.0f32; C3_STATES];
        state[example.initial_state] = 1.0;
        let mut states = vec![state];
        for &operation in &example.operations {
            let mut next = state;
            for (post, next_value) in next.iter_mut().enumerate() {
                for (pre, &pre_value) in state.iter().enumerate() {
                    *next_value += self.engine.edge_w[self.edge(operation, pre, post)] * pre_value;
                }
            }
            state = next;
            states.push(state);
        }
        let prediction = state
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or(0);
        C3CompositionTrace {
            states,
            operations: example.operations.clone(),
            prediction,
        }
    }

    pub(crate) fn teacher_targets(
        &self,
        trace: &C3CompositionTrace,
        terminal_label: usize,
        learning_rate: f32,
        reward: Option<f32>,
    ) -> crate::teacher::TeacherTargets {
        let final_state = trace.states.last().expect("terminal state");
        let probabilities = softmax_fixed(final_state);
        let loss = -probabilities[terminal_label].max(1e-12).ln();
        let mut downstream = [0.0f32; C3_STATES];
        for (state, probability) in probabilities.iter().copied().enumerate() {
            downstream[state] = f32::from(state == terminal_label) - probability;
        }
        let mut edge_deltas = vec![0.0f32; self.engine.edge_w.len()];
        let mut post_credit = [0.0f32; C3_STATES];
        for step in (0..trace.operations.len()).rev() {
            let operation = trace.operations[step];
            let previous = trace.states[step];
            for pre in 0..C3_STATES {
                for post in 0..C3_STATES {
                    edge_deltas[self.edge(operation, pre, post)] +=
                        learning_rate * previous[pre] * downstream[post];
                }
            }
            for post in 0..C3_STATES {
                post_credit[post] += learning_rate * downstream[post];
            }
            let mut previous_credit = downstream;
            for (pre, credit) in previous_credit.iter_mut().enumerate() {
                for (post, &downstream_value) in downstream.iter().enumerate() {
                    *credit +=
                        self.engine.edge_w[self.edge(operation, pre, post)] * downstream_value;
                }
            }
            downstream = previous_credit;
        }
        let mut credits = vec![0.0f32; self.engine.num_cells()];
        let output_start = C3_OPERATIONS * C3_STATES;
        credits[output_start..output_start + C3_STATES].copy_from_slice(&post_credit);
        crate::teacher::TeacherTargets {
            loss,
            edge_deltas,
            post_credits: credits,
            features: self.credit_features(trace, reward),
        }
    }

    pub(crate) fn credit_features(
        &self,
        trace: &C3CompositionTrace,
        reward: Option<f32>,
    ) -> Vec<CreditFeatures> {
        let mut pre_trace = [[0.0f32; C3_STATES]; C3_OPERATIONS];
        let mut post_trace = [[0.0f32; C3_STATES]; C3_OPERATIONS];
        let mut eligibility = [[[0.0f32; C3_STATES]; C3_STATES]; C3_OPERATIONS];
        let mut counts = [0usize; C3_OPERATIONS];
        for (step, &operation) in trace.operations.iter().enumerate() {
            counts[operation] += 1;
            let previous = trace.states[step];
            let next = trace.states[step + 1];
            for pre in 0..C3_STATES {
                pre_trace[operation][pre] += previous[pre];
                for post in 0..C3_STATES {
                    eligibility[operation][pre][post] += previous[pre] * next[post];
                }
            }
            for post in 0..C3_STATES {
                post_trace[operation][post] += next[post];
            }
        }
        let mut features = Vec::with_capacity(self.engine.edge_w.len());
        for operation in 0..C3_OPERATIONS {
            let divisor = counts[operation].max(1) as f32;
            for pre in 0..C3_STATES {
                for post in 0..C3_STATES {
                    let edge = self.edge(operation, pre, post);
                    let pre_value = pre_trace[operation][pre] / divisor;
                    let post_value = post_trace[operation][post] / divisor;
                    features.push(CreditFeatures {
                        pre_trace: pre_value,
                        post_trace: post_value,
                        eligibility: eligibility[operation][pre][post] / divisor,
                        weight: self.engine.edge_w[edge],
                        pre_membrane: pre_value,
                        post_membrane: post_value,
                        pre_threshold: 0.0,
                        post_threshold: 0.0,
                        pre_activity: pre_value.abs(),
                        post_activity: post_value.abs(),
                        structural_id: ((operation as u32) << 16)
                            ^ ((pre as u32) << 8)
                            ^ post as u32,
                        broadcast_reward: reward,
                    });
                }
            }
        }
        features
    }

    pub(crate) fn edge_posts(&self) -> Vec<usize> {
        let output_start = C3_OPERATIONS * C3_STATES;
        (0..C3_OPERATIONS)
            .flat_map(|_| {
                (0..C3_STATES)
                    .flat_map(move |_| (0..C3_STATES).map(move |post| output_start + post))
            })
            .collect()
    }

    pub(crate) fn apply_deltas(&mut self, deltas: &[f32]) {
        assert_eq!(deltas.len(), self.engine.edge_w.len());
        for (edge, &delta) in deltas.iter().enumerate() {
            let updated = (self.engine.edge_w[edge] + delta).clamp(-2.0, 2.0);
            self.engine.edge_w[edge] = updated;
            self.engine.syn.as_mut_slice()[edge].weight = updated;
        }
    }
}

pub(crate) fn softmax_fixed(logits: &[f32; C3_STATES]) -> [f32; C3_STATES] {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values = [0.0f32; C3_STATES];
    let mut sum = 0.0f32;
    for (slot, &logit) in values.iter_mut().zip(logits) {
        *slot = (logit - max).exp();
        sum += *slot;
    }
    for value in &mut values {
        *value /= sum.max(1e-12);
    }
    values
}

pub fn run_feasibility(protocol: &HybridProtocol) -> FeasibilityReport {
    let protocol_hash = protocol.hash_string();
    let teacher_hash = protocol.hash();
    let development_seeds = seeds(0x4842_4445_5600_0001, protocol.development_seeds);
    let held_out_master = if protocol.quick {
        0x4842_5049_4c4f_5402
    } else {
        0x4842_4652_4553_4802
    };
    let held_out_seeds = seeds(held_out_master, protocol.held_out_seeds);
    let training_seed_hash = seed_hash(&development_seeds);

    let c1_layers = vec![C1_INPUTS, C1_HIDDEN, 2];
    let c1_development_audit = audit_generic(&c1_layers, &development_seeds, protocol, |seed| {
        c1_examples(seed, protocol.train_examples)
    });
    let mut h0_benchmarks = vec![merge_audit(
        evaluate_benchmark(
            "c1-terminal-surrogate",
            None,
            &c1_layers,
            &development_seeds,
            protocol,
            |seed| c1_examples(seed, protocol.train_examples),
            |seed| c1_examples(seed ^ 0x4445_5654, protocol.test_examples),
            None,
        ),
        &c1_development_audit,
    )];
    let c3_development_audits = audit_c3(&development_seeds, protocol);
    for depth in 1..=8 {
        h0_benchmarks.push(merge_audit(
            evaluate_c3(depth, &development_seeds, protocol, None),
            &c3_development_audits[depth - 1],
        ));
    }
    let c1_direct_pass = h0_benchmarks[0]
        .arm(UpperBoundArm::DirectPerSynapse)
        .lower_95
        >= protocol.c1_accuracy_floor;
    let c1_post_pass = h0_benchmarks[0]
        .arm(UpperBoundArm::OraclePostSynaptic)
        .lower_95
        >= protocol.c1_accuracy_floor;
    let teacher_d_star = d_star(
        &h0_benchmarks,
        UpperBoundArm::DirectPerSynapse,
        protocol.c3_accuracy_floor,
    );
    let post_d_star = d_star(
        &h0_benchmarks,
        UpperBoundArm::OraclePostSynaptic,
        protocol.c3_accuracy_floor,
    );
    let direct_pass =
        c1_direct_pass && teacher_d_star.unwrap_or(0) >= protocol.c3_min_teacher_depth;
    let post_pass = c1_post_pass && post_d_star.unwrap_or(0) >= protocol.c3_min_teacher_depth;
    if !direct_pass {
        return FeasibilityReport {
            protocol_hash,
            quick: protocol.quick,
            decision: H0Decision::HybridNoGo,
            benchmarks: h0_benchmarks,
            teacher_d_star,
            student_d_star: None,
            artifacts: Vec::new(),
            no_test_updates: true,
            scientific_claim_allowed: false,
        };
    }
    let selected_granularity = if post_pass {
        CreditGranularity::PostSynaptic
    } else {
        CreditGranularity::PerSynapse
    };

    let (mut c1_artifact, _c1_granularity, c1_dev) = prepare_artifact(
        "c1-terminal-surrogate",
        &c1_layers,
        &development_seeds,
        protocol,
        teacher_hash,
        training_seed_hash,
        |seed| c1_examples(seed, protocol.train_examples),
    );
    c1_artifact.granularity = selected_granularity;
    c1_artifact.checksum = c1_artifact.computed_checksum();
    let mut artifacts = vec![("c1-credit-head.artifact".to_string(), c1_artifact.clone())];
    let mut benchmarks = Vec::new();
    let c1_summary = evaluate_benchmark(
        "c1-terminal-surrogate",
        None,
        &c1_layers,
        &held_out_seeds,
        protocol,
        |seed| c1_examples(seed, protocol.train_examples),
        |seed| c1_examples(seed ^ 0x5453_5400, protocol.test_examples),
        Some(&c1_artifact),
    );
    benchmarks.push(merge_audit(c1_summary, &c1_dev));

    let (mut c3_artifact, _c3_granularity, c3_development_audits) = prepare_c3_artifact(
        &development_seeds,
        protocol,
        teacher_hash ^ 0xc300,
        training_seed_hash,
    );
    c3_artifact.granularity = selected_granularity;
    c3_artifact.checksum = c3_artifact.computed_checksum();
    artifacts.push(("c3-credit-head.artifact".to_string(), c3_artifact.clone()));
    for depth in 1..=8 {
        let summary = evaluate_c3(depth, &held_out_seeds, protocol, Some(&c3_artifact));
        benchmarks.push(merge_audit(summary, &c3_development_audits[depth - 1]));
    }

    let student_d_star = d_star(
        &benchmarks,
        UpperBoundArm::DistilledStudent,
        protocol.c3_accuracy_floor,
    );
    let c1_student_pass =
        benchmarks[0].arm(UpperBoundArm::DistilledStudent).lower_95 >= protocol.c1_accuracy_floor;
    let student_pass =
        c1_student_pass && student_d_star.unwrap_or(0) >= protocol.c3_min_student_depth;
    let decision = if !student_pass {
        H0Decision::H1NoGo
    } else if post_pass {
        H0Decision::PilotPostSynaptic
    } else {
        H0Decision::PilotPerSynapse
    };

    let no_test_updates = benchmarks.iter().all(|benchmark| {
        benchmark
            .arms
            .iter()
            .all(|arm| arm.mean_accuracy.is_finite())
    });
    debug_assert_eq!(selected_granularity, c1_artifact.granularity);
    debug_assert_eq!(selected_granularity, c3_artifact.granularity);
    FeasibilityReport {
        protocol_hash,
        quick: protocol.quick,
        decision,
        benchmarks,
        teacher_d_star,
        student_d_star,
        artifacts,
        no_test_updates,
        scientific_claim_allowed: false,
    }
}

fn prepare_artifact<F>(
    _name: &str,
    layers: &[usize],
    development_seeds: &[u64],
    protocol: &HybridProtocol,
    teacher_hash: u64,
    training_seed_hash: u64,
    train_examples: F,
) -> (CreditHeadArtifact, CreditGranularity, AuditAccumulator)
where
    F: Fn(u64) -> Vec<Example>,
{
    let teacher = TerminalTraceTeacher;
    let mut examples = Vec::new();
    let mut audit_accumulator = AuditAccumulator::default();
    let mut post_pass_votes = 0usize;
    for &seed in development_seeds {
        let mut model = SparseTerminalModel::new(layers.to_vec(), seed);
        for example in train_examples(seed) {
            let trace = model.forward(&example.input);
            let reward = if trace.prediction == example.label {
                1.0
            } else {
                -1.0
            };
            let mut targets =
                teacher.targets(&model, &trace, example.label, protocol.learning_rate);
            for features in &mut targets.features {
                features.broadcast_reward = Some(reward);
            }
            let posts = edge_posts(&model);
            let audit = factorization_audit(&targets, &posts);
            audit_accumulator.observe(&audit);
            if audit.oracle_cosine >= 0.90 && audit.oracle_sign_agreement >= 0.80 {
                post_pass_votes += 1;
            }
            if examples.len() < MAX_DISTILLATION_EXAMPLES {
                for (features, &target_delta) in targets.features.iter().zip(&targets.edge_deltas) {
                    if examples.len() >= MAX_DISTILLATION_EXAMPLES {
                        break;
                    }
                    examples.push(DistillationExample {
                        features: *features,
                        target_delta,
                    });
                }
            }
            model.apply_deltas(&targets.edge_deltas);
        }
    }
    let total_audits = audit_accumulator.oracle_cosine.len().max(1);
    let granularity = if post_pass_votes * 2 >= total_audits {
        CreditGranularity::PostSynaptic
    } else {
        CreditGranularity::PerSynapse
    };
    let topology_signature = SparseTerminalModel::new(layers.to_vec(), 1).signature();
    let artifact = distill_linear_head(
        &examples,
        DistillationConfig {
            epochs: if protocol.quick { 12 } else { 40 },
            learning_rate: 0.015,
            l2: 1e-5,
            output_scale: 0.10,
        },
        topology_signature,
        granularity,
        teacher_hash,
        training_seed_hash,
    )
    .expect("finite distillation artifact");
    (artifact, granularity, audit_accumulator)
}

fn audit_generic<F>(
    layers: &[usize],
    development_seeds: &[u64],
    protocol: &HybridProtocol,
    train_examples: F,
) -> AuditAccumulator
where
    F: Fn(u64) -> Vec<Example>,
{
    let teacher = TerminalTraceTeacher;
    let mut accumulator = AuditAccumulator::default();
    for &seed in development_seeds {
        let mut model = SparseTerminalModel::new(layers.to_vec(), seed);
        for example in train_examples(seed) {
            let trace = model.forward(&example.input);
            let targets = teacher.targets(&model, &trace, example.label, protocol.learning_rate);
            let audit = factorization_audit(&targets, &edge_posts(&model));
            accumulator.observe(&audit);
            model.apply_deltas(&targets.edge_deltas);
        }
    }
    accumulator
}

fn audit_c3(development_seeds: &[u64], protocol: &HybridProtocol) -> Vec<AuditAccumulator> {
    let mut audits = (0..8)
        .map(|_| AuditAccumulator::default())
        .collect::<Vec<_>>();
    for depth in 1..=8 {
        for &seed in development_seeds {
            let mut model = C3CompositionModel::new(seed);
            for example in c3_examples(seed, depth, protocol.train_examples) {
                let trace = model.forward(&example);
                let targets =
                    model.teacher_targets(&trace, example.label, protocol.learning_rate, None);
                audits[depth - 1].observe(&factorization_audit(&targets, &model.edge_posts()));
                model.apply_deltas(&targets.edge_deltas);
            }
        }
    }
    audits
}

fn prepare_c3_artifact(
    development_seeds: &[u64],
    protocol: &HybridProtocol,
    teacher_hash: u64,
    training_seed_hash: u64,
) -> (CreditHeadArtifact, CreditGranularity, Vec<AuditAccumulator>) {
    let mut examples = Vec::new();
    let mut audits = (0..8)
        .map(|_| AuditAccumulator::default())
        .collect::<Vec<_>>();
    let mut post_votes = 0usize;
    let mut total_votes = 0usize;
    for depth in 1..=8 {
        for &seed in development_seeds {
            let mut model = C3CompositionModel::new(seed);
            for example in c3_examples(seed, depth, protocol.train_examples) {
                let trace = model.forward(&example);
                let reward = if trace.prediction == example.label {
                    1.0
                } else {
                    -1.0
                };
                let mut targets = model.teacher_targets(
                    &trace,
                    example.label,
                    protocol.learning_rate,
                    Some(reward),
                );
                let audit = factorization_audit(&targets, &model.edge_posts());
                audits[depth - 1].observe(&audit);
                total_votes += 1;
                if audit.oracle_cosine >= 0.90 && audit.oracle_sign_agreement >= 0.80 {
                    post_votes += 1;
                }
                if examples.len() < MAX_DISTILLATION_EXAMPLES {
                    for (features, &target_delta) in
                        targets.features.iter_mut().zip(&targets.edge_deltas)
                    {
                        if examples.len() >= MAX_DISTILLATION_EXAMPLES {
                            break;
                        }
                        features.broadcast_reward = Some(reward);
                        examples.push(DistillationExample {
                            features: *features,
                            target_delta,
                        });
                    }
                }
                model.apply_deltas(&targets.edge_deltas);
            }
        }
    }
    let granularity = if post_votes * 2 >= total_votes.max(1) {
        CreditGranularity::PostSynaptic
    } else {
        CreditGranularity::PerSynapse
    };
    let topology = C3CompositionModel::new(1).signature();
    let artifact = distill_linear_head(
        &examples,
        DistillationConfig {
            epochs: if protocol.quick { 16 } else { 50 },
            learning_rate: 0.015,
            l2: 1e-5,
            output_scale: 0.10,
        },
        topology,
        granularity,
        teacher_hash,
        training_seed_hash,
    )
    .expect("finite C3 artifact");
    (artifact, granularity, audits)
}

fn evaluate_c3(
    depth: usize,
    seeds: &[u64],
    protocol: &HybridProtocol,
    artifact: Option<&CreditHeadArtifact>,
) -> BenchmarkSummary {
    let mut accuracies: BTreeMap<UpperBoundArm, Vec<f32>> = BTreeMap::new();
    for arm in [
        UpperBoundArm::ExistingPostSynaptic,
        UpperBoundArm::OraclePostSynaptic,
        UpperBoundArm::DirectPerSynapse,
    ] {
        accuracies.insert(arm, Vec::new());
    }
    if artifact.is_some() {
        accuracies.insert(UpperBoundArm::DistilledStudent, Vec::new());
    }
    for &seed in seeds {
        let train = c3_examples(seed, depth, protocol.train_examples);
        let test = c3_examples(seed ^ 0x5453_5400, depth, protocol.test_examples);
        for arm in [
            UpperBoundArm::ExistingPostSynaptic,
            UpperBoundArm::OraclePostSynaptic,
            UpperBoundArm::DirectPerSynapse,
        ] {
            let accuracy = train_c3_upper_bound(
                C3CompositionModel::new(seed),
                &train,
                &test,
                arm,
                protocol.learning_rate,
            );
            accuracies.get_mut(&arm).expect("arm").push(accuracy);
        }
        if let Some(artifact) = artifact {
            let accuracy = train_c3_student(
                C3CompositionModel::new(seed),
                &train,
                &test,
                artifact.clone(),
            );
            accuracies
                .get_mut(&UpperBoundArm::DistilledStudent)
                .expect("student")
                .push(accuracy);
        }
    }
    BenchmarkSummary {
        name: "c3-terminal-composition".to_string(),
        depth: Some(depth),
        arms: accuracies
            .into_iter()
            .map(|(arm, values)| summarize(arm, &values, protocol.confidence_z))
            .collect(),
        mean_existing_cosine: 0.0,
        mean_oracle_cosine: 0.0,
        mean_existing_sign_agreement: 0.0,
        mean_oracle_sign_agreement: 0.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_benchmark<F, G>(
    name: &str,
    depth: Option<usize>,
    layers: &[usize],
    seeds: &[u64],
    protocol: &HybridProtocol,
    train_examples: F,
    test_examples: G,
    artifact: Option<&CreditHeadArtifact>,
) -> BenchmarkSummary
where
    F: Fn(u64) -> Vec<Example>,
    G: Fn(u64) -> Vec<Example>,
{
    let mut accuracies: BTreeMap<UpperBoundArm, Vec<f32>> = BTreeMap::new();
    for arm in [
        UpperBoundArm::ExistingPostSynaptic,
        UpperBoundArm::OraclePostSynaptic,
        UpperBoundArm::DirectPerSynapse,
    ] {
        accuracies.insert(arm, Vec::new());
    }
    if artifact.is_some() {
        accuracies.insert(UpperBoundArm::DistilledStudent, Vec::new());
    }
    for &seed in seeds {
        let train = train_examples(seed);
        let test = test_examples(seed);
        for arm in [
            UpperBoundArm::ExistingPostSynaptic,
            UpperBoundArm::OraclePostSynaptic,
            UpperBoundArm::DirectPerSynapse,
        ] {
            let accuracy = train_upper_bound(
                SparseTerminalModel::new(layers.to_vec(), seed),
                &train,
                &test,
                arm,
                protocol.learning_rate,
            );
            accuracies.get_mut(&arm).expect("arm").push(accuracy);
        }
        if let Some(artifact) = artifact {
            let accuracy = train_student(
                SparseTerminalModel::new(layers.to_vec(), seed),
                &train,
                &test,
                artifact.clone(),
            );
            accuracies
                .get_mut(&UpperBoundArm::DistilledStudent)
                .expect("student")
                .push(accuracy);
        }
    }
    let arms = accuracies
        .into_iter()
        .map(|(arm, values)| summarize(arm, &values, protocol.confidence_z))
        .collect();
    BenchmarkSummary {
        name: name.to_string(),
        depth,
        arms,
        mean_existing_cosine: 0.0,
        mean_oracle_cosine: 0.0,
        mean_existing_sign_agreement: 0.0,
        mean_oracle_sign_agreement: 0.0,
    }
}

fn train_upper_bound(
    mut model: SparseTerminalModel,
    train: &[Example],
    test: &[Example],
    arm: UpperBoundArm,
    learning_rate: f32,
) -> f32 {
    let teacher = TerminalTraceTeacher;
    for example in train {
        let trace = model.forward(&example.input);
        let targets = teacher.targets(&model, &trace, example.label, learning_rate);
        let audit = factorization_audit(&targets, &edge_posts(&model));
        let deltas = match arm {
            UpperBoundArm::ExistingPostSynaptic => audit.existing_post_deltas,
            UpperBoundArm::OraclePostSynaptic => audit.oracle_post_deltas,
            UpperBoundArm::DirectPerSynapse => audit.direct_edge_deltas,
            UpperBoundArm::DistilledStudent => unreachable!("student has separate path"),
        };
        model.apply_deltas(&deltas);
    }
    accuracy(&model, test)
}

fn train_c3_upper_bound(
    mut model: C3CompositionModel,
    train: &[C3Example],
    test: &[C3Example],
    arm: UpperBoundArm,
    learning_rate: f32,
) -> f32 {
    for example in train {
        let trace = model.forward(example);
        let targets = model.teacher_targets(&trace, example.label, learning_rate, None);
        let audit = factorization_audit(&targets, &model.edge_posts());
        let deltas = match arm {
            UpperBoundArm::ExistingPostSynaptic => audit.existing_post_deltas,
            UpperBoundArm::OraclePostSynaptic => audit.oracle_post_deltas,
            UpperBoundArm::DirectPerSynapse => audit.direct_edge_deltas,
            UpperBoundArm::DistilledStudent => unreachable!("student has separate path"),
        };
        model.apply_deltas(&deltas);
    }
    c3_accuracy(&model, test)
}

fn train_c3_student(
    mut model: C3CompositionModel,
    train: &[C3Example],
    test: &[C3Example],
    artifact: CreditHeadArtifact,
) -> f32 {
    let signature = model.signature();
    let learner = HybridLearner::load(artifact, signature).expect("C3 topology");
    for example in train {
        let trace = model.forward(example);
        let reward = if trace.prediction == example.label {
            1.0
        } else {
            -1.0
        };
        for (edge, features) in model
            .credit_features(&trace, Some(reward))
            .into_iter()
            .enumerate()
        {
            learner
                .apply_edge(&mut model.engine, edge, features)
                .expect("student C3 update");
        }
    }
    let before_test = fingerprint_engine_weights(&model.engine);
    let result = c3_accuracy(&model, test);
    assert_eq!(
        before_test,
        fingerprint_engine_weights(&model.engine),
        "C3 test evaluation changed weights"
    );
    result
}

fn train_student(
    mut model: SparseTerminalModel,
    train: &[Example],
    test: &[Example],
    artifact: CreditHeadArtifact,
) -> f32 {
    let signature = model.signature();
    let learner = HybridLearner::load(artifact, signature).expect("matching topology");
    for example in train {
        let trace = model.forward(&example.input);
        let reward = if trace.prediction == example.label {
            1.0
        } else {
            -1.0
        };
        let features = model.credit_features(&trace, Some(reward));
        for (edge, edge_features) in features.into_iter().enumerate() {
            learner
                .apply_edge(&mut model.engine, edge, edge_features)
                .expect("finite student update");
        }
    }
    let before_test = fingerprint_weights(&model);
    let result = accuracy(&model, test);
    assert_eq!(
        before_test,
        fingerprint_weights(&model),
        "test evaluation changed weights"
    );
    result
}

fn accuracy(model: &SparseTerminalModel, examples: &[Example]) -> f32 {
    examples
        .iter()
        .filter(|example| model.forward(&example.input).prediction == example.label)
        .count() as f32
        / examples.len().max(1) as f32
}

pub(crate) fn c3_accuracy(model: &C3CompositionModel, examples: &[C3Example]) -> f32 {
    examples
        .iter()
        .filter(|example| model.forward(example).prediction == example.label)
        .count() as f32
        / examples.len().max(1) as f32
}

fn merge_audit(mut summary: BenchmarkSummary, audit: &AuditAccumulator) -> BenchmarkSummary {
    summary.mean_existing_cosine = mean(&audit.existing_cosine);
    summary.mean_oracle_cosine = mean(&audit.oracle_cosine);
    summary.mean_existing_sign_agreement = mean(&audit.existing_sign);
    summary.mean_oracle_sign_agreement = mean(&audit.oracle_sign);
    summary
}

fn edge_posts(model: &SparseTerminalModel) -> Vec<usize> {
    let mut posts = Vec::with_capacity(model.engine.edge_w.len());
    for pre in 0..model.n_cells() {
        for edge in edge_range(&model.engine.conn, pre) {
            posts.push(model.engine.conn.col[edge] as usize);
        }
    }
    posts
}

fn summarize(arm: UpperBoundArm, values: &[f32], z: f32) -> ArmSummary {
    let mean_accuracy = mean(values);
    let variance = variance(values, mean_accuracy);
    let lower_95 = if values.len() > 1 {
        mean_accuracy - z * (variance / values.len() as f32).sqrt()
    } else {
        mean_accuracy
    };
    ArmSummary {
        arm,
        mean_accuracy,
        variance,
        lower_95,
    }
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn variance(values: &[f32], mean: f32) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f32>()
        / (values.len() - 1) as f32
}

fn d_star(
    benchmarks: &[BenchmarkSummary],
    arm: UpperBoundArm,
    accuracy_floor: f32,
) -> Option<usize> {
    benchmarks
        .iter()
        .filter_map(|benchmark| benchmark.depth.map(|depth| (depth, benchmark)))
        .filter(|(_, benchmark)| benchmark.arm(arm).lower_95 >= accuracy_floor)
        .map(|(depth, _)| depth)
        .max()
}

fn c1_examples(seed: u64, n: usize) -> Vec<Example> {
    let mut rng = LocalRng::new(seed);
    (0..n)
        .map(|_| {
            let input = (0..C1_INPUTS)
                .map(|_| if rng.next_f32() >= 0.5 { 1.0 } else { -1.0 })
                .collect::<Vec<_>>();
            let temporal =
                0.9 * input[1] - 0.7 * input[5] + 0.4 * input[2] - 0.3 * input[6] + 0.2 * input[0];
            let label = usize::from(temporal > 0.0);
            Example { input, label }
        })
        .collect()
}

pub(crate) fn c3_examples(seed: u64, depth: usize, n: usize) -> Vec<C3Example> {
    let mut rng = LocalRng::new(seed ^ (depth as u64).wrapping_mul(0x9e37_79b9));
    (0..n)
        .map(|_| {
            let mut state = rng.index(C3_STATES);
            let initial_state = state;
            let mut operations = Vec::with_capacity(depth);
            for _ in 0..depth {
                let operation = rng.index(2);
                operations.push(operation);
                state = if operation == 0 {
                    (state + 1) % C3_STATES
                } else {
                    [1, 3, 0, 2][state]
                };
            }
            C3Example {
                initial_state,
                operations,
                label: state,
            }
        })
        .collect()
}

pub(crate) fn seeds(master: u64, n: usize) -> Vec<u64> {
    (0..n)
        .map(|index| master ^ (index as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .collect()
}

fn seed_hash(seeds: &[u64]) -> u64 {
    let mut bytes = Vec::with_capacity(seeds.len() * 8);
    for seed in seeds {
        bytes.extend_from_slice(&seed.to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn fingerprint_weights(model: &SparseTerminalModel) -> u64 {
    fingerprint_engine_weights(&model.engine)
}

fn fingerprint_engine_weights(engine: &Engine) -> u64 {
    let mut bytes = Vec::with_capacity(engine.edge_w.len() * 4);
    for weight in &engine.edge_w {
        bytes.extend_from_slice(&weight.to_bits().to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn optional_depth(value: Option<usize>) -> String {
    value
        .map(|depth| depth.to_string())
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_does_not_update_weights() {
        let model = SparseTerminalModel::new(vec![C1_INPUTS, C1_HIDDEN, 2], 31);
        let test = c1_examples(37, 16);
        let before = fingerprint_weights(&model);
        let _ = accuracy(&model, &test);
        assert_eq!(before, fingerprint_weights(&model));
    }

    #[test]
    fn c3_generation_is_deterministic_and_terminal_only() {
        let a = c3_examples(7, 4, 20);
        let b = c3_examples(7, 4, 20);
        assert_eq!(a, b);
    }

    #[test]
    fn c3_terminal_teacher_matches_finite_difference() {
        let model = C3CompositionModel::new(17);
        let example = c3_examples(23, 3, 1).remove(0);
        let trace = model.forward(&example);
        let analytic = model.teacher_targets(&trace, example.label, 1.0, None);
        let epsilon = 1e-3f32;
        for edge in 0..model.engine.edge_w.len() {
            let mut plus = C3CompositionModel::new(17);
            plus.engine.edge_w = model.engine.edge_w.clone();
            plus.engine.edge_w[edge] += epsilon;
            plus.engine.syn.rebuild_from_weights(&plus.engine.edge_w, 1);
            let plus_trace = plus.forward(&example);
            let plus_loss = plus
                .teacher_targets(&plus_trace, example.label, 1.0, None)
                .loss;

            let mut minus = C3CompositionModel::new(17);
            minus.engine.edge_w = model.engine.edge_w.clone();
            minus.engine.edge_w[edge] -= epsilon;
            minus
                .engine
                .syn
                .rebuild_from_weights(&minus.engine.edge_w, 1);
            let minus_trace = minus.forward(&example);
            let minus_loss = minus
                .teacher_targets(&minus_trace, example.label, 1.0, None)
                .loss;
            let descent = -(plus_loss - minus_loss) / (2.0 * epsilon);
            assert!(
                (analytic.edge_deltas[edge] - descent).abs() < 3e-3,
                "edge {edge}: analytic={} numeric={descent}",
                analytic.edge_deltas[edge]
            );
        }
    }

    #[test]
    fn terminal_labels_cannot_change_forward_traces() {
        let model = C3CompositionModel::new(41);
        let mut original = c3_examples(43, 5, 1).remove(0);
        let first = model.forward(&original);
        original.label = (original.label + 1) % C3_STATES;
        let second = model.forward(&original);
        assert_eq!(first.states, second.states);
        assert_eq!(first.operations, second.operations);
        assert_eq!(first.prediction, second.prediction);
    }

    #[test]
    fn feasibility_replay_is_deterministic() {
        let protocol = HybridProtocol {
            development_seeds: 1,
            held_out_seeds: 1,
            train_examples: 12,
            test_examples: 8,
            ..HybridProtocol::quick()
        };
        let first = run_feasibility(&protocol);
        let second = run_feasibility(&protocol);
        assert_eq!(first.protocol_hash, second.protocol_hash);
        assert_eq!(first.decision, second.decision);
        assert_eq!(first.benchmarks, second.benchmarks);
        assert_eq!(first.teacher_d_star, second.teacher_d_star);
        assert!(!first.scientific_claim_allowed);
    }

    #[test]
    fn no_go_stops_before_student_training_and_artifact_emission() {
        let protocol = HybridProtocol::quick();
        let report = run_feasibility(&protocol);
        assert_eq!(report.decision, H0Decision::HybridNoGo);
        assert!(report.artifacts.is_empty());
        assert!(report.student_d_star.is_none());
        assert!(report.benchmarks.iter().all(|benchmark| {
            benchmark
                .arms
                .iter()
                .all(|arm| arm.arm != UpperBoundArm::DistilledStudent)
        }));
    }
}
