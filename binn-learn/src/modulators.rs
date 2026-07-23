//! Broadcast modulators (U10).
//!
//! Third-factor signals applied uniformly to synapses. The scalar gate used by
//! [`crate::ThreeFactor`] is `attention · (reward + novelty)` so that zeroing
//! attention or the value channels ablates plasticity as predicted.

/// Broadcast neuromodulatory third factors.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Modulators {
    /// Reward / value prediction error channel.
    pub reward: f32,
    /// Novelty / surprise channel.
    pub novelty: f32,
    /// Attention / plasticity gate (typically in `[0, 1]`).
    pub attention: f32,
}

impl Modulators {
    /// All channels zero — no plasticity drive.
    #[inline]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Construct from the three channels.
    #[inline]
    pub fn new(reward: f32, novelty: f32, attention: f32) -> Self {
        Self {
            reward,
            novelty,
            attention,
        }
    }

    /// Rewarded, attentive update (novelty off).
    #[inline]
    pub fn reward(reward: f32) -> Self {
        Self {
            reward,
            novelty: 0.0,
            attention: 1.0,
        }
    }

    /// Scalar third-factor gate: `attention · (reward + novelty)`.
    #[inline]
    pub fn scalar(self) -> f32 {
        self.attention * (self.reward + self.novelty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_is_attention_times_value() {
        let m = Modulators::new(0.5, 0.25, 2.0);
        assert!((m.scalar() - 1.5).abs() < 1e-6);
    }

    #[test]
    fn zero_modulator_is_zero_gate() {
        assert_eq!(Modulators::zero().scalar(), 0.0);
    }
}
