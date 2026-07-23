use std::hint::black_box;

use binn_areas::k_wta;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn kwta(c: &mut Criterion) {
    let scores: Vec<_> = (0..10_000u32)
        .map(|cell| (cell, ((cell.wrapping_mul(2_654_435_761) >> 8) as f32)))
        .collect();
    let mut group = c.benchmark_group("kwta");
    group.throughput(Throughput::Elements(scores.len() as u64));
    group.bench_function("n10000_k100", |b| {
        b.iter(|| black_box(k_wta(black_box(&scores), 100)));
    });
    group.finish();
}

criterion_group!(benches, kwta);
criterion_main!(benches);
