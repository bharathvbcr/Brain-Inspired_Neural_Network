use std::hint::black_box;

use binn_areas::{boundary_below, k_wta, k_wta_straight_through, k_wta_with_margin};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn kwta(c: &mut Criterion) {
    let scores: Vec<_> = (0..10_000u32)
        .map(|cell| (cell, ((cell.wrapping_mul(2_654_435_761) >> 8) as f32)))
        .collect();
    let winners = k_wta(&scores, 100);
    let mut group = c.benchmark_group("kwta");
    group.throughput(Throughput::Elements(scores.len() as u64));
    group.bench_function("n10000_k100", |b| {
        b.iter(|| black_box(k_wta(black_box(&scores), 100)));
    });
    group.bench_function("n10000_k100_with_margin", |b| {
        b.iter(|| black_box(k_wta_with_margin(black_box(&scores), 100)));
    });
    group.bench_function("n10000_k100_boundary_below", |b| {
        b.iter(|| black_box(boundary_below(black_box(&scores), black_box(&winners))));
    });
    group.bench_function("n10000_k100_straight_through_hard", |b| {
        b.iter(|| black_box(k_wta_straight_through(black_box(&scores), 100, 0.0)));
    });
    group.finish();
}

criterion_group!(benches, kwta);
criterion_main!(benches);
