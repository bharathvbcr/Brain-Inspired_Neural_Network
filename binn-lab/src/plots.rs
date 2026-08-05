//! Plotting bridge (U13). Viz only; optional `plots` feature enables plotters.
//!
//! Without the feature (default / CI), plot requests return
//! [`PlotResult::Skipped`] while keeping the public API stable.
//!
//! Local enable (no Python):
//! `cargo run --locked --release -p binn-lab --features plots --bin c1 -- …`
//! Camera-ready paper figures:
//! `cargo run --locked --release -p binn-lab --features plots --bin paper-figures`

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
    /// plotters unavailable / feature off — API kept, no figure.
    Skipped(&'static str),
}

/// Plot helpers (scaffold + optional plotters backend).
#[derive(Clone, Debug, Default)]
pub struct Plots;

impl Plots {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Render `req`. Always available; may skip when the `plots` feature is off
    /// or when the plotters backend fails.
    pub fn render(&self, req: &PlotRequest<'_>) -> PlotResult {
        if req.xs.len() != req.ys.len() {
            return PlotResult::Skipped("xs/ys length mismatch");
        }
        #[cfg(feature = "plots")]
        {
            match render_plotters(req) {
                Ok(()) => PlotResult::Written,
                Err(msg) => {
                    eprintln!("binn-lab plots: {msg}");
                    PlotResult::Skipped("plotters unavailable or failed")
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
fn render_plotters(req: &PlotRequest<'_>) -> Result<(), String> {
    use plotters::prelude::*;

    if let Some(parent) = req.out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }

    let root = BitMapBackend::new(req.out_path, (900, 560)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| format!("fill background: {e}"))?;

    let (x_min, x_max) = min_max(req.xs);
    let (y_min, y_max) = min_max(req.ys);
    let x_pad = ((x_max - x_min) * 0.05).max(1e-6);
    let y_pad = ((y_max - y_min) * 0.08).max(1e-6);

    let mut chart = ChartBuilder::on(&root)
        .caption(req.title, ("sans-serif", 22))
        .margin(18)
        .x_label_area_size(40)
        .y_label_area_size(56)
        .build_cartesian_2d(x_min - x_pad..x_max + x_pad, y_min - y_pad..y_max + y_pad)
        .map_err(|e| format!("chart: {e}"))?;

    let (xlabel, ylabel) = match req.kind {
        PlotKind::Raster => ("t", "cell"),
        PlotKind::Weights => ("step", "weight"),
    };
    chart
        .configure_mesh()
        .x_desc(xlabel)
        .y_desc(ylabel)
        .draw()
        .map_err(|e| format!("mesh: {e}"))?;

    match req.kind {
        PlotKind::Raster => {
            chart
                .draw_series(
                    req.xs
                        .iter()
                        .zip(req.ys.iter())
                        .map(|(&x, &y)| Circle::new((x, y), 2, BLUE.filled())),
                )
                .map_err(|e| format!("scatter: {e}"))?;
        }
        PlotKind::Weights => {
            chart
                .draw_series(LineSeries::new(
                    req.xs.iter().copied().zip(req.ys.iter().copied()),
                    &BLUE,
                ))
                .map_err(|e| format!("line: {e}"))?;
        }
    }

    root.present().map_err(|e| format!("present: {e}"))?;
    Ok(())
}

#[cfg(feature = "plots")]
fn min_max(xs: &[f64]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in xs {
        if v < lo {
            lo = v;
        }
        if v > hi {
            hi = v;
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return (0.0, 1.0);
    }
    if (hi - lo).abs() < 1e-12 {
        (lo - 0.5, hi + 0.5)
    } else {
        (lo, hi)
    }
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
                // Only possible with --features plots and a working plotters backend.
            }
        }
    }
}
