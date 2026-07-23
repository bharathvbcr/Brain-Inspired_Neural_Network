//! U04 timing-wheel criterion bench host (GC5).
//!
//! Wired as `[[bench]] timing_wheel` with `harness = false`. Package
//! `autotests = false` so this file is not also an integration test.
//!
//! Sizes include 1e3 and 1e5 so append-only slot insert stays the same order of
//! magnitude per event (no quadratic bucket-insert cliff).

use std::hint::black_box;

use binn_engine::{Event, TimingWheel};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Insert `n` near-horizon events, then pop them all (O(n) append per insert).
fn fill_and_drain(n: usize) {
    let mut q = TimingWheel::new();
    // Bounded delays keep work in the hot level-0 window (realistic engine load).
    for i in 0..n {
        let at = (i % 128) as u64;
        q.insert(at, Event::new(i as u64));
    }
    while q.pop_earliest().is_some() {}
}

fn timing_wheel_per_op(c: &mut Criterion) {
    let mut group = c.benchmark_group("timing_wheel_insert_pop");
    for &n in &[1_000usize, 10_000, 50_000, 100_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| fill_and_drain(black_box(n)));
        });
    }
    group.finish();
}

criterion_group!(benches, timing_wheel_per_op);
criterion_main!(benches);
