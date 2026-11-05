//! Sparse CSR connectivity storage (U01).
//!
//! Public names stay `binn_core::{Csr, Csc, CsrError}`. Validation, sorting,
//! and stored column width come from [`sparsl`]; BINN call sites keep the
//! historical `from_parts(row_ptr, col)` signature and public `row_ptr` /
//! `col` fields.

/// Re-exported sparsl CSR validation errors (includes `RowUnsorted`,
/// `ColumnOutOfRange`, and CSC offset overflow).
pub use sparsl::CsrError;

/// Compressed-sparse-row connectivity graph.
///
/// `row_ptr` has length `nrows + 1`. Neighbors of row `r` are the column
/// indices `col[row_ptr[r] as usize .. row_ptr[r + 1] as usize]`.
///
/// `ncols` is the declared operator width (`max(nrows, max_col + 1)` when
/// inferred by [`Csr::from_parts`] / [`Csr::from_adjacency`]), so square
/// graphs of `N` adjacency lists keep width ≥ `N`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Csr {
    pub row_ptr: Vec<u32>,
    pub col: Vec<u32>,
    ncols: usize,
}

impl Default for Csr {
    /// Structurally valid zero-row CSR (`row_ptr == [0]`, `ncols == 0`).
    #[inline]
    fn default() -> Self {
        Self::empty(0)
    }
}

impl Csr {
    /// Build from explicit arrays after validating CSR invariants.
    ///
    /// Infers `ncols = max(nrows, max_col + 1)` (or `nrows` when there are no
    /// edges) so existing BINN call sites that omit an explicit width still
    /// compile and keep square cell graphs wide enough for every row index.
    pub fn from_parts(row_ptr: Vec<u32>, col: Vec<u32>) -> Result<Self, CsrError> {
        let ncols = infer_ncols(&row_ptr, &col);
        Self::from_parts_with_ncols(row_ptr, col, ncols)
    }

    /// Build with an explicit column count (trailing empty columns allowed).
    pub fn from_parts_with_ncols(
        row_ptr: Vec<u32>,
        col: Vec<u32>,
        ncols: usize,
    ) -> Result<Self, CsrError> {
        validate_csr(&row_ptr, &col, ncols)?;
        Ok(Self {
            row_ptr,
            col,
            ncols,
        })
    }

    /// Build from explicit arrays without validation (caller guarantees shape).
    ///
    /// Infers `ncols` the same way as [`Csr::from_parts`]. An unchecked CSR
    /// still cannot reach a Metal kernel: sparsl `Device::prepare` re-validates.
    #[inline]
    pub fn from_parts_unchecked(row_ptr: Vec<u32>, col: Vec<u32>) -> Self {
        let ncols = infer_ncols(&row_ptr, &col);
        Self {
            row_ptr,
            col,
            ncols,
        }
    }

    /// Build from per-row adjacency lists.
    ///
    /// Each row is stable-sorted (sparsl contract); duplicates are preserved.
    pub fn from_adjacency(rows: &[Vec<u32>]) -> Self {
        let nrows = rows.len();
        let row_ptr_len = nrows
            .checked_add(1)
            .expect("CSR row-pointer length overflowed usize");
        let nnz = rows
            .iter()
            .try_fold(0usize, |total, row| total.checked_add(row.len()))
            .expect("CSR adjacency non-zero count overflowed usize");
        u32::try_from(nnz)
            .expect("CSR adjacency has more non-zeros than u32 offsets can represent");

        let mut row_ptr = Vec::with_capacity(row_ptr_len);
        let mut col = Vec::with_capacity(nnz);
        let mut max_col: Option<u32> = None;
        row_ptr.push(0);
        for row in rows {
            let start = col.len();
            col.extend_from_slice(row);
            col[start..].sort();
            if let Some(&last) = col[start..].last() {
                max_col = Some(max_col.map_or(last, |m| m.max(last)));
            }
            row_ptr.push(
                u32::try_from(col.len())
                    .expect("CSR adjacency offset exceeded the prevalidated u32 range"),
            );
        }
        let ncols = match max_col {
            Some(m) => nrows.max(m as usize + 1),
            None => nrows,
        };
        Self {
            row_ptr,
            col,
            ncols,
        }
    }

    /// Empty graph with `nrows` rows and no edges (square: `ncols = nrows`).
    pub fn empty(nrows: usize) -> Self {
        let row_ptr_len = nrows
            .checked_add(1)
            .expect("CSR row-pointer length overflowed usize");
        Self {
            row_ptr: vec![0; row_ptr_len],
            col: Vec::new(),
            ncols: nrows,
        }
    }

    /// Number of rows.
    #[inline]
    pub fn nrows(&self) -> usize {
        self.row_ptr.len().saturating_sub(1)
    }

