//! Parallel backends for CSR SpMV and LIF integration.
//!
//! Provides parallel matrix-vector multiplication (`y = A · x`) and LIF membrane
//! integration for large-scale cell networks.
//!
//! # Backend honesty invariant
//!
//! This module previously exposed a `use_gpu: bool` that [`SpmvBackend::spmv`]
//! never read, so a "GPU" backend and a "CPU" backend executed byte-identical
//! rayon code. Benchmarks built on it reported ~1.00x "speedups" that were pure
//! measurement noise, and reports labelled CPU numbers as GPU numbers.
//!
//! The fix is structural, not cosmetic:
//!
//! 1. The backend selector is an explicit [`Backend`] enum, not a bool.
//! 2. [`METAL_GPU_DISPATCH_IMPLEMENTED`] is the single source of truth for
//!    whether real Metal dispatch exists. It is currently `false`.
//! 3. [`SpmvBackend::try_new`] **refuses to construct** an unimplemented
//!    backend. There is no code path that yields a `Backend::MetalGpu` handle
//!    which silently runs on the CPU.
//! 4. [`SpmvBackend::label`] returns the backend that *actually executed*, so a
//!    report cannot mislabel a run even if the caller is confused.
//!
//! To land real Metal: implement the dispatch bodies in [`MetalGpuContext`],
//! then flip [`METAL_GPU_DISPATCH_IMPLEMENTED`] to `true`. The guard tests at
//! the bottom of this file will start exercising the GPU path automatically.

#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

use std::fmt;

use crate::sparse::Csr;

/// Whether native Metal GPU kernel dispatch is actually implemented end-to-end.
///
/// **Do not flip this to `true` until [`MetalGpuContext::spmv`],
/// [`MetalGpuContext::batch_lif_integrate`] and
/// [`MetalGpuContext::fused_spmv_lif_integrate`] perform real GPU dispatch and
/// pass `metal_gpu_matches_cpu_reference`.**
///
/// While `false`, [`Backend::MetalGpu`] is unconstructible and any benchmark
/// that asks for it fails loudly instead of quietly timing the CPU path.
pub const METAL_GPU_DISPATCH_IMPLEMENTED: bool = false;

