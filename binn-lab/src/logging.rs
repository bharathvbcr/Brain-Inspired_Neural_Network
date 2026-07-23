//! Structured run logging (U13).
//!
//! **GC7:** the harness refuses to emit results without an `activity-sparsity`
//! / `activity_sparsity` field.

use std::fmt::Write as _;

/// Why [`StructuredLogger::emit_results`] refused to write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitError {
    /// GC7: results emission requires a recorded activity-sparsity value.
    MissingActivitySparsity,
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitError::MissingActivitySparsity => {
                write!(
                    f,
                    "GC7: refuse emit_results without activity_sparsity / activity-sparsity"
                )
            }
        }
    }
}

impl std::error::Error for EmitError {}

/// One structured run record prior to emission.
#[derive(Clone, Debug, PartialEq)]
pub struct RunLog {
    /// Config hash string (`c1-…`).
    pub config_hash: String,
    /// Seed for this replicate.
    pub seed: u64,
    /// Condition label (`local-assembly`, `dense-local`, `gradient-reference`).
    pub condition: String,
    /// Held-out accuracy in `[0, 1]`.
    pub accuracy: f32,
    /// GC7 field. Must be `Some` before [`StructuredLogger::emit_results`].
    ///
    /// Serialized under both `activity_sparsity` and `activity-sparsity`.
    pub activity_sparsity: Option<f32>,
    /// Optional work-per-accuracy (honest F5 metric).
    pub work_per_accuracy: Option<f64>,
    /// Free-form note.
    pub note: String,
}

impl RunLog {
    /// Builder with no sparsity yet (emit will fail until set).
    pub fn new(config_hash: impl Into<String>, seed: u64, condition: impl Into<String>) -> Self {
        Self {
            config_hash: config_hash.into(),
            seed,
            condition: condition.into(),
            accuracy: 0.0,
            activity_sparsity: None,
            work_per_accuracy: None,
            note: String::new(),
        }
    }

    /// Attach GC7 activity-sparsity (`active / N`).
    #[inline]
    pub fn with_activity_sparsity(mut self, sparsity: f32) -> Self {
        self.activity_sparsity = Some(sparsity);
        self
    }
}

/// Structured logger that gates emission on GC7.
#[derive(Clone, Debug, Default)]
pub struct StructuredLogger {
    lines: Vec<String>,
}

impl StructuredLogger {
    /// Empty logger.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Emitted lines so far (empty if every emit was refused).
    #[inline]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Emit one results line. **Refuses** when `activity_sparsity` is missing (GC7).
    pub fn emit_results(&mut self, log: &RunLog) -> Result<String, EmitError> {
        let sparsity = log
            .activity_sparsity
            .ok_or(EmitError::MissingActivitySparsity)?;
        let mut line = String::new();
        let _ = write!(
            &mut line,
            "config_hash={} seed={} condition={} accuracy={:.6} activity_sparsity={:.6} activity-sparsity={:.6}",
            log.config_hash, log.seed, log.condition, log.accuracy, sparsity, sparsity
        );
        if let Some(wpa) = log.work_per_accuracy {
            let _ = write!(&mut line, " work_per_accuracy={wpa:.6}");
        }
        if !log.note.is_empty() {
            let _ = write!(&mut line, " note={}", log.note.replace(' ', "_"));
        }
        self.lines.push(line.clone());
        Ok(line)
    }

    /// Join emitted lines with newlines.
    pub fn render(&self) -> String {
        self.lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gc7_refuses_emit_without_activity_sparsity() {
        let mut log = StructuredLogger::new();
        let entry = RunLog::new("c1-deadbeef", 1, "local-assembly");
        let err = log.emit_results(&entry).unwrap_err();
        assert_eq!(err, EmitError::MissingActivitySparsity);
        assert!(log.lines().is_empty());
    }

    #[test]
    fn gc7_emits_when_activity_sparsity_present() {
        let mut log = StructuredLogger::new();
        let entry = RunLog::new("c1-deadbeef", 1, "local-assembly").with_activity_sparsity(0.02);
        let mut with_acc = entry;
        with_acc.accuracy = 0.75;
        let line = log.emit_results(&with_acc).unwrap();
        assert!(line.contains("activity_sparsity=0.020000"));
        assert!(line.contains("activity-sparsity=0.020000"));
        assert_eq!(log.lines().len(), 1);
    }
}
