//! F1 parallelism characterization: sequential vs adaptive vs always-rayon.
//!
//! Engineering bench only — not a biology claim and not a G2 reopen.

use std::hint::black_box;

use binn_core::Csr;
use binn_engine::{Engine, PartitionPlan};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn sparse_schedule(n_cells: usize) -> (Csr, Vec<(u32, u64)>) {
    let conn = Csr::from_adjacency(&(0..n_cells).map(|_| Vec::<u32>::new()).collect::<Vec<_>>());
    let mut events = Vec::new();
    let n_events = n_cells * 4;
    for i in 0..n_events {
        let in_burst = i % 64 < 8;
        let (cell, tick) = if in_burst {
            let burst_id = i / 64;
            let slot = i % 8;
            ((slot % n_cells) as u32, 1 + burst_id as u64)
        } else {
            ((i * 17 % n_cells) as u32, 1_000 + i as u64)
        };
        events.push((cell, tick));
    }
    (conn, events)
}

fn primed(n_cells: usize, conn: &Csr, events: &[(u32, u64)]) -> Engine {
    let mut eng = Engine::with_cells(n_cells);
    eng.set_connectivity(conn.clone(), vec![0.0; conn.nnz()]);
    for &(cell, tick) in events {
        eng.force_spike(cell, tick);
    }
    eng
}

fn f1_parallelism(c: &mut Criterion) {
    let mut group = c.benchmark_group("f1_parallelism");
    for &n in &[128usize, 512] {
        let (conn, events) = sparse_schedule(n);
        let plan = PartitionPlan::degree_balanced(&conn, 4);
        group.throughput(Throughput::Elements(events.len() as u64));

        group.bench_with_input(BenchmarkId::new("sequential", n), &n, |b, &_n| {
            b.iter(|| {
                let mut eng = primed(n, &conn, &events);
                black_box(eng.step_until(1_000 + (n * 4) as u64));
            });
        });
        group.bench_with_input(BenchmarkId::new("adaptive_partitioned", n), &n, |b, &_n| {
            b.iter(|| {
                let mut eng = primed(n, &conn, &events);
                black_box(eng.step_until_partitioned(1_000 + (n * 4) as u64, &plan));
            });
        });
        group.bench_with_input(BenchmarkId::new("always_rayon", n), &n, |b, &_n| {
            b.iter(|| {
                let mut eng = primed(n, &conn, &events);
                black_box(eng.step_until_partitioned_threshold(1_000 + (n * 4) as u64, &plan, 1));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, f1_parallelism);
criterion_main!(benches);
