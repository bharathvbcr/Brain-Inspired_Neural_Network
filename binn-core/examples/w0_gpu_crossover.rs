//! W0: does Metal GPU CSR SpMV beat the rayon CPU path at BINN's sizes?
//!
//! Thin labelled-backend driver over [`binn_core::SpmvBackend`]. Resident and
//! per-tick arms use the same Metal substrate as `Backend::MetalGpu` (sparsl
//! `SparseOp`), not a second kernel stack.
//!
//! Three arms:
//!
//! * `cpu`            — `SpmvBackend::cpu()`
//! * `gpu (resident)` — weights/`x` already on device; dispatch + sync only
//! * `gpu (per-tick)` — resident matrix; host `x` write and `y` read each call
//!
//! Run: `cargo run --release -p binn-core --features gpu --example w0_gpu_crossover`

use std::time::{Duration, Instant};

use binn_core::{Backend, Csr, Rng, SpmvBackend, SpmvBackendConfig};

const SIZES: &[usize] = &[1000, 5000, 10000, 20000];
const RAMP: usize = 50;

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

fn ms_per_iter(total: Duration, iters: usize) -> f64 {
    total.as_secs_f64() * 1000.0 / iters as f64
}

fn time_cpu(cpu: &SpmvBackend, csr: &Csr, weights: &[f32], x: &[f32], n: usize, iters: usize) -> f64 {
    let mut y = vec![0.0f32; n];
    let start = Instant::now();
    for _ in 0..iters {
        y.fill(0.0);
        cpu.spmv(csr, weights, x, &mut y);
    }
    ms_per_iter(start.elapsed(), iters)
}

fn time_gpu_per_tick(
    gpu: &SpmvBackend,
    csr: &Csr,
    weights: &[f32],
    x: &[f32],
    n: usize,
    iters: usize,
) -> f64 {
    let mut y = vec![0.0f32; n];
    let start = Instant::now();
    for _ in 0..iters {
        y.fill(0.0);
        gpu.spmv(csr, weights, x, &mut y);
    }
    ms_per_iter(start.elapsed(), iters)
}

fn time_gpu_resident(op: &sparsl::SparseOp, x: &[f32], n: usize, iters: usize) -> f64 {
    let mut y = vec![0.0f32; n];
    op.write_x(x).expect("write_x");
    op.write_y(&y).expect("write_y");
    let start = Instant::now();
    for _ in 0..iters {
        op.write_y(&vec![0.0f32; n]).expect("zero y");
        op.spmv_resident().expect("spmv_resident");
    }
    op.sync_y(&mut y).expect("sync_y");
    ms_per_iter(start.elapsed(), iters)
}

fn main() {
    if !Backend::MetalGpu.is_available() {
        eprintln!(
            "Backend::MetalGpu unavailable: {:?}. W0 cannot run.",
            Backend::MetalGpu.unavailable_reason()
        );
        std::process::exit(1);
    }

    let gpu = SpmvBackend::try_new(SpmvBackendConfig {
        backend: Backend::MetalGpu,
        batch_size: 1024,
    })
    .expect("MetalGpu constructible when is_available");
    let device = gpu.metal_device().expect("MetalGpu holds a device");
    println!(
        "Metal device: {}",
        device.device_name().unwrap_or_else(|| "Metal".to_string())
    );
    println!("backend label: {}\n", gpu.label());

    println!(
        "| N | nnz | cpu (ms) | gpu resident (ms) | gpu per-tick (ms) | upload (ms) | \
         resident speedup | per-tick speedup | max |Δ| | pass spread |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|");

    let cpu = SpmvBackend::cpu();

    for &n in SIZES {
        let (csr, weights, x) = build_problem(n);
        let nnz = csr.nnz();
        let iters = if nnz > 2_000_000 { 50 } else { 200 };

        let upload_start = Instant::now();
        let op = gpu
            .prepare_sparse(&csr, &weights)
            .expect("prepare SparseOp");
        let upload = upload_start.elapsed();

        // ---- Parity first -------------------------------------------------
        let mut y_cpu = vec![0.0f32; n];
        cpu.spmv(&csr, &weights, &x, &mut y_cpu);

        let mut y_gpu = vec![0.0f32; n];
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
        let max_diff = y_cpu
            .iter()
            .zip(y_gpu.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            max_diff <= tol,
            "GPU/CPU mismatch at N={n}: max |Δ| = {max_diff} (tol={tol})"
        );

        let _ = time_cpu(&cpu, &csr, &weights, &x, n, RAMP);
        let _ = time_gpu_resident(&op, &x, n, RAMP);
        let _ = time_gpu_per_tick(&gpu, &csr, &weights, &x, n, RAMP);

        let cpu_a = time_cpu(&cpu, &csr, &weights, &x, n, iters);
        let res_a = time_gpu_resident(&op, &x, n, iters);
        let tick_a = time_gpu_per_tick(&gpu, &csr, &weights, &x, n, iters);
        let tick_b = time_gpu_per_tick(&gpu, &csr, &weights, &x, n, iters);
        let res_b = time_gpu_resident(&op, &x, n, iters);
        let cpu_b = time_cpu(&cpu, &csr, &weights, &x, n, iters);

        let cpu_ms = cpu_a.min(cpu_b);
        let gpu_resident_ms = res_a.min(res_b);
        let gpu_tick_ms = tick_a.min(tick_b);
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
