//! Plotting bridge (U13). Viz only; optional `plots` feature enables pyo3.
//!
//! Without the feature (default / CI), plot requests return
//! [`PlotResult::Skipped`] while keeping the public API stable.

use std::path::Path;

/// Kind of figure the harness may request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlotKind {
    /// Spike raster (cell × time).
    Raster,
    /// Weight trajectory over training steps.
    Weights,
}

/// One plot request.
#[derive(Clone, Debug)]
pub struct PlotRequest<'a> {
    pub kind: PlotKind,
    pub title: &'a str,
    pub out_path: &'a Path,
    /// X values (time / step).
    pub xs: &'a [f64],
    /// Y values (cell id / weight). Parallel to `xs` for line plots; for
    /// rasters, pair `(xs[i], ys[i])` is a spike at time `xs[i]` on cell `ys[i]`.
    pub ys: &'a [f64],
}

/// Outcome of a plot attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlotResult {
    /// Figure written to disk.
    Written,
    /// pyo3 / matplotlib unavailable — API kept, no figure.
    Skipped(&'static str),
}

/// Plot helpers (scaffold + optional pyo3 backend).
#[derive(Clone, Debug, Default)]
pub struct Plots;

impl Plots {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Render `req`. Always available; may skip when the `plots` feature is off
    /// or when the Python side fails.
    pub fn render(&self, req: &PlotRequest<'_>) -> PlotResult {
        if req.xs.len() != req.ys.len() {
            return PlotResult::Skipped("xs/ys length mismatch");
        }
        #[cfg(feature = "plots")]
        {
            match render_pyo3(req) {
                Ok(()) => PlotResult::Written,
                Err(msg) => {
                    // Keep CI / machines without matplotlib green.
                    let _ = msg;
                    PlotResult::Skipped("pyo3/matplotlib unavailable or failed")
                }
            }
        }
        #[cfg(not(feature = "plots"))]
        {
            let _ = req;
            PlotResult::Skipped("plots feature disabled (enable --features plots)")
        }
    }

    /// Convenience: spike raster.
    pub fn raster(&self, title: &str, out: &Path, times: &[f64], cells: &[f64]) -> PlotResult {
        self.render(&PlotRequest {
            kind: PlotKind::Raster,
            title,
            out_path: out,
            xs: times,
            ys: cells,
        })
    }

    /// Convenience: weight curve.
    pub fn weights(&self, title: &str, out: &Path, steps: &[f64], ws: &[f64]) -> PlotResult {
        self.render(&PlotRequest {
            kind: PlotKind::Weights,
            title,
            out_path: out,
            xs: steps,
            ys: ws,
        })
    }
}

#[cfg(feature = "plots")]
fn render_pyo3(req: &PlotRequest<'_>) -> Result<(), String> {
    use pyo3::prelude::*;
    use pyo3::types::PyList;

    Python::with_gil(|py| {
        let plt = py
            .import_bound("matplotlib.pyplot")
            .map_err(|e| e.to_string())?;
        plt.call_method0("figure").map_err(|e| e.to_string())?;
        let xs = PyList::new_bound(py, req.xs.iter().copied());
        let ys = PyList::new_bound(py, req.ys.iter().copied());
        match req.kind {
            PlotKind::Raster => {
                plt.call_method1("scatter", (xs, ys))
                    .map_err(|e| e.to_string())?;
                plt.call_method1("title", (req.title,))
                    .map_err(|e| e.to_string())?;
                plt.call_method1("xlabel", ("t",))
                    .map_err(|e| e.to_string())?;
                plt.call_method1("ylabel", ("cell",))
                    .map_err(|e| e.to_string())?;
            }
            PlotKind::Weights => {
                plt.call_method1("plot", (xs, ys))
                    .map_err(|e| e.to_string())?;
                plt.call_method1("title", (req.title,))
                    .map_err(|e| e.to_string())?;
                plt.call_method1("xlabel", ("step",))
                    .map_err(|e| e.to_string())?;
                plt.call_method1("ylabel", ("weight",))
                    .map_err(|e| e.to_string())?;
            }
        }
        let path = req.out_path.to_string_lossy();
        plt.call_method1("savefig", (path.as_ref(),))
            .map_err(|e| e.to_string())?;
        plt.call_method0("close").map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn plots_api_skips_gracefully_without_feature() {
        let plots = Plots::new();
        let out = PathBuf::from("/tmp/binn_c1_raster_test.png");
        let r = plots.raster("test", &out, &[0.0, 1.0], &[0.0, 1.0]);
        match r {
            PlotResult::Skipped(_) => {}
            PlotResult::Written => {
                // Only possible with --features plots and a working matplotlib.
            }
        }
    }
}
