//! CLI refuse-without-flag smoke for post-G2 exploratory binaries.
//!
//! These spawn the real `c2` / `c3` / `r1` / `r2` / `extensions` / `efficiency`
//! binaries with no override and expect a nonzero exit (kill-gate intact).

use std::process::Command;

fn bin(name: &str) -> Command {
    // Prefer cargo-built binaries from CARGO_BIN_EXE_* when available.
    let key = format!("CARGO_BIN_EXE_{name}");
    let mut cmd = if let Ok(path) = std::env::var(&key) {
        Command::new(path)
    } else {
        let mut c = Command::new("cargo");
        c.args(["run", "-q", "-p", "binn-lab", "--bin", name, "--"]);
        c
    };
    cmd.env_remove("BINN_OVERRIDE_G2_FOR");
    cmd
}

#[test]
fn c2_refuses_without_override_flag() {
    let out = bin("c2").output().expect("spawn c2");
    assert_ne!(
        out.status.code(),
        Some(0),
        "c2 must refuse without override"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kill-gate") || err.contains("enable-c2"),
        "stderr should mention override: {err}"
    );
}

#[test]
fn c3_refuses_without_override_flag() {
    let out = bin("c3").output().expect("spawn c3");
    assert_ne!(
        out.status.code(),
        Some(0),
        "c3 must refuse without override"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kill-gate") || err.contains("enable-c3"),
        "stderr should mention override: {err}"
    );
}

#[test]
fn r1_refuses_without_override_flag() {
    let out = bin("r1").output().expect("spawn r1");
    assert_ne!(
        out.status.code(),
        Some(0),
        "r1 must refuse without override"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kill-gate") || err.contains("enable-r1"),
        "stderr should mention override: {err}"
    );
}

#[test]
fn r2_refuses_without_override_flag() {
    let out = bin("r2").output().expect("spawn r2");
    assert_ne!(
        out.status.code(),
        Some(0),
        "r2 must refuse without override"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("kill-gate") || err.contains("enable-r2"),
        "stderr should mention override: {err}"
    );
}

#[test]
fn extensions_refuses_without_override_flag() {
    let out = bin("extensions").output().expect("spawn extensions");
    assert_ne!(
        out.status.code(),
        Some(0),
        "extensions must refuse without override"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("enable-extensions") || err.contains("post-G2"),
        "stderr should mention override: {err}"
    );
}

#[test]
fn efficiency_refuses_without_override_flag() {
    let out = bin("efficiency").output().expect("spawn efficiency");
    assert_ne!(
        out.status.code(),
        Some(0),
        "efficiency must refuse without override"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("enable-efficiency") || err.contains("post-G2"),
        "stderr should mention override: {err}"
    );
}
