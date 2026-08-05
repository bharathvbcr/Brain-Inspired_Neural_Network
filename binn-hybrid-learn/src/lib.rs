//! Teacher-free learned local updates for BINN-Hybrid.
//!
//! This crate deliberately has no dependency on `binn-hybrid-lab`. Teacher
//! construction and distillation belong to the training-only lab crate.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use binn_engine::Engine;

pub const FEATURE_SCHEMA_VERSION: u32 = 1;
pub const CREDIT_FEATURE_COUNT: usize = 12;
const ARTIFACT_FORMAT: &str = "binn-hybrid-credit-head-v1";

/// Credit resolution selected by the H0 feasibility gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditGranularity {
    PostSynaptic,
    PerSynapse,
}

impl CreditGranularity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PostSynaptic => "post-synaptic",
            Self::PerSynapse => "per-synapse",
        }
    }
}

impl FromStr for CreditGranularity {
    type Err = ArtifactError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "post-synaptic" => Ok(Self::PostSynaptic),
            "per-synapse" => Ok(Self::PerSynapse),
            _ => Err(ArtifactError::InvalidField("granularity")),
        }
    }
}

/// Causal information available at a synapse when an online update is made.
///
/// Labels, teacher deltas, future activity, and intermediate task targets are
/// intentionally absent.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CreditFeatures {
    pub pre_trace: f32,
    pub post_trace: f32,
    pub eligibility: f32,
    pub weight: f32,
    pub pre_membrane: f32,
    pub post_membrane: f32,
    pub pre_threshold: f32,
    pub post_threshold: f32,
    pub pre_activity: f32,
    pub post_activity: f32,
    pub structural_id: u32,
    pub broadcast_reward: Option<f32>,
}

impl CreditFeatures {
    pub fn values(self) -> [f32; CREDIT_FEATURE_COUNT] {
        [
            self.pre_trace,
            self.post_trace,
            self.eligibility,
            self.weight,
            self.pre_membrane,
            self.post_membrane,
            self.pre_threshold,
            self.post_threshold,
            self.pre_activity,
            self.post_activity,
            normalize_structural_id(self.structural_id),
            self.broadcast_reward.unwrap_or(0.0),
        ]
    }

    pub fn all_finite(self) -> bool {
        self.values().into_iter().all(f32::is_finite)
    }
}

fn normalize_structural_id(value: u32) -> f32 {
    let folded = (value ^ (value >> 16)) & 0xffff;
    folded as f32 / u16::MAX as f32 * 2.0 - 1.0
}

/// Frozen, checksummed student parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditHeadArtifact {
    pub feature_schema_version: u32,
    pub topology_signature: u64,
    pub granularity: CreditGranularity,
    pub teacher_protocol_hash: u64,
    pub training_seed_hash: u64,
    pub coefficients: [f32; CREDIT_FEATURE_COUNT],
    pub bias: f32,
    pub output_scale: f32,
    pub checksum: u64,
}

impl CreditHeadArtifact {
    pub fn new(
        topology_signature: u64,
        granularity: CreditGranularity,
        teacher_protocol_hash: u64,
        training_seed_hash: u64,
        coefficients: [f32; CREDIT_FEATURE_COUNT],
        bias: f32,
        output_scale: f32,
    ) -> Result<Self, ArtifactError> {
        if !coefficients.into_iter().all(f32::is_finite)
            || !bias.is_finite()
            || !output_scale.is_finite()
            || output_scale <= 0.0
        {
            return Err(ArtifactError::NonFinite);
        }
        let mut artifact = Self {
            feature_schema_version: FEATURE_SCHEMA_VERSION,
            topology_signature,
            granularity,
            teacher_protocol_hash,
            training_seed_hash,
            coefficients,
            bias,
            output_scale,
            checksum: 0,
        };
        artifact.checksum = artifact.computed_checksum();
        Ok(artifact)
    }

