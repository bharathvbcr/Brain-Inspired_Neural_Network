//! U06–U08 acceptance + GC3 determinism + Gate G1.

use binn_areas::wiring::{intra_area_edge_fraction, intra_area_event_fraction};
use binn_areas::{
    associate, overlap, overlap_count, project, wire, ActivityLog, Area, AreaRole, Assembly, Pos,
    WiringPrior,
};
use binn_core::Rng;
use binn_engine::{CellId, Engine};

// ─── helpers ───────────────────────────────────────────────────────────────

fn single_area_setup(n: usize, k: usize, seed: u64, p_intra: f32) -> (Engine, Area) {
    let mut eng = Engine::with_cells(n);
    // One-element `Vec<Range<_>>` is intentional (area partition), not a range collect.
    #[allow(clippy::single_range_in_vec_init)]
    let prior = WiringPrior::new(seed, vec![0..n as CellId], p_intra, 0.0);
    let conn = wire(AreaRole::Association, Pos::new(0), &prior);
    let weights = vec![0.05; conn.nnz()];
    eng.set_connectivity(conn, weights);
    let area = Area::new(0..n as CellId, k);
    (eng, area)
}

// ─── U06 · Area + k-WTA ────────────────────────────────────────────────────

#[test]
fn u06_kwta_at_most_k_winners_per_cycle() {
    let n = 200usize;
    let k = 10usize;
    let (mut eng, mut area) = single_area_setup(n, k, 0xA06_0001, 0.15);
    let mut rng = Rng::new(0xA06_5EED);

    for _ in 0..32 {
        let src = Assembly::random_in_area(&area, k, &mut rng);
        let winners = project(&mut eng, &src, &mut area);
        assert!(winners.len() <= k, "winners {} exceed k={k}", winners.len());
        for &c in &winners.members {
            assert!(area.contains(c));
        }
    }

    // Activity log: every cycle recorded, winners <= k.
    assert_eq!(area.activity.len(), 32);
    for sample in area.activity.samples() {
        assert!(sample.winners <= k);
        assert_eq!(sample.n, n);
        assert!((sample.activity_sparsity - sample.winners as f32 / n as f32).abs() < 1e-6);
        // GC7 field present on every sample.
        let _ = sample.activity_sparsity;
    }
}

#[test]
fn u06_measured_activity_approx_k_over_n() {
    let n = 500usize;
    let k = 10usize;
    let (mut eng, mut area) = single_area_setup(n, k, 0xA06_0002, 0.2);
    let mut rng = Rng::new(0xA06_00AC);

    for _ in 0..64 {
        let src = Assembly::random_in_area(&area, k, &mut rng);
        let _ = project(&mut eng, &src, &mut area);
    }

    let mean = area.activity.mean_activity_sparsity();
    let expected = k as f32 / n as f32;
    assert!(
        (mean - expected).abs() < 1e-5,
        "mean activity {mean} not approx k/N={expected}"
    );

    // Direct GC7 hook: ActivityLog records activity_sparsity.
    let mut log = ActivityLog::new();
    log.record(n, k);
    assert!((log.samples()[0].activity_sparsity - expected).abs() < 1e-6);
}

// ─── U07 · project convergence · Gate G1 ───────────────────────────────────

/// Disclosed seed sweep for Gate G1 (wiring seed xor assembly seed per trial).
/// Spec requires convergence across a disclosed multi-seed sweep, not one fixed seed.
const G1_DISCLOSED_SEEDS: &[u64] = &[
    0xB1_5EED,
    0xB1_5EED ^ 0x9E37_79B9_7F4A_7C15,
    0xC0FF_EE00_D15C_105E,
    0xA11C_E555_0000_0007,
    0xDEAD_BEEF_CAFE_BABE,
];

