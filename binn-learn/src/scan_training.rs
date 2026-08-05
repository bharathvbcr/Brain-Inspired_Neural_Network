//! Reset-aware chunked-scan forward training trace (U19 / F1).
//!
//! Linear sub-threshold segments use `binn_core::assoc_scan`; declared spike
//! resets remain sequential barriers. This is a forward/local training path,
//! not BPTT and not a claim that reset dynamics parallelize through time.

use binn_core::{assoc_scan, State, DEFAULT_CHUNK_SIZE};

#[derive(Clone, Debug, PartialEq)]
pub struct ScanTrainingTrace {
    /// Membrane value after every input step.
    pub membrane: Vec<f32>,
    /// Local pre×post eligibility proxy per step.
    pub local_eligibility: Vec<f32>,
    /// Number of reset-free scan segments.
    pub segments: usize,
    /// Number of sequential reset barriers.
    pub reset_barriers: usize,
    /// Mean length of reset-free segments (steps).
    pub mean_segment_len: f64,
    /// Shortest reset-free segment.
    pub min_segment_len: usize,
    /// Longest reset-free segment.
    pub max_segment_len: usize,
    /// Steps that sit in segments longer than [`DEFAULT_CHUNK_SIZE`] (scan
    /// parallelizes across chunks inside those segments).
    pub parallelizable_steps: usize,
    /// `reset_barriers / n_steps` — fraction of timeline that forces a
    /// sequential restart of the scan monoid.
    pub barrier_fraction: f64,
    /// `1 - barrier_fraction` capped to `[0, 1]`: fraction of steps that are
    /// *not* barrier events. Does **not** claim full wall-clock speedup.
    pub scan_headroom: f64,
}

/// Scan a known reset schedule using parallel reset-free chunks.
///
/// `reset_after[t]` means the membrane is reset after recording step `t`.
pub fn forward_scan_training(
    inputs: &[f32],
    reset_after: &[bool],
    initial_v: f32,
    reset_v: f32,
    tau: f32,
    dt: f32,
) -> ScanTrainingTrace {
    assert_eq!(inputs.len(), reset_after.len());
    assert!(tau > 0.0 && dt > 0.0);
    if inputs.is_empty() {
        return ScanTrainingTrace {
            membrane: Vec::new(),
            local_eligibility: Vec::new(),
            segments: 0,
            reset_barriers: 0,
            mean_segment_len: 0.0,
            min_segment_len: 0,
            max_segment_len: 0,
            parallelizable_steps: 0,
            barrier_fraction: 0.0,
            scan_headroom: 0.0,
        };
    }

    let steps: Vec<State> = inputs
        .iter()
        .map(|&input| State::leak_step(input, tau, dt))
        .collect();
    let mut membrane = vec![0.0; inputs.len()];
    let mut local_eligibility = vec![0.0; inputs.len()];
    let mut start = 0usize;
    let mut v0 = initial_v;
    let mut segments = 0usize;
    let mut reset_barriers = 0usize;
    let mut segment_lens: Vec<usize> = Vec::new();
    let mut parallelizable_steps = 0usize;
    while start < inputs.len() {
        let end = reset_after[start..]
            .iter()
            .position(|&reset| reset)
            .map(|offset| start + offset + 1)
            .unwrap_or(inputs.len());
        let seg_len = end - start;
        segment_lens.push(seg_len);
        if seg_len > DEFAULT_CHUNK_SIZE {
            parallelizable_steps = parallelizable_steps.saturating_add(seg_len);
        }
        let scanned = assoc_scan(&steps[start..end], State::combine);
        for (offset, state) in scanned.into_iter().enumerate() {
            let index = start + offset;
            let value = state.apply(v0);
            membrane[index] = value;
            local_eligibility[index] = inputs[index] * value;
        }
        segments += 1;
        if reset_after[end - 1] {
            reset_barriers += 1;
            v0 = reset_v;
        } else {
            v0 = membrane[end - 1];
        }
        start = end;
    }
    let n = inputs.len();
    let mean_segment_len = segment_lens.iter().sum::<usize>() as f64 / segments as f64;
    let min_segment_len = segment_lens.iter().copied().min().unwrap_or(0);
    let max_segment_len = segment_lens.iter().copied().max().unwrap_or(0);
    let barrier_fraction = reset_barriers as f64 / n as f64;
    let scan_headroom = (1.0 - barrier_fraction).clamp(0.0, 1.0);
    ScanTrainingTrace {
        membrane,
        local_eligibility,
        segments,
        reset_barriers,
        mean_segment_len,
        min_segment_len,
        max_segment_len,
        parallelizable_steps,
        barrier_fraction,
        scan_headroom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequential(
        inputs: &[f32],
        reset_after: &[bool],
        mut v: f32,
        reset_v: f32,
        tau: f32,
        dt: f32,
    ) -> Vec<f32> {
        let mut out = Vec::new();
        for (&input, &reset) in inputs.iter().zip(reset_after) {
            v = State::leak_step(input, tau, dt).apply(v);
            out.push(v);
            if reset {
                v = reset_v;
            }
        }
        out
    }

    #[test]
    fn scan_training_matches_sequential_across_declared_resets() {
        let inputs: Vec<f32> = (0..1_000).map(|i| (i % 13) as f32 / 13.0).collect();
        let resets: Vec<bool> = (0..inputs.len()).map(|i| i % 97 == 96).collect();
        let trace = forward_scan_training(&inputs, &resets, 0.2, 0.0, 20.0, 1.0);
        let expected = sequential(&inputs, &resets, 0.2, 0.0, 20.0, 1.0);
        assert_eq!(trace.membrane.len(), expected.len());
        for (a, b) in trace.membrane.iter().zip(expected) {
            assert!((a - b).abs() < 2e-5, "{a} != {b}");
        }
        assert_eq!(trace.reset_barriers, resets.iter().filter(|&&x| x).count());
        assert_eq!(trace.segments, trace.reset_barriers + 1);
        assert!(trace.mean_segment_len > 0.0);
        assert!(trace.max_segment_len >= trace.min_segment_len);
        assert!((0.0..=1.0).contains(&trace.barrier_fraction));
        assert!((trace.scan_headroom - (1.0 - trace.barrier_fraction)).abs() < 1e-12);
    }

    #[test]
    fn eligibility_is_local_pre_times_post() {
        let trace = forward_scan_training(&[1.0, 0.5], &[false, false], 0.0, 0.0, 2.0, 1.0);
        assert_eq!(trace.local_eligibility[0], trace.membrane[0]);
        assert_eq!(trace.local_eligibility[1], 0.5 * trace.membrane[1]);
        assert_eq!(trace.reset_barriers, 0);
        assert_eq!(trace.segments, 1);
        assert_eq!(trace.scan_headroom, 1.0);
    }

    #[test]
    fn dense_resets_kill_scan_headroom() {
        let n = 64usize;
        let inputs = vec![0.1f32; n];
        let resets = vec![true; n];
        let trace = forward_scan_training(&inputs, &resets, 0.0, 0.0, 10.0, 1.0);
        assert_eq!(trace.reset_barriers, n);
        assert_eq!(trace.segments, n);
        assert_eq!(trace.max_segment_len, 1);
        assert_eq!(trace.parallelizable_steps, 0);
        assert!((trace.barrier_fraction - 1.0).abs() < 1e-12);
        assert!(trace.scan_headroom.abs() < 1e-12);
    }
}
