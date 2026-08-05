//! Replay export (viz only, adjunct to U13 logging).
//!
//! Serializes the full spike log plus topology and trial windows so the
//! offline HTML viewer (`viz/replay_viewer.html`) can animate activation flow.
//!
//! Enabled by `BINN_REPLAY_OUT=<path>` (or `--replay <path>` on the `c1`
//! binary). Capture is read-only over engine state: it has no effect on
//! config hashes, accuracies, budgets, or the GC7 structured log.
//!
//! Output is hand-rolled JSON (matching the harness convention; GC2: no new
//! dependencies) with deterministic field and element order:
//! edges in CSR nnz order, spikes in recording order.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

use binn_core::Tick;
use binn_engine::Engine;

/// Format tag for forward compatibility of the viewer.
pub const REPLAY_FORMAT: &str = "binn-replay-v1";

/// One contiguous cell-id range with a display name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayGroup {
    /// Display name (`input`, `hidden`, `readout`).
    pub name: String,
    /// First cell id in the group (inclusive).
    pub start: u32,
    /// One past the last cell id (exclusive).
    pub end: u32,
}

/// One trial window on the shared engine clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayTrial {
    /// `train` or `test`.
    pub phase: &'static str,
    /// Task label for the trial.
    pub label: u32,
    /// First tick of the trial window (inclusive).
    pub t0: Tick,
    /// Engine time when the trial finished (inclusive upper bound).
    pub t1: Tick,
    /// Prediction correctness (test trials only).
    pub correct: Option<bool>,
}

/// Complete replay payload for one condition run.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayExport {
    /// Experiment tag (`c1`).
    pub experiment: String,
    /// Config hash string (`c1-…`).
    pub config_hash: String,
    /// Replicate seed.
    pub seed: u64,
    /// Condition label (`local-assembly`, …).
    pub condition: String,
    /// Total cell count.
    pub n_cells: u32,
    /// k-WTA winner budget of the hidden area.
    pub k_wta: u32,
    /// Named cell-id ranges.
    pub groups: Vec<ReplayGroup>,
    /// Directed edges `(pre, post, weight, delay)` in CSR nnz order.
    pub edges: Vec<(u32, u32, f32, Tick)>,
    /// Trial windows in run order.
    pub trials: Vec<ReplayTrial>,
    /// Spikes `(t, cell)` in recording order.
    pub spikes: Vec<(Tick, u32)>,
}

impl ReplayExport {
    /// Snapshot topology, final weights, and the full spike log from `eng`.
    ///
    /// `groups` should partition the meaningful cell ranges; `trials` are the
    /// windows recorded by the caller on the same clock as `eng.spikes()`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_engine(
        experiment: impl Into<String>,
        config_hash: impl Into<String>,
        seed: u64,
        condition: impl Into<String>,
        k_wta: usize,
        groups: Vec<ReplayGroup>,
        trials: Vec<ReplayTrial>,
        eng: &Engine,
    ) -> Self {
        let mut edges = Vec::with_capacity(eng.conn.nnz());
        for (i, (pre, post)) in eng.conn.edges().enumerate() {
            let w = eng.edge_w.get(i).copied().unwrap_or(0.0);
            let d = eng.syn.get(i).map(|s| s.delay).unwrap_or(1);
            edges.push((pre, post, w, d));
        }
        let spikes = eng
            .spikes()
            .iter()
            .map(|sp| (sp.t, sp.cell))
            .collect::<Vec<_>>();
        Self {
            experiment: experiment.into(),
            config_hash: config_hash.into(),
            seed,
            condition: condition.into(),
            n_cells: eng.num_cells() as u32,
            k_wta: k_wta as u32,
            groups,
            edges,
            trials,
            spikes,
        }
    }

    /// Deterministic JSON rendering (stable field and element order).
    pub fn render_json(&self) -> String {
        let mut s = String::with_capacity(64 + self.edges.len() * 24 + self.spikes.len() * 12);
        let _ = write!(
            &mut s,
            "{{\"format\":\"{}\",\"experiment\":\"{}\",\"config_hash\":\"{}\",\"seed\":{},\"condition\":\"{}\",\"n_cells\":{},\"k_wta\":{}",
            REPLAY_FORMAT,
            self.experiment,
            self.config_hash,
            self.seed,
            self.condition,
            self.n_cells,
            self.k_wta
        );
        s.push_str(",\"groups\":[");
        for (i, g) in self.groups.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(
                &mut s,
                "{{\"name\":\"{}\",\"start\":{},\"end\":{}}}",
                g.name, g.start, g.end
            );
        }
        s.push_str("],\"edges\":[");
        for (i, (pre, post, w, d)) in self.edges.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(&mut s, "[{pre},{post},{w:.6},{d}]");
        }
        s.push_str("],\"trials\":[");
        for (i, t) in self.trials.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(
                &mut s,
                "{{\"phase\":\"{}\",\"label\":{},\"t0\":{},\"t1\":{}",
                t.phase, t.label, t.t0, t.t1
            );
            if let Some(ok) = t.correct {
                let _ = write!(&mut s, ",\"correct\":{ok}");
            }
            s.push('}');
        }
        s.push_str("],\"spikes\":[");
        for (i, (t, cell)) in self.spikes.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            let _ = write!(&mut s, "[{t},{cell}]");
        }
        s.push_str("]}");
        s
    }

    /// Write the JSON payload to `path`, creating parent dirs as needed.
    pub fn write(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, self.render_json())
    }
}

