//! W0: does Metal GPU CSR SpMV actually beat the rayon CPU path at BINN's sizes?
//!
//! This is the gate for the Metal dispatch workstream. Before implementing
//! `MetalGpuContext::spmv` / `batch_lif_integrate` / `fused_spmv_lif_integrate`
//! for real, measure whether a GPU arm can win at all on the problem shapes the
//! throughput benchmarks actually use.
//!
//! Deliberately kept **outside** [`binn_core::SpmvBackend`]:
//! `METAL_GPU_DISPATCH_IMPLEMENTED` stays `false` and no GPU-labelled backend
//! handle is constructed anywhere. This example encodes its own dispatch using
//! the public device/queue/pipeline handles on [`MetalGpuContext`], so the
//! honesty invariant in `metal_backend.rs` is untouched by the measurement.
//!
//! Three arms, because they answer different questions:
//!
//! * `cpu`            — `SpmvBackend::cpu()`, the incumbent.
//! * `gpu (resident)` — dispatch only, every buffer already on the device. This
//!   is the *upper bound* on any GPU win; nothing real can beat it.
//! * `gpu (per-tick)` — resident matrix, but `x` written and `y` read back each
//!   call. This is what an SNN tick loop actually costs, because the spike
//!   vector changes every tick and the result is consumed on the host.
//!
//! Run: `cargo run --release -p binn-core --features gpu --example w0_gpu_crossover`

use std::ffi::c_void;
use std::time::{Duration, Instant};

use binn_core::metal_backend::MetalGpuContext;
use binn_core::{Csr, Rng, SpmvBackend};
use metal::{Buffer, MTLResourceOptions, MTLSize};
use objc::rc::autoreleasepool;

/// Network sizes to sweep. 1000/5000/10000 are the sizes `c1_enhanced` and
/// `multi_area_scaling` benchmark; 20000 is included to locate a crossover if
/// it sits above the range those experiments currently cover.
const SIZES: &[usize] = &[1000, 5000, 10000, 20000];

/// Matches the synthetic topology both throughput benchmarks build, so a
/// crossover measured here transfers to them directly.
///
/// The benchmarks fill `weights` with a constant `0.1` and `x` with a constant
/// `1.0`. That makes every row a sum of identical terms, under which CPU and
/// GPU agree bitwise no matter how either one reduces — a parity check against
/// it proves nothing. Values here are seeded-random so the parity assertion has
/// something to fail on. The topology, nnz and access pattern are unchanged, so
/// the timings still describe the benchmarks' problem.
fn build_problem(n: usize) -> (Csr, Vec<f32>, Vec<f32>) {
    let density = 0.05;
    let nnz_per_row = ((n as f32) * density) as usize;
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (r, row) in adj.iter_mut().enumerate() {
        for i in 0..nnz_per_row {
            row.push(((r + i * 3) % n) as u32);
        }
    }
    let csr = Csr::from_adjacency(&adj);
    let mut rng = Rng::new(0x5713_2026);
    let weights = (0..csr.nnz()).map(|_| rng.next_f32() - 0.5).collect();
    let x = (0..n).map(|_| rng.next_f32() - 0.5).collect();
    (csr, weights, x)
}

fn buffer_from_slice<T>(device: &metal::Device, data: &[T]) -> Buffer {
    assert!(!data.is_empty(), "Metal rejects zero-length buffers");
    device.new_buffer_with_data(
        data.as_ptr() as *const c_void,
        std::mem::size_of_val(data) as u64,
        MTLResourceOptions::StorageModeShared,
    )
}

/// CSR SpMV with all operands resident on the device.
struct ResidentSpmv<'a> {
    ctx: &'a MetalGpuContext,
    row_ptr: Buffer,
    col: Buffer,
    values: Buffer,
    x: Buffer,
    y: Buffer,
    n: usize,
    threads_per_group: u64,
}

