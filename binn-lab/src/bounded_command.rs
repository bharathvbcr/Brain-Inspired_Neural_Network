//! Run a child process under a deadline, and never inherit a terminal's stdin.
//!
//! Every `Command` in this workspace waited forever. That is the same defect
//! that left `scripts/check_gc4.sh` at 0% CPU for two days and that
//! `scripts/gate_f_rust.py` carried until it grew a per-cell budget: a process
//! that stops making progress and a process that is merely slow are
//! indistinguishable to a caller with no clock, so the caller waits.
//!
//! Two hazards are closed here, not one:
//!
//! * **No deadline.** `Command::output()` returns only when the child exits.
//!   [`run_bounded`] polls instead, and kills the child's process group when
//!   the budget is spent.
//! * **Inherited stdin.** A child spawned with no stdin configuration inherits
//!   the parent's, so a child that reads stdin blocks on a terminal nobody is
//!   typing into. That is *precisely* how the ripgrep call in GC4 hung. Every
//!   child started here gets `/dev/null`.
//!
//! The pipes are drained on their own threads. A child writing more than the
//! pipe buffer — 64 KiB on Linux, often less on macOS — blocks in `write` once
//! the buffer fills, so a caller that waits for exit before reading deadlocks
//! against a merely chatty child. `Command::output()` handles this; a
//! hand-rolled `spawn` + `wait` does not, and reproducing that bug inside the
//! fix for a different one would be a poor trade.

use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// How often the child is checked for exit. Short enough that a fast child is
/// not delayed noticeably, long enough not to spin a core while waiting.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// How long to wait for the child's output after it has exited or been killed.
/// Only reached when something still holds the pipe's write end open.
const OUTPUT_GRACE: Duration = Duration::from_secs(5);

/// What a bounded run produced.
#[derive(Debug)]
pub struct BoundedOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Why a bounded run did not produce an exit status.
#[derive(Debug)]
pub enum BoundedError {
    /// The child outlived its budget and was killed. Carries whatever it had
    /// written, which is often the only clue about where it stopped.
    TimedOut {
        after: Duration,
        stdout: String,
        stderr: String,
    },
    /// The child could not be started, or could not be waited on.
    Spawn(std::io::Error),
}

impl std::fmt::Display for BoundedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundedError::TimedOut { after, stderr, .. } => write!(
                f,
                "child exceeded its {:.0}s budget and was killed{}",
                after.as_secs_f64(),
                if stderr.trim().is_empty() {
                    String::new()
                } else {
                    format!("; last stderr: {}", stderr.trim())
                }
            ),
            BoundedError::Spawn(err) => write!(f, "could not run child: {err}"),
        }
    }
}

impl std::error::Error for BoundedError {}

/// Put the child in its own process group so the kill reaches its descendants.
///
/// Killing only the direct child leaves a grandchild holding the pipes, and the
/// drain threads then never see EOF — the wait blocks again, inside the code
/// that exists to stop it blocking.
#[cfg(unix)]
fn own_process_group(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    // `process_group(0)` makes the child its own group leader, so its pid is
    // also its process-group id. Safe; no `pre_exec` and no libc needed.
    cmd.process_group(0);
}

#[cfg(not(unix))]
fn own_process_group(_cmd: &mut Command) {}

#[cfg(unix)]
fn kill_group(child: &mut Child) {
    extern "C" {
        fn killpg(pgrp: i32, sig: i32) -> i32;
    }
    const SIGKILL: i32 = 9;
    // `own_process_group` made the child its own group leader, so its pid is
    // the group id. A stale or already-reaped pid makes `killpg` return an
    // error we ignore; it cannot signal an unrelated group, because the group
    // was created for this child and a group id is not reused while any member
    // remains, which the child does until we reap it below.
    //
    // SAFETY: a plain signal send with no memory operands, and the invariant above names the group.
    unsafe {
        killpg(child.id() as i32, SIGKILL);
    }
    // SIGKILL cannot be caught, but a process wedged in uninterruptible I/O can
    // still outlive it, so the direct kill is a second attempt, not a
    // substitute.
    let _ = child.kill();
}

#[cfg(not(unix))]
fn kill_group(child: &mut Child) {
    let _ = child.kill();
}

