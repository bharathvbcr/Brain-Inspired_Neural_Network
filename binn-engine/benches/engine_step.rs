use std::hint::black_box;

use binn_core::{Csr, Rng};
use binn_engine::{matched_null, Engine, RestingNull, RestingRaster};
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

    // Build a 4,096-cell, ~524k-edge graph to measure repeated `max_synaptic_delay`
    // queries across multi-window stepping.
    let n = 4_096usize;
    let mut rng = Rng::new(0xE16E_BE1C);
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let mut r: Vec<u32> = (0..128).map(|_| rng.gen_index(n) as u32).collect();
        r.sort_unstable();
        r.dedup();
        rows.push(r);
    }
    let csr = Csr::from_adjacency(&rows);
    let weights = vec![0.05f32; csr.nnz()];
    let mut connected = Engine::with_cells(n);
    connected.set_connectivity(csr, weights);
    group.throughput(Throughput::Elements(connected.conn.nnz() as u64));
    group.bench_function("max_synaptic_delay_500k", |b| {
        b.iter(|| {
            black_box(connected.max_synaptic_delay());
        });
    });
    group.finish();
}

fn resting_nulls(c: &mut Criterion) {
    let mut group = c.benchmark_group("resting_nulls");
    let n_cells = 1_024usize;
    let ticks = 128usize;
    let k = 128usize;
    let mut rng = Rng::new(0xAE57_BE1C);
    let spikes_by_tick: Vec<Vec<u32>> = (0..ticks)
        .map(|_| {
            let mut row: Vec<u32> = (0..k).map(|_| rng.gen_index(n_cells) as u32).collect();
            row.sort_unstable();
            row.dedup();
            row
        })
        .collect();
    let raster = RestingRaster {
        n_cells,
        spikes_by_tick,
    };
    group.throughput(Throughput::Elements((ticks * k) as u64));
    group.bench_function("rate_matched_1024x128", |b| {
        b.iter(|| {
            black_box(matched_null(
                black_box(&raster),
                RestingNull::RateMatched,
                7,
            ));
        });
    });
    group.bench_function("activity_matched_1024x128", |b| {
        b.iter(|| {
            black_box(matched_null(
                black_box(&raster),
                RestingNull::ActivityMatched,
                7,
            ));
        });
    });
    group.finish();
}

criterion_group!(benches, engine_step, resting_nulls);
criterion_main!(benches);
