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
use std::cell::{Cell, RefCell};

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

/// Block width (64 bytes = four 128-bit SIMD vectors) for bitwise `f32` equality.
#[cfg(feature = "gpu")]
const BIT_EQ_LANES: usize = 16;

#[cfg(feature = "gpu")]
fn f32_bits_eq(cached_bits: &[u32], weights: &[f32]) -> bool {
    if cached_bits.len() != weights.len() {
        return false;
    }
    let (b_chunks, b_rem) = cached_bits.as_chunks::<BIT_EQ_LANES>();
    let (w_chunks, w_rem) = weights.as_chunks::<BIT_EQ_LANES>();
    for (bc, wc) in b_chunks.iter().zip(w_chunks.iter()) {
        if wc.map(f32::to_bits) != *bc {
            return false;
        }
    }
    b_rem
        .iter()
        .zip(w_rem.iter())
        .all(|(&b, &w)| b == w.to_bits())
}

#[cfg(feature = "gpu")]
fn overwrite_f32_bits(dst: &mut Vec<u32>, weights: &[f32]) {
    dst.resize(weights.len(), 0);
    for (slot, &w) in dst.iter_mut().zip(weights.iter()) {
        *slot = w.to_bits();
    }
}

/// Cached sparsl operator: weights stay resident across ticks with the same
/// topology and bit-identical weights; rebuilt when shape or CSR indices
/// change, and re-uploaded via `set_weights` only when weight bits change.
#[cfg(feature = "gpu")]
struct MetalOpCache {
    ncols: usize,
    row_ptr: Vec<u32>,
    col: Vec<u32>,
    weights_bits: Vec<u32>,
    op: sparsl::SparseOp,
}

