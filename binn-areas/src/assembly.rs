//! Assembly: sparse co-active set of size k (U07).

use binn_core::Rng;
use binn_engine::CellId;

use crate::area::Area;

/// Stable co-active set (typically size `k` within an area).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assembly {
    /// Member cell ids (sorted ascending, unique).
    pub members: Vec<CellId>,
}

impl Assembly {
    /// Empty assembly.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from members; sorts and deduplicates.
    pub fn from_members(mut members: Vec<CellId>) -> Self {
        members.sort_unstable();
        members.dedup();
        Self { members }
    }

    /// Number of members.
    #[inline]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// True when no members.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// True when `id` is a member.
    #[inline]
    pub fn contains(&self, id: CellId) -> bool {
        self.members.binary_search(&id).is_ok()
    }

    /// Sample a uniform random size-`k` assembly from `area`.
    ///
    /// # Panics
    ///
    /// Panics if `k == 0` or `k > area.len()`.
    pub fn random_in_area(area: &Area, k: usize, rng: &mut Rng) -> Self {
        assert!(k > 0, "assembly requires k > 0");
        assert!(
            k <= area.len(),
            "k ({k}) exceeds area population ({})",
            area.len()
        );
        let n = area.len();
        // Partial Fisher–Yates over local indices, then map to cell ids.
        let mut idx: Vec<usize> = (0..n).collect();
        for i in 0..k {
            let j = i + rng.gen_index(n - i);
            idx.swap(i, j);
        }
        let start = area.cells.start;
        let members: Vec<CellId> = idx[..k].iter().map(|&i| start + i as CellId).collect();
        Self::from_members(members)
    }
}

#[inline]
fn is_strictly_sorted(xs: &[CellId]) -> bool {
    xs.windows(2).all(|w| w[0] < w[1])
}

/// `|A ∩ B|`.
pub fn overlap_count(a: &Assembly, b: &Assembly) -> usize {
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let sm = &small.members;
    let lg = &large.members;
    if sm.is_empty() {
        return 0;
    }
    if lg.len() <= sm.len().saturating_mul(4) && is_strictly_sorted(sm) && is_strictly_sorted(lg) {
        let mut i = 0usize;
        let mut j = 0usize;
        let mut count = 0usize;
        while i < sm.len() && j < lg.len() {
            let x = sm[i];
            let y = lg[j];
            if x == y {
                count += 1;
                i += 1;
                j += 1;
            } else if x < y {
                i += 1;
            } else {
                j += 1;
            }
        }
        return count;
    }
    sm.iter().filter(|id| large.contains(**id)).count()
}

/// Assembly overlap `|A ∩ B| / k` with `k = max(|A|, |B|)` (0 when both empty).
///
/// For equal-size assemblies this is the standard Assembly-Calculus overlap;
/// `E[|A ∩ B|] = k²/N` so `E[overlap] = k/N`.
pub fn overlap(a: &Assembly, b: &Assembly) -> f32 {
    let k = a.len().max(b.len());
    if k == 0 {
        return 0.0;
    }
    overlap_count(a, b) as f32 / k as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_count_merge_matches_binary_search_across_skew_ratios() {
        let balanced_a = Assembly::from_members((0..128u32).map(|i| i * 3).collect());
        let balanced_b = Assembly::from_members((0..128u32).map(|i| i * 4).collect());
        let expected_bal = balanced_a
            .members
            .iter()
            .filter(|id| balanced_b.contains(**id))
            .count();
        assert_eq!(overlap_count(&balanced_a, &balanced_b), expected_bal);
        assert_eq!(overlap_count(&balanced_b, &balanced_a), expected_bal);

        // Skewed (>4x): exercises the binary-search branch symmetrically.
        let tiny = Assembly::from_members(vec![0, 12, 99, 381]);
        let expected_skew = tiny
            .members
            .iter()
            .filter(|id| balanced_a.contains(**id))
            .count();
        assert_eq!(overlap_count(&tiny, &balanced_a), expected_skew);
        assert_eq!(overlap_count(&balanced_a, &tiny), expected_skew);
        assert_eq!(overlap_count(&Assembly::new(), &balanced_a), 0);
    }
}