/// Gate G1 helper: repeated projection converges for one `(wiring, assembly)` seed pair.
fn g1_assert_project_converges(wiring_seed: u64, assembly_seed: u64) {
    let n = 400usize;
    let k = 20usize;
    let total = 2 * n;
    let mut eng = Engine::with_cells(total);
    let ranges = vec![0..n as CellId, n as CellId..(2 * n) as CellId];
    let prior = WiringPrior::new(wiring_seed, ranges, 0.1, 0.1);
    let conn = wire(AreaRole::Association, Pos::new(1), &prior);
    let initial_nnz = conn.nnz();
    eng.set_connectivity(conn, vec![0.05; initial_nnz]);

    let area_a = Area::new(0..n as CellId, k);
    let mut area_b = Area::new(n as CellId..(2 * n) as CellId, k);
    let mut rng = Rng::new(assembly_seed);
    let src = Assembly::random_in_area(&area_a, k, &mut rng);

    let mut curr = project(&mut eng, &src, &mut area_b);
    assert_eq!(
        curr.len(),
        k,
        "seed={assembly_seed:#x}: first project must return k winners"
    );

    const MAX_ROUNDS: usize = 20;
    let mut converged = false;
    let mut last_overlap = 0.0f32;
    for round in 0..MAX_ROUNDS {
        let next = project(&mut eng, &src, &mut area_b);
        assert!(next.len() <= k);
        last_overlap = overlap(&curr, &next);
        curr = next;
        if last_overlap > 0.9 {
            converged = true;
            let next2 = project(&mut eng, &src, &mut area_b);
            let ov2 = overlap(&curr, &next2);
            assert!(
                ov2 > 0.9,
                "G1: lost convergence at follow-up round (ov={ov2}, round={round}, seed={assembly_seed:#x})"
            );
            break;
        }
    }
    assert!(
        converged,
        "G1 FAIL: seed={assembly_seed:#x} successive overlap only reached {last_overlap} within {MAX_ROUNDS} rounds"
    );
    assert_eq!(
        eng.conn.nnz(),
        initial_nnz,
        "G1 plasticity must not create structural edges (seed={assembly_seed:#x})"
    );
}

/// Gate G1: repeated projection converges — successive overlap > 0.9 within N rounds.
///
/// Fixed source assembly in area A repeatedly projects into area B. Each
/// `project` Hebbian-imprints `src → winners`, so the support in B stabilizes.
#[test]
fn g1_project_converges_successive_overlap() {
    g1_assert_project_converges(0xB1_C0DE, 0xB1_5EED);
}

/// Gate G1 disclosed multi-seed sweep: every listed seed converges (overlap > 0.9).
#[test]
fn g1_project_converges_disclosed_seed_sweep() {
    const WIRING_SEED: u64 = 0xB1_C0DE;
    for &seed in G1_DISCLOSED_SEEDS {
        g1_assert_project_converges(WIRING_SEED ^ seed, seed);
    }
}

#[test]
fn u07_random_assembly_overlap_approx_k_sq_over_n() {
    let n = 1000usize;
    let k = 20usize;
    let area = Area::new(0..n as CellId, k);
    let mut rng = Rng::new(0x0B17_01A9);
    const TRIALS: usize = 400;

    let mut sum_inter = 0.0f64;
    for _ in 0..TRIALS {
        let a = Assembly::random_in_area(&area, k, &mut rng);
        let b = Assembly::random_in_area(&area, k, &mut rng);
        sum_inter += overlap_count(&a, &b) as f64;
    }
    let mean_inter = sum_inter / TRIALS as f64;
    // E[|A n B|] = k^2/N for uniform size-k subsets.
    let expected = (k * k) as f64 / n as f64;
    assert!(
        (mean_inter - expected).abs() < 0.35,
        "mean |AnB|={mean_inter} not approx k^2/N={expected}"
    );
}

// ─── U08 · associate + wiring prior ────────────────────────────────────────