/// Run `cmd` to completion, or kill it once `budget` is spent.
///
/// `stdin` is always `/dev/null`: a child that reads stdin must see EOF, never
/// a terminal.
pub fn run_bounded(mut cmd: Command, budget: Duration) -> Result<BoundedOutput, BoundedError> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    own_process_group(&mut cmd);

    let mut child = cmd.spawn().map_err(BoundedError::Spawn)?;
    let mut out_pipe = child.stdout.take();
    let mut err_pipe = child.stderr.take();

    // Collected over a channel rather than by joining the threads. `join()` is
    // unbounded, and after a timeout the pipes may still be held open by a
    // process that outlived the kill — so joining would block forever, which is
    // the defect this whole module exists to prevent, one layer down. Measured:
    // with the group kill mutated out, a joining version took 600s to return
    // (the full lifetime of the surviving grandchild).
    fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> Receiver<String> {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = String::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_string(&mut buf);
            }
            // The receiver may already have given up; that is not an error.
            let _ = tx.send(buf);
        });
        rx
    }
    let out_rx = drain(out_pipe.take());
    let err_rx = drain(err_pipe.take());

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(err) => return Err(BoundedError::Spawn(err)),
        }
        if started.elapsed() >= budget {
            kill_group(&mut child);
            // Reap so the child does not linger as a zombie. It has been
            // SIGKILLed and its group is gone, so this wait is not the
            // unbounded one we just removed.
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(POLL_INTERVAL);
    };

    // The child is gone by now, so its pipes should reach EOF at once. This
    // grace is for the case where they do not: a survivor still holds the write
    // end. Losing the output of a run that already failed is a far better
    // outcome than never returning from it.
    let stdout = out_rx.recv_timeout(OUTPUT_GRACE).unwrap_or_default();
    let stderr = err_rx.recv_timeout(OUTPUT_GRACE).unwrap_or_default();
    match status {
        Some(status) => Ok(BoundedOutput {
            status,
            stdout,
            stderr,
        }),
        None => Err(BoundedError::TimedOut {
            after: budget,
            stdout,
            stderr,
        }),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn sh(script: &str) -> Command {
        let mut c = Command::new("sh");
        c.arg("-c").arg(script);
        c
    }

    #[test]
    fn a_quick_child_returns_its_output() {
        let out = run_bounded(sh("echo hello; echo oops >&2"), Duration::from_secs(30))
            .expect("a child that exits immediately must not time out");
        assert!(out.status.success());
        assert_eq!(out.stdout.trim(), "hello");
        assert_eq!(out.stderr.trim(), "oops");
    }

    #[test]
    fn a_failing_child_is_a_status_not_an_error() {
        // Distinguishing "ran and failed" from "could not run" is the whole
        // point; collapsing them is how a check comes to mean nothing.
        let out = run_bounded(sh("exit 3"), Duration::from_secs(30)).expect("exit 3 is a result");
        assert_eq!(out.status.code(), Some(3));
    }

    #[test]
    fn a_hung_child_is_killed_within_its_budget() {
        let started = Instant::now();
        let err = run_bounded(sh("sleep 600"), Duration::from_millis(300))
            .expect_err("a child sleeping ten minutes must not be waited on");
        let elapsed = started.elapsed();
        assert!(
            matches!(err, BoundedError::TimedOut { .. }),
            "expected a timeout, got {err:?}"
        );
        assert!(
            elapsed < Duration::from_secs(20),
            "gave up only after {elapsed:?}, which is not a bound"
        );
    }

    #[test]
    fn a_grandchild_does_not_outlive_the_kill() {
        // Killing the direct child alone leaves a grandchild holding the write
        // end of the pipe, so the drain threads never see EOF and the join
        // blocks — the same hang, one level down.
        // Qualified by pid. `pgrep -f` searches the whole machine's process
        // table, so a fixed marker makes this test fail whenever a second copy
        // of it runs anywhere on the host — another `cargo test`, a CI matrix
        // sharing a runner, a retry overlapping its own first attempt. Measured
        // before this change: two concurrent copies failed 7 of 12 runs while
        // `run_bounded` was behaving perfectly. `shd_matched.rs` and
        // `transfer_bundle.rs` already qualify their temp paths this way.
        let marker = format!("binn_bounded_grandchild_{}", std::process::id());
        let started = Instant::now();
        let _ = run_bounded(
            // `: {marker}` is a second command, which matters: `sh -c` with a
            // single simple command execs it directly, so `sh -c 'sleep 600 #
            // {marker}'` becomes a bare `sleep 600` and the marker vanishes
            // from the process table. Written that way, the only process
            // carrying the marker was the *direct child* — so the pgrep half of
            // this test could never have observed a surviving grandchild, and
            // the mutation that removes the group kill was being caught only by
            // the elapsed-time assertion above. Verified: with the second
            // command present, killing the direct child alone leaves exactly
            // one marker-bearing survivor.
            sh(&format!("sh -c 'sleep 600; : {marker}' & sleep 600")),
            Duration::from_millis(300),
        );
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "the call did not return, so the grandchild held the pipes open"
        );
        // Reaping a SIGKILLed orphan is asynchronous, so poll to a bound
        // rather than sleeping a fixed amount tuned to this machine.
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut survivors = usize::MAX;
        while Instant::now() < deadline {
            survivors = Command::new("pgrep")
                .args(["-f", marker.as_str()])
                .output()
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .split_whitespace()
                        .count()
                })
                .unwrap_or(0);
            if survivors == 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(survivors, 0, "a grandchild survived the timeout");
    }

    #[test]
    fn stdin_is_never_the_terminal() {
        // GC4's two-day hang in one assertion: a child that reads stdin must
        // see EOF, not a terminal nobody is typing into.
        let started = Instant::now();
        let out = run_bounded(sh("cat; echo drained"), Duration::from_secs(20))
            .expect("a child reading stdin must see EOF, not block");
        assert_eq!(out.stdout.trim(), "drained");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the child blocked on stdin"
        );
    }

    #[test]
    fn a_chatty_child_does_not_deadlock_on_a_full_pipe() {
        // More than any pipe buffer. A caller that waits for exit before
        // reading would deadlock here against a child blocked in `write`.
        let out = run_bounded(
            sh("i=0; while [ $i -lt 4000 ]; do \
                echo 0123456789012345678901234567890123456789; i=$((i+1)); done"),
            Duration::from_secs(60),
        )
        .expect("a child writing past the pipe buffer must still complete");
        assert!(out.status.success());
        assert_eq!(out.stdout.lines().count(), 4000);
    }

    #[test]
    fn a_zero_budget_still_terminates() {
        let started = Instant::now();
        let err = run_bounded(sh("sleep 600"), Duration::ZERO)
            .expect_err("a zero budget cannot succeed for a long child");
        assert!(matches!(err, BoundedError::TimedOut { .. }), "{err:?}");
        assert!(started.elapsed() < Duration::from_secs(20));
    }

    #[test]
    fn a_missing_program_is_a_spawn_error_not_a_hang() {
        let err = run_bounded(
            Command::new("binn-no-such-program-exists"),
            Duration::from_secs(30),
        )
        .expect_err("a missing program cannot succeed");
        assert!(matches!(err, BoundedError::Spawn(_)), "{err:?}");
    }
}
