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
        let expected_local = p_local as f64 * local_candidates as f64;
        let expected_remote = p_remote as f64 * remote_candidates as f64;
        let (take_local, take_remote) = degree_budget(
            expected_local,
            expected_remote,
            prior.max_fan_out,
            local_candidates,
            remote_candidates,
        );

        sample_range_excluding(local, pre_id, take_local, &mut rng, row);
        sample_outside_range(n, local, take_remote, &mut rng, row);
        row.sort_unstable();
    }

    Csr::from_adjacency(&rows)
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
    out: &mut Vec<CellId>,
) {
    let candidates = range
        .len()
        .saturating_sub(usize::from(range.contains(&excluded)));
    sample_unique(
        take.min(candidates),
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
    out: &mut Vec<CellId>,
) {
    let candidates = n.saturating_sub(excluded.len());
    sample_unique(
        take.min(candidates),
        out,
        || rng.gen_index(n) as CellId,
        |candidate| !excluded.contains(&candidate),
    );
}

fn sample_unique(
    take: usize,
    out: &mut Vec<CellId>,
    mut draw: impl FnMut() -> CellId,
    mut eligible: impl FnMut(CellId) -> bool,
) {
    let start_len = out.len();
    while out.len() - start_len < take {
        let candidate = draw();
        if eligible(candidate) && !out[start_len..].contains(&candidate) {
            out.push(candidate);
        }
    }
}

/// Fraction of directed edges whose endpoints lie in the same area.
pub fn intra_area_edge_fraction(conn: &Csr, prior: &WiringPrior) -> f32 {
    let mut intra = 0usize;
    let mut total = 0usize;
    for (pre, post) in conn.edges() {
        total += 1;
        if prior.area_of(pre) == prior.area_of(post) && prior.area_of(pre).is_some() {
            intra += 1;
        }
    }
    if total == 0 {
        return 1.0;
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
    let mut intra = 0u128;
    let mut total = 0u128;
    for (pre, post) in conn.edges() {
        let events = presynaptic_spikes[pre as usize] as u128;
        total += events;
        if prior.area_of(pre) == prior.area_of(post) && prior.area_of(pre).is_some() {
            intra += events;
        }
    }
    if total == 0 {
        1.0
    } else {
        intra as f64 / total as f64
    }
}
