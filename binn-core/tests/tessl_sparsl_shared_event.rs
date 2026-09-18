//! Tessl GEMM → SharedEvent wait → sparsl SpMV → signal handoff.
//!
//! Requires `--features tessl-interop` (Metal sparsl + path tessl). Queues stay
//! separate (Metal 4 vs Metal 3); only the SharedEvent timeline crosses crates.

#![cfg(all(feature = "tessl-interop", target_os = "macos"))]

use sparsl::{Backend, Csr, Device};
use tessl::gemm::{gemm, GemmBackend};
use tessl::GpuRuntime;

#[test]
fn tessl_gemm_then_sparsl_spmv_via_shared_event() {
    let Ok(rt) = GpuRuntime::new() else {
        return;
    };
    let Ok(device) = Device::try_new(Backend::Metal) else {
        return;
    };

    let a = rt.alloc_tensor_f32(&[2, 2]).expect("A");
    let b = rt.alloc_tensor_f32(&[2, 2]).expect("B");
    let c = rt.alloc_tensor_f32(&[2, 2]).expect("C");
    a.write_f32(&[1.0, 0.0, 0.0, 1.0]).expect("write A");
    b.write_f32(&[3.0, 4.0, 5.0, 6.0]).expect("write B");
    c.write_f32(&[0.0; 4]).expect("write C");
    gemm(&a, &b, &c, GemmBackend::TensorOps).expect("gemm");
    rt.synchronize().expect("tessl sync");

    let event = rt.shared_event();
    let signaled = rt.last_signaled_value();
    assert!(signaled > 0, "tessl commit must advance SharedEvent");

    let csr = Csr::from_adjacency(&[vec![0], vec![1]]);
    let weights = vec![1.0f32, 1.0];
    let op = device.prepare(&csr, 2, &weights).expect("prepare");

    // Cross-crate handoff gate: sparsl must wait on tessl's timeline before
    // consuming GEMM output.
    op.wait_shared_event(event, signaled, 5_000)
        .expect("sparsl wait on tessl SharedEvent");

    let x_host = c.read_f32().expect("read C");
    // Identity @ B ⇒ C == B in row-major; first column is [3, 5].
    let x_col = [x_host[0], x_host[2]];
    assert_eq!(x_col, [3.0, 5.0]);
    let mut y = [0.0f32; 2];
    op.spmv(&x_col, &mut y).expect("spmv after wait");
    assert_eq!(y, x_col);

    let next = signaled + 1;
    op.signal_shared_event(event, next).expect("signal");
    // Tessl-side wait is expressed through sparsl's wait API on the same event.
    op.wait_shared_event(event, next, 5_000)
        .expect("wait on sparsl signal");

    let err = op
        .wait_shared_event(event, next + 1, 0)
        .expect_err("unsignaled future value must time out");
    assert!(err.to_string().contains("timed out"), "{err}");
}