/// Env var consulted by the runner for opt-in replay capture.
pub const REPLAY_OUT_ENV: &str = "BINN_REPLAY_OUT";

/// Destination path when replay capture is requested, else `None`.
pub fn replay_out_path() -> Option<std::path::PathBuf> {
    std::env::var_os(REPLAY_OUT_ENV).map(std::path::PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_core::Csr;

    fn tiny_export() -> ReplayExport {
        let mut eng = Engine::with_cells(4);
        let conn = Csr::from_adjacency(&[vec![2], vec![2], vec![3], vec![]]);
        eng.set_connectivity(conn, vec![0.5, 0.25, 1.0]);
        eng.force_spike(0, 1);
        let _ = eng.step_until(10);
        ReplayExport::from_engine(
            "c1",
            "c1-deadbeef",
            7,
            "local-assembly",
            1,
            vec![
                ReplayGroup {
                    name: "input".into(),
                    start: 0,
                    end: 2,
                },
                ReplayGroup {
                    name: "hidden".into(),
                    start: 2,
                    end: 3,
                },
                ReplayGroup {
                    name: "readout".into(),
                    start: 3,
                    end: 4,
                },
            ],
            vec![ReplayTrial {
                phase: "test",
                label: 1,
                t0: 0,
                t1: 10,
                correct: Some(true),
            }],
            &eng,
        )
    }

    #[test]
    fn render_json_is_deterministic() {
        let e = tiny_export();
        assert_eq!(e.render_json(), e.render_json());
        assert_eq!(e.clone().render_json(), e.render_json());
    }

    #[test]
    fn render_json_has_expected_shape() {
        let e = tiny_export();
        let j = e.render_json();
        assert!(j.starts_with("{\"format\":\"binn-replay-v1\""));
        assert!(j.contains("\"config_hash\":\"c1-deadbeef\""));
        assert!(j.contains("\"groups\":[{\"name\":\"input\",\"start\":0,\"end\":2}"));
        assert!(j.contains("\"edges\":[[0,2,0.500000,1],[1,2,0.250000,1],[2,3,1.000000,1]]"));
        assert!(j.contains(
            "\"trials\":[{\"phase\":\"test\",\"label\":1,\"t0\":0,\"t1\":10,\"correct\":true}]"
        ));
        assert!(j.ends_with("]}"));
        // Balanced braces / brackets (cheap structural sanity without a parser dep).
        let open = j.matches('{').count() + j.matches('[').count();
        let close = j.matches('}').count() + j.matches(']').count();
        assert_eq!(open, close);
    }

    #[test]
    fn edges_follow_csr_nnz_order_and_spikes_recorded() {
        let e = tiny_export();
        assert_eq!(e.edges.len(), 3);
        assert_eq!(e.edges[0].0, 0);
        assert!(!e.spikes.is_empty(), "forced spike must appear in the log");
        assert_eq!(e.spikes[0], (1, 0));
    }

    #[test]
    fn write_creates_parent_dirs() {
        let e = tiny_export();
        let dir = std::env::temp_dir().join("binn_replay_test");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("nested").join("replay.json");
        e.write(&path).expect("write must succeed");
        let back = fs::read_to_string(&path).expect("readback");
        assert_eq!(back, e.render_json());
        let _ = fs::remove_dir_all(&dir);
    }
}
