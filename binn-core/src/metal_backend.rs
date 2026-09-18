//! Parallel backends for CSR SpMV and LIF integration.
//!
//! Provides parallel matrix-vector multiplication (`y = A · x`) and LIF membrane
//! integration for large-scale cell networks.
//!
//! # Backend honesty invariant
//!
//! 1. The backend selector is an explicit [`Backend`] enum, not a bool.
//! 2. [`METAL_GPU_DISPATCH_IMPLEMENTED`] is the single source of truth for
//!    whether real Metal dispatch exists via [`sparsl`].
//! 3. [`SpmvBackend::try_new`] **refuses to construct** an unavailable backend.
//!    There is no code path that yields a `Backend::MetalGpu` handle which
//!    silently runs on the CPU.
//! 4. [`SpmvBackend::label`] returns the backend that *actually executed*.
//!
//! Metal work is delegated to [`sparsl::Device`] / [`sparsl::SparseOp`]. The
//! duplicate one-thread-per-row MSL stack that lived in this crate is gone.

#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

use std::fmt;

#[cfg(feature = "gpu")]
use std::cell::RefCell;

use crate::sparse::Csr;

/// Whether native Metal GPU kernel dispatch is implemented end-to-end via sparsl.
///
/// Flip only after `metal_gpu_matches_cpu_reference` (and related differential
/// checks) pass. While `true`, [`Backend::MetalGpu`] is still unavailable when
/// the `gpu` feature is off or no Metal device can open.
pub const METAL_GPU_DISPATCH_IMPLEMENTED: bool = true;

/// Which execution substrate a backend handle actually runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Backend {
    /// Multi-threaded CPU execution via rayon. Always available.
    CpuParallel,
    /// Native Metal GPU dispatch through sparsl. Requires the `gpu` cargo
    /// feature, [`METAL_GPU_DISPATCH_IMPLEMENTED`], and a live Metal device.
    MetalGpu,
}

impl Backend {
    /// Human-readable label. Report generators must use this rather than a
    /// hardcoded column heading.
    pub const fn label(self) -> &'static str {
        match self {
            Backend::CpuParallel => "CPU parallel (rayon)",
            Backend::MetalGpu => "Metal GPU",
        }
    }

    /// Whether this backend can actually execute work right now.
    pub fn is_available(self) -> bool {
        self.unavailable_reason().is_none()
    }

    /// Why the backend is unavailable, or `None` if it is available.
    pub fn unavailable_reason(self) -> Option<&'static str> {
        match self {
            Backend::CpuParallel => None,
            Backend::MetalGpu => {
                if !METAL_GPU_DISPATCH_IMPLEMENTED {
                    Some(
                        "Metal kernel dispatch is not implemented \
                         (METAL_GPU_DISPATCH_IMPLEMENTED == false); \
                         refusing to fall back to CPU under a GPU label",
                    )
                } else if !cfg!(feature = "gpu") {
                    Some("binn-core was built without the `gpu` cargo feature")
                } else {
                    sparsl::Backend::Metal.unavailable_reason()
                }
            }
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Returned instead of silently falling back to a different substrate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendUnavailable {
    pub requested: Backend,
    pub reason: &'static str,
}

impl fmt::Display for BackendUnavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "backend `{}` is unavailable: {}",
            self.requested.label(),
            self.reason
        )
    }
}

impl std::error::Error for BackendUnavailable {}

/// Configuration for parallel SpMV / LIF execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpmvBackendConfig {
    pub backend: Backend,
    pub batch_size: usize,
}

impl Default for SpmvBackendConfig {
    fn default() -> Self {
        Self {
            backend: Backend::CpuParallel,
            batch_size: 1024,
        }
    }
}

/// Cached sparsl operator: weights stay resident across ticks with the same
/// topology; rebuilt when shape / topology identity changes.
#[cfg(feature = "gpu")]
struct MetalOpCache {
    nrows: usize,
    ncols: usize,
    nnz: usize,
    row_ptr_addr: usize,
    col_addr: usize,
    op: sparsl::SparseOp,
}