/// Metal device plus a prepared [`sparsl::SparseOp`] cache.
#[cfg(feature = "gpu")]
struct MetalGpuState {
    device: sparsl::Device,
    cache: RefCell<Option<MetalOpCache>>,
    prepares: Cell<u64>,
    weight_uploads: Cell<u64>,
    lif_uploads: Cell<u64>,
    lif_syncs: Cell<u64>,
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
            prepares: Cell::new(0),
            weight_uploads: Cell::new(0),
            lif_uploads: Cell::new(0),
            lif_syncs: Cell::new(0),
        })
    }

    fn ensure_op(&self, csr: &Csr, weights: &[f32]) -> Result<(), String> {
        let ncols = csr.ncols();

        let mut slot = self.cache.borrow_mut();
        let rebuild = match slot.as_ref() {
            None => true,
            Some(c) => c.ncols != ncols || c.row_ptr != csr.row_ptr || c.col != csr.col,
        };
        if rebuild {
            let inner = csr.to_sparsl();
            let op = self
                .device
                .prepare(&inner, ncols, weights)
                .map_err(|e| e.to_string())?;
            let mut weights_bits = Vec::with_capacity(weights.len());
            overwrite_f32_bits(&mut weights_bits, weights);
            self.prepares.set(self.prepares.get() + 1);
            self.weight_uploads.set(self.weight_uploads.get() + 1);
            *slot = Some(MetalOpCache {
                ncols,
                row_ptr: csr.row_ptr.clone(),
                col: csr.col.clone(),
                weights_bits,
                op,
            });
        } else if let Some(c) = slot.as_mut() {
            if !f32_bits_eq(&c.weights_bits, weights) {
                c.op.set_weights(weights).map_err(|e| e.to_string())?;
                overwrite_f32_bits(&mut c.weights_bits, weights);
                self.weight_uploads.set(self.weight_uploads.get() + 1);
            }
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

    /// Cumulative `(operator_prepares, weight_uploads)` executed on this Metal
    /// handle's cached operator path (`None` on [`Backend::CpuParallel`]).
    #[cfg(feature = "gpu")]
    pub fn metal_cache_stats(&self) -> Option<(u64, u64)> {
        self.metal
            .as_ref()
            .map(|m| (m.prepares.get(), m.weight_uploads.get()))
    }

    /// Cumulative `(lif_state_uploads, lif_state_syncs)` executed on this Metal
    /// handle (`None` on [`Backend::CpuParallel`]).
    #[cfg(feature = "gpu")]
    pub fn metal_lif_transfer_stats(&self) -> Option<(u64, u64)> {
        self.metal
            .as_ref()
            .map(|m| (m.lif_uploads.get(), m.lif_syncs.get()))
    }

    /// Prepare a substrate-matched [`sparsl::SparseOp`] with resident weights.
    ///
    /// Works on both [`Backend::CpuParallel`] and [`Backend::MetalGpu`]. Use
    /// this in multi-tick loops so `SparseOp::spmv` / `SparseOp::fused_spmv_lif`
    /// run with `O(1)` host setup (zero per-tick CSR or weight slice scans),
    /// calling [`sparsl::SparseOp::set_weights`] only when plasticity mutates
    /// weights.
    pub fn prepare_sparse(
        &self,
        csr: &Csr,
        weights: &[f32],
    ) -> Result<sparsl::SparseOp, sparsl::SparsePlanError> {
        let inner = csr.to_sparsl();
        match self.config.backend {
            Backend::CpuParallel => {
                sparsl::Device::cpu_parallel().prepare(&inner, csr.ncols(), weights)
            }
            Backend::MetalGpu => {
                #[cfg(feature = "gpu")]
                {
                    let state = self
                        .metal
                        .as_ref()
                        .expect("MetalGpu SpmvBackend must hold a device");
                    state.device.prepare(&inner, csr.ncols(), weights)
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
                        .with_op_mut(csr, weights, |op| op.spmv(x, y).map_err(|e| e.to_string()))
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

        y.par_iter_mut()
            .enumerate()
            .with_min_len(64)
            .for_each(|(r, y_val)| {
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
                    .with_min_len(64)
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
                    state.lif_uploads.set(state.lif_uploads.get() + 1);
                    state.lif_syncs.set(state.lif_syncs.get() + 1);
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
                    .with_min_len(64)
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
                    state.lif_uploads.set(state.lif_uploads.get() + 1);
                    state.lif_syncs.set(state.lif_syncs.get() + 1);
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

    /// Multi-tick fused CSR SpMV + LIF integration keeping `(v, theta, spikes)`
    /// resident on Metal across all `xs` steps (1 upload + 1 sync instead of
    /// `xs.len()` uploads and `xs.len()` syncs).
    pub fn fused_spmv_lif_steps(
        &self,
        csr: &Csr,
        weights: &[f32],
        xs: &[&[f32]],
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
        let params = sparsl::LifParams::new(decay, v_reset, delta_theta)
            .unwrap_or_else(|e| panic!("invalid LIF params: {e}"));
        for (step_idx, x) in xs.iter().enumerate() {
            assert!(
                x.len() >= csr.ncols(),
                "x[{step_idx}] dimension mismatch: got {}, need >= {}",
                x.len(),
                csr.ncols()
            );
        }
        if xs.is_empty() {
            return;
        }

        match self.config.backend {
            Backend::CpuParallel => {
                let _ = params;
                for x in xs {
                    self.fused_spmv_lif_integrate(
                        csr,
                        weights,
                        x,
                        v,
                        theta,
                        spikes,
                        decay,
                        v_reset,
                        delta_theta,
                    );
                }
            }
            Backend::MetalGpu => {
                #[cfg(feature = "gpu")]
                {
                    let state = self
                        .metal
                        .as_ref()
                        .expect("MetalGpu SpmvBackend must hold a device");
                    let ncols = csr.ncols();
                    state
                        .with_op_mut(csr, weights, |op| {
                            op.write_lif_state(v, theta).map_err(|e| e.to_string())?;
                            for x in xs {
                                op.write_x(&x[..ncols]).map_err(|e| e.to_string())?;
                                op.fused_spmv_lif_resident(params)
                                    .map_err(|e| e.to_string())?;
                            }
                            op.sync_lif_state(v, theta, spikes)
                                .map_err(|e| e.to_string())
                        })
                        .unwrap_or_else(|e| panic!("Metal multi-tick fused SpMV+LIF failed: {e}"));
                    state.lif_uploads.set(state.lif_uploads.get() + 1);
                    state.lif_syncs.set(state.lif_syncs.get() + 1);
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
    state: MetalGpuState,
}

#[cfg(feature = "gpu")]
impl MetalGpuContext {
    /// Open the system Metal device through sparsl, or `None` if unavailable.
    pub fn new() -> Option<Self> {
        MetalGpuState::open().ok().map(|state| Self { state })
    }

    #[cfg(test)]
    fn cache_stats(&self) -> (u64, u64) {
        (self.state.prepares.get(), self.state.weight_uploads.get())
    }

    /// Underlying sparsl device.
    pub fn device(&self) -> &sparsl::Device {
        &self.state.device
    }

    /// Device name for reports.
    pub fn device_name(&self) -> String {
        self.state
            .device
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
        let op = self.state.device.prepare(&inner, csr.ncols(), weights)?;
        self.state.prepares.set(self.state.prepares.get() + 1);
        self.state
            .weight_uploads
            .set(self.state.weight_uploads.get() + 1);
        Ok(op)
    }

    /// `y += A · x` reusing the cached operator when CSR topology is unchanged.
    pub fn spmv(&self, csr: &Csr, weights: &[f32], x: &[f32], y: &mut [f32]) {
        self.state
            .with_op_mut(csr, weights, |op| op.spmv(x, y).map_err(|e| e.to_string()))
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
        self.state
            .device
            .lif_integrate(v, theta, currents, spikes, params)
            .unwrap_or_else(|e| panic!("Metal LIF failed: {e}"));
        self.state.lif_uploads.set(self.state.lif_uploads.get() + 1);
        self.state.lif_syncs.set(self.state.lif_syncs.get() + 1);
    }

    /// Fused SpMV + LIF reusing the cached operator when CSR topology is unchanged.
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
        self.state
            .with_op_mut(csr, weights, |op| {
                op.fused_spmv_lif(x, v, theta, spikes, params)
                    .map_err(|e| e.to_string())
            })
            .unwrap_or_else(|e| panic!("Metal fused SpMV+LIF failed: {e}"));
        self.state.lif_uploads.set(self.state.lif_uploads.get() + 1);
        self.state.lif_syncs.set(self.state.lif_syncs.get() + 1);
    }

    /// Multi-tick fused SpMV + LIF keeping `(v, theta, spikes)` resident on
    /// Metal across all `xs` steps (1 upload + 1 sync).
    pub fn fused_spmv_lif_steps(
        &self,
        csr: &Csr,
        weights: &[f32],
        xs: &[&[f32]],
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
        let params = sparsl::LifParams::new(decay, v_reset, delta_theta)
            .unwrap_or_else(|e| panic!("invalid LIF params: {e}"));
        let ncols = csr.ncols();
        for (step_idx, x) in xs.iter().enumerate() {
            assert!(
                x.len() >= ncols,
                "x[{step_idx}] dimension mismatch: got {}, need >= {ncols}",
                x.len()
            );
        }
        if xs.is_empty() {
            return;
        }
        self.state
            .with_op_mut(csr, weights, |op| {
                op.write_lif_state(v, theta).map_err(|e| e.to_string())?;
                for x in xs {
                    op.write_x(&x[..ncols]).map_err(|e| e.to_string())?;
                    op.fused_spmv_lif_resident(params)
                        .map_err(|e| e.to_string())?;
                }
                op.sync_lif_state(v, theta, spikes)
                    .map_err(|e| e.to_string())
            })
            .unwrap_or_else(|e| panic!("Metal multi-tick fused SpMV+LIF failed: {e}"));
        self.state.lif_uploads.set(self.state.lif_uploads.get() + 1);
        self.state.lif_syncs.set(self.state.lif_syncs.get() + 1);
    }
}

/// One gate shared with the integration tests (`tests/common`).
#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod metal_gate;

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
        if !metal_gate::require_metal(
            "metal_gpu_backend_constructs_when_available",
            Backend::MetalGpu.is_available(),
        ) {
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
        eprintln!(
            "RAN: metal_gpu_backend_constructs_when_available on {}",
            backend.label()
        );
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
        const { assert!(METAL_GPU_DISPATCH_IMPLEMENTED) };
        if !metal_gate::require_metal(
            "metal_gpu_matches_cpu_reference",
            Backend::MetalGpu.is_available(),
        ) {
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
        assert_eq!(gpu.label(), "Metal GPU");
        eprintln!("RAN: metal_gpu_matches_cpu_reference on {}", gpu.label());

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
        const { assert!(METAL_GPU_DISPATCH_IMPLEMENTED) };
        if !metal_gate::require_metal(
            "metal_fused_and_lif_match_cpu_reference",
            Backend::MetalGpu.is_available(),
        ) {
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
            &mut v_l,
            &mut th_l,
            &currents,
            &mut spk_l,
            decay,
            v_reset,
            delta_theta,
        );
        gpu.batch_lif_integrate(
            &mut v_g,
            &mut th_g,
            &currents,
            &mut spk_g,
            decay,
            v_reset,
            delta_theta,
        );
        for i in 0..n {
            assert!((v_l[i] - v_g[i]).abs() <= 1e-5, "LIF v[{i}]");
            assert!((th_l[i] - th_g[i]).abs() <= 1e-5, "LIF theta[{i}]");
            assert_eq!(spk_l[i], spk_g[i], "LIF spike[{i}]");
        }
        assert_eq!(gpu.label(), "Metal GPU");
        eprintln!(
            "RAN: metal_fused_and_lif_match_cpu_reference on {}",
            gpu.label()
        );
    }

    #[test]
    #[should_panic(expected = "refusing to report a pass")]
    fn metal_gate_panics_when_expected_and_missing() {
        metal_gate::metal_gate_decision("t", false, true, false);
    }

    #[test]
    fn metal_gate_skips_only_when_opted_out_or_not_expected() {
        assert!(!metal_gate::metal_gate_decision("t", false, true, true));
        assert!(!metal_gate::metal_gate_decision("t", false, false, false));
        assert!(metal_gate::metal_gate_decision("t", true, true, false));
    }

    #[test]
    fn metal_dispatch_flag_matches_implementation() {
        // Honesty pin: the flag must stay true only while MetalGpu dispatch
        // paths above are real (sparsl-backed), not stubs.
        const { assert!(METAL_GPU_DISPATCH_IMPLEMENTED) };
    }

    #[test]
    fn prepare_sparse_supports_cpu_parallel_and_tracks_set_weights() {
        let (csr, mut weights, x) = tiny_problem();
        let cpu = SpmvBackend::cpu();
        let mut op = cpu
            .prepare_sparse(&csr, &weights)
            .expect("CPU prepare_sparse");
        let mut y_op = vec![0.0; csr.nrows()];
        let mut y_ref = vec![0.0; csr.nrows()];
        op.spmv(&x, &mut y_op).expect("CPU SparseOp::spmv");
        cpu.spmv(&csr, &weights, &x, &mut y_ref);
        assert_eq!(y_op, y_ref);

        weights[0] = -3.5;
        op.set_weights(&weights).expect("CPU SparseOp::set_weights");
        y_op.fill(0.0);
        y_ref.fill(0.0);
        op.spmv(&x, &mut y_op)
            .expect("CPU SparseOp::spmv after set_weights");
        cpu.spmv(&csr, &weights, &x, &mut y_ref);
        assert_eq!(y_op, y_ref);
    }

    /// Repeated ticks over an unchanged `(csr, weights)` pair must keep weights
    /// resident on Metal instead of calling `SparseOp::set_weights` every tick,
    /// while any in-place weight mutation (including 1-ULP or sign-bit changes)
    /// must re-upload and match CPU parity.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_skips_redundant_weight_upload_and_tracks_in_place_weight_edits() {
        if !metal_gate::require_metal(
            "metal_skips_redundant_weight_upload_and_tracks_in_place_weight_edits",
            Backend::MetalGpu.is_available(),
        ) {
            return;
        }
        let (csr, mut weights, x) = tiny_problem();
        let cpu = SpmvBackend::cpu();
        let gpu = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu available");
        let state = gpu.metal.as_ref().expect("MetalGpu state");

        for _ in 0..4 {
            let mut y_gpu = vec![0.0; csr.nrows()];
            gpu.spmv(&csr, &weights, &x, &mut y_gpu);
            let mut y_cpu = vec![0.0; csr.nrows()];
            cpu.spmv(&csr, &weights, &x, &mut y_cpu);
            for (a, b) in y_cpu.iter().zip(y_gpu.iter()) {
                assert!((a - b).abs() <= 1e-5, "GPU/CPU mismatch: {a} vs {b}");
            }
        }
        let mut v_gpu = vec![0.2; csr.nrows()];
        let mut th_gpu = vec![0.5; csr.nrows()];
        let mut spk_gpu = vec![false; csr.nrows()];
        gpu.fused_spmv_lif_integrate(
            &csr,
            &weights,
            &x,
            &mut v_gpu,
            &mut th_gpu,
            &mut spk_gpu,
            0.95,
            0.0,
            0.1,
        );
        assert_eq!(state.prepares.get(), 1, "unchanged CSR must prepare once");
        assert_eq!(
            state.weight_uploads.get(),
            1,
            "unchanged weights must upload once across spmv + fused_spmv_lif_integrate"
        );

        // Adversarial 1: in-place weight edit at the same slice address.
        let ptr_before = weights.as_ptr();
        weights[0] = -7.25;
        assert_eq!(weights.as_ptr(), ptr_before, "must test in-place mutation");
        let mut y_gpu = vec![0.0; csr.nrows()];
        let mut y_cpu = vec![0.0; csr.nrows()];
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        cpu.spmv(&csr, &weights, &x, &mut y_cpu);
        assert_eq!(state.prepares.get(), 1, "CSR unchanged on weight edit");
        assert_eq!(
            state.weight_uploads.get(),
            2,
            "in-place weight edit must trigger a weight upload"
        );
        for (a, b) in y_cpu.iter().zip(y_gpu.iter()) {
            assert!((a - b).abs() <= 1e-5, "stale GPU weights: {a} vs {b}");
        }

        // Adversarial 2: 1-ULP tweak at tail of slice, then repeat unchanged.
        weights[4] = f32::from_bits(weights[4].to_bits() + 1);
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        assert_eq!(state.weight_uploads.get(), 3, "1-ULP change must upload");
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        assert_eq!(
            state.weight_uploads.get(),
            3,
            "second call with 1-ULP weights must hit cache"
        );
    }

    /// Because `Csr::row_ptr` and `Csr::col` are public `Vec<u32>` fields, an
    /// in-place topology mutation keeps the same heap buffer pointers. The
    /// Metal cache must rebuild instead of silently executing stale topology.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_rebuilds_on_in_place_csr_mutation_and_handles_empty_csr() {
        if !metal_gate::require_metal(
            "metal_rebuilds_on_in_place_csr_mutation_and_handles_empty_csr",
            Backend::MetalGpu.is_available(),
        ) {
            return;
        }
        let (mut csr, weights, x) = tiny_problem();
        let cpu = SpmvBackend::cpu();
        let gpu = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu available");

        let mut y_gpu = vec![0.0; csr.nrows()];
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);

        // Mutate row 1's single column from 0 -> 2 in-place (still sorted & valid).
        let col_ptr_before = csr.col.as_ptr();
        csr.col[2] = 2;
        assert_eq!(csr.col.as_ptr(), col_ptr_before, "in-place col edit");

        let mut y_gpu_after = vec![0.0; csr.nrows()];
        let mut y_cpu_after = vec![0.0; csr.nrows()];
        gpu.spmv(&csr, &weights, &x, &mut y_gpu_after);
        cpu.spmv(&csr, &weights, &x, &mut y_cpu_after);
        for (a, b) in y_cpu_after.iter().zip(y_gpu_after.iter()) {
            assert!(
                (a - b).abs() <= 1e-5,
                "in-place CSR edit produced stale GPU output: cpu={a} vs gpu={b}"
            );
        }

        // Empty-edge CSR boundary case (`nnz == 0`, `nrows == 4`).
        let empty = Csr::empty(4);
        let mut y_empty = vec![1.25; 4];
        gpu.spmv(&empty, &[], &[1.0; 4], &mut y_empty);
        assert_eq!(y_empty, vec![1.25; 4]);
    }

    /// Exercises multi-chunk (`>> BIT_EQ_LANES`, non-multiple remainder)
    /// weight-bit checks, `-0.0` vs `+0.0` sign-bit sensitivity, and
    /// fail-closed rejection when an in-place CSR edit makes a column index out
    /// of bounds.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_large_slice_cache_and_fail_closed_invalid_csr() {
        if !metal_gate::require_metal(
            "metal_large_slice_cache_and_fail_closed_invalid_csr",
            Backend::MetalGpu.is_available(),
        ) {
            return;
        }
        let n = 1_000usize;
        let deg = 71usize; // 71,000 edges = 4,437 x 16-lane chunks + 8-element remainder tail
        let adj: Vec<Vec<u32>> = (0..n)
            .map(|r| (0..deg).map(|k| ((r + k * 7) % n) as u32).collect())
            .collect();
        let csr = Csr::from_adjacency(&adj);
        let mut weights = vec![0.0_f32; csr.nnz()];
        for (i, w) in weights.iter_mut().enumerate() {
            *w = ((i % 13) as f32 - 6.0) * 0.125;
        }
        let x: Vec<f32> = (0..n).map(|i| (i % 5) as f32 * 0.25).collect();
        let gpu = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu available");
        let cpu = SpmvBackend::cpu();

        let mut y_gpu = vec![0.0; n];
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        assert_eq!(gpu.metal_cache_stats(), Some((1, 1)));

        // Flip a +0.0 entry in the tail chunk to -0.0: bit-level cache detects it,
        // and a subsequent unchanged call hits cache again.
        let zero_idx = weights
            .iter()
            .rposition(|&w| w.to_bits() == 0.0_f32.to_bits())
            .expect("has +0.0 weight");
        weights[zero_idx] = -0.0_f32;
        y_gpu.fill(0.0);
        let mut y_cpu = vec![0.0; n];
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        cpu.spmv(&csr, &weights, &x, &mut y_cpu);
        assert_eq!(gpu.metal_cache_stats(), Some((1, 2)));
        gpu.spmv(&csr, &weights, &x, &mut y_gpu);
        assert_eq!(gpu.metal_cache_stats(), Some((1, 2)));
        for (a, b) in y_cpu.iter().zip(y_gpu.iter()) {
            assert!((a * 2.0 - b).abs() <= 1e-3, "large parity: 2*{a} vs {b}");
        }

        // Fail-closed: corrupting an already-cached CSR in-place with an
        // out-of-range column must panic in `prepare` instead of silently
        // running the previously cached valid operator.
        let (mut bad_csr, small_w, _) = tiny_problem();
        let big_x = vec![1.0_f32; 16];
        let mut small_y = vec![0.0_f32; bad_csr.nrows()];
        gpu.spmv(&bad_csr, &small_w, &big_x, &mut small_y);
        bad_csr.col[0] = 15; // >= ncols (3), though < big_x.len() (16)
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gpu.spmv(&bad_csr, &small_w, &big_x, &mut small_y);
        }));
        assert!(
            caught.is_err(),
            "in-place invalid CSR column must fail closed via prepare()"
        );
    }

    /// Regression test: `MetalGpuContext::spmv` and `fused_spmv_lif_integrate`
    /// must reuse a cached `SparseOp` across ticks rather than re-preparing CSR
    /// and weight buffers on every call.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_gpu_context_reuses_cached_operator() {
        if !metal_gate::require_metal(
            "metal_gpu_context_reuses_cached_operator",
            Backend::MetalGpu.is_available(),
        ) {
            return;
        }
        let ctx = MetalGpuContext::new().expect("MetalGpuContext available");
        let (csr, mut weights, x) = tiny_problem();
        let mut y = vec![0.0_f32; csr.nrows()];
        for _ in 0..3 {
            ctx.spmv(&csr, &weights, &x, &mut y);
        }
        let mut v = vec![0.2_f32; csr.nrows()];
        let mut theta = vec![1.0_f32; csr.nrows()];
        let mut spikes = vec![false; csr.nrows()];
        for _ in 0..3 {
            ctx.fused_spmv_lif_integrate(
                &csr,
                &weights,
                &x,
                &mut v,
                &mut theta,
                &mut spikes,
                0.9,
                0.0,
                0.1,
            );
        }
        assert_eq!(
            ctx.cache_stats(),
            (1, 1),
            "6 ticks on static CSR+weights must prepare once and upload weights once"
        );

        // Mutating weights updates weights in place without rebuilding CSR buffers.
        weights[0] += 0.5;
        y.fill(0.0);
        ctx.spmv(&csr, &weights, &x, &mut y);
        let mut y_cpu = vec![0.0_f32; csr.nrows()];
        SpmvBackend::cpu().spmv(&csr, &weights, &x, &mut y_cpu);
        for (a, b) in y_cpu.iter().zip(y.iter()) {
            assert!((a - b).abs() <= 1e-5, "mutated weight parity: {a} vs {b}");
        }
        assert_eq!(
            ctx.cache_stats(),
            (1, 2),
            "weight-only update must call set_weights once without rebuilding CSR topology"
        );
    }

    /// Multi-tick `fused_spmv_lif_steps` keeps `(v, theta)` resident on Metal
    /// across all `T` steps (1 upload + 1 sync instead of `T` uploads + `T`
    /// syncs) and agrees with CPU step-by-step execution.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_fused_spmv_lif_steps_uploads_and_syncs_state_once_and_matches_cpu() {
        if !metal_gate::require_metal(
            "metal_fused_spmv_lif_steps_uploads_and_syncs_state_once_and_matches_cpu",
            Backend::MetalGpu.is_available(),
        ) {
            return;
        }
        let (csr, weights, _) = tiny_problem();
        let n = csr.nrows();
        let frames: Vec<Vec<f32>> = (0..16usize)
            .map(|t| {
                (0..csr.ncols())
                    .map(|c| ((t * 3 + c * 5) % 7) as f32 * 0.15)
                    .collect()
            })
            .collect();
        let xs: Vec<&[f32]> = frames.iter().map(Vec::as_slice).collect();

        let cpu = SpmvBackend::cpu();
        let gpu = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect("MetalGpu available");

        let mut v_cpu = vec![0.15_f32; n];
        let mut th_cpu = vec![0.8_f32; n];
        let mut spk_cpu = vec![false; n];
        cpu.fused_spmv_lif_steps(
            &csr,
            &weights,
            &xs,
            &mut v_cpu,
            &mut th_cpu,
            &mut spk_cpu,
            0.92,
            0.0,
            0.12,
        );

        let mut v_gpu = vec![0.15_f32; n];
        let mut th_gpu = vec![0.8_f32; n];
        let mut spk_gpu = vec![false; n];
        // Empty slice is a zero-transfer no-op.
        gpu.fused_spmv_lif_steps(
            &csr,
            &weights,
            &[],
            &mut v_gpu,
            &mut th_gpu,
            &mut spk_gpu,
            0.92,
            0.0,
            0.12,
        );
        assert_eq!(gpu.metal_lif_transfer_stats(), Some((0, 0)));

        gpu.fused_spmv_lif_steps(
            &csr,
            &weights,
            &xs,
            &mut v_gpu,
            &mut th_gpu,
            &mut spk_gpu,
            0.92,
            0.0,
            0.12,
        );
        assert_eq!(gpu.metal_cache_stats(), Some((1, 1)));
        assert_eq!(
            gpu.metal_lif_transfer_stats(),
            Some((1, 1)),
            "16 resident LIF ticks must upload (v, theta) once and sync once"
        );
        for i in 0..n {
            assert!(
                (v_cpu[i] - v_gpu[i]).abs() <= 1e-4,
                "v[{i}] mismatch: cpu={} vs gpu={}",
                v_cpu[i],
                v_gpu[i]
            );
            assert!(
                (th_cpu[i] - th_gpu[i]).abs() <= 1e-4,
                "theta[{i}] mismatch: cpu={} vs gpu={}",
                th_cpu[i],
                th_gpu[i]
            );
            assert_eq!(spk_cpu[i], spk_gpu[i], "spike[{i}] mismatch");
        }

        // Fail-closed: an undersized x at a later step must be rejected up front
        // before mutating state or issuing partial GPU uploads.
        let bad_x = [0.5_f32];
        let bad_xs: [&[f32]; 2] = [xs[0], &bad_x];
        let v_before = v_gpu.clone();
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gpu.fused_spmv_lif_steps(
                &csr,
                &weights,
                &bad_xs,
                &mut v_gpu,
                &mut th_gpu,
                &mut spk_gpu,
                0.92,
                0.0,
                0.12,
            );
        }));
        assert!(caught.is_err(), "undersized step input must panic up front");
        assert_eq!(v_gpu, v_before);
        assert_eq!(gpu.metal_lif_transfer_stats(), Some((1, 1)));
    }
}