impl<'a> ResidentSpmv<'a> {
    /// Uploads every operand once. Returns the handle and the upload cost, so
    /// the report can separate "what caching buys" from "what it costs".
    fn new(ctx: &'a MetalGpuContext, csr: &Csr, weights: &[f32], x: &[f32]) -> (Self, Duration) {
        let n = csr.nrows();
        let start = Instant::now();
        let row_ptr = buffer_from_slice(&ctx.device, &csr.row_ptr);
        let col = buffer_from_slice(&ctx.device, &csr.col);
        let values = buffer_from_slice(&ctx.device, weights);
        let x_buf = buffer_from_slice(&ctx.device, x);
        let y = ctx.device.new_buffer(
            (n * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let upload = start.elapsed();

        // `dispatch_threads` uses non-uniform threadgroups, so N need not be a
        // multiple of the group size and the kernel needs no bounds guard.
        let max_group = ctx.csr_spmv_kernel.max_total_threads_per_threadgroup();
        let width = ctx.csr_spmv_kernel.thread_execution_width();
        let threads_per_group = (width * 4).min(max_group).max(width);

        (
            Self {
                ctx,
                row_ptr,
                col,
                values,
                x: x_buf,
                y,
                n,
                threads_per_group,
            },
            upload,
        )
    }

    fn dispatch(&self) {
        let cb = self.ctx.command_queue.new_command_buffer();
        let enc = cb.new_compute_command_encoder();
        enc.set_compute_pipeline_state(&self.ctx.csr_spmv_kernel);
        enc.set_buffer(0, Some(&self.row_ptr), 0);
        enc.set_buffer(1, Some(&self.col), 0);
        enc.set_buffer(2, Some(&self.values), 0);
        enc.set_buffer(3, Some(&self.x), 0);
        enc.set_buffer(4, Some(&self.y), 0);
        enc.dispatch_threads(
            MTLSize {
                width: self.n as u64,
                height: 1,
                depth: 1,
            },
            MTLSize {
                width: self.threads_per_group,
                height: 1,
                depth: 1,
            },
        );
        enc.end_encoding();
        cb.commit();
        cb.wait_until_completed();
    }

    /// Host -> device copy of the spike vector. Unified memory, so this is a
    /// memcpy into shared storage rather than a bus transfer.
    fn write_x(&self, x: &[f32]) {
        assert_eq!(x.len(), self.n);
        // SAFETY: `self.x` was allocated at `n * size_of::<f32>()` bytes with
        // shared storage, so `contents()` is a valid, Metal-aligned host
        // pointer for that many floats; the assert above pins `x.len() == n`,
        // so the copy cannot overrun either side. `x` is a separate borrow, so
        // source and destination cannot overlap. Every caller runs this between
        // dispatches, after `wait_until_completed`, so no GPU command is
        // reading the buffer concurrently.
        // The invariant: `x.len() == n`, no aliasing, GPU idle.
        unsafe {
            std::ptr::copy_nonoverlapping(x.as_ptr(), self.x.contents() as *mut f32, x.len());
        }
    }

    fn read_y(&self, out: &mut [f32]) {
        assert_eq!(out.len(), self.n);
        // SAFETY: as `write_x`, in the other direction. `self.y` holds `n`
        // floats and the assert pins `out.len() == n`. Callers read only after
        // `wait_until_completed`, so the GPU has finished writing; a read that
        // did race would observe a torn value rather than undefined behaviour,
        // and the comparison against the CPU reference would fail rather than
        // the read.
        // The invariant: `out.len() == n`, no aliasing, GPU idle.
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.y.contents() as *const f32,
                out.as_mut_ptr(),
                out.len(),
            );
        }
    }

    fn zero_y(&self) {
        // SAFETY: `self.y` holds exactly `n` floats and nothing else aliases
        // it. `write_bytes` counts elements of the pointee type, so `self.n`
        // here is `n` floats, not `n` bytes — zeroing the whole buffer and no
        // more. Called between dispatches, after the previous one completed.
        // The invariant: `y` holds `n` floats, no aliasing, GPU idle.
        unsafe {
            std::ptr::write_bytes(self.y.contents() as *mut f32, 0, self.n);
        }
    }
}

fn ms_per_iter(total: Duration, iters: usize) -> f64 {
    total.as_secs_f64() * 1000.0 / iters as f64
}

/// Iterations run through every arm before any arm is timed.
///
/// Without this, the first-timed arm pays for the GPU clock ramp and the last
/// one does not, which shows up as the physically impossible result that adding
/// a host memcpy to each iteration makes it *faster*. Every arm is exercised
/// first, then all arms are timed twice in opposite orders.
const RAMP: usize = 50;

fn time_cpu(
    cpu: &SpmvBackend,
    csr: &Csr,
    weights: &[f32],
    x: &[f32],
    n: usize,
    iters: usize,
) -> f64 {
    let mut y = vec![0.0f32; n];
    let start = Instant::now();
    for _ in 0..iters {
        cpu.spmv(csr, weights, x, &mut y);
    }
    ms_per_iter(start.elapsed(), iters)
}