    /// Number of stored non-zeros (edges).
    #[inline]
    pub fn nnz(&self) -> usize {
        self.col.len()
    }

    /// Declared number of columns (operator width). O(1).
    #[inline]
    pub fn ncols(&self) -> usize {
        self.ncols
    }

    /// Column indices of neighbors for `row`.
    ///
    /// Panics if `row >= nrows()`.
    #[inline]
    pub fn row_cols(&self, row: usize) -> &[u32] {
        let start = self.row_ptr[row] as usize;
        let end = self.row_ptr[row + 1] as usize;
        &self.col[start..end]
    }

    /// Iterate neighbor column indices for `row`.
    ///
    /// Panics if `row >= nrows()`.
    #[inline]
    pub fn neighbors(&self, row: usize) -> impl Iterator<Item = u32> + '_ {
        self.row_cols(row).iter().copied()
    }

    /// Iterate `(row, col)` pairs over all edges in row-major order.
    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..self.nrows()).flat_map(move |r| {
            let row = r as u32;
            self.neighbors(r).map(move |c| (row, c))
        })
    }

    /// Build a CSC reverse index over this CSR (uses stored [`Csr::ncols`]).
    #[inline]
    pub fn to_csc(&self) -> Csc {
        Csc::from_csr(self)
    }

    /// Convert to a sparsl CSR for device prepare / SpMV.
    pub(crate) fn to_sparsl(&self) -> sparsl::Csr {
        sparsl::Csr::from_parts_unchecked(self.row_ptr.clone(), self.col.clone(), self.ncols)
    }
}

/// Compressed-sparse-column reverse index over CSR edge storage.
///
/// Checked `u32` degree / pointer / edge-id arithmetic matches [`sparsl::Csc`]
/// without cloning the source [`Csr`]. Public field layout matches the
/// historical BINN `Csc`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Csc {
    pub col_ptr: Vec<u32>,
    pub row: Vec<u32>,
    pub edge_idx: Vec<u32>,
}

impl Default for Csc {
    fn default() -> Self {
        Self::empty(0)
    }
}

impl Csc {
    /// Empty reverse index with `ncols` columns and no edges.
    pub fn empty(ncols: usize) -> Self {
        let col_ptr_len = ncols
            .checked_add(1)
            .expect("CSC column-pointer length overflowed usize");
        Self {
            col_ptr: vec![0; col_ptr_len],
            row: Vec::new(),
            edge_idx: Vec::new(),
        }
    }

    /// Build CSC fan-in from a CSR graph (uses [`Csr::ncols`]).
    pub fn from_csr(csr: &Csr) -> Self {
        Self::from_csr_rect(csr, csr.ncols()).unwrap_or_else(|e| {
            panic!("CSC build over a square graph: {e}");
        })
    }

    /// Build CSC fan-in with an explicit column count.
    pub fn from_csr_rect(csr: &Csr, ncols: usize) -> Result<Self, CsrError> {
        validate_csr(&csr.row_ptr, &csr.col, ncols)?;
        if ncols == 0 {
            return match csr.col.first() {
                Some(&col) => Err(CsrError::ColumnOutOfRange { col, ncols }),
                None => Ok(Self::empty(0)),
            };
        }
        let nnz = csr.nnz();
        let mut degrees = vec![0u32; ncols];
        for &c in &csr.col {
            let col = c as usize;
            if col >= ncols {
                return Err(CsrError::ColumnOutOfRange { col: c, ncols });
            }
            degrees[col] = degrees[col]
                .checked_add(1)
                .ok_or(CsrError::OffsetOverflow {
                    what: "column degree",
                })?;
        }

        let mut col_ptr = Vec::with_capacity(ncols + 1);
        col_ptr.push(0);
        let mut acc = 0u32;
        for &d in &degrees {
            acc = acc.checked_add(d).ok_or(CsrError::OffsetOverflow {
                what: "column pointer",
            })?;
            col_ptr.push(acc);
        }

        let mut row = vec![0u32; nnz];
        let mut edge_idx = vec![0u32; nnz];
        degrees.copy_from_slice(&col_ptr[..ncols]);
        let next = &mut degrees;
        for r in 0..csr.nrows() {
            let start = csr.row_ptr[r] as usize;
            let end = csr.row_ptr[r + 1] as usize;
            let row_u32 =
                u32::try_from(r).map_err(|_| CsrError::OffsetOverflow { what: "row index" })?;
            for e in start..end {
                let c = csr.col[e] as usize;
                let slot = next[c] as usize;
                row[slot] = row_u32;
                edge_idx[slot] = u32::try_from(e)
                    .map_err(|_| CsrError::OffsetOverflow { what: "edge index" })?;
                next[c] = next[c].checked_add(1).ok_or(CsrError::OffsetOverflow {
                    what: "column cursor",
                })?;
            }
        }

        Ok(Self {
            col_ptr,
            row,
            edge_idx,
        })
    }