/// Metal device plus a prepared [`sparsl::SparseOp`] cache.
#[cfg(feature = "gpu")]
struct MetalGpuState {
    device: sparsl::Device,
    cache: RefCell<Option<MetalOpCache>>,
}

#[cfg(feature = "gpu")]
impl MetalGpuState {
    fn open() -> Result<Self, BackendUnavailable> {
        let device =
            sparsl::Device::try_new(sparsl::Backend::Metal).map_err(|e| BackendUnavailable {
                requested: Backend::MetalGpu,
                reason: e.reason,
            })?;
        Ok(Self {
            device,
            cache: RefCell::new(None),
        })
    }

    fn ensure_op(&self, csr: &Csr, weights: &[f32]) -> Result<(), String> {
        let nrows = csr.nrows();
        let ncols = csr.ncols();
        let nnz = csr.nnz();
        let row_ptr_addr = csr.row_ptr.as_ptr() as usize;
        let col_addr = csr.col.as_ptr() as usize;

        let mut slot = self.cache.borrow_mut();
        let rebuild = match slot.as_ref() {
            None => true,
            Some(c) => {
                c.nrows != nrows
                    || c.ncols != ncols
                    || c.nnz != nnz
                    || c.row_ptr_addr != row_ptr_addr
                    || c.col_addr != col_addr
            }
        };
        if rebuild {
            let inner = csr.to_sparsl();
            let op = self
                .device
                .prepare(&inner, ncols, weights)
                .map_err(|e| e.to_string())?;
            *slot = Some(MetalOpCache {
                nrows,
                ncols,
                nnz,
                row_ptr_addr,
                col_addr,
                op,
            });
        } else if let Some(c) = slot.as_mut() {
            c.op.set_weights(weights).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn with_op_mut<R>(
        &self,
        csr: &Csr,
        weights: &[f32],
        f: impl FnOnce(&mut sparsl::SparseOp) -> Result<R, String>,
    ) -> Result<R, String> {
        self.ensure_op(csr, weights)?;
        let mut slot = self.cache.borrow_mut();
        let cache = slot
            .as_mut()
            .expect("ensure_op must leave a prepared SparseOp");
        f(&mut cache.op)
    }
}

/// Parallel SpMV and LIF integration execution engine.
///
/// A handle can only exist for a backend that is actually available, so
/// `backend.label()` is always a truthful description of what ran.
pub struct SpmvBackend {
    config: SpmvBackendConfig,
    #[cfg(feature = "gpu")]
    metal: Option<MetalGpuState>,
}

impl Clone for SpmvBackend {
    fn clone(&self) -> Self {
        // Metal state is not shared across clones: each handle opens its own
        // device path and rebuilds the SparseOp cache on first use.
        match Self::try_new(self.config) {
            Ok(b) => b,
            Err(e) => panic!("SpmvBackend::clone failed: {e}"),
        }
    }
}

impl fmt::Debug for SpmvBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpmvBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Default for SpmvBackend {
    fn default() -> Self {
        Self::cpu()
    }
}

impl SpmvBackend {
    /// CPU-parallel backend with default batching. Infallible.
    pub fn cpu() -> Self {
        Self {
            config: SpmvBackendConfig::default(),
            #[cfg(feature = "gpu")]
            metal: None,
        }
    }

    /// Construct a backend, or fail if the requested substrate cannot execute.
    ///
    /// This never falls back to a different backend. Callers that want a
    /// fallback must ask for it explicitly and relabel their output.
    pub fn try_new(config: SpmvBackendConfig) -> Result<Self, BackendUnavailable> {
        if let Some(reason) = config.backend.unavailable_reason() {
            return Err(BackendUnavailable {
                requested: config.backend,
                reason,
            });
        }
        #[cfg(feature = "gpu")]
        let metal = match config.backend {
            Backend::MetalGpu => Some(MetalGpuState::open()?),
            Backend::CpuParallel => None,
        };
        Ok(Self {
            config,
            #[cfg(feature = "gpu")]
            metal,
        })
    }

    /// Construct a backend, panicking with the unavailability reason.
    ///
    /// # Panics
    ///
    /// Panics if `config.backend` is not available. This is intentional: a
    /// benchmark that requests a GPU arm must abort rather than silently time
    /// the CPU path.
    pub fn new(config: SpmvBackendConfig) -> Self {
        match Self::try_new(config) {
            Ok(backend) => backend,
            Err(e) => panic!("{e}"),
        }
    }

    /// The substrate this handle actually executes on.
    pub const fn backend(&self) -> Backend {
        self.config.backend
    }

    /// Truthful label for report generators.
    pub const fn label(&self) -> &'static str {
        self.config.backend.label()
    }

    pub const fn config(&self) -> &SpmvBackendConfig {
        &self.config
    }

    /// Underlying sparsl device when this handle is Metal GPU.
    ///
    /// Used by labelled crossover drivers that need resident `SparseOp` ticks
    /// without standing up a second Metal stack.
    #[cfg(feature = "gpu")]
    pub fn metal_device(&self) -> Option<&sparsl::Device> {
        self.metal.as_ref().map(|m| &m.device)
    }

    /// Prepare a sparsl operator with resident weights on this Metal handle.
    #[cfg(feature = "gpu")]
    pub fn prepare_sparse(
        &self,
        csr: &Csr,
        weights: &[f32],
    ) -> Result<sparsl::SparseOp, sparsl::SparsePlanError> {
        let state = self
            .metal
            .as_ref()
            .expect("prepare_sparse requires Backend::MetalGpu");
        let inner = csr.to_sparsl();
        state.device.prepare(&inner, csr.ncols(), weights)
    }

    /// Execute Sparse Matrix-Vector Multiply: `y = y + A · x`
    ///
    /// # Panics
    ///
    /// Panics if `x.len() < A.ncols()`, `y.len() != A.nrows()`, or
    /// `weights.len() != A.nnz()`.
    pub fn spmv(&self, csr: &Csr, weights: &[f32], x: &[f32], y: &mut [f32]) {
        assert!(x.len() >= csr.ncols(), "x dimension mismatch");
        assert_eq!(y.len(), csr.nrows(), "y dimension mismatch");
        assert_eq!(weights.len(), csr.nnz(), "weights dimension mismatch");

        match self.config.backend {
            Backend::CpuParallel => Self::spmv_cpu(csr, weights, x, y),
            Backend::MetalGpu => {
                #[cfg(feature = "gpu")]
                {
                    let state = self
                        .metal
                        .as_ref()
                        .expect("MetalGpu SpmvBackend must hold a device");
                    state
                        .with_op_mut(csr, weights, |op| {
                            op.spmv(x, y).map_err(|e| e.to_string())
                        })
                        .unwrap_or_else(|e| panic!("Metal SpMV failed: {e}"));
                }
                #[cfg(not(feature = "gpu"))]
                {
                    unreachable!(
                        "SpmvBackend holds Backend::MetalGpu without the gpu feature; \
                         SpmvBackend::try_new must reject it"
                    );
                }
            }
        }
    }

    fn spmv_cpu(csr: &Csr, weights: &[f32], x: &[f32], y: &mut [f32]) {
        use rayon::prelude::*;

        y.par_iter_mut().enumerate().for_each(|(r, y_val)| {
            let row_start = csr.row_ptr[r] as usize;
            let row_end = csr.row_ptr[r + 1] as usize;
            let mut sum = 0.0f32;
            for i in row_start..row_end {
                let col = csr.col[i] as usize;
                sum += weights[i] * x[col];
            }
            *y_val += sum;
        });
    }

    /// Parallel batch LIF membrane integrate and threshold spike check.
    pub fn batch_lif_integrate(
        &self,
        v: &mut [f32],
        theta: &mut [f32],
        currents: &[f32],
        spikes: &mut [bool],
        decay: f32,
        v_reset: f32,
        delta_theta: f32,
    ) {
        let n = v.len();
        assert_eq!(theta.len(), n);
        assert_eq!(currents.len(), n);
        assert_eq!(spikes.len(), n);

        match self.config.backend {
            Backend::CpuParallel => {
                use rayon::prelude::*;

                v.par_iter_mut()
                    .zip(theta.par_iter_mut())
                    .zip(currents.par_iter())
                    .zip(spikes.par_iter_mut())
                    .for_each(|(((v_i, th_i), &curr_i), spk_i)| {
                        let voltage = *v_i * decay + curr_i;
                        if voltage >= *th_i {
                            *spk_i = true;
                            *v_i = v_reset;
                            *th_i += delta_theta;
                        } else {
                            *spk_i = false;
                            *v_i = voltage;
                        }
                    });
            }
            Backend::MetalGpu => {
                #[cfg(feature = "gpu")]
                {
                    let state = self
                        .metal
                        .as_ref()
                        .expect("MetalGpu SpmvBackend must hold a device");
                    let params = sparsl::LifParams::new(decay, v_reset, delta_theta)
                        .unwrap_or_else(|e| panic!("invalid LIF params: {e}"));
                    state
                        .device
                        .lif_integrate(v, theta, currents, spikes, params)
                        .unwrap_or_else(|e| panic!("Metal LIF failed: {e}"));
                }
                #[cfg(not(feature = "gpu"))]
                {
                    unreachable!(
                        "SpmvBackend holds Backend::MetalGpu without the gpu feature; \
                         SpmvBackend::try_new must reject it"
                    );
                }
            }
        }
    }

    /// Fused CSR SpMV + LIF integration (single pass).
    pub fn fused_spmv_lif_integrate(
        &self,
        csr: &Csr,
        weights: &[f32],
        x: &[f32],
        v: &mut [f32],
        theta: &mut [f32],
        spikes: &mut [bool],
        decay: f32,
        v_reset: f32,
        delta_theta: f32,
    ) {
        let n = v.len();
        assert_eq!(n, csr.nrows());
        assert_eq!(theta.len(), n);
        assert_eq!(spikes.len(), n);
        assert_eq!(weights.len(), csr.nnz());
        assert!(x.len() >= csr.ncols());

        match self.config.backend {
            Backend::CpuParallel => {
                use rayon::prelude::*;

                v.par_iter_mut()
                    .zip(theta.par_iter_mut())
                    .zip(spikes.par_iter_mut())
                    .enumerate()
                    .for_each(|(r, ((v_i, th_i), spk_i))| {
                        let row_start = csr.row_ptr[r] as usize;
                        let row_end = csr.row_ptr[r + 1] as usize;
                        let mut synaptic_sum = 0.0f32;
                        for i in row_start..row_end {
                            let col = csr.col[i] as usize;
                            synaptic_sum += weights[i] * x[col];
                        }
                        let voltage = *v_i * decay + synaptic_sum;
                        if voltage >= *th_i {
                            *spk_i = true;
                            *v_i = v_reset;
                            *th_i += delta_theta;
                        } else {
                            *spk_i = false;
                            *v_i = voltage;
                        }
                    });
            }
            Backend::MetalGpu => {
                #[cfg(feature = "gpu")]
                {
                    let state = self
                        .metal
                        .as_ref()
                        .expect("MetalGpu SpmvBackend must hold a device");
                    let params = sparsl::LifParams::new(decay, v_reset, delta_theta)
                        .unwrap_or_else(|e| panic!("invalid LIF params: {e}"));
                    state
                        .with_op_mut(csr, weights, |op| {
                            op.fused_spmv_lif(x, v, theta, spikes, params)
                                .map_err(|e| e.to_string())
                        })
                        .unwrap_or_else(|e| panic!("Metal fused SpMV+LIF failed: {e}"));
                }
                #[cfg(not(feature = "gpu"))]
                {
                    unreachable!(
                        "SpmvBackend holds Backend::MetalGpu without the gpu feature; \
                         SpmvBackend::try_new must reject it"
                    );
                }
            }
        }
    }
}

/// Backend arms a throughput benchmark may legitimately compare.
///
/// Returns only substrates that are actually available, so a benchmark loop
/// built on this can never emit a "GPU vs CPU" table where both arms ran on the
/// CPU.
pub fn benchmarkable_backends() -> Vec<Backend> {
    [Backend::CpuParallel, Backend::MetalGpu]
        .into_iter()
        .filter(|b| b.is_available())
        .collect()
}

/// Thin labelled handle around a sparsl Metal [`sparsl::Device`].
///
/// Prefer [`SpmvBackend`] with [`Backend::MetalGpu`] for SpMV/LIF. This type
/// exists so crossover examples can prepare a resident [`sparsl::SparseOp`]
/// without a second kernel stack.
#[cfg(feature = "gpu")]
pub struct MetalGpuContext {
    device: sparsl::Device,
}

#[cfg(feature = "gpu")]
impl MetalGpuContext {
    /// Open the system Metal device through sparsl, or `None` if unavailable.
    pub fn new() -> Option<Self> {
        sparsl::Device::try_new(sparsl::Backend::Metal)
            .ok()
            .map(|device| Self { device })
    }

