//! k-winners-take-all via partial select (U06).

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
    }

    #[test]
    fn k_wta_tie_breaks_by_cell_id() {
        let scores = [(3, 1.0), (1, 1.0), (2, 1.0)];
        let winners = k_wta(&scores, 2);
        assert_eq!(winners, vec![1, 2]);
    }
}
