use std::hint::black_box;

use binn_areas::wiring::intra_area_edge_fraction;
use binn_areas::{
    overlap_count, wire, AreaRole, Assembly, InterAreaStepOpts, MultiAreaNetwork, Pos, WiringPrior,
};
use binn_core::{Csr, Rng};
use binn_engine::Engine;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn bench_wiring(c: &mut Criterion) {
    let mut group = c.benchmark_group("wiring");
    let areas = vec![0..32u32, 32..10_032u32, 10_032..10_034u32];
    let prior = WiringPrior::new(0xA55E_2026, areas, 0.08, 0.012).with_max_fan_out(256);

    group.throughput(Throughput::Elements(10_034));
    group.bench_function("wire_10k_fan256", |b| {
        b.iter(|| {
            let csr = wire(
                black_box(AreaRole::Association),
                black_box(Pos::new(1)),
                black_box(&prior),
            );
            black_box(csr.nnz());
        });
    });

    let csr = wire(AreaRole::Association, Pos::new(1), &prior);
    group.throughput(Throughput::Elements(csr.nnz() as u64));
    group.bench_function("intra_area_edge_fraction_10k", |b| {
        b.iter(|| {
            black_box(intra_area_edge_fraction(black_box(&csr), black_box(&prior)));
        });
    });
    group.finish();
}

fn bench_multi_area_and_overlap(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_area_and_overlap");
    let cells_per_area = 2_048usize;
    let k = 64usize;
    let mut net = MultiAreaNetwork::new(4, cells_per_area, k);
    let mut engine = Engine::with_cells(4 * cells_per_area);
    let mut rng = Rng::new(0x4D55_4C54);
    for a in 0..3 {
        let mut rows = Vec::with_capacity(cells_per_area);
        for _ in 0..cells_per_area {
            let mut r: Vec<u32> = (0..32)
                .map(|_| rng.gen_index(cells_per_area) as u32)
                .collect();
            r.sort_unstable();
            r.dedup();
            rows.push(r);
        }
        let csr = Csr::from_adjacency(&rows);
        let weights = vec![0.15f32; csr.nnz()];
        net.add_projection(a, a + 1, csr, weights);
    }
    let src_winners: Vec<u32> = (0..k as u32).map(|i| i * 16).collect();
    let extra = vec![0.01f32; cells_per_area];

    group.throughput(Throughput::Elements(cells_per_area as u64));
    group.bench_function("fused_inter_area_step_2048", |b| {
        b.iter(|| {
            let winners = net.fused_inter_area_step(
                black_box(&mut engine),
                black_box(0),
                black_box(1),
                black_box(&src_winners),
                InterAreaStepOpts::new(0.0, 0).with_bias(black_box(&extra)),
            );
            black_box(winners.len());
        });
    });

    let asm_a = Assembly::from_members((0..256u32).map(|i| i * 3).collect());
    let asm_b = Assembly::from_members((0..256u32).map(|i| i * 4).collect());
    group.throughput(Throughput::Elements(256));
    group.bench_function("overlap_count_256", |b| {
        b.iter(|| {
            black_box(overlap_count(black_box(&asm_a), black_box(&asm_b)));
        });
    });
    group.finish();
}

criterion_group!(benches, bench_wiring, bench_multi_area_and_overlap);
criterion_main!(benches);
