use std::hint::black_box;

use binn_core::Csr;
use binn_engine::Engine;
use binn_learn::three_factor::{coincidence_engine, run_coincidence_trial};
use binn_learn::{
    shd_matched_forward_arm_tau, shd_matched_loss_only_and_gradient_arm_tau_with_workspace,
    AttentionConfig, AttentionParams, Learner, MatchedArm, MatchedShdSample, Modulators,
    ShdAlifArm, ShdAlifConfig, ShdAlifRule, ShdArmWeightLayout, ShdArmWeights, ShdArmWorkspace,
    ShdDfa, ShdExample, ShdMatchedWeights, ShdTrainConfig, ThreeFactor, MATCHED_PHYSICAL_TAU_MS,
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn build_sparse_engine(n_cells: usize, fanout: usize) -> Engine {
    let mut eng = Engine::with_cells(n_cells);
    let mut row_ptr = Vec::with_capacity(n_cells + 1);
    let mut col = Vec::with_capacity(n_cells * fanout);
    row_ptr.push(0);
    for pre in 0..n_cells {
        let base = pre.wrapping_mul(131);
        let mut row_cols: Vec<u32> = (0..fanout)
            .map(|k| ((base + k * 17 + 1) % n_cells) as u32)
            .collect();
        row_cols.sort_unstable();
        row_cols.dedup();
        col.extend_from_slice(&row_cols);
        row_ptr.push(col.len() as u32);
    }
    let nnz = col.len();
    let conn = Csr::from_parts(row_ptr, col).expect("sparse bench CSR");
    eng.set_connectivity(conn, vec![0.05; nnz]);
    eng
}

fn plasticity_update(c: &mut Criterion) {
    c.bench_function("three_factor_coincidence_trial", |b| {
        b.iter(|| {
            let mut engine = coincidence_engine(0.1);
            let mut learner = ThreeFactor::new(0.2, 0.0, 30.0);
            run_coincidence_trial(&mut engine, &mut learner, Modulators::reward(1.0), 10);
            black_box(engine.edge_w);
        });
    });

    // Sparse network plasticity step (1% active cells across ~64k synapses).
    let mut sparse_engine = build_sparse_engine(1024, 64);
    let nnz = sparse_engine.edge_w.len() as u64;
    let mut sparse_learner = ThreeFactor::new(0.05, 1e-4, 30.0);
    let mut tick = 10u64;
    let mut group = c.benchmark_group("sparse_three_factor");
    group.throughput(Throughput::Elements(nnz));
    group.bench_function("n1024_nnz64k_1pct_active", |b| {
        b.iter(|| {
            for k in 0..10u32 {
                let cell = ((tick as u32).wrapping_mul(37).wrapping_add(k * 97)) & 1023;
                sparse_engine.inject_weighted(cell, 0, tick, 2.0);
            }
            sparse_engine.step_until(tick + 2);
            sparse_learner.update(&mut sparse_engine, Modulators::reward(1.0));
            tick += 5;
            black_box(sparse_engine.edge_w[0]);
        });
    });
    group.finish();

    // Wave 4 SHD training & eval mega-kernels (sparse 700-channel & 140-channel rasters).
    let (t_steps, n_in_700, hidden, n_classes) = (100usize, 700usize, 128usize, 20usize);
    let mut frames_700 = vec![0.0f32; t_steps * n_in_700];
    for t in 0..t_steps {
        for rep in 0..8usize {
            let ch = (t * 37 + rep * 83) % n_in_700;
            frames_700[t * n_in_700 + ch] = 1.0;
        }
    }
    let shd_ex = ShdExample {
        frames: frames_700,
        t: t_steps,
        n_in: n_in_700,
        label: 3,
    };
    let shd_test = [shd_ex.clone()];

    let mut alif_group = c.benchmark_group("wave4_shd_training");
    let alif_ff_cfg = ShdAlifConfig::feedforward_fixed(hidden, n_classes, 0.02, 1);
    let mut alif_ff = ShdAlifArm::new(&shd_ex, &alif_ff_cfg, ShdAlifRule::Dfa, 42);
    alif_group.bench_function("alif_ff_fixed_step_700x128_t100", |b| {
        b.iter(|| {
            black_box(alif_ff.train_and_evaluate(1, std::slice::from_ref(&shd_ex), &shd_test));
        });
    });

    let alif_rec_cfg = alif_ff_cfg.with_recurrent(true).with_adaptive(true);
    let mut alif_rec = ShdAlifArm::new(&shd_ex, &alif_rec_cfg, ShdAlifRule::Dfa, 42);
    alif_group.bench_function("alif_rec_adaptive_step_700x128_t100", |b| {
        b.iter(|| {
            black_box(alif_rec.train_and_evaluate(1, std::slice::from_ref(&shd_ex), &shd_test));
        });
    });

    let eprop_cfg = ShdTrainConfig {
        hidden,
        n_classes,
        lr: 0.02,
        beta: 5.0,
        epochs: 1,
    };
    let mut dfa_arm = ShdDfa::new(&shd_ex, eprop_cfg, 42);
    alif_group.bench_function("eprop_dfa_step_700x128_t100", |b| {
        b.iter(|| {
            black_box(dfa_arm.train_and_evaluate(1, std::slice::from_ref(&shd_ex), &shd_test));
        });
    });

    let n_in_140 = 140usize;
    let sparse_frames: Vec<Vec<(usize, f32)>> = (0..t_steps)
        .map(|t| {
            (0..6)
                .map(|k| ((t * 17 + k * 23) % n_in_140, 1.0))
                .collect()
        })
        .collect();
    let matched_sample = MatchedShdSample::new(3, sparse_frames, n_in_140, 2.0);
    let base_w = ShdMatchedWeights::deterministic(n_in_140, hidden, n_classes, 42);
    let w_rec: Vec<f32> = (0..hidden * hidden)
        .map(|i| (((i % 19) as f32) - 9.0) * 5e-3)
        .collect();
    let attn = AttentionParams::deterministic(
        hidden,
        n_classes,
        AttentionConfig::new(32, 1).expect("attn cfg"),
        99,
    )
    .expect("attn params");
    let arm_weights =
        ShdArmWeights::new_attentive(base_w, MatchedArm::ALL_ATTENTION[3], w_rec, attn)
            .expect("arm weights");
    let arm_layout = ShdArmWeightLayout::prepare(&arm_weights);
    let mut arm_ws = ShdArmWorkspace::default();

    alif_group.bench_function("matched_rec_alif_attn_forward_only_140x128_t100", |b| {
        b.iter(|| {
            black_box(
                shd_matched_forward_arm_tau(
                    &arm_weights,
                    &arm_layout,
                    &matched_sample,
                    MATCHED_PHYSICAL_TAU_MS,
                )
                .expect("forward_arm_tau"),
            );
        });
    });

    alif_group.bench_function("matched_rec_alif_attn_loss_grad_ws_140x128_t100", |b| {
        b.iter(|| {
            black_box(
                shd_matched_loss_only_and_gradient_arm_tau_with_workspace(
                    &arm_weights,
                    &arm_layout,
                    &matched_sample,
                    1.0,
                    MATCHED_PHYSICAL_TAU_MS,
                    &mut arm_ws,
                )
                .expect("loss_only_and_gradient_arm_tau_with_workspace"),
            );
        });
    });
    alif_group.finish();
}

criterion_group!(benches, plasticity_update);
criterion_main!(benches);
