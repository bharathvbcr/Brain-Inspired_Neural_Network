//! Compositional credit-depth examples in the dense temporal form.
//!
//! The sibling of [`crate::shd_dense`] for
//! [`binn_data::credit_depth::CreditDepthTask`], and the missing hop between a
//! task that was written for depth studies and a stack that can run one.
//! `CreditDepthTask` had no callers at all before this.
//!
//! # The encoding, and the one decision in it
//!
//! One operation per timestep, so the network sees the composition in the order
//! it is applied — that order is the whole point, since `op0 ∘ op1 ≠ op1 ∘ op0`
//! under the task's oracle.
//!
//! The start state is presented at **every** timestep, not only the first. That
//! is a deliberate choice and it is the one thing here that could bias a depth
//! result: presenting it once would additionally require the network to *retain*
//! it across the sequence, so a depth effect would confound credit assignment
//! with memory. Holding it constant removes the memory demand and leaves the
//! composition. A study of retention would make the opposite choice, and should
//! say so.

use binn_core::Rng;
use binn_data::credit_depth::{draw_example, CreditDepthConfig};
use binn_learn::DenseTemporalExample;

/// Input width for a given config: one-hot start state, then one-hot operation.
#[inline]
pub fn credit_depth_input_width(config: &CreditDepthConfig) -> usize {
    config.n_states + config.n_operations
}

/// Chance accuracy for a config, `1 / n_states`.
#[inline]
pub fn credit_depth_chance(config: &CreditDepthConfig) -> f32 {
    1.0 / config.n_states.max(1) as f32
}

/// Draw `n` examples in the dense temporal form.
///
/// `rng` is threaded rather than seeded here so a caller can draw disjoint train
/// and test splits from one stream without the two silently sharing draws.
///
/// # Panics
///
/// Panics if the config has no states or no operations, which would make the
/// label space or the input encoding empty.
pub fn credit_depth_examples(
    config: &CreditDepthConfig,
    rng: &mut Rng,
    n: usize,
) -> Vec<DenseTemporalExample> {
    assert!(
        config.n_states >= 2,
        "a task with <2 states has no labels to tell apart"
    );
    assert!(
        config.n_operations >= 1,
        "a task with no operations has no composition"
    );
    let n_in = credit_depth_input_width(config);
    let timesteps = config.depth.max(1);
    (0..n)
        .map(|_| {
            let example = draw_example(rng, config.depth, config.n_states);
            let mut frames = vec![0.0f32; timesteps * n_in];
            for step in 0..timesteps {
                // Start state, held for the whole sequence. See the module header.
                frames[step * n_in + example.start] = 1.0;
                if let Some(&op) = example.operations.get(step) {
                    frames[step * n_in + config.n_states + (op % config.n_operations)] = 1.0;
                }
            }
            DenseTemporalExample {
                frames,
                timesteps,
                n_in,
                label: example.target as u32,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> CreditDepthConfig {
        CreditDepthConfig {
            seed: 7,
            n_states: 8,
            n_operations: 2,
            depth: 4,
        }
    }

    #[test]
    fn every_example_matches_the_shape_the_stack_validates() {
        let cfg = config();
        let mut rng = Rng::new(11);
        let examples = credit_depth_examples(&cfg, &mut rng, 32);
        assert_eq!(examples.len(), 32);
        for example in &examples {
            assert_eq!(example.timesteps, cfg.depth);
            assert_eq!(example.n_in, credit_depth_input_width(&cfg));
            assert_eq!(example.frames.len(), example.timesteps * example.n_in);
            assert!(
                (example.label as usize) < cfg.n_states,
                "label {} is outside the state space",
                example.label
            );
        }
    }

    #[test]
    fn the_operation_order_reaches_the_encoding() {
        // The task is non-commutative, so an encoding that lost order would make
        // the whole suite measure nothing. Two examples with the same start and
        // the same operation multiset in different orders must differ.
        let cfg = CreditDepthConfig {
            seed: 7,
            n_states: 4,
            n_operations: 2,
            depth: 2,
        };
        let n_in = credit_depth_input_width(&cfg);
        let build = |ops: [usize; 2]| {
            let mut frames = vec![0.0f32; 2 * n_in];
            for (step, &op) in ops.iter().enumerate() {
                frames[step * n_in] = 1.0;
                frames[step * n_in + cfg.n_states + op] = 1.0;
            }
            frames
        };
        assert_ne!(
            build([0, 1]),
            build([1, 0]),
            "the encoding is order-blind; a depth study on it would measure nothing"
        );
    }

    #[test]
    fn labels_are_not_all_one_class() {
        // A constant label column would make every accuracy the majority rate and
        // every arm look identical, which is the failure this workspace keeps
        // finding. Cheap to assert, and it would have caught it.
        let cfg = config();
        let mut rng = Rng::new(3);
        let examples = credit_depth_examples(&cfg, &mut rng, 200);
        let first = examples[0].label;
        assert!(
            examples.iter().any(|e| e.label != first),
            "every drawn example carries label {first}"
        );
    }

    #[test]
    fn draws_advance_the_stream_so_splits_do_not_overlap() {
        let cfg = config();
        let mut rng = Rng::new(5);
        let train = credit_depth_examples(&cfg, &mut rng, 8);
        let test = credit_depth_examples(&cfg, &mut rng, 8);
        assert_ne!(
            train.iter().map(|e| e.label).collect::<Vec<_>>(),
            test.iter().map(|e| e.label).collect::<Vec<_>>(),
            "the second draw repeated the first; train and test would be the same data"
        );
    }

    #[test]
    fn chance_is_one_over_the_state_count() {
        assert!((credit_depth_chance(&config()) - 0.125).abs() < 1e-6);
    }
}