#[test]
fn u08_associate_raises_inter_assembly_overlap() {
    let n = 300usize;
    let k = 15usize;
    let (mut eng, mut area) = single_area_setup(n, k, 0xA08_A5C0, 0.08);
    let mut rng = Rng::new(0xA08_A5C1);

    // Two well-separated random assemblies.
    let a = Assembly::random_in_area(&area, k, &mut rng);
    let mut b = Assembly::random_in_area(&area, k, &mut rng);
    // Resample b until overlap with a is low.
    for _ in 0..32 {
        if overlap(&a, &b) < 0.2 {
            break;
        }
        b = Assembly::random_in_area(&area, k, &mut rng);
    }

    let before = overlap(&project(&mut eng, &a, &mut area), &b);

    associate(&mut eng, &a, &b);

    let after = overlap(&project(&mut eng, &a, &mut area), &b);
    assert!(
        after > before,
        "associate must raise inter-assembly overlap (before={before}, after={after})"
    );
    assert!(
        after >= 0.5,
        "after associate, project(a) should recruit most of b (after={after})"
    );
}

#[test]
fn u08_wiring_prior_deterministic_same_seed() {
    let areas = vec![0..100u32, 100..200, 200..300];
    let prior = WiringPrior::new(0xB177_01FE, areas, 0.1, 0.001);
    let role = AreaRole::Hub;
    let pos = Pos::new(1);

    let a = wire(role, pos, &prior);
    let b = wire(role, pos, &prior);
    assert_eq!(a, b, "same seed must yield identical CSR");
    assert!(a.nnz() > 0);

    let mut prior2 = prior.clone();
    prior2.seed ^= 0x9E37_79B9_7F4A_7C15;
    let c = wire(role, pos, &prior2);
    assert_ne!(a, c, "different seeds must diverge");
}

#[test]
fn u08_wiring_respects_fan_out_cap_at_many_area_scale() {
    let areas: Vec<_> = (0..100u32)
        .map(|area| area * 100..(area + 1) * 100)
        .collect();
    let prior = WiringPrior::new(7, areas, 0.2, 0.01).with_max_fan_out(64);
    let conn = wire(AreaRole::Association, Pos::new(50), &prior);
    assert_eq!(conn.nrows(), 10_000);
    assert!(
        (0..conn.nrows()).all(|row| conn.row_cols(row).len() <= 64),
        "every row must respect the configured fan-out cap"
    );
    assert!(conn.nnz() <= conn.nrows() * 64);
}

/// Long-range wiring stays local -- **and the graph has long-range edges to be
/// local about**, which this gate did not check until 2026-08-22.
///
/// It ran at `p_inter = 0.001` and asserted `frac > 0.90`. At that setting every
/// role produces **zero** inter-area edges, so the fraction is exactly 1.0 by
/// construction and the assertion tested nothing -- it would have passed under a
/// total inversion of the role modulation.
///
/// The cause is in `wiring.rs::degree_budget`: out-degree is `round(expected)`
/// per cell rather than a Bernoulli draw per pair, so any `p_inter` whose
/// expected remote degree falls below 0.5 rounds to zero. `p_inter` is
/// documented as an "edge probability" and is not one in the realized model.
///
/// See `results/FINDING_2026-08-22_A_SWEEP_OF_BINN_PROPER.md` section 1.2.
#[test]
fn u08_wiring_yields_gt_90_percent_intra_area_events() {
    let areas = vec![0..120u32, 120..240, 240..360];
    // The smallest setting on the model's own step function that produces
    // long-range edges at all. Chosen by that rule and not by which value
    // passes: the measured locality at the rungs above and below is pinned in
    // `locality_falls_below_the_gate_at_a_denser_p_inter` and in the finding.
    let prior = WiringPrior::new(0xB177_10C4, areas, 0.12, 0.0021);
    for (idx, role) in [AreaRole::Sensory, AreaRole::Association, AreaRole::Hub]
        .into_iter()
        .enumerate()
    {
        let conn = wire(role, Pos::new(idx), &prior);

        // Refuse a vacuous pass. Without this the gate certifies locality on a
        // graph with nothing non-local in it.
        let inter: usize = conn
            .edges()
            .filter(|(pre, post)| prior.area_of(*pre) != prior.area_of(*post))
            .count();
        assert!(
            inter > 0,
            "role={role:?} produced no inter-area edges, so the locality \
             fraction is 1.0 by construction and this gate tests nothing"
        );

        let frac = intra_area_edge_fraction(&conn, &prior);
        assert!(
            frac > 0.90,
            "intra-area edge fraction {frac} <= 0.90 for role={role:?}"
        );
        let mut workload = vec![1u64; conn.nrows()];
        for (cell, spikes) in workload.iter_mut().enumerate() {
            *spikes = 1 + (cell % 7) as u64;
        }
        let event_frac = intra_area_event_fraction(&conn, &prior, &workload);
        assert!(
            event_frac > 0.90,
            "intra-area event fraction {event_frac} <= 0.90 for role={role:?}"
        );
    }
}