    /// Underlying sparsl device.
    pub fn device(&self) -> &sparsl::Device {
        &self.device
    }

    /// Device name for reports.
    pub fn device_name(&self) -> String {
        self.device
            .device_name()
            .unwrap_or_else(|| "Metal".to_string())
    }

    /// Prepare a resident sparse operator (weights uploaded once).
    pub fn prepare(
        &self,
        csr: &Csr,
        weights: &[f32],
    ) -> Result<sparsl::SparseOp, sparsl::SparsePlanError> {
        let inner = csr.to_sparsl();
        self.device.prepare(&inner, csr.ncols(), weights)
    }

    /// `y += A · x` via a freshly prepared operator.
    pub fn spmv(&self, csr: &Csr, weights: &[f32], x: &[f32], y: &mut [f32]) {
        let op = self
            .prepare(csr, weights)
            .unwrap_or_else(|e| panic!("Metal prepare failed: {e}"));
        op.spmv(x, y)
            .unwrap_or_else(|e| panic!("Metal SpMV failed: {e}"));
    }

    /// Dense LIF integrate via sparsl.
    pub fn batch_lif_integrate(
        &self,
        v: &mut [f32],
        theta: &mut [f32],
        currents: &[f32],
        spikes: &mut [bool],
        decay: f32,
        v_reset: f32,
        delta_theta: f32,
    ) {
        let params = sparsl::LifParams::new(decay, v_reset, delta_theta)
            .unwrap_or_else(|e| panic!("invalid LIF params: {e}"));
        self.device
            .lif_integrate(v, theta, currents, spikes, params)
            .unwrap_or_else(|e| panic!("Metal LIF failed: {e}"));
    }

