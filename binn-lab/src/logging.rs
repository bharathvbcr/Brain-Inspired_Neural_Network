//! Structured run logging (U13) and offline JSONL trace export.
//!
//! **GC7:** the harness refuses to emit results without an `activity-sparsity`
//! / `activity_sparsity` field.
//!
//! Trace export is stdlib-only JSON (no `serde`); one object per line.

use std::fmt::Write as _;
use std::path::Path;

/// Write a finished report to `path`, creating its parent directory.
///
/// The canonical report sink for the experiment binaries. Both the `mkdir -p`
/// and the write surface their `io::Error` as the `String` the binaries'
/// `main` already returns, so a report that could not be written aborts the
/// run instead of leaving a run whose result exists only in stdout.
pub fn write_report(path: &Path, report: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, report).map_err(|error| error.to_string())
}

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
    /// Optional F5: modeled event work from disjoint counters.
    pub event_work: Option<f64>,
    /// Optional F5: naive `n_cells × activity` proxy.
    pub naive_activity_work: Option<f64>,
    /// Optional F5: `event_work / naive_activity_work` (≫1 ⇒ activity understates compute).
    pub work_vs_activity_ratio: Option<f64>,
    /// Optional F5: source spike count.
    pub source_spikes: Option<u64>,
    /// Optional F5: synaptic delivery count.
    pub synaptic_deliveries: Option<u64>,
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
            event_work: None,
            naive_activity_work: None,
            work_vs_activity_ratio: None,
            source_spikes: None,
            synaptic_deliveries: None,
            note: String::new(),
        }
    }

    /// Attach F5 activity≠compute accounting fields (optional; emit still requires sparsity).
    pub fn with_f5_account(
        mut self,
        event_work: f64,
        naive_activity_work: f64,
        work_vs_activity_ratio: f64,
        source_spikes: u64,
        synaptic_deliveries: u64,
    ) -> Self {
        self.event_work = Some(event_work);
        self.naive_activity_work = Some(naive_activity_work);
        self.work_vs_activity_ratio = Some(work_vs_activity_ratio);
        self.source_spikes = Some(source_spikes);
        self.synaptic_deliveries = Some(synaptic_deliveries);
        self
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
        if let Some(ew) = log.event_work {
            let _ = write!(&mut line, " event_work={ew:.6}");
        }
        if let Some(nw) = log.naive_activity_work {
            let _ = write!(&mut line, " naive_activity_work={nw:.6}");
        }
        if let Some(ratio) = log.work_vs_activity_ratio {
            let _ = write!(&mut line, " work_vs_activity_ratio={ratio:.6}");
        }
        if let Some(spikes) = log.source_spikes {
            let _ = write!(&mut line, " source_spikes={spikes}");
        }
        if let Some(deliv) = log.synaptic_deliveries {
            let _ = write!(&mut line, " synaptic_deliveries={deliv}");
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

// ---------------------------------------------------------------------------
// Offline JSONL trace export (stdlib-only; no serde)
// ---------------------------------------------------------------------------

/// Escape a string for embedding in a JSON string value.
pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(&mut out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn write_f32(buf: &mut String, v: f32) {
    if v.is_finite() {
        let _ = write!(buf, "{v}");
    } else if v.is_nan() {
        buf.push_str("null");
    } else if v.is_sign_negative() {
        buf.push_str("-1e999");
    } else {
        buf.push_str("1e999");
    }
}

fn write_f64(buf: &mut String, v: f64) {
    if v.is_finite() {
        let _ = write!(buf, "{v}");
    } else if v.is_nan() {
        buf.push_str("null");
    } else if v.is_sign_negative() {
        buf.push_str("-1e999");
    } else {
        buf.push_str("1e999");
    }
}

fn write_u32_array(buf: &mut String, xs: &[u32]) {
    buf.push('[');
    for (i, x) in xs.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{x}");
    }
    buf.push(']');
}

/// Area range for topology / flow records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceArea {
    pub id: u32,
    pub name: String,
    pub start: u32,
    pub end: u32,
}

/// Projection edge-count descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceProjection {
    pub src: u32,
    pub dst: u32,
    pub nnz: u64,
    /// Optional per-edge or aggregate coupling scores (R1 `flow_static`).
    pub coupling: Option<Vec<f32>>,
}