    /// Number of columns (postsynaptic cells).
    #[inline]
    pub fn ncols(&self) -> usize {
        self.col_ptr.len().saturating_sub(1)
    }

    /// Number of stored non-zeros (edges).
    #[inline]
    pub fn nnz(&self) -> usize {
        self.edge_idx.len()
    }

    /// CSR edge indices of incoming synapses for postsynaptic `col`.
    ///
    /// Panics if `col >= ncols()`.
    #[inline]
    pub fn col_edge_indices(&self, col: usize) -> &[u32] {
        let start = self.col_ptr[col] as usize;
        let end = self.col_ptr[col + 1] as usize;
        &self.edge_idx[start..end]
    }

    /// Iterate `(pre, csr_edge_idx)` pairs for postsynaptic `col`.
    ///
    /// Panics if `col >= ncols()`.
    #[inline]
    pub fn incoming(&self, col: usize) -> impl Iterator<Item = (u32, u32)> + '_ {
        let start = self.col_ptr[col] as usize;
        let end = self.col_ptr[col + 1] as usize;
        (start..end).map(move |i| (self.row[i], self.edge_idx[i]))
    }
}

fn validate_csr(row_ptr: &[u32], col: &[u32], ncols: usize) -> Result<(), CsrError> {
    if row_ptr.is_empty() {
        return Err(CsrError::EmptyRowPtr);
    }
    if row_ptr[0] != 0 {
        return Err(CsrError::NonZeroStart { start: row_ptr[0] });
    }
    for i in 1..row_ptr.len() {
        if row_ptr[i] < row_ptr[i - 1] {
            return Err(CsrError::NotMonotonic { index: i });
        }
    }
    let end = *row_ptr.last().expect("row_ptr non-empty");
    if end as usize != col.len() {
        return Err(CsrError::NnzMismatch {
            row_ptr_end: end,
            col_len: col.len(),
        });
    }
    for &c in col {
        if c as usize >= ncols {
            return Err(CsrError::ColumnOutOfRange { col: c, ncols });
        }
    }
    for row in 0..row_ptr.len().saturating_sub(1) {
        let start = row_ptr[row] as usize;
        let end = row_ptr[row + 1] as usize;
        for edge in start.saturating_add(1)..end {
            if col[edge] < col[edge - 1] {
                return Err(CsrError::RowUnsorted { row, edge });
            }
        }
    }
    Ok(())
}

fn infer_ncols(row_ptr: &[u32], col: &[u32]) -> usize {
    let nrows = row_ptr.len().saturating_sub(1);
    match col.iter().copied().max() {
        Some(m) => nrows.max(m as usize + 1),
        None => nrows,
    }
}

#[cfg(test)]
mod tests {
    use super::{Csc, Csr, CsrError};
    use proptest::prelude::*;

    #[test]
    fn csr_from_adjacency_neighbors() {
        let csr = Csr::from_adjacency(&[vec![1, 2], vec![0], vec![]]);
        assert_eq!(csr.nrows(), 3);
        assert_eq!(csr.ncols(), 3);
        assert_eq!(csr.nnz(), 3);
        assert_eq!(csr.row_cols(0), &[1, 2]);
        assert_eq!(csr.neighbors(1).collect::<Vec<_>>(), vec![0]);
        assert_eq!(csr.neighbors(2).count(), 0);
    }

    #[test]
    fn csr_from_parts_rejects_bad_shape() {
        assert_eq!(Csr::from_parts(vec![], vec![]), Err(CsrError::EmptyRowPtr));
        assert_eq!(
            Csr::from_parts(vec![1, 1], vec![]),
            Err(CsrError::NonZeroStart { start: 1 })
        );
        assert_eq!(
            Csr::from_parts(vec![0, 2, 1], vec![0, 1]),
            Err(CsrError::NotMonotonic { index: 2 })
        );
        assert_eq!(
            Csr::from_parts(vec![0, 1], vec![0, 1]),
            Err(CsrError::NnzMismatch {
                row_ptr_end: 1,
                col_len: 2
            })
        );
    }

    #[test]
    fn csr_from_parts_rejects_unsorted_row() {
        assert_eq!(
            Csr::from_parts(vec![0, 2], vec![2, 0]),
            Err(CsrError::RowUnsorted { row: 0, edge: 1 })
        );
    }