fn time_gpu_resident(gpu: &ResidentSpmv, iters: usize) -> f64 {
    autoreleasepool(|| {
        let start = Instant::now();
        for _ in 0..iters {
            gpu.dispatch();
        }
        ms_per_iter(start.elapsed(), iters)
    })
}

fn time_gpu_per_tick(gpu: &ResidentSpmv, x: &[f32], n: usize, iters: usize) -> f64 {
    let mut y_out = vec![0.0f32; n];
    autoreleasepool(|| {
        let start = Instant::now();
        for _ in 0..iters {
            gpu.write_x(x);
            gpu.dispatch();
            gpu.read_y(&mut y_out);
        }
        ms_per_iter(start.elapsed(), iters)
    })
}

fn main() {
    let Some(ctx) = MetalGpuContext::new() else {
        eprintln!(
            "MetalGpuContext::new() returned None: no Metal device, or one of the six MSL \
             kernels failed to compile. W0 cannot run."
        );
        std::process::exit(1);
    };
    println!("Metal device: {}", ctx.device.name());
    println!(
        "csr_spmv_kernel: thread_execution_width={}, max_threads_per_threadgroup={}\n",
        ctx.csr_spmv_kernel.thread_execution_width(),
        ctx.csr_spmv_kernel.max_total_threads_per_threadgroup()
    );

    println!(
        "| N | nnz | cpu (ms) | gpu resident (ms) | gpu per-tick (ms) | upload (ms) | \
         resident speedup | per-tick speedup | max |Δ| | pass spread |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|");

    for &n in SIZES {
        let (csr, weights, x) = build_problem(n);
        let nnz = csr.nnz();
        let iters = if nnz > 2_000_000 { 50 } else { 200 };

        let cpu = SpmvBackend::cpu();
        let (gpu, upload) = ResidentSpmv::new(&ctx, &csr, &weights, &x);

        // ---- Parity first. Timing a wrong kernel is worse than not timing. --
        let mut y_cpu = vec![0.0f32; n];
        cpu.spmv(&csr, &weights, &x, &mut y_cpu);
        gpu.zero_y();
        gpu.dispatch();
        let mut y_gpu = vec![0.0f32; n];
        gpu.read_y(&mut y_gpu);
        let max_diff = y_cpu
            .iter()
            .zip(y_gpu.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            max_diff < 1e-3,
            "GPU/CPU mismatch at N={n}: max |Δ| = {max_diff}; timings below would be meaningless"
        );

        // ---- Ramp every arm before timing any of them --------------------
        let _ = time_cpu(&cpu, &csr, &weights, &x, n, RAMP);
        let _ = time_gpu_resident(&gpu, RAMP);
        let _ = time_gpu_per_tick(&gpu, &x, n, RAMP);

        // ---- Time twice, in opposite orders, and take the best of each ------
        let cpu_a = time_cpu(&cpu, &csr, &weights, &x, n, iters);
        let res_a = time_gpu_resident(&gpu, iters);
        let tick_a = time_gpu_per_tick(&gpu, &x, n, iters);
        let tick_b = time_gpu_per_tick(&gpu, &x, n, iters);
        let res_b = time_gpu_resident(&gpu, iters);
        let cpu_b = time_cpu(&cpu, &csr, &weights, &x, n, iters);

        let cpu_ms = cpu_a.min(cpu_b);
        let gpu_resident_ms = res_a.min(res_b);
        let gpu_tick_ms = tick_a.min(tick_b);
        // Ratio of the two passes per arm. Anything far from 1.00 means the
        // ordering still matters and the numbers below are not stable.
        let spread = [(cpu_a, cpu_b), (res_a, res_b), (tick_a, tick_b)]
            .iter()
            .map(|(a, b)| a.max(*b) / a.min(*b))
            .fold(1.0f64, f64::max);

        println!(
            "| {n} | {nnz} | {cpu_ms:.3} | {gpu_resident_ms:.3} | {gpu_tick_ms:.3} | {:.3} | \
             {:.2}x | {:.2}x | {max_diff:.2e} | {spread:.2} |",
            upload.as_secs_f64() * 1000.0,
            cpu_ms / gpu_resident_ms,
            cpu_ms / gpu_tick_ms,
        );
    }
}
