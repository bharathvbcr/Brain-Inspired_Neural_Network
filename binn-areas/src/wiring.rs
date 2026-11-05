//! Wiring prior: role/position → CSR (U08).

use std::ops::Range;

use binn_core::{Csr, Rng};
use binn_engine::CellId;

/// Functional role of an area in the wiring prior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AreaRole {
    /// Primarily local; minimal long-range fan-out.
    Sensory = 1,
    /// Default recurrent association area.
    Association = 2,
    /// Slightly elevated inter-area probability (still locality-dominated).
    Hub = 3,
}

/// Position of an area within the prior's area list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    /// Index into [`WiringPrior::areas`].
    pub area_index: usize,
}

impl Pos {
    /// Position at `area_index`.
    #[inline]
    pub fn new(area_index: usize) -> Self {
        Self { area_index }
    }
}

/// Compact description of multi-area random connectivity.
#[derive(Clone, Debug)]
pub struct WiringPrior {
    /// RNG seed (GC3 / determinism).
    pub seed: u64,
    /// Contiguous cell ranges, one per area (non-overlapping, covering `0..n`).
    pub areas: Vec<Range<CellId>>,
    /// Directed edge probability within an area.
    pub p_intra: f32,
    /// Directed edge probability across areas (base; role may modulate).
    pub p_inter: f32,
    /// Hard per-cell fan-out bound used by the scalable sampler.
    pub max_fan_out: usize,
}

impl WiringPrior {
    /// Prior over `areas` with the given edge probabilities.
    pub fn new(seed: u64, areas: Vec<Range<CellId>>, p_intra: f32, p_inter: f32) -> Self {
        assert!(!areas.is_empty(), "wiring prior requires at least one area");
        assert!((0.0..=1.0).contains(&p_intra), "p_intra must be in [0, 1]");
        assert!((0.0..=1.0).contains(&p_inter), "p_inter must be in [0, 1]");
        Self {
            seed,
            areas,
            p_intra,
            p_inter,
            max_fan_out: 256,
        }
    }

    /// Override the default per-cell fan-out bound.
    pub fn with_max_fan_out(mut self, max_fan_out: usize) -> Self {
        assert!(max_fan_out > 0, "max_fan_out must be positive");
        self.max_fan_out = max_fan_out;
        self
    }

    /// Total cell count (end of last area range).
    pub fn num_cells(&self) -> usize {
        self.areas.iter().map(|r| r.end as usize).max().unwrap_or(0)
    }

    /// Area index containing `cell`, if any.
    pub fn area_of(&self, cell: CellId) -> Option<usize> {
        self.areas.iter().position(|r| r.contains(&cell))
    }
}

const NO_AREA: u32 = u32::MAX;

struct SeenSet {
    stamps: Vec<u32>,
    gen_id: u32,
}

impl SeenSet {
    fn with_capacity(n: usize) -> Self {
        Self {
            stamps: vec![0; n],
            gen_id: 0,
        }
    }

    #[inline]
    fn begin(&mut self) {
        self.gen_id = self.gen_id.wrapping_add(1);
        if self.gen_id == 0 {
            self.stamps.fill(0);
            self.gen_id = 1;
        }
    }

    #[inline]
    fn insert(&mut self, candidate: CellId) -> bool {
        let idx = candidate as usize;
        if self.stamps[idx] == self.gen_id {
            false
        } else {
            self.stamps[idx] = self.gen_id;
            true
        }
    }
}

fn build_cell_area_table(prior: &WiringPrior, min_len: usize) -> Vec<u32> {
    let table_len = prior.num_cells().max(min_len);
    let mut table = vec![NO_AREA; table_len];
    // Iterate in reverse so the earliest area in `prior.areas` wins on overlap,
    // matching `prior.areas.iter().position(|r| r.contains(&cell))` identically.
    for (area_idx, range) in prior.areas.iter().enumerate().rev() {
        let start = (range.start as usize).min(table_len);
        let end = (range.end as usize).min(table_len);
        let tag = u32::try_from(area_idx).expect("area index exceeds u32::MAX - 1");
        debug_assert_ne!(tag, NO_AREA);
        table[start..end].fill(tag);
    }
    table
}