    pub fn verify(&self) -> Result<(), ArtifactError> {
        if self.feature_schema_version != FEATURE_SCHEMA_VERSION {
            return Err(ArtifactError::SchemaMismatch {
                expected: FEATURE_SCHEMA_VERSION,
                actual: self.feature_schema_version,
            });
        }
        if !self.coefficients.into_iter().all(f32::is_finite)
            || !self.bias.is_finite()
            || !self.output_scale.is_finite()
            || self.output_scale <= 0.0
        {
            return Err(ArtifactError::NonFinite);
        }
        let actual = self.computed_checksum();
        if actual != self.checksum {
            return Err(ArtifactError::ChecksumMismatch {
                expected: self.checksum,
                actual,
            });
        }
        Ok(())
    }

    pub fn computed_checksum(&self) -> u64 {
        fnv1a64(self.canonical_body().as_bytes())
    }

    fn canonical_body(&self) -> String {
        let coefficients = self
            .coefficients
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "format={ARTIFACT_FORMAT}\n\
             feature_schema_version={}\n\
             topology_signature={:016x}\n\
             granularity={}\n\
             teacher_protocol_hash={:016x}\n\
             training_seed_hash={:016x}\n\
             coefficients={coefficients}\n\
             bias={:08x}\n\
             output_scale={:08x}\n",
            self.feature_schema_version,
            self.topology_signature,
            self.granularity.as_str(),
            self.teacher_protocol_hash,
            self.training_seed_hash,
            self.bias.to_bits(),
            self.output_scale.to_bits(),
        )
    }

    pub fn encode(&self) -> Result<String, ArtifactError> {
        self.verify()?;
        Ok(format!(
            "{}checksum={:016x}\n",
            self.canonical_body(),
            self.checksum
        ))
    }

    pub fn decode(text: &str) -> Result<Self, ArtifactError> {
        let mut fields = std::collections::BTreeMap::new();
        for line in text.lines() {
            let (key, value) = line.split_once('=').ok_or(ArtifactError::MalformedLine)?;
            if fields.insert(key, value).is_some() {
                return Err(ArtifactError::DuplicateField);
            }
        }
        if fields.get("format").copied() != Some(ARTIFACT_FORMAT) {
            return Err(ArtifactError::InvalidField("format"));
        }
        for key in fields.keys().copied() {
            if !matches!(
                key,
                "format"
                    | "feature_schema_version"
                    | "topology_signature"
                    | "granularity"
                    | "teacher_protocol_hash"
                    | "training_seed_hash"
                    | "coefficients"
                    | "bias"
                    | "output_scale"
                    | "checksum"
            ) {
                return Err(ArtifactError::UnknownField);
            }
        }
        let feature_schema_version = parse_u32(&fields, "feature_schema_version")?;
        let topology_signature = parse_hex_u64(&fields, "topology_signature")?;
        let granularity = required(&fields, "granularity")?.parse()?;
        let teacher_protocol_hash = parse_hex_u64(&fields, "teacher_protocol_hash")?;
        let training_seed_hash = parse_hex_u64(&fields, "training_seed_hash")?;
        let raw_coefficients = required(&fields, "coefficients")?;
        let coefficient_words = raw_coefficients.split(',').collect::<Vec<_>>();
        if coefficient_words.len() != CREDIT_FEATURE_COUNT {
            return Err(ArtifactError::InvalidField("coefficients"));
        }
        let mut coefficients = [0.0; CREDIT_FEATURE_COUNT];
        for (slot, word) in coefficients.iter_mut().zip(coefficient_words) {
            *slot = f32::from_bits(parse_hex_u32_value(word, "coefficients")?);
        }
        let bias = f32::from_bits(parse_hex_u32_value(required(&fields, "bias")?, "bias")?);
        let output_scale = f32::from_bits(parse_hex_u32_value(
            required(&fields, "output_scale")?,
            "output_scale",
        )?);
        let checksum = parse_hex_u64(&fields, "checksum")?;
        let artifact = Self {
            feature_schema_version,
            topology_signature,
            granularity,
            teacher_protocol_hash,
            training_seed_hash,
            coefficients,
            bias,
            output_scale,
            checksum,
        };
        artifact.verify()?;
        Ok(artifact)
    }
}

