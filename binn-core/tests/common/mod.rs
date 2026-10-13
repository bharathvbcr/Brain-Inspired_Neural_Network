//! Shared gate for tests that need a live Metal device.
//!
//! A check that could not run must never look like one that ran and passed.
//! On Apple silicon (where Metal is expected) a missing device is a hard
//! failure unless `BINN_ALLOW_NO_METAL=1` is set, in which case the skip is
//! announced on stderr. On other targets the skip is always announced.
//! Not `#[ignore]`: an ignored test is never run by a plain `cargo test`, so
//! it would stop guarding the machines where it matters.
#![allow(dead_code)]

pub const OPT_OUT_ENV: &str = "BINN_ALLOW_NO_METAL";

pub const METAL_EXPECTED: bool = cfg!(all(target_os = "macos", target_arch = "aarch64"));

/// Pure decision: `true` = run the body, `false` = skipped (already announced),
/// panics when Metal is expected, missing, and not opted out.
pub fn metal_gate_decision(test: &str, available: bool, expected: bool, opted_out: bool) -> bool {
    if available {
        return true;
    }
    if expected && !opted_out {
        panic!(
            "{test}: Metal is unavailable on a target where it is expected; refusing to \
             report a pass for a check that did not run (set {OPT_OUT_ENV}=1 to skip explicitly)"
        );
    }
    eprintln!("SKIPPED: Metal unavailable ({test}); body NOT executed");
    false
}

/// Returns `true` when the test body should run. `available` is the caller's
/// device-availability probe.
pub fn require_metal(test: &str, available: bool) -> bool {
    let opted_out = std::env::var(OPT_OUT_ENV).is_ok_and(|v| v == "1");
    metal_gate_decision(test, available, METAL_EXPECTED, opted_out)
}