    /// Fused SpMV + LIF via a freshly prepared operator.
    pub fn fused_spmv_lif_integrate(
        &self,
        csr: &Csr,
        weights: &[f32],
        x: &[f32],
        v: &mut [f32],
        theta: &mut [f32],
        spikes: &mut [bool],
        decay: f32,
        v_reset: f32,
        delta_theta: f32,
    ) {
        let params = sparsl::LifParams::new(decay, v_reset, delta_theta)
            .unwrap_or_else(|e| panic!("invalid LIF params: {e}"));
        let op = self
            .prepare(csr, weights)
            .unwrap_or_else(|e| panic!("Metal prepare failed: {e}"));
        op.fused_spmv_lif(x, v, theta, spikes, params)
            .unwrap_or_else(|e| panic!("Metal fused SpMV+LIF failed: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_problem() -> (Csr, Vec<f32>, Vec<f32>) {
        let adj = vec![vec![1, 2], vec![0], vec![0, 1]];
        let csr = Csr::from_adjacency(&adj);
        let weights = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let x = vec![0.5, 1.0, 1.5];
        (csr, weights, x)
    }

    #[test]
    fn cpu_spmv_correctness() {
        let (csr, weights, x) = tiny_problem();
        let mut y = vec![0.0; 3];

        let backend = SpmvBackend::cpu();
        backend.spmv(&csr, &weights, &x, &mut y);

        // Row 0: 1.0*x[1] + 2.0*x[2] = 4.0
        // Row 1: 3.0*x[0] = 1.5
        // Row 2: 4.0*x[0] + 5.0*x[1] = 7.0
        assert!((y[0] - 4.0).abs() < 1e-5);
        assert!((y[1] - 1.5).abs() < 1e-5);
        assert!((y[2] - 7.0).abs() < 1e-5);
    }

    /// While Metal dispatch is unimplemented *or* the gpu feature / device is
    /// missing, it must be impossible to obtain a GPU-labelled handle.
    #[test]
    fn unavailable_gpu_backend_is_unconstructible() {
        if Backend::MetalGpu.is_available() {
            return;
        }
        let err = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect_err("Backend::MetalGpu must not be constructible when unavailable");
        assert_eq!(err.requested, Backend::MetalGpu);
        assert!(!Backend::MetalGpu.is_available());
    }

    /// When Metal is available, a GPU-labelled handle must construct and not
    /// pretend to be the CPU arm.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_gpu_backend_constructs_when_available() {
        if !Backend::MetalGpu.is_available() {
            return;
        }
        let backend = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu must construct when is_available");
        assert_eq!(backend.backend(), Backend::MetalGpu);
        assert_eq!(backend.label(), "Metal GPU");
        assert!(backend.metal_device().is_some());
    }

    /// A benchmark driven by `benchmarkable_backends()` can never produce a
    /// two-arm table whose arms are the same substrate.
    #[test]
    fn benchmarkable_backends_are_distinct_and_available() {
        let arms = benchmarkable_backends();
        assert!(arms.contains(&Backend::CpuParallel));
        for a in &arms {
            assert!(a.is_available(), "{a} advertised but unavailable");
        }
        let mut seen = arms.clone();
        seen.sort_by_key(|b| b.label());
        seen.dedup();
        assert_eq!(seen.len(), arms.len(), "duplicate backend arms");
        if !Backend::MetalGpu.is_available() {
            assert_eq!(
                arms.len(),
                1,
                "only the CPU arm may be benchmarked until Metal is available"
            );
        }
    }

    #[test]
    fn labels_describe_the_executing_substrate() {
        assert_eq!(SpmvBackend::cpu().label(), "CPU parallel (rayon)");
        assert_eq!(SpmvBackend::cpu().backend(), Backend::CpuParallel);
    }

    /// Activates when Metal is available under `--features gpu`.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_gpu_matches_cpu_reference() {
        if !METAL_GPU_DISPATCH_IMPLEMENTED || !Backend::MetalGpu.is_available() {
            return;
        }
        let (csr, weights, x) = tiny_problem();
        let mut y_cpu = vec![0.0; 3];
        SpmvBackend::cpu().spmv(&csr, &weights, &x, &mut y_cpu);

        let gpu = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu available");
        let mut y_gpu = vec![0.0; 3];
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);