impl Display for CreditHeadArtifact {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.encode() {
            Ok(encoded) => formatter.write_str(&encoded),
            Err(_) => formatter.write_str("<invalid BINN-Hybrid artifact>"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtifactError {
    MalformedLine,
    DuplicateField,
    UnknownField,
    MissingField(&'static str),
    InvalidField(&'static str),
    NonFinite,
    SchemaMismatch { expected: u32, actual: u32 },
    TopologyMismatch { expected: u64, actual: u64 },
    ChecksumMismatch { expected: u64, actual: u64 },
    EdgeOutOfBounds { edge: usize, len: usize },
}

impl Display for ArtifactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ArtifactError {}

/// Runtime-only learner. It cannot construct or call a teacher.
#[derive(Clone, Debug)]
pub struct HybridLearner {
    artifact: CreditHeadArtifact,
}

impl HybridLearner {
    pub fn load(
        artifact: CreditHeadArtifact,
        topology_signature: u64,
    ) -> Result<Self, ArtifactError> {
        artifact.verify()?;
        if artifact.topology_signature != topology_signature {
            return Err(ArtifactError::TopologyMismatch {
                expected: artifact.topology_signature,
                actual: topology_signature,
            });
        }
        Ok(Self { artifact })
    }

    pub fn artifact(&self) -> &CreditHeadArtifact {
        &self.artifact
    }

    pub fn predict_delta(&self, features: CreditFeatures) -> Result<f32, ArtifactError> {
        if !features.all_finite() {
            return Err(ArtifactError::NonFinite);
        }
        let activation = self
            .artifact
            .coefficients
            .into_iter()
            .zip(features.values())
            .fold(self.artifact.bias, |sum, (coefficient, value)| {
                sum + coefficient * value
            });
        Ok(activation.tanh() * self.artifact.output_scale)
    }

    pub fn apply_edge(
        &self,
        engine: &mut Engine,
        edge: usize,
        features: CreditFeatures,
    ) -> Result<f32, ArtifactError> {
        if edge >= engine.edge_w.len() {
            return Err(ArtifactError::EdgeOutOfBounds {
                edge,
                len: engine.edge_w.len(),
            });
        }
        let delta = self.predict_delta(features)?;
        let updated = (engine.edge_w[edge] + delta).clamp(-8.0, 8.0);
        engine.edge_w[edge] = updated;
        engine.syn.as_mut_slice()[edge].weight = updated;
        Ok(delta)
    }

    /// Reward-only online adaptation of the frozen head.
    ///
    /// No label or teacher target is accepted by this interface.
    pub fn adapt_from_reward(
        &mut self,
        features: CreditFeatures,
        reward: f32,
        eta: f32,
    ) -> Result<(), ArtifactError> {
        if !reward.is_finite() || !eta.is_finite() || eta < 0.0 || !features.all_finite() {
            return Err(ArtifactError::NonFinite);
        }
        let prediction = self.predict_delta(features)?;
        let target = reward * features.eligibility;
        let error = (target - prediction).clamp(-1.0, 1.0);
        for (coefficient, value) in self.artifact.coefficients.iter_mut().zip(features.values()) {
            *coefficient += eta * error * value;
        }
        self.artifact.bias += eta * error;
        self.artifact.checksum = self.artifact.computed_checksum();
        Ok(())
    }
}

pub fn topology_signature(engine: &Engine) -> u64 {
    let mut bytes = Vec::with_capacity((engine.conn.row_ptr.len() + engine.conn.col.len()) * 8);
    for &value in &engine.conn.row_ptr {
        bytes.extend_from_slice(&(value as u64).to_le_bytes());
    }
    for &value in &engine.conn.col {
        bytes.extend_from_slice(&u64::from(value).to_le_bytes());
    }
    fnv1a64(&bytes)
}

pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn required<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    field: &'static str,
) -> Result<&'a str, ArtifactError> {
    fields
        .get(field)
        .copied()
        .ok_or(ArtifactError::MissingField(field))
}

fn parse_u32(
    fields: &std::collections::BTreeMap<&str, &str>,
    field: &'static str,
) -> Result<u32, ArtifactError> {
    required(fields, field)?
        .parse()
        .map_err(|_| ArtifactError::InvalidField(field))
}

fn parse_hex_u64(
    fields: &std::collections::BTreeMap<&str, &str>,
    field: &'static str,
) -> Result<u64, ArtifactError> {
    u64::from_str_radix(required(fields, field)?, 16)
        .map_err(|_| ArtifactError::InvalidField(field))
}

fn parse_hex_u32_value(value: &str, field: &'static str) -> Result<u32, ArtifactError> {
    u32::from_str_radix(value, 16).map_err(|_| ArtifactError::InvalidField(field))
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_engine::Engine;

    fn artifact() -> CreditHeadArtifact {
        CreditHeadArtifact::new(
            7,
            CreditGranularity::PerSynapse,
            11,
            13,
            [0.1; CREDIT_FEATURE_COUNT],
            -0.25,
            0.5,
        )
        .expect("artifact")
    }

    #[test]
    fn artifact_round_trip_is_exact() {
        let artifact = artifact();
        let encoded = artifact.encode().expect("encode");
        assert_eq!(CreditHeadArtifact::decode(&encoded), Ok(artifact));
    }

    #[test]
    fn artifact_rejects_tampering() {
        let encoded = artifact().encode().expect("encode");
        let tampered = encoded.replace("bias=be800000", "bias=00000000");
        assert!(matches!(
            CreditHeadArtifact::decode(&tampered),
            Err(ArtifactError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn artifact_rejects_unknown_and_duplicate_fields() {
        let encoded = artifact().encode().expect("encode");
        let unknown = format!("{encoded}future_field=1\n");
        assert_eq!(
            CreditHeadArtifact::decode(&unknown),
            Err(ArtifactError::UnknownField)
        );
        let duplicate = format!("{encoded}bias=00000000\n");
        assert_eq!(
            CreditHeadArtifact::decode(&duplicate),
            Err(ArtifactError::DuplicateField)
        );
    }

    #[test]
    fn topology_mismatch_is_rejected() {
        assert!(matches!(
            HybridLearner::load(artifact(), 8),
            Err(ArtifactError::TopologyMismatch { .. })
        ));
    }

    #[test]
    fn runtime_update_changes_engine_and_synapse_together() {
        let mut engine = Engine::with_cells(2);
        engine.set_connectivity(binn_core_for_test::one_edge(), vec![0.25]);
        let signature = topology_signature(&engine);
        let artifact = CreditHeadArtifact::new(
            signature,
            CreditGranularity::PerSynapse,
            1,
            2,
            [0.2; CREDIT_FEATURE_COUNT],
            0.0,
            0.1,
        )
        .expect("artifact");
        let learner = HybridLearner::load(artifact, signature).expect("load");
        let delta = learner
            .apply_edge(
                &mut engine,
                0,
                CreditFeatures {
                    eligibility: 1.0,
                    broadcast_reward: Some(1.0),
                    ..CreditFeatures::default()
                },
            )
            .expect("apply");
        assert!(delta > 0.0);
        assert_eq!(engine.edge_w[0], engine.syn.as_slice()[0].weight);
    }

    #[test]
    fn topology_signature_ignores_weights_but_rejects_edge_changes() {
        let mut a = Engine::with_cells(3);
        a.set_connectivity(
            binn_core::Csr::from_adjacency(&[vec![1], vec![2], vec![]]),
            vec![0.1, 0.2],
        );
        let mut same_edges = Engine::with_cells(3);
        same_edges.set_connectivity(
            binn_core::Csr::from_adjacency(&[vec![1], vec![2], vec![]]),
            vec![-7.0, 3.5],
        );
        let mut changed_edges = Engine::with_cells(3);
        changed_edges.set_connectivity(
            binn_core::Csr::from_adjacency(&[vec![2], vec![2], vec![]]),
            vec![0.1, 0.2],
        );
        assert_eq!(topology_signature(&a), topology_signature(&same_edges));
        assert_ne!(topology_signature(&a), topology_signature(&changed_edges));
    }

    mod binn_core_for_test {
        pub fn one_edge() -> binn_core::Csr {
            binn_core::Csr::from_adjacency(&[vec![1], vec![]])
        }
    }
}