/// Generate directed CSR connectivity from a wiring prior.
///
/// `role` and `pos` modulate outgoing density for the area at `pos` (hubs a bit
/// more global, sensory more local). Expected probability-derived degrees are
/// capped by [`WiringPrior::max_fan_out`], so generation is O(N * fan-out)
/// rather than enumerating all O(N²) candidate pairs.
///
/// Same `(role, pos, prior.seed, prior.areas, prior.p_*)` always yields an
/// identical CSR (GC3).
pub fn wire(role: AreaRole, pos: Pos, prior: &WiringPrior) -> Csr {
    assert!(
        pos.area_index < prior.areas.len(),
        "pos.area_index {} out of range ({} areas)",
        pos.area_index,
        prior.areas.len()
    );

    let n = prior.num_cells();
    let seed = prior.seed
        ^ ((role as u64) << 48)
        ^ ((pos.area_index as u64) << 32)
        ^ ((prior.areas.len() as u64) << 16);
    let mut rng = Rng::new(seed);

    // Role modulation applies to edges that touch the area at `pos`.
    let (p_intra_touch, p_inter_touch) = match role {
        AreaRole::Sensory => (prior.p_intra.min(1.0), prior.p_inter * 0.5),
        AreaRole::Association => (prior.p_intra, prior.p_inter),
        AreaRole::Hub => (
            prior.p_intra,
            (prior.p_inter * 1.5).min(prior.p_intra * 0.25),
        ),
    };

    let cell_area = build_cell_area_table(prior, n);
    let mut seen = SeenSet::with_capacity(n);
    let mut row_ptr = Vec::with_capacity(n.checked_add(1).expect("CSR row_ptr overflow"));
    let mut col = Vec::with_capacity(n.saturating_mul(prior.max_fan_out.min(n)));
    let mut max_col_width = n;
    row_ptr.push(0);

    for (pre, &pre_area_tag) in cell_area.iter().enumerate() {
        if pre_area_tag == NO_AREA {
            row_ptr.push(u32::try_from(col.len()).expect("CSR adjacency offset overflowed u32"));
            continue;
        }
        let pre_area = pre_area_tag as usize;
        let pre_id = pre as CellId;
        let local = &prior.areas[pre_area];
        let local_candidates = local.len().saturating_sub(1);
        let remote_candidates = n.saturating_sub(local.len());
        let touches_focus = pre_area == pos.area_index;
        let p_local = if touches_focus {
            p_intra_touch
        } else {
            prior.p_intra
        };
        let p_remote = if touches_focus {
            p_inter_touch
        } else {
            prior.p_inter
        };
        let expected_local = p_local as f64 * local_candidates as f64;
        let expected_remote = p_remote as f64 * remote_candidates as f64;
        let (take_local, take_remote) = degree_budget(
            expected_local,
            expected_remote,
            prior.max_fan_out,
            local_candidates,
            remote_candidates,
        );

        let row_start = col.len();
        sample_range_excluding(local, pre_id, take_local, &mut rng, &mut seen, &mut col);
        sample_outside_range(n, local, take_remote, &mut rng, &mut seen, &mut col);
        col[row_start..].sort_unstable();
        if let Some(&last) = col[row_start..].last() {
            max_col_width = max_col_width.max(last as usize + 1);
        }
        row_ptr.push(u32::try_from(col.len()).expect("CSR adjacency offset overflowed u32"));
    }

    Csr::from_parts_with_ncols(row_ptr, col, max_col_width)
        .expect("wire produced structurally valid sorted CSR")
}

fn degree_budget(
    expected_local: f64,
    expected_remote: f64,
    cap: usize,
    local_candidates: usize,
    remote_candidates: usize,
) -> (usize, usize) {
    let total = expected_local + expected_remote;
    if total == 0.0 {
        return (0, 0);
    }
    let scale = (cap as f64 / total).min(1.0);
    let local = (expected_local * scale).round() as usize;
    let remote = (expected_remote * scale).round() as usize;
    let mut local = local.min(local_candidates);
    let mut remote = remote.min(remote_candidates);
    while local + remote > cap {
        if remote > local {
            remote -= 1;
        } else {
            local -= 1;
        }
    }
    (local, remote)
}

