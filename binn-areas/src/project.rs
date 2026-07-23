//! Assembly projection and association (U07-U08).
//!
//! [`project`] is the scientific path: source cells are activated through the
//! event engine, postsynaptic charge is measured after weighted/delayed routing,
//! and area inhibition selects the k winners. [`project_reference`] is a labeled
//! algebraic oracle for parity/debugging only.

use binn_engine::{CellId, Engine};

use crate::area::Area;
use crate::assembly::Assembly;
use crate::wta::k_wta;

const ASSOCIATE_BOOST: f32 = 5.0;
const PROJECT_HEBB: f32 = 1.0;

/// Event-driven Assembly-Calculus projection used by Gate G1.
///
/// The external `fire` primitive activates `src`; ordinary engine fan-out then
/// delivers weighted, delayed impulses. The k-WTA cap is applied to measured
/// delivered charge. Plasticity changes weights on existing random synapses
/// only: this function never creates an edge.
pub fn project(engine: &mut Engine, src: &Assembly, dst: &mut Area) -> Assembly {
    validate(engine, dst);
    let fire_at = engine
        .time()
        .checked_add(1)
        .expect("projection time overflow");
    for &cell in &src.members {
        engine.force_spike(cell, fire_at);
    }

    let settle_until = fire_at
        .checked_add(engine.max_synaptic_delay().max(1))
        .expect("projection settle time overflow");
    let _ = engine.step_until(settle_until);

    let scores: Vec<(CellId, f32)> = dst
        .cells
        .clone()
        .map(|cell| (cell, engine.last_step_charge(cell)))
        .collect();
    let winners = k_wta(&scores, dst.effective_k());
    dst.log_activity(winners.len());
    let assembly = Assembly::from_members(winners);

    // The k-WTA cycle is an explicitly inhibited measurement window. Any
    // later recurrent activity belongs to a different cycle and is suppressed.
    engine.close_inhibited_cycle();
    potentiate_existing(engine, src, &assembly, PROJECT_HEBB);
    assembly
}

/// Direct algebraic projection oracle for debugging and parity checks.
///
/// This function does not establish emergent neural dynamics and must not be
/// used as the G1 or C1 scientific result.
pub fn project_reference(engine: &mut Engine, src: &Assembly, dst: &mut Area) -> Assembly {
    validate(engine, dst);
    let mut scores = vec![0.0f32; dst.len()];
    let base = dst.cells.start;
    for &pre in &src.members {
        if (pre as usize) >= engine.conn.nrows() {
            continue;
        }
        let row = pre as usize;
        let start = engine.conn.row_ptr[row] as usize;
        let end = engine.conn.row_ptr[row + 1] as usize;
        for edge in start..end {
            let post = engine.conn.col[edge];
            if dst.contains(post) {
                scores[(post - base) as usize] += engine.edge_w[edge];
            }
        }
    }
    let pairs: Vec<(CellId, f32)> = scores
        .iter()
        .enumerate()
        .map(|(i, &score)| (base + i as CellId, score))
        .collect();
    let winners = k_wta(&pairs, dst.effective_k());
    dst.log_activity(winners.len());
    let assembly = Assembly::from_members(winners);
    potentiate_existing(engine, src, &assembly, PROJECT_HEBB);
    assembly
}

/// Hebbian association on existing random synapses only.
pub fn associate(engine: &mut Engine, a: &Assembly, b: &Assembly) {
    potentiate_existing(engine, a, b, ASSOCIATE_BOOST);
    potentiate_existing(engine, b, a, ASSOCIATE_BOOST);
}

fn validate(engine: &Engine, dst: &Area) {
    let n = engine.num_cells();
    assert!((dst.cells.end as usize) <= n, "area exceeds engine cells");
    assert_eq!(engine.conn.nrows(), n, "connectivity must cover all cells");
    assert_eq!(engine.edge_w.len(), engine.conn.nnz());
    assert_eq!(engine.syn.len(), engine.conn.nnz());
}

fn potentiate_existing(engine: &mut Engine, pre_set: &Assembly, post_set: &Assembly, boost: f32) {
    let mut selected = Vec::new();
    for &pre in &pre_set.members {
        let row = pre as usize;
        if row >= engine.conn.nrows() {
            continue;
        }
        let start = engine.conn.row_ptr[row] as usize;
        let end = engine.conn.row_ptr[row + 1] as usize;
        for edge in start..end {
            if post_set.contains(engine.conn.col[edge]) {
                selected.push(edge);
            }
        }
    }
    for edge in selected {
        engine.potentiate_edge(edge, boost);
    }
}