/// Which execution substrate a backend handle actually runs on.
///
/// This is deliberately an enum rather than a `bool`: a bool invites the
/// "flag is set but never read" failure mode that this module previously had.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Backend {
    /// Multi-threaded CPU execution via rayon. Always available.
    CpuParallel,
    /// Native Metal GPU dispatch. Requires the `gpu` cargo feature *and*
    /// [`METAL_GPU_DISPATCH_IMPLEMENTED`].
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
    pub const fn is_available(self) -> bool {
        match self {
            Backend::CpuParallel => true,
            Backend::MetalGpu => METAL_GPU_DISPATCH_IMPLEMENTED && cfg!(feature = "gpu"),
        }
    }

    /// Why the backend is unavailable, or `None` if it is available.
    pub fn unavailable_reason(self) -> Option<&'static str> {
        match self {
            Backend::CpuParallel => None,
            Backend::MetalGpu => {
                if !cfg!(feature = "gpu") {
                    Some("binn-core was built without the `gpu` cargo feature")
                } else if !METAL_GPU_DISPATCH_IMPLEMENTED {
                    Some(
                        "Metal kernel dispatch is not implemented \
                         (METAL_GPU_DISPATCH_IMPLEMENTED == false); \
                         refusing to fall back to CPU under a GPU label",
                    )
                } else {
                    None
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

/// Parallel SpMV and LIF integration execution engine.
///
/// A handle can only exist for a backend that is actually available, so
/// `backend.label()` is always a truthful description of what ran.
#[derive(Clone, Debug)]
pub struct SpmvBackend {
    config: SpmvBackendConfig,
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
        Ok(Self { config })
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
            // Unreachable while `try_new` guards construction; kept as a
            // belt-and-braces guard so adding a constructor cannot reintroduce
            // the silent-fallback bug.
            Backend::MetalGpu => unreachable!(
                "SpmvBackend holds Backend::MetalGpu but dispatch is unimplemented; \
                 SpmvBackend::try_new must reject it"
            ),
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
        assert_eq!(
            self.config.backend,
            Backend::CpuParallel,
            "only the CPU backend is implemented"
        );

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
        assert_eq!(
            self.config.backend,
            Backend::CpuParallel,
            "only the CPU backend is implemented"
        );

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
}

/// Backend arms a throughput benchmark may legitimately compare.
///
/// Returns only substrates that are actually available, so a benchmark loop
/// built on this can never emit a "GPU vs CPU" table where both arms ran on the
/// CPU. When Metal lands, this starts returning two arms with no change to the
/// benchmark call sites.
pub fn benchmarkable_backends() -> Vec<Backend> {
    [Backend::CpuParallel, Backend::MetalGpu]
        .into_iter()
        .filter(|b| b.is_available())
        .collect()
}

// ---------------------------------------------------------------------------
// Real Metal dispatch scaffold
// ---------------------------------------------------------------------------

/// Metal device/kernel handles.
///
/// # Status: kernels compile, dispatch is NOT implemented
///
/// [`MetalGpuContext::new`] genuinely compiles the MSL sources, so a build with
/// `--features gpu` proves the kernels are syntactically valid. The dispatch
/// bodies below are unimplemented.
///
/// To finish, for each method: allocate `metal::Buffer`s with
/// `device.new_buffer_with_data` for the read-only inputs and
/// `new_buffer(len, MTLResourceOptions::StorageModeShared)` for outputs; create
/// a command buffer from `command_queue`; set the corresponding pipeline state
/// and buffers on a compute encoder; dispatch with a threadgroup size derived
/// from `pipeline.thread_execution_width()`; `commit()` and
/// `wait_until_completed()`; then copy the output buffer back into the `&mut`
/// slice. Once all three pass `metal_gpu_matches_cpu_reference`, flip
/// [`METAL_GPU_DISPATCH_IMPLEMENTED`] to `true`.
#[cfg(feature = "gpu")]
pub struct MetalGpuContext {
    pub device: metal::Device,
    pub command_queue: metal::CommandQueue,
    pub csr_spmv_kernel: metal::ComputePipelineState,
    pub lif_integrate_kernel: metal::ComputePipelineState,
    pub lif_spmv_fused_simdgroup_kernel: metal::ComputePipelineState,
    pub elig_decay_kernel: metal::ComputePipelineState,
    pub margin_credit_kernel: metal::ComputePipelineState,
    pub fused_training_step_kernel: metal::ComputePipelineState,
}

#[cfg(feature = "gpu")]
const METAL_DISPATCH_TODO: &str = "Metal kernel dispatch is not implemented. \
     This is a scaffold: implement the encoder body, verify against \
     `SpmvBackend::cpu()` in `metal_gpu_matches_cpu_reference`, then set \
     METAL_GPU_DISPATCH_IMPLEMENTED = true. Until then no caller may treat \
     this as a GPU result.";

#[cfg(feature = "gpu")]
impl MetalGpuContext {
    /// Creates a new Metal GPU context by discovering the system default device
    /// and compiling the MSL kernels.
    pub fn new() -> Option<Self> {
        let device = metal::Device::system_default()?;
        let command_queue = device.new_command_queue();

        let source = include_str!("metal_spmv.metal");
        let compile_options = metal::CompileOptions::new();
        let library = device
            .new_library_with_source(source, &compile_options)
            .ok()?;

        let get_pipeline = |name: &str| -> Option<metal::ComputePipelineState> {
            let function = library.get_function(name, None).ok()?;
            device
                .new_compute_pipeline_state_with_function(&function)
                .ok()
        };

        let csr_spmv_kernel = get_pipeline("csr_spmv_kernel")?;
        let lif_integrate_kernel = get_pipeline("lif_integrate_kernel")?;
        let lif_spmv_fused_simdgroup_kernel = get_pipeline("lif_spmv_fused_simdgroup_kernel")?;
        let elig_decay_kernel = get_pipeline("elig_decay_kernel")?;
        let margin_credit_kernel = get_pipeline("margin_credit_kernel")?;

        let source_train = include_str!("metal_training.metal");
        let library_train = device
            .new_library_with_source(source_train, &compile_options)
            .ok()?;
        let get_pipeline_train = |name: &str| -> Option<metal::ComputePipelineState> {
            let function = library_train.get_function(name, None).ok()?;
            device
                .new_compute_pipeline_state_with_function(&function)
                .ok()
        };
        let fused_training_step_kernel = get_pipeline_train("fused_training_step_kernel")?;

        Some(Self {
            device,
            command_queue,
            csr_spmv_kernel,
            lif_integrate_kernel,
            lif_spmv_fused_simdgroup_kernel,
            elig_decay_kernel,
            margin_credit_kernel,
            fused_training_step_kernel,
        })
    }

    /// TODO(metal): encode `csr_spmv_kernel`. See type-level docs.
    pub fn spmv(&self, _csr: &Csr, _weights: &[f32], _x: &[f32], _y: &mut [f32]) {
        todo!("{}", METAL_DISPATCH_TODO)
    }

    /// TODO(metal): encode `lif_integrate_kernel`. See type-level docs.
    pub fn batch_lif_integrate(
        &self,
        _v: &mut [f32],
        _theta: &mut [f32],
        _currents: &[f32],
        _spikes: &mut [bool],
        _decay: f32,
        _v_reset: f32,
        _delta_theta: f32,
    ) {
        todo!("{}", METAL_DISPATCH_TODO)
    }

    /// TODO(metal): encode `lif_spmv_fused_simdgroup_kernel`. See type-level docs.
    pub fn fused_spmv_lif_integrate(
        &self,
        _csr: &Csr,
        _weights: &[f32],
        _x: &[f32],
        _v: &mut [f32],
        _theta: &mut [f32],
        _spikes: &mut [bool],
        _decay: f32,
        _v_reset: f32,
        _delta_theta: f32,
    ) {
        todo!("{}", METAL_DISPATCH_TODO)
    }

    /// TODO(metal): encode `elig_decay_kernel`. See type-level docs.
    pub fn elig_decay(
        &self,
        _eligibility: &mut [f32],
        _elig_slow: &mut [f32],
        _dt: f32,
        _tau_fast: f32,
        _tau_slow: f32,
        _alpha: f32,
    ) {
        todo!("{}", METAL_DISPATCH_TODO)
    }

    /// TODO(metal): encode `margin_credit_kernel`. See type-level docs.
    pub fn margin_credit(
        &self,
        _membranes: &[f32],
        _weights_out: &mut [f32],
        _v_boundary: f32,
        _inv_2sigma2: f32,
    ) {
        todo!("{}", METAL_DISPATCH_TODO)
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

    /// Regression guard for the "GPU flag never read" bug.
    ///
    /// While Metal dispatch is unimplemented, it must be impossible to obtain a
    /// backend handle labelled "Metal GPU". If this test ever fails, some code
    /// path is handing out a GPU-labelled handle that runs on the CPU.
    #[test]
    fn unimplemented_gpu_backend_is_unconstructible() {
        if METAL_GPU_DISPATCH_IMPLEMENTED && cfg!(feature = "gpu") {
            return; // real dispatch landed; covered by the parity test below
        }
        let err = SpmvBackend::try_new(SpmvBackendConfig {
            backend: Backend::MetalGpu,
            batch_size: 1024,
        })
        .expect_err("Backend::MetalGpu must not be constructible without real dispatch");
        assert_eq!(err.requested, Backend::MetalGpu);
        assert!(!Backend::MetalGpu.is_available());
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
        if !METAL_GPU_DISPATCH_IMPLEMENTED {
            assert_eq!(
                arms.len(),
                1,
                "only the CPU arm may be benchmarked until Metal dispatch lands"
            );
        }
    }

    #[test]
    fn labels_describe_the_executing_substrate() {
        assert_eq!(SpmvBackend::cpu().label(), "CPU parallel (rayon)");
        assert_eq!(SpmvBackend::cpu().backend(), Backend::CpuParallel);
    }

    /// Activates automatically when Metal dispatch lands.
    #[cfg(feature = "gpu")]
    #[test]
    fn metal_gpu_matches_cpu_reference() {
        if !METAL_GPU_DISPATCH_IMPLEMENTED {
            return;
        }
        let (csr, weights, x) = tiny_problem();
        let mut y_cpu = vec![0.0; 3];
        SpmvBackend::cpu().spmv(&csr, &weights, &x, &mut y_cpu);

        let ctx = MetalGpuContext::new().expect("Metal device unavailable");
        let mut y_gpu = vec![0.0; 3];
        ctx.spmv(&csr, &weights, &x, &mut y_gpu);

        for (a, b) in y_cpu.iter().zip(y_gpu.iter()) {
            assert!((a - b).abs() < 1e-4, "GPU/CPU mismatch: {a} vs {b}");
        }
    }
}
