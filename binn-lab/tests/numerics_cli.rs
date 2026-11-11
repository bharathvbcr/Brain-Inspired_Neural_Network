//! `shd-instrument train-cell --numerics`: the exit status and message a launcher
//! sees. These spawn the real binary, because the contract is the process exit
//! (status 2, message on stderr) and not the function that produces it.
//!
//! Every case passes only `--numerics`. The mode is resolved before any other
//! flag is read, so a refusal here proves it happened before an event file was
//! opened, and an acceptance shows up as the *next* missing flag.

use std::process::{Command, Output};

fn train_cell(extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_shd-instrument"))
        .arg("train-cell")
        .args(extra)
        .output()
        .expect("spawn shd-instrument")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn an_unknown_numerics_value_exits_2_naming_the_choices() {
    let output = train_cell(&["--numerics", "fast"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("--numerics must be exact or blas, got \"fast\""),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_numerics_flag_with_no_value_exits_2() {
    let output = train_cell(&["--numerics"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("--numerics requires a value"),
        "{}",
        stderr(&output)
    );
}

/// `exact`, spelled out or omitted, is accepted: the run goes on to ask for the
/// next required flag rather than refusing the mode.
#[test]
fn exact_is_accepted_whether_or_not_it_is_spelled_out() {
    for extra in [&["--numerics", "exact"][..], &[][..]] {
        let output = train_cell(extra);
        assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
        assert!(
            stderr(&output).contains("--train-events requires a value"),
            "{extra:?}: {}",
            stderr(&output)
        );
    }
}

#[cfg(not(all(feature = "accelerate", target_os = "macos")))]
#[test]
fn blas_without_the_accelerate_feature_exits_2_with_the_contract_message() {
    let output = train_cell(&["--numerics", "blas"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output)
            .contains("--numerics blas needs a build with the accelerate feature on macOS"),
        "{}",
        stderr(&output)
    );
}

#[cfg(all(feature = "accelerate", target_os = "macos"))]
#[test]
fn blas_with_the_accelerate_feature_is_accepted() {
    let output = train_cell(&["--numerics", "blas"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("--train-events requires a value"),
        "{}",
        stderr(&output)
    );
}
