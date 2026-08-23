//! k-winners-take-all via partial select (U06).

use binn_core::Rng;
use binn_engine::CellId;

/// Select up to `k` winners from `(cell, score)` pairs.
///
/// Uses `select_nth_unstable_by` (average O(n)) then sorts winners by cell id
/// for a deterministic emission order. Ties break toward the higher score,
/// then the lower `CellId`.
///
/// Returns at most `k` cell ids (fewer when `scores` is shorter than `k`).
pub fn k_wta(scores: &[(CellId, f32)], k: usize) -> Vec<CellId> {
    if k == 0 || scores.is_empty() {
        return Vec::new();
    }

    let mut items: Vec<(CellId, f32)> = scores
        .iter()
        .copied()
        .filter(|(_, s)| s.is_finite())
        .collect();
    if items.is_empty() {
        return Vec::new();
    }

    let take = k.min(items.len());
    // Highest score first; lower CellId wins ties. Treat NaN as already filtered.
    let cmp = |a: &(CellId, f32), b: &(CellId, f32)| match b.1.partial_cmp(&a.1) {
        Some(ord) => ord.then_with(|| a.0.cmp(&b.0)),
        None => a.0.cmp(&b.0),
    };

    if take < items.len() {
        items.select_nth_unstable_by(take, cmp);
        items.truncate(take);
    }
    // Deterministic emission order.
    items.sort_unstable_by_key(|(id, _)| *id);
    items.into_iter().map(|(id, _)| id).collect()
}

/// Score every cell in `[start, end)` with `score_fn`, then run [`k_wta`].
pub fn k_wta_range(
    start: CellId,
    end: CellId,
    k: usize,
    mut score_fn: impl FnMut(CellId) -> f32,
) -> Vec<CellId> {
    let mut scores = Vec::with_capacity((end - start) as usize);
    let mut id = start;
    while id < end {
        scores.push((id, score_fn(id)));
        id += 1;
    }
    k_wta(&scores, k)
}

/// Soft/relaxed k-WTA: sample up to `k` distinct winners with probability
/// proportional to `softmax(score / temperature)`.
///
/// When `temperature <= 0` or non-finite, falls back to hard [`k_wta`].
/// Deterministic given `seed`. Used by live C1 protocol 21 (one disclosed T).
pub fn soft_k_wta(scores: &[(CellId, f32)], k: usize, temperature: f32, seed: u64) -> Vec<CellId> {
    if k == 0 || scores.is_empty() {
        return Vec::new();
    }
    if !temperature.is_finite() || temperature <= 0.0 {
        return k_wta(scores, k);
    }
    let mut pool: Vec<(CellId, f32)> = scores
        .iter()
        .copied()
        .filter(|(_, s)| s.is_finite())
        .collect();
    if pool.is_empty() {
        return Vec::new();
    }
    let take = k.min(pool.len());
    let mut rng = Rng::new(seed ^ 0x50F7_A7A0_0001);
    let mut winners = Vec::with_capacity(take);
    for _ in 0..take {
        let max_s = pool
            .iter()
            .map(|(_, s)| *s)
            .fold(f32::NEG_INFINITY, f32::max);
        let mut weights = Vec::with_capacity(pool.len());
        let mut total = 0.0f32;
        for (_, s) in &pool {
            let w = ((s - max_s) / temperature).exp().max(0.0);
            total += w;
            weights.push(w);
        }
        if total <= 0.0 || !total.is_finite() {
            // Degenerate: finish with hard select on remaining.
            let rest = k_wta(&pool, take - winners.len());
            winners.extend(rest);
            break;
        }
        let mut draw = rng.next_f32() * total;
        let mut picked = 0usize;
        for (i, w) in weights.iter().enumerate() {
            draw -= *w;
            if draw <= 0.0 {
                picked = i;
                break;
            }
            picked = i;
        }
        winners.push(pool[picked].0);
        pool.swap_remove(picked);
    }
    winners.sort_unstable();
    winners.dedup();
    winners
}

/// Straight-through k-WTA: hard winners for forward dynamics,
/// soft credit weights for backward credit assignment.
///
/// Returns `(hard_winners, soft_weights)` where:
/// - `hard_winners` is the deterministic hard k-WTA output (GC3-compliant)
/// - `soft_weights` is a `(CellId, f32)` vec with softmax-tempered weights
///   for ALL neurons in `scores`, used to scale credit signals
///
/// The straight-through trick: forward pass uses hard selection,
/// credit assignment uses the soft relaxation gradient. This preserves
/// spiking dynamics while allowing credit to flow through the
/// winner selection boundary.
pub fn k_wta_straight_through(
    scores: &[(CellId, f32)],
    k: usize,
    temperature: f32,
) -> (Vec<CellId>, Vec<(CellId, f32)>) {
    let hard_winners = k_wta(scores, k);
    let mut soft_weights = Vec::with_capacity(scores.len());

    if !temperature.is_finite() || temperature <= 0.0 {
        for &(id, _) in scores {
            let w = if hard_winners.contains(&id) { 1.0 } else { 0.0 };
            soft_weights.push((id, w));
        }
        return (hard_winners, soft_weights);
    }

    let max_s = scores
        .iter()
        .map(|&(_, s)| s)
        .fold(f32::NEG_INFINITY, f32::max);

    let mut total = 0.0;
    for &(id, s) in scores {
        let w = ((s - max_s) / temperature).exp().max(0.0);
        soft_weights.push((id, w));
        total += w;
    }

    if total > 0.0 && total.is_finite() {
        for (_, w) in &mut soft_weights {
            *w /= total;
        }
        // Normalize so max weight is 1.0
        let max_w = soft_weights.iter().map(|&(_, w)| w).fold(0.0f32, f32::max);
        if max_w > 0.0 {
            for (_, w) in &mut soft_weights {
                *w /= max_w;
            }
        }
    } else {
        // Degenerate case fallback
        soft_weights.clear();
        for &(id, _) in scores {
            let w = if hard_winners.contains(&id) { 1.0 } else { 0.0 };
            soft_weights.push((id, w));
        }
    }

    (hard_winners, soft_weights)
}

