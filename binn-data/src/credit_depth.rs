//! Compositional credit-depth tasks (U15 / C3).
//!
//! A sequence of hidden-state transforms of tunable length (`depth`) maps a
//! start state to a terminal target. Local learners see only the terminal
//! reward; a disclosed gradient / supervised reference may use per-step
//! targets (GC1-exempt when labeled as such in the harness).

use binn_core::Rng;

/// Public knobs for a credit-depth stream.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditDepthConfig {
    /// RNG seed (GC3).
    pub seed: u64,
    /// Number of discrete hidden states.
    pub n_states: usize,
    /// Number of operations available at each layer (currently 2 in the oracle).
    pub n_operations: usize,
    /// Compositional depth (≥ 1).
    pub depth: usize,
}

impl CreditDepthConfig {
    /// Default scientific-sized state space.
    pub fn new(seed: u64, depth: usize) -> Self {
        Self {
            seed,
            n_states: 4,
            n_operations: 2,
            depth: depth.max(1),
        }
    }

    /// Compact quick/PILOT knobs.
    pub fn quick(seed: u64, depth: usize) -> Self {
        Self {
            seed,
            n_states: 4,
            n_operations: 2,
            depth: depth.max(1),
        }
    }

    /// Stable fingerprint of the public config (not drawn samples).
    pub fn fingerprint(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for word in [
            self.seed,
            self.n_states as u64,
            self.n_operations as u64,
            self.depth as u64,
        ] {
            h ^= word;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        h
    }
}

/// One compositional example: start → ops → terminal target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreditDepthExample {
    /// Initial hidden state.
    pub start: usize,
    /// Per-layer operation indices (length = depth).
    pub operations: Vec<usize>,
    /// Terminal target after composing `operations`.
    pub target: usize,
}

/// Deterministic oracle transition used by the task generator.
#[inline]
pub fn true_transition(state: usize, operation: usize, n_states: usize) -> usize {
    let n = n_states.max(1);
    match operation % 2 {
        0 => (state + 1) % n,
        _ => (state.wrapping_mul(n.saturating_sub(1)) + 1) % n,
    }
}

/// Compose `operations` from `start` under the oracle.
#[inline]
pub fn compose_target(start: usize, operations: &[usize], n_states: usize) -> usize {
    operations
        .iter()
        .fold(start, |s, &op| true_transition(s, op, n_states))
}

/// Seeded credit-depth task stream.
#[derive(Clone, Debug)]
pub struct CreditDepthTask {
    config: CreditDepthConfig,
    rng: Rng,
}

impl CreditDepthTask {
    /// Open a stream from config.
    pub fn new(config: CreditDepthConfig) -> Self {
        let seed = config.seed;
        Self {
            config,
            rng: Rng::new(seed),
        }
    }

    /// Borrow config.
    #[inline]
    pub fn config(&self) -> &CreditDepthConfig {
        &self.config
    }

    /// Config fingerprint for harness logs.
    #[inline]
    pub fn config_fingerprint(&self) -> u64 {
        self.config.fingerprint()
    }

    /// Draw one compositional example at the configured depth.
    pub fn next_example(&mut self) -> CreditDepthExample {
        draw_example(&mut self.rng, self.config.depth, self.config.n_states)
    }
}

/// Draw a single example (shared by harnesses that own their own RNG).
pub fn draw_example(rng: &mut Rng, depth: usize, n_states: usize) -> CreditDepthExample {
    let depth = depth.max(1);
    let n_states = n_states.max(1);
    let start = rng.gen_index(n_states);
    let operations: Vec<usize> = (0..depth).map(|_| rng.gen_index(2)).collect();
    let target = compose_target(start, &operations, n_states);
    CreditDepthExample {
        start,
        operations,
        target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_matches_composed_transitions() {
        let mut rng = Rng::new(7);
        for depth in 1..=8 {
            for _ in 0..32 {
                let ex = draw_example(&mut rng, depth, 4);
                let expected = compose_target(ex.start, &ex.operations, 4);
                assert_eq!(ex.target, expected);
                assert_eq!(ex.operations.len(), depth);
            }
        }
    }

    #[test]
    fn same_seed_reproduces_examples() {
        let mut a = CreditDepthTask::new(CreditDepthConfig::quick(42, 3));
        let mut b = CreditDepthTask::new(CreditDepthConfig::quick(42, 3));
        for _ in 0..16 {
            assert_eq!(a.next_example(), b.next_example());
        }
        assert_eq!(a.config_fingerprint(), b.config_fingerprint());
    }

    #[test]
    fn fingerprint_sensitive_to_depth() {
        let a = CreditDepthConfig::new(1, 2);
        let b = CreditDepthConfig::new(1, 3);
        assert_ne!(a.fingerprint(), b.fingerprint());
    }
}
