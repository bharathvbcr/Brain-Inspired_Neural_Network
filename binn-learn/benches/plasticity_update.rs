use std::hint::black_box;

use binn_learn::three_factor::{coincidence_engine, run_coincidence_trial};
use binn_learn::{Modulators, ThreeFactor};
use criterion::{criterion_group, criterion_main, Criterion};

fn plasticity_update(c: &mut Criterion) {
    c.bench_function("three_factor_coincidence_trial", |b| {
        b.iter(|| {
            let mut engine = coincidence_engine(0.1);
            let mut learner = ThreeFactor::new(0.2, 0.0, 30.0);
            run_coincidence_trial(&mut engine, &mut learner, Modulators::reward(1.0), 10);
            black_box(engine.edge_w);
        });
    });
}

criterion_group!(benches, plasticity_update);
criterion_main!(benches);