/// Synapse edge for weight frames.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceWeightEdge {
    pub pre: u32,
    pub post: u32,
    pub w: f32,
}

/// Synapse edge for eligibility snapshots (nonzero `|e|` / `|dw|` only).
#[derive(Clone, Debug, PartialEq)]
pub struct TraceEligEdge {
    pub pre: u32,
    pub post: u32,
    pub w: f32,
    pub e: f32,
    pub dw: f32,
}

/// Competitor score pair `(cell, v)` for k-WTA records.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceScore {
    pub cell: u32,
    pub v: f32,
}

/// Append-only JSONL trace recorder for offline viewers.
#[derive(Clone, Debug, Default)]
pub struct TraceRecorder {
    lines: Vec<String>,
}

impl TraceRecorder {
    /// Empty recorder.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Recorded JSONL lines so far.
    #[inline]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Append a raw JSON object line (must already be a single object).
    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// Write all lines to `path` (newline-terminated JSONL).
    pub fn write_jsonl(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(path.as_ref())?;
        for line in &self.lines {
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
        }
        Ok(())
    }

    /// `meta`: run identity for the viewer header.
    #[allow(clippy::too_many_arguments)]
    pub fn emit_meta(
        &mut self,
        config_hash: &str,
        seed: u64,
        condition: &str,
        experiment: &str,
        n_classes: u32,
        k_wta: u32,
        n_hidden: u32,
    ) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"meta","config_hash":"{}","seed":{},"condition":"{}","experiment":"{}","n_classes":{},"k_wta":{},"n_hidden":{}}}"#,
            json_escape(config_hash),
            seed,
            json_escape(condition),
            json_escape(experiment),
            n_classes,
            k_wta,
            n_hidden
        );
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `topology`: area ranges + projection nnz.
    pub fn emit_topology(&mut self, areas: &[TraceArea], projections: &[TraceProjection]) -> &str {
        let mut line = String::from(r#"{"type":"topology","areas":["#);
        for (i, a) in areas.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            let _ = write!(
                &mut line,
                r#"{{"id":{},"name":"{}","start":{},"end":{}}}"#,
                a.id,
                json_escape(&a.name),
                a.start,
                a.end
            );
        }
        line.push_str(r#"],"projections":["#);
        for (i, p) in projections.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            let _ = write!(
                &mut line,
                r#"{{"src":{},"dst":{},"nnz":{}}}"#,
                p.src, p.dst, p.nnz
            );
        }
        line.push_str("]}");
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `flow_static`: R1 static flow (nnz + optional coupling; no spikes).
    pub fn emit_flow_static(&mut self, projections: &[TraceProjection]) -> &str {
        let mut line = String::from(r#"{"type":"flow_static","projections":["#);
        for (i, p) in projections.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            let _ = write!(
                &mut line,
                r#"{{"src":{},"dst":{},"nnz":{}"#,
                p.src, p.dst, p.nnz
            );
            if let Some(ref coupling) = p.coupling {
                line.push_str(r#","coupling":["#);
                for (j, c) in coupling.iter().enumerate() {
                    if j > 0 {
                        line.push(',');
                    }
                    write_f32(&mut line, *c);
                }
                line.push(']');
            }
            line.push('}');
        }
        line.push_str("]}");
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `stimulus`: trial window + label + phase.
    pub fn emit_stimulus(&mut self, trial: u32, label: u32, t0: u64, t1: u64, phase: &str) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"stimulus","trial":{},"label":{},"t0":{},"t1":{},"phase":"{}"}}"#,
            trial,
            label,
            t0,
            t1,
            json_escape(phase)
        );
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `spike`: one spike event.
    pub fn emit_spike(&mut self, t: u64, cell: u32, trial: u32) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"spike","t":{},"cell":{},"trial":{}}}"#,
            t, cell, trial
        );
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `kwta`: winners + competitor scores at decision time.
    pub fn emit_kwta(
        &mut self,
        trial: u32,
        area: &str,
        t: u64,
        winners: &[u32],
        scores: &[TraceScore],
    ) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"kwta","trial":{},"area":"{}","t":{},"winners":"#,
            trial,
            json_escape(area),
            t
        );
        write_u32_array(&mut line, winners);
        line.push_str(r#","scores":["#);
        for (i, s) in scores.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            line.push('[');
            let _ = write!(&mut line, "{},", s.cell);
            write_f32(&mut line, s.v);
            line.push(']');
        }
        line.push_str("]}");
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `assembly_class`: per-label member cells (+ optional hit counts).
    pub fn emit_assembly_class(
        &mut self,
        label: u32,
        members: &[u32],
        hits: Option<&[u32]>,
    ) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"assembly_class","label":{},"members":"#,
            label
        );
        write_u32_array(&mut line, members);
        if let Some(h) = hits {
            line.push_str(r#","hits":"#);
            write_u32_array(&mut line, h);
        }
        line.push('}');
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `weight_frame`: before/after weight snapshot for a focused subgraph.
    pub fn emit_weight_frame(&mut self, step: u32, kind: &str, edges: &[TraceWeightEdge]) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"weight_frame","step":{},"kind":"{}","edges":["#,
            step,
            json_escape(kind)
        );
        for (i, e) in edges.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            let _ = write!(&mut line, r#"{{"pre":{},"post":{},"w":"#, e.pre, e.post);
            write_f32(&mut line, e.w);
            line.push('}');
        }
        line.push_str("]}");
        self.lines.push(line);
        self.lines.last().unwrap()
    }

    /// `elig_event`: plasticity-time eligibility / Δw snapshot.
    pub fn emit_elig_event(
        &mut self,
        trial: u32,
        step: u32,
        reward: f64,
        edges: &[TraceEligEdge],
    ) -> &str {
        let mut line = String::new();
        let _ = write!(
            &mut line,
            r#"{{"type":"elig_event","trial":{},"step":{},"reward":"#,
            trial, step
        );
        write_f64(&mut line, reward);
        line.push_str(r#","edges":["#);
        for (i, e) in edges.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            let _ = write!(&mut line, r#"{{"pre":{},"post":{},"w":"#, e.pre, e.post);
            write_f32(&mut line, e.w);
            line.push_str(r#","e":"#);
            write_f32(&mut line, e.e);
            line.push_str(r#","dw":"#);
            write_f32(&mut line, e.dw);
            line.push('}');
        }
        line.push_str("]}");
        self.lines.push(line);
        self.lines.last().unwrap()
    }
}

/// Env var for opt-in C1 JSONL trace export path (`--export-trace`).
pub const TRACE_OUT_ENV: &str = "BINN_TRACE_OUT";

/// Env var selecting which seed emits the trace (one seed only).
pub const TRACE_SEED_ENV: &str = "BINN_TRACE_SEED";

/// Destination path when trace export is requested, else `None`.
pub fn trace_out_path() -> Option<std::path::PathBuf> {
    std::env::var_os(TRACE_OUT_ENV).map(std::path::PathBuf::from)
}

/// Seed that should emit the JSONL trace, else `None`.
pub fn trace_export_seed() -> Option<u64> {
    std::env::var(TRACE_SEED_ENV)
        .ok()
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn looks_like_json_object(line: &str) -> bool {
        let t = line.trim();
        t.starts_with('{') && t.ends_with('}') && t.contains('"')
    }

    fn has_key(line: &str, key: &str) -> bool {
        line.contains(&format!("\"{key}\""))
    }

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

    #[test]
    fn f5_account_fields_emit_when_set() {
        let mut log = StructuredLogger::new();
        let entry = RunLog::new("c1-deadbeef", 1, "local-assembly")
            .with_activity_sparsity(0.02)
            .with_f5_account(1700.0, 200.0, 8.5, 100, 800);
        let line = log.emit_results(&entry).unwrap();
        assert!(line.contains("event_work=1700.000000"));
        assert!(line.contains("naive_activity_work=200.000000"));
        assert!(line.contains("work_vs_activity_ratio=8.500000"));
        assert!(line.contains("source_spikes=100"));
        assert!(line.contains("synaptic_deliveries=800"));
    }

    #[test]
    fn json_escape_quotes_and_controls() {
        assert_eq!(json_escape(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(json_escape("x\ny"), r#"x\ny"#);
    }

    #[test]
    fn emit_meta_has_required_keys() {
        let mut tr = TraceRecorder::new();
        let line = tr
            .emit_meta("c1-abc", 7, "local-assembly", "c1", 10, 2, 64)
            .to_string();
        assert!(looks_like_json_object(&line));
        for key in [
            "type",
            "config_hash",
            "seed",
            "condition",
            "experiment",
            "n_classes",
            "k_wta",
            "n_hidden",
        ] {
            assert!(has_key(&line, key), "missing {key} in {line}");
        }
        assert!(line.contains(r#""type":"meta""#));
    }

    #[test]
    fn emit_topology_and_flow_static() {
        let areas = [TraceArea {
            id: 0,
            name: "input".into(),
            start: 0,
            end: 10,
        }];
        let projections = [TraceProjection {
            src: 0,
            dst: 1,
            nnz: 42,
            coupling: Some(vec![0.5, 0.25]),
        }];
        let mut tr = TraceRecorder::new();
        let topo = tr.emit_topology(&areas, &projections).to_string();
        assert!(looks_like_json_object(&topo));
        assert!(topo.contains(r#""type":"topology""#));
        for key in ["areas", "projections"] {
            assert!(has_key(&topo, key));
        }
        let flow = tr.emit_flow_static(&projections).to_string();
        assert!(looks_like_json_object(&flow));
        assert!(flow.contains(r#""type":"flow_static""#));
        assert!(has_key(&flow, "projections"));
        assert!(flow.contains(r#""coupling":["#));
    }

    #[test]
    fn emit_stimulus_spike_kwta() {
        let mut tr = TraceRecorder::new();
        let stim = tr.emit_stimulus(3, 1, 100, 120, "test").to_string();
        assert!(looks_like_json_object(&stim));
        assert!(stim.contains(r#""type":"stimulus""#));
        for key in ["trial", "label", "t0", "t1", "phase"] {
            assert!(has_key(&stim, key));
        }
        let spike = tr.emit_spike(105, 12, 3).to_string();
        assert!(looks_like_json_object(&spike));
        assert!(spike.contains(r#""type":"spike""#));
        for key in ["t", "cell", "trial"] {
            assert!(has_key(&spike, key));
        }
        let scores = [
            TraceScore { cell: 12, v: 0.9 },
            TraceScore { cell: 13, v: 0.4 },
        ];
        let kwta = tr.emit_kwta(3, "hidden", 110, &[12], &scores).to_string();
        assert!(looks_like_json_object(&kwta));
        assert!(kwta.contains(r#""type":"kwta""#));
        for key in ["trial", "area", "t", "winners", "scores"] {
            assert!(has_key(&kwta, key));
        }
    }

    #[test]
    fn emit_assembly_weight_elig() {
        let mut tr = TraceRecorder::new();
        let asm = tr
            .emit_assembly_class(2, &[10, 11], Some(&[3, 1]))
            .to_string();
        assert!(looks_like_json_object(&asm));
        assert!(asm.contains(r#""type":"assembly_class""#));
        for key in ["label", "members", "hits"] {
            assert!(has_key(&asm, key));
        }
        let edges = [TraceWeightEdge {
            pre: 1,
            post: 2,
            w: 0.1,
        }];
        let wf = tr.emit_weight_frame(0, "before", &edges).to_string();
        assert!(looks_like_json_object(&wf));
        assert!(wf.contains(r#""type":"weight_frame""#));
        for key in ["step", "kind", "edges"] {
            assert!(has_key(&wf, key));
        }
        let elig = [TraceEligEdge {
            pre: 1,
            post: 2,
            w: 0.1,
            e: 0.05,
            dw: 0.01,
        }];
        let ee = tr.emit_elig_event(0, 1, 1.0, &elig).to_string();
        assert!(looks_like_json_object(&ee));
        assert!(ee.contains(r#""type":"elig_event""#));
        for key in ["trial", "step", "reward", "edges"] {
            assert!(has_key(&ee, key));
        }
    }

    #[test]
    fn write_jsonl_roundtrip() {
        let mut tr = TraceRecorder::new();
        tr.emit_meta("c1-x", 1, "local", "c1", 2, 1, 8);
        tr.emit_spike(1, 0, 0);
        let dir = std::env::temp_dir();
        let path = dir.join("binn_trace_recorder_test.jsonl");
        tr.write_jsonl(&path).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        let file_lines: Vec<_> = body.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(file_lines.len(), 2);
        assert!(file_lines[0].contains(r#""type":"meta""#));
        assert!(file_lines[1].contains(r#""type":"spike""#));
        let _ = std::fs::remove_file(&path);
    }
}