/// k-WTA that also returns the margin boundary potential.
///
/// Returns `(winners, v_boundary)` where `v_boundary` is the (k+1)-th
/// highest score (or `f32::NEG_INFINITY` if `scores.len() <= k`).
/// Used by `MarginScaledCredit` to focus plasticity near the decision boundary.
pub fn k_wta_with_margin(scores: &[(CellId, f32)], k: usize) -> (Vec<CellId>, f32) {
    let winners = k_wta(scores, k);
    let v_boundary = if scores.len() <= k {
        f32::NEG_INFINITY
    } else {
        scores
            .iter()
            .filter(|(id, s)| !winners.contains(id) && s.is_finite())
            .map(|(_, s)| *s)
            .fold(f32::NEG_INFINITY, f32::max)
    };
    (winners, v_boundary)
}

/// Temperature annealer for Soft-to-Hard WTA schedule.
#[derive(Clone, Copy, Debug)]
pub struct WtaAnnealer {
    pub t_start: f32,
    pub t_end: f32,
    pub max_epochs: usize,
}

impl WtaAnnealer {
    pub fn new(t_start: f32, t_end: f32, max_epochs: usize) -> Self {
        Self {
            t_start,
            t_end,
            max_epochs,
        }
    }

    /// Calculate temperature at epoch index `epoch` (0-indexed).
    pub fn temperature_at(&self, epoch: usize) -> f32 {
        if self.max_epochs <= 1 {
            return self.t_end;
        }
        let progress = (epoch as f32 / (self.max_epochs - 1) as f32).clamp(0.0, 1.0);
        if self.t_start <= 0.0 || self.t_end <= 0.0 {
            return self.t_end;
        }
        self.t_start * (self.t_end / self.t_start).powf(progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_wta_selects_top_k() {
        let scores = [(0, 1.0), (1, 5.0), (2, 3.0), (3, 4.0), (4, 0.5)];
        let winners = k_wta(&scores, 3);
        assert_eq!(winners.len(), 3);
        assert_eq!(winners, vec![1, 2, 3]);
    }

    #[test]
    fn k_wta_at_most_k() {
        let scores: Vec<_> = (0..20u32).map(|i| (i, i as f32)).collect();
        let winners = k_wta(&scores, 5);
        assert!(winners.len() <= 5);
        assert_eq!(winners.len(), 5);
        // Which cells win, not just how many. `k_wta` returns `min(k, finite)`
        // structurally, so the two length assertions above hold for any
        // selection at all -- including the k *lowest* scores. Scores here are
        // `i as f32`, so the winners are cells 15..19; the emission order is
        // ascending cell id, not descending score, because the final
        // `sort_unstable_by_key` above pins a deterministic order rather than a
        // ranking.
        assert_eq!(
            winners,
            [15, 16, 17, 18, 19],
            "k-WTA did not select the highest-scoring cells"
        );
    }

    #[test]
    fn k_wta_tie_breaks_by_cell_id() {
        let scores = [(3, 1.0), (1, 1.0), (2, 1.0)];
        let winners = k_wta(&scores, 2);
        assert_eq!(winners, vec![1, 2]);
    }

    #[test]
    fn soft_k_wta_is_seeded_and_bounded() {
        let scores = [(0, 1.0), (1, 5.0), (2, 3.0), (3, 4.0), (4, 0.5)];
        let a = soft_k_wta(&scores, 2, 1.0, 7);
        let b = soft_k_wta(&scores, 2, 1.0, 7);
        let c = soft_k_wta(&scores, 2, 1.0, 8);
        assert_eq!(a, b);
        assert!(a.len() <= 2);
        assert_ne!(a, c);
        let hardish = soft_k_wta(&scores, 2, 0.0, 1);
        assert_eq!(hardish, k_wta(&scores, 2));
    }

    #[test]
    fn straight_through_k_wta_matches_hard_and_soft() {
        let scores = [(0, 1.0), (1, 5.0), (2, 3.0), (3, 4.0), (4, 0.5)];
        let (hard, soft) = k_wta_straight_through(&scores, 2, 1.0);
        assert_eq!(hard, vec![1, 3]);
        assert_eq!(soft.len(), 5);
        // max score is 5.0 at id 1, so its soft weight should be 1.0
        let w_1 = soft.iter().find(|&&(id, _)| id == 1).unwrap().1;
        assert!((w_1 - 1.0).abs() < 1e-6);
        let w_4 = soft.iter().find(|&&(id, _)| id == 4).unwrap().1;
        assert!(w_4 < 1.0 && w_4 > 0.0);
    }

    #[test]
    fn k_wta_with_margin_finds_boundary() {
        let scores = [(0, 1.0), (1, 5.0), (2, 3.0), (3, 4.0), (4, 0.5)];
        // k=2: winners are [1, 3] (scores 5, 4). The next highest is id 2 (score 3).
        let (winners, v_b) = k_wta_with_margin(&scores, 2);
        assert_eq!(winners, vec![1, 3]);
        assert!((v_b - 3.0).abs() < 1e-6);

        // k >= len: boundary is -inf
        let (_, v_inf) = k_wta_with_margin(&scores, 5);
        assert_eq!(v_inf, f32::NEG_INFINITY);
    }
}