        let max_abs_term = weights
            .iter()
            .zip(csr.col.iter())
            .map(|(&w, &c)| (w * x[c as usize]).abs())
            .fold(0.0f32, f32::max);
        let max_row_nnz = (0..csr.nrows())
            .map(|r| csr.row_cols(r).len())
            .max()
            .unwrap_or(0);
        let max_abs_result = y_cpu.iter().copied().fold(0.0f32, |a, b| a.max(b.abs()));
        let tol = sparsl::tolerance_for_spmv(max_row_nnz, max_abs_term, max_abs_result);

        for (a, b) in y_cpu.iter().zip(y_gpu.iter()) {
            assert!(
                (a - b).abs() <= tol,
                "GPU/CPU mismatch: {a} vs {b} (tol={tol})"
            );
        }
    }

    /// Fused SpMV+LIF and standalone LIF must agree with the CPU arm within
    /// the sparsl SpMV tolerance envelope (LIF itself is elementwise).
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_fused_and_lif_match_cpu_reference() {
        if !METAL_GPU_DISPATCH_IMPLEMENTED || !Backend::MetalGpu.is_available() {
            return;
        }
        let (csr, weights, x) = tiny_problem();
        let n = csr.nrows();
        let decay = 0.95;
        let v_reset = 0.0;
        let delta_theta = 0.1;

        let mut v_cpu = vec![0.2; n];
        let mut th_cpu = vec![0.5; n];
        let mut spk_cpu = vec![false; n];
        SpmvBackend::cpu().fused_spmv_lif_integrate(
            &csr,
            &weights,
            &x,
            &mut v_cpu,
            &mut th_cpu,
            &mut spk_cpu,
            decay,
            v_reset,
            delta_theta,
        );

        let gpu = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu available");
        let mut v_gpu = vec![0.2; n];
        let mut th_gpu = vec![0.5; n];
        let mut spk_gpu = vec![false; n];
        gpu.fused_spmv_lif_integrate(
            &csr,
            &weights,
            &x,
            &mut v_gpu,
            &mut th_gpu,
            &mut spk_gpu,
            decay,
            v_reset,
            delta_theta,
        );

        let max_abs_term = weights
            .iter()
            .zip(csr.col.iter())
            .map(|(&w, &c)| (w * x[c as usize]).abs())
            .fold(0.0f32, f32::max)
            .max(1.0);
        let max_row_nnz = (0..csr.nrows())
            .map(|r| csr.row_cols(r).len())
            .max()
            .unwrap_or(0);
        let tol = sparsl::tolerance_for_spmv(max_row_nnz, max_abs_term, max_abs_term);

        for i in 0..n {
            assert!(
                (v_cpu[i] - v_gpu[i]).abs() <= tol,
                "v[{i}]: {} vs {} (tol={tol})",
                v_cpu[i],
                v_gpu[i]
            );
            assert!(
                (th_cpu[i] - th_gpu[i]).abs() <= tol,
                "theta[{i}]: {} vs {}",
                th_cpu[i],
                th_gpu[i]
            );
            assert_eq!(spk_cpu[i], spk_gpu[i], "spike[{i}]");
        }

        let mut currents = vec![0.0; n];
        SpmvBackend::cpu().spmv(&csr, &weights, &x, &mut currents);
        let mut v_l = vec![0.1; n];
        let mut th_l = vec![1.0; n];
        let mut spk_l = vec![false; n];
        let mut v_g = v_l.clone();
        let mut th_g = th_l.clone();
        let mut spk_g = spk_l.clone();
        SpmvBackend::cpu().batch_lif_integrate(
            &mut v_l, &mut th_l, &currents, &mut spk_l, decay, v_reset, delta_theta,
        );
        gpu.batch_lif_integrate(
            &mut v_g, &mut th_g, &currents, &mut spk_g, decay, v_reset, delta_theta,
        );
        for i in 0..n {
            assert!((v_l[i] - v_g[i]).abs() <= 1e-5, "LIF v[{i}]");
            assert!((th_l[i] - th_g[i]).abs() <= 1e-5, "LIF theta[{i}]");
            assert_eq!(spk_l[i], spk_g[i], "LIF spike[{i}]");
        }
    }

    #[test]
    fn metal_dispatch_flag_matches_implementation() {
        // Honesty pin: the flag must stay true only while MetalGpu dispatch
        // paths above are real (sparsl-backed), not stubs.
        assert!(METAL_GPU_DISPATCH_IMPLEMENTED);
    }
}