/// The 0.90 locality claim is conditional on `p_inter`, and the gate above never
/// said so because it could not discover it.
///
/// At `p_inter = 0.01` locality falls to 0.84-0.89 across the three roles --
/// below the gate's own bar. That is a property of the model, not a defect, but
/// leaving it unstated would let "wiring is >90% local" be read as unconditional.
/// Pinned here so the scope travels with the claim.
#[test]
fn locality_falls_below_the_gate_at_a_denser_p_inter() {
    let areas = vec![0..120u32, 120..240, 240..360];
    let prior = WiringPrior::new(0xB177_10C4, areas, 0.12, 0.01);
    let mut worst = 1.0f32;
    for (idx, role) in [AreaRole::Sensory, AreaRole::Association, AreaRole::Hub]
        .into_iter()
        .enumerate()
    {
        let conn = wire(role, Pos::new(idx), &prior);
        worst = worst.min(intra_area_edge_fraction(&conn, &prior));
    }
    assert!(
        worst < 0.90,
        "locality is {worst} at p_inter=0.01, at or above the 0.90 bar. If the \
         wiring model changed, the scope statement in \
         results/FINDING_2026-08-22_A_SWEEP_OF_BINN_PROPER.md must change with it"
    );
}

// ─── GC3 · seed => identical wiring / assembly hash ─────────────────────────

fn fingerprint(seed: u64) -> u64 {
    let n = 80usize;
    let k = 8usize;
    let areas = vec![0..40u32, 40..80];
    let prior = WiringPrior::new(seed, areas, 0.15, 0.002);
    let conn = wire(AreaRole::Association, Pos::new(0), &prior);

    let mut eng = Engine::with_cells(n);
    eng.set_connectivity(conn.clone(), vec![0.05; conn.nnz()]);
    let mut area = Area::new(0..n as CellId, k);
    let mut rng = Rng::new(seed ^ 0xA55E);
    let mut asm = Assembly::random_in_area(&area, k, &mut rng);
    for _ in 0..8 {
        asm = project(&mut eng, &asm, &mut area);
    }

    // FNV-1a over CSR edges + final assembly members + activity sparsity bits.
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for (r, c) in eng.conn.edges() {
        hash ^= ((r as u64) << 32) | (c as u64);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    for &m in &asm.members {
        hash ^= m as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    for s in area.activity.samples() {
        hash ^= s.winners as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
        hash ^= s.activity_sparsity.to_bits() as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

#[test]
fn gc3_same_seed_identical_wiring_assembly_hash() {
    let seed = 0xB177_C0DE_0000_0007;
    let h1 = fingerprint(seed);
    let h2 = fingerprint(seed);
    assert_eq!(h1, h2, "same seed must yield identical fingerprint");

    let h_other = fingerprint(seed ^ 0x9E37_79B9_7F4A_7C15);
    assert_ne!(h1, h_other, "different seeds must not collide");
}
