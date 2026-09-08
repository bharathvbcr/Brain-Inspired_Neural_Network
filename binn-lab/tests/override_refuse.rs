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

// ---- the canonical record is not a scratch file, 2026-09-07 ----------------
//
// Every `c1` matched suite defaults its `--out` to the suite's COMMITTED record
// -- `results/c1_match.md`, `results/c1_dfa.md` and so on. Those are documents
// other documents cite and `scripts/check_every_number.py` traces the draft's
// numbers to. A run at a non-canonical config wrote over them silently: the file
// changed, the run printed its verdict, and nothing said the document no longer
// described what its name claimed.
//
// That happened. On 2026-09-07 a hash-resolution scan run without `--out`
// overwrote `results/c1_dfa.md` and `results/c1_eventprop.md`. Neither run
// noticed. What caught it, two steps later, was a paper-level check reporting
// that the draft's 0.9150 no longer appeared in the source it is traced to --
// a check standing in for a guard that belonged in the binary.

/// One override flag per suite that is enough to move it off its canonical
/// config. `--mech` takes only `--config-hash`, so it is given a real preset:
/// an unknown hash exits for a different reason and would test nothing here.
const MOVED_RUNS: &[(&str, &[&str], &str)] = &[
    ("matched-arch", &["--matched-arch", "--max-lag", "3"], "results/c1_match.md"),
    ("matched-dfa", &["--matched-dfa", "--max-lag", "3"], "results/c1_dfa.md"),
    ("matched-rl", &["--matched-rl", "--matched-forward", "recurrent"], "results/c1_rl.md"),
    ("eventprop", &["--eventprop", "--matched-forward", "recurrent"], "results/c1_eventprop.md"),
    (
        "mech",
        &["--mech", "--config-hash", "c1-mech-adfc5d6fd9e48e02"],
        "results/c1_credit_mech.md",
    ),
];

#[test]
fn a_moved_config_may_not_overwrite_its_suites_committed_record() {
    for (suite, args, record) in MOVED_RUNS {
        let out = bin("c1").args(*args).output().expect("spawn c1");
        assert_eq!(
            out.status.code(),
            Some(2),
            "{suite}: a moved config with no --out must refuse, not run"
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(record),
            "{suite}: the refusal must name the record it is protecting, so the \
             reader knows which document was at risk; got: {err}"
        );
        assert!(
            err.contains("--out"),
            "{suite}: the refusal must say what to do instead; got: {err}"
        );
    }
}

#[test]
fn the_refusal_costs_nothing_because_it_precedes_the_run() {
    // A guard that fires at the write step is correct and wasteful: it spends
    // the whole suite first. `--matched-arch` scientific is twenty seeds.
    let start = std::time::Instant::now();
    let out = bin("c1")
        .args(["--matched-arch", "--config-hash", "c1-match-6f6000f148f7d30c"])
        .output()
        .expect("spawn c1");
    assert_eq!(out.status.code(), Some(2));
    assert!(
        start.elapsed() < std::time::Duration::from_secs(20),
        "the refusal took {:?}; it is firing after the run rather than before it",
        start.elapsed()
    );
}

#[test]
fn an_explicit_out_is_honoured_and_the_guard_stays_quiet() {
    // The positive control. Without it every assertion above would still pass
    // if the guard refused unconditionally, which would break record
    // generation entirely.
    let dir = std::env::temp_dir().join(format!("binn-c1-guard-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("moved.md");
    let out = bin("c1")
        .args(["--matched-arch", "--quick", "--max-lag", "3", "--out"])
        .arg(&path)
        .output()
        .expect("spawn c1");
    assert_eq!(
        out.status.code(),
        Some(0),
        "a moved config WITH --out must run: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(path.is_file(), "the report was not written to the given path");
    let body = std::fs::read_to_string(&path).expect("read report");
    assert!(
        body.contains("c1-match-"),
        "the written file is not a matched-arch report"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
