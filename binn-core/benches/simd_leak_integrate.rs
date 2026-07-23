//! Criterion benches for `binn-core` hot paths (U02 / U03 / GC5).
//!
//! Lives at `tests/determinism.rs` so the path stays inside the scaffold's
//! planned file set (GC3 still requires this file to exist; the GC3 *test*
//! itself runs as a unit test in `src/lib.rs`).

use binn_core::{assoc_scan, simd_leak_integrate, Rng, State};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn fill_leak_inputs(n: usize, seed: u64) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut rng = Rng::new(seed);
    let v: Vec<f32> = (0..n).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
    let input: Vec<f32> = (0..n).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
    let tau: Vec<f32> = (0..n).map(|_| 0.5 + rng.next_f32() * 4.0).collect();
    (v, input, tau)
}

fn fill_states(n: usize, seed: u64) -> Vec<State> {
    let mut rng = Rng::new(seed);
    (0..n)
        .map(|_| {
            let tau = 0.5 + rng.next_f32() * 4.0;
            let input = rng.next_f32() * 2.0 - 1.0;
            State::leak_step(input, tau, 1.0)
        })
        .collect()
}

fn bench_simd_leak_integrate(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_leak_integrate");
    for &n in &[1_024usize, 16_384, 262_144] {
        group.throughput(Throughput::Elements(n as u64));
        let (v0, input, tau) = fill_leak_inputs(n, 0x51_4D_44_BE);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            let mut v = v0.clone();
            b.iter(|| {
                simd_leak_integrate(
                    black_box(&mut v),
                    black_box(&input),
                    black_box(&tau),
                    black_box(1),
                );
            });
        });
    }
    group.finish();
}

fn bench_assoc_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("assoc_scan");
    for &n in &[1_024usize, 16_384, 65_536] {
        group.throughput(Throughput::Elements(n as u64));
        let xs = fill_states(n, 0x5CA1_BE11);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let out = assoc_scan(black_box(&xs), State::combine);
                black_box(out);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_simd_leak_integrate, bench_assoc_scan);
criterion_main!(benches);