fn sample_range_excluding(
    range: &Range<CellId>,
    excluded: CellId,
    take: usize,
    rng: &mut Rng,
    seen: &mut SeenSet,
    out: &mut Vec<CellId>,
) {
    let candidates = range
        .len()
        .saturating_sub(usize::from(range.contains(&excluded)));
    sample_unique(
        take.min(candidates),
        seen,
        out,
        || range.start + rng.gen_index(range.len()) as CellId,
        |candidate| candidate != excluded,
    );
}

fn sample_outside_range(
    n: usize,
    excluded: &Range<CellId>,
    take: usize,
    rng: &mut Rng,
    seen: &mut SeenSet,
    out: &mut Vec<CellId>,
) {
    let candidates = n.saturating_sub(excluded.len());
    sample_unique(
        take.min(candidates),
        seen,
        out,
        || rng.gen_index(n) as CellId,
        |candidate| !excluded.contains(&candidate),
    );
}

fn sample_unique(
    take: usize,
    seen: &mut SeenSet,
    out: &mut Vec<CellId>,
    mut draw: impl FnMut() -> CellId,
    mut eligible: impl FnMut(CellId) -> bool,
) {
    if take == 0 {
        return;
    }
    seen.begin();
    let start_len = out.len();
    while out.len() - start_len < take {
        let candidate = draw();
        if eligible(candidate) && seen.insert(candidate) {
            out.push(candidate);
        }
    }
}

/// Fraction of directed edges whose endpoints lie in the same area.
pub fn intra_area_edge_fraction(conn: &Csr, prior: &WiringPrior) -> f32 {
    let total = conn.nnz();
    if total == 0 {
        return 1.0;
    }
    let cell_area = build_cell_area_table(prior, conn.nrows().max(conn.ncols()));
    let mut intra = 0usize;
    for pre in 0..conn.nrows() {
        let start = conn.row_ptr[pre] as usize;
        let end = conn.row_ptr[pre + 1] as usize;
        if start == end {
            continue;
        }
        let pre_area = cell_area.get(pre).copied().unwrap_or(NO_AREA);
        if pre_area == NO_AREA {
            continue;
        }
        for &post in &conn.col[start..end] {
            if cell_area.get(post as usize).copied() == Some(pre_area) {
                intra += 1;
            }
        }
    }
    intra as f32 / total as f32
}

/// Fraction of routed synaptic deliveries that remain within an area for a
/// disclosed presynaptic spike workload.
pub fn intra_area_event_fraction(
    conn: &Csr,
    prior: &WiringPrior,
    presynaptic_spikes: &[u64],
) -> f64 {
    assert_eq!(presynaptic_spikes.len(), conn.nrows());
    let cell_area = build_cell_area_table(prior, conn.nrows().max(conn.ncols()));
    let mut intra = 0u128;
    let mut total = 0u128;
    for (pre, &spk) in presynaptic_spikes.iter().enumerate() {
        let start = conn.row_ptr[pre] as usize;
        let end = conn.row_ptr[pre + 1] as usize;
        let deg = end - start;
        if deg == 0 || spk == 0 {
            continue;
        }
        let events = spk as u128;
        total += events * (deg as u128);
        let pre_area = cell_area.get(pre).copied().unwrap_or(NO_AREA);
        if pre_area == NO_AREA {
            continue;
        }
        let mut row_intra = 0usize;
        for &post in &conn.col[start..end] {
            if cell_area.get(post as usize).copied() == Some(pre_area) {
                row_intra += 1;
            }
        }
        intra += events * (row_intra as u128);
    }
    if total == 0 {
        1.0
    } else {
        intra as f64 / total as f64
    }
}

#[cfg(test)]
mod parity_tests {
    use super::*;

