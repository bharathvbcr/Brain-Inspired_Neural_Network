//! Tessl GEMM → SharedEvent wait → sparsl SpMV → signal handoff.
//!
//! Requires `--features tessl-interop` (Metal sparsl + path tessl). Queues stay
//! separate (Metal 4 vs Metal 3); only the SharedEvent timeline crosses crates.

#![cfg(all(feature = "tessl-interop", target_os = "macos"))]

mod common;

use sparsl::{Backend, Csr, Device};
use tessl::gemm::{gemm, GemmBackend};
use tessl::GpuRuntime;

#[test]
fn tessl_gemm_then_sparsl_spmv_via_shared_event() {
    let rt = GpuRuntime::new();
    let device = Device::try_new(Backend::Metal);
    if !common::require_metal(
        "tessl_gemm_then_sparsl_spmv_via_shared_event",
        rt.is_ok() && device.is_ok(),
    ) {
        return;
    }
    let (rt, device) = (rt.expect("tessl runtime"), device.expect("sparsl device"));

    let a = rt.alloc_tensor_f32(&[2, 2]).expect("A");
    let b = rt.alloc_tensor_f32(&[2, 2]).expect("B");
    let c = rt.alloc_tensor_f32(&[2, 2]).expect("C");
    // tessl 0.1.4 puts host IO on the buffer and panics (not Result) on misuse.
    a.buffer.write_f32(&[1.0, 0.0, 0.0, 1.0]);
    b.buffer.write_f32(&[3.0, 4.0, 5.0, 6.0]);
    c.buffer.write_f32(&[0.0; 4]);
    gemm(&a, &b, &c, GemmBackend::TensorOps).expect("gemm");
    rt.synchronize().expect("tessl sync");

    // tessl 0.1.4 exposes the event as a public field and has no accessor for
    // the last signaled value, so read it back through sparsl's own wait API.
    let event = &rt.metal4.shared_event;

    let csr = Csr::from_adjacency(&[vec![0], vec![1]]);
    let weights = vec![1.0f32, 1.0];
    let op = device.prepare(&csr, 2, &weights).expect("prepare");

    // Largest `v` for which a zero-timeout wait succeeds == signaledValue
    // (waits are monotone in `v`; value 0 always succeeds).
    let (mut lo, mut hi) = (0u64, 1u64 << 32);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if op.wait_shared_event(event, mid, 0).is_ok() {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let signaled = lo;
    assert!(signaled > 0, "tessl commit must advance SharedEvent");

    // Cross-crate handoff gate: sparsl must wait on tessl's timeline before
    // consuming GEMM output.
    op.wait_shared_event(event, signaled, 5_000)
        .expect("sparsl wait on tessl SharedEvent");

    let x_host = c.buffer.read_f32();
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
