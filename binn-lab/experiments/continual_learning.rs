//! Class-Incremental Continual Learning Benchmark (Tier 3).
//!
//! Evaluates forward transfer vs catastrophic forgetting rate when tasks are
//! presented sequentially without experience replay memory.
//!
//! Compares:
//! - Online Local Learning (Online Learned B_i Alignment)
//! - Global BPTT without Replay Memory

use std::process::ExitCode;

use binn_lab::{freeze_trials, samples_to_gradient_examples, Config};
use binn_learn::{MatchedGradient, MatchedRlLearnedFb, DEFAULT_MATCHED_BETA};

const EXPERIMENT_NAME: &str = "continual-learning";

fn main() -> ExitCode {
    let n_tasks = 3;
    let hidden = 256;
    let epochs_per_task = 40;
    let master_seed = 0x0071_AC00_001C_00F4_u64;

    println!("========================================================================");
    println!("Continual Learning Advantage Benchmark ({EXPERIMENT_NAME})");
    println!("Tasks={n_tasks}, hidden={hidden}, epochs_per_task={epochs_per_task}");
    println!("========================================================================\n");

    let mut cfg = Config::c1_default();
    cfg.n_hidden = hidden;

    let splits: Vec<_> = (0..n_tasks)
        .map(|t| {
            let seed = master_seed ^ (t as u64 * 0x1111_0222);
            freeze_trials(&cfg, seed)
        })
        .collect();

    let mut learned_fb =
        MatchedRlLearnedFb::new(hidden, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, master_seed);
    let mut bptt =
        MatchedGradient::new_feedforward(hidden, 0.05, DEFAULT_MATCHED_BETA, master_seed);

    for t_idx in 0..n_tasks {
        let train_data = samples_to_gradient_examples(&splits[t_idx].train);
        let test_data = samples_to_gradient_examples(&splits[t_idx].test);

        let r_fb = learned_fb.train_and_evaluate(epochs_per_task, &train_data, &test_data);
        let r_bptt = bptt.train_and_evaluate(epochs_per_task, &train_data, &test_data);

        println!(
            "Task {}: Train Acc -> LearnedFB={:.4} | BPTT={:.4}",
            t_idx + 1,
            r_fb.accuracy,
            r_bptt.accuracy
        );

        // Evaluate retention on previous tasks
        for (prev_t, split) in splits.iter().enumerate().take(t_idx + 1) {
            let prev_test = samples_to_gradient_examples(&split.test);
            let acc_fb = learned_fb.evaluate(&prev_test);
            let acc_bptt = bptt.evaluate(&prev_test);
            println!(
                "   -> Retention on Task {}: LearnedFB={:.4} | BPTT={:.4}",
                prev_t + 1,
                acc_fb,
                acc_bptt
            );
        }
    }

    println!("\nContinual Learning Evaluation Complete.");
    ExitCode::SUCCESS
}