    #[test]
    fn csr_from_parts_infers_square_width() {
        // Four rows, columns only touch index 2 → width must stay ≥ 4.
        let csr = Csr::from_parts(vec![0, 1, 2, 2, 3], vec![2, 2, 2]).expect("csr");
        assert_eq!(csr.nrows(), 4);
        assert_eq!(csr.ncols(), 4);
    }

    #[test]
    fn csr_edges_row_major_sorted() {
        // from_adjacency stable-sorts each row.
        let csr = Csr::from_adjacency(&[vec![2, 0], vec![1]]);
        let edges: Vec<_> = csr.edges().collect();
        assert_eq!(edges, vec![(0, 0), (0, 2), (1, 1)]);
    }

    #[test]
    fn csc_fan_in_matches_csr_edges() {
        // Edges: 0→2, 1→2, 3→2 (coincidence wiring).
        let csr = Csr::from_parts(vec![0, 1, 2, 2, 3], vec![2, 2, 2]).expect("csr");
        let csc = Csc::from_csr(&csr);
        assert_eq!(csc.ncols(), 4);
        assert_eq!(csc.nnz(), 3);
        assert!(csc.incoming(0).next().is_none());
        assert!(csc.incoming(1).next().is_none());
        let into_2: Vec<_> = csc.incoming(2).collect();
        assert_eq!(into_2, vec![(0, 0), (1, 1), (3, 2)]);
        assert!(csc.incoming(3).next().is_none());
    }

    #[test]
    fn csc_round_trip_edge_ids_cover_csr() {
        let csr = Csr::from_adjacency(&[vec![1, 2], vec![0, 2], vec![0]]);
        let csc = csr.to_csc();
        let mut seen = vec![false; csr.nnz()];
        for (pre, post) in csr.edges() {
            let hit = csc
                .incoming(post as usize)
                .any(|(p, e)| p == pre && csr.col[e as usize] == post);
            assert!(hit, "missing reverse entry for ({pre},{post})");
        }
        for e in csc.edge_idx.iter().map(|&e| e as usize) {
            seen[e] = true;
        }
        assert!(seen.iter().all(|&v| v));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Neighbor iteration recovers each adjacency list after stable-sort.
        #[test]
        fn csr_neighbor_iteration_matches_adjacency(
            rows in proptest::collection::vec(
                proptest::collection::vec(0u32..64, 0..8),
                0..16,
            )
        ) {
            let csr = Csr::from_adjacency(&rows);
            prop_assert_eq!(csr.nrows(), rows.len());
            prop_assert_eq!(
                csr.nnz(),
                rows.iter().map(Vec::len).sum::<usize>()
            );

            for (r, expected) in rows.iter().enumerate() {
                let mut sorted = expected.clone();
                sorted.sort();
                let via_iter: Vec<u32> = csr.neighbors(r).collect();
                prop_assert_eq!(&via_iter, &sorted);
                prop_assert_eq!(csr.row_cols(r), sorted.as_slice());

                let start = csr.row_ptr[r] as usize;
                let end = csr.row_ptr[r + 1] as usize;
                prop_assert_eq!(&csr.col[start..end], sorted.as_slice());
            }

            let flat_got: Vec<u32> = csr.edges().map(|(_, c)| c).collect();
            let mut flat_expected: Vec<u32> = Vec::new();
            for row in &rows {
                let mut sorted = row.clone();
                sorted.sort();
                flat_expected.extend(sorted);
            }
            prop_assert_eq!(flat_got, flat_expected);
        }

        /// `from_parts` must accept adjacency-built CSR and preserve neighbors.
        #[test]
        fn csr_from_parts_preserves_neighbors(
            rows in proptest::collection::vec(
                proptest::collection::vec(0u32..32, 0..6),
                0..12,
            )
        ) {
            let built = Csr::from_adjacency(&rows);
            let csr = Csr::from_parts(built.row_ptr.clone(), built.col.clone())
                .expect("adjacency CSR must be valid");
            for (r, expected) in rows.iter().enumerate() {
                let mut sorted = expected.clone();
                sorted.sort();
                let got: Vec<u32> = csr.neighbors(r).collect();
                prop_assert_eq!(&got, &sorted);
            }
        }
    }

    #[test]
    fn csr_from_parts_moves_vectors_without_cloning() {
        let row_ptr = vec![0u32, 2, 3];
        let col = vec![0u32, 2, 1];
        let expected_rp_ptr = row_ptr.as_ptr();
        let expected_col_ptr = col.as_ptr();
        let csr = Csr::from_parts(row_ptr, col).expect("valid csr");
        assert_eq!(
            csr.row_ptr.as_ptr(),
            expected_rp_ptr,
            "Csr::from_parts must move row_ptr without cloning"
        );
        assert_eq!(
            csr.col.as_ptr(),
            expected_col_ptr,
            "Csr::from_parts must move col without cloning"
        );
    }
}