    fn wire_naive_reference(role: AreaRole, pos: Pos, prior: &WiringPrior) -> Csr {
        let n = prior.num_cells();
        let seed = prior.seed
            ^ ((role as u64) << 48)
            ^ ((pos.area_index as u64) << 32)
            ^ ((prior.areas.len() as u64) << 16);
        let mut rng = Rng::new(seed);
        let (p_intra_touch, p_inter_touch) = match role {
            AreaRole::Sensory => (prior.p_intra.min(1.0), prior.p_inter * 0.5),
            AreaRole::Association => (prior.p_intra, prior.p_inter),
            AreaRole::Hub => (
                prior.p_intra,
                (prior.p_inter * 1.5).min(prior.p_intra * 0.25),
            ),
        };
        let mut rows: Vec<Vec<u32>> = vec![Vec::new(); n];
        for (pre, row) in rows.iter_mut().enumerate() {
            let pre_id = pre as CellId;
            let Some(pre_area) = prior.area_of(pre_id) else {
                continue;
            };
            let local = &prior.areas[pre_area];
            let local_candidates = local.len().saturating_sub(1);
            let remote_candidates = n.saturating_sub(local.len());
            let touches_focus = pre_area == pos.area_index;
            let p_local = if touches_focus {
                p_intra_touch
            } else {
                prior.p_intra
            };
            let p_remote = if touches_focus {
                p_inter_touch
            } else {
                prior.p_inter
            };
            let (take_local, take_remote) = degree_budget(
                p_local as f64 * local_candidates as f64,
                p_remote as f64 * remote_candidates as f64,
                prior.max_fan_out,
                local_candidates,
                remote_candidates,
            );
            let local_take = take_local.min(
                local
                    .len()
                    .saturating_sub(usize::from(local.contains(&pre_id))),
            );
            let s0 = row.len();
            while row.len() - s0 < local_take {
                let c = local.start + rng.gen_index(local.len()) as CellId;
                if c != pre_id && !row[s0..].contains(&c) {
                    row.push(c);
                }
            }
            let remote_take = take_remote.min(n.saturating_sub(local.len()));
            let s1 = row.len();
            while row.len() - s1 < remote_take {
                let c = rng.gen_index(n) as CellId;
                if !local.contains(&c) && !row[s1..].contains(&c) {
                    row.push(c);
                }
            }
            row.sort_unstable();
        }
        Csr::from_adjacency(&rows)
    }

    #[test]
    fn wire_and_fractions_match_naive_reference_bit_for_bit() {
        // Includes gapped cell ids (10..15 unmapped) and high collision density.
        let areas = vec![0..10u32, 15..95u32, 95..140u32];
        let mut checked = 0usize;
        for seed in [1u64, 42, 0xDEAD_BEEF] {
            let prior = WiringPrior::new(seed, areas.clone(), 0.75, 0.35).with_max_fan_out(64);
            for role in [AreaRole::Sensory, AreaRole::Association, AreaRole::Hub] {
                for area_idx in 0..areas.len() {
                    let got = wire(role, Pos::new(area_idx), &prior);
                    let exp = wire_naive_reference(role, Pos::new(area_idx), &prior);
                    assert_eq!(got, exp, "CSR mismatch for role={role:?} area={area_idx}");

                    let ref_intra_edges = got
                        .edges()
                        .filter(|&(pre, post)| {
                            prior.area_of(pre) == prior.area_of(post)
                                && prior.area_of(pre).is_some()
                        })
                        .count();
                    let ref_edge_frac = ref_intra_edges as f32 / got.nnz() as f32;
                    assert_eq!(
                        intra_area_edge_fraction(&got, &prior).to_bits(),
                        ref_edge_frac.to_bits()
                    );

                    let spikes: Vec<u64> = (0..got.nrows()).map(|i| (i as u64 % 5) + 1).collect();
                    let mut ref_ev_intra = 0u128;
                    let mut ref_ev_total = 0u128;
                    for (pre, post) in got.edges() {
                        let ev = spikes[pre as usize] as u128;
                        ref_ev_total += ev;
                        if prior.area_of(pre) == prior.area_of(post) && prior.area_of(pre).is_some()
                        {
                            ref_ev_intra += ev;
                        }
                    }
                    let ref_ev_frac = ref_ev_intra as f64 / ref_ev_total as f64;
                    assert_eq!(
                        intra_area_event_fraction(&got, &prior, &spikes).to_bits(),
                        ref_ev_frac.to_bits()
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 27);
    }
}
