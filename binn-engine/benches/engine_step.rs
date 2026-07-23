use std::hint::black_box;

use binn_engine::Engine;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn engine_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_step");
    group.throughput(Throughput::Elements(1_024));
    group.bench_function("1024_external_events", |b| {
        b.iter(|| {
            let mut engine = Engine::with_cells(1_024);
            for cell in 0..1_024u32 {
                engine.inject(cell, 0, 1);
            }
            black_box(engine.step_until(1));
        });
    });
    group.finish();
}

criterion_group!(benches, engine_step);
criterion_main!(benches);
