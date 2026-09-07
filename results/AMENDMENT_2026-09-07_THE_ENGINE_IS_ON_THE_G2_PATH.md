# Amendment — `binn-engine` is on the Gate G2 path, and the register says it is not

**Date:** 2026-09-07
**Amends:** [`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md) §2b and §3
**Also amends:** [`TODO_2026-08-07_OPEN_WORK.md`](TODO_2026-08-07_OPEN_WORK.md) §6, first entry

The register's honesty about its own gaps is the reason it is worth keeping. This
amendment corrects one claim inside that gap list, and closes part of it.

---

## 1. The correction

The register says of `binn-engine`, `binn-areas` and `binn-core`, in two places:

> §2b — "**BINN proper remains semantically unaudited**, and no result in this
> repository currently depends on it."
>
> §3 — "Not on the instrument path, so no current result depends on them."

**The second half of each sentence is false.** Gate G2 — the paper's lead
negative, the frozen hash `c1-118207fbc3eaba53` — executes `binn-engine` and
`binn-areas` on its canonical path.

The chain, each link read rather than assumed:

| step | evidence |
|---|---|
| the G2 replay command names the `c1` binary | `PAPER_DRAFT.md` §5; `cargo run -p binn-lab --bin c1 -- --config-hash c1-118207fbc3eaba53` |
| `c1` is `experiments/c1.rs` | `binn-lab/Cargo.toml` `[[bin]] name = "c1"`, `path = "experiments/c1.rs"` |
| it runs through the shared harness | `c1.rs:41` `use binn_lab::{...}` |
| the harness imports the engine and the areas | `binn-lab/src/runner.rs:13` `use binn_areas::{...}`, `:21` `use binn_engine::{CellId, Engine, K}` |
| and steps the engine on the trial path | `runner.rs:2049` `fn run_trial(...)`, containing `:2254` `eng.step_until(readout_until)` — one of nine `step_until` sites |
| the canonical protocol is the same substrate | `runner.rs:585` — every variant protocol describes itself as "same k-WTA / single-pass C1 substrate as v2" and as not reopening `c1-118207fbc3eaba53` |

What the register got right is narrower than what it wrote: these crates are not
on the **SHD instrument** path. `shd_instrument.rs` does not touch them. But the
matched-architecture and engine programmes are not the SHD instrument, and the
lead negative is theirs.

**Why this matters.** The sentence was load-bearing in exactly the wrong
direction: it is the reason the §3 risk column reads as deferrable, and it is
why `TODO_2026-08-07_OPEN_WORK.md` §6 could carry "whatever Gate 2 eventually
says is worth nothing until this is done" while the register beside it said
nothing depended on the code. Those two statements cannot both be true. The
TODO's is the correct one.

This is the same defect class as the two found on 2026-09-07 in the same
register family — `matrix_authorized` outliving its decision by five weeks, and
the record checks pointing at a corpus that had moved: **a document nothing
verifies against the artefacts it describes.**

---

## 2. What was swept on 2026-09-07, and what it found

The register names three invariants as untouched:

> "The event-driven engine's own invariants — timing-wheel ordering, event-queue
> correctness, `sparse.rs` CSR/CSC consistency — are untouched, and those are
> where a BINN-proper defect would actually live."

All three were read. **All three are covered, and better than the register
implies.**

**`binn-engine/src/queue.rs` (941 lines) — timing wheel and event queue.**
Sixteen tests, all executing, none `#[ignore]`d. The suite is built on parity
against a `BinaryHeap` reference implementation — scripted
(`parity_vs_binary_heap_scripted`) and property-based twice over
(`prop_parity_vs_heap`, and `prop_parity_vs_heap_large_ticks` with offsets to
three billion, so all eight wheel levels are reached). Beyond parity it pins
equal-tick tie-break as insertion order, cursor monotonicity against a naive
scan, and the occupancy mask against brute force. The file also **documents its
own coverage boundary** rather than leaving it to a reader — it records that the
small-offset property test "structurally cannot exercise
`skip_empty_and_cascade`'s forward-only slot scan" and adds the tests that do.

**`binn-core/src/sparse.rs` (402 lines) — CSR/CSC consistency.** Seven tests,
all executing. `csc_fan_in_matches_csr_edges` and
`csc_round_trip_edge_ids_cover_csr` pin the consistency invariant directly, with
two property tests on neighbour iteration and `from_parts` round-tripping.

**One apparent gap was probed and is not one.** `level_for` — which decides the
wheel level an entry is filed at — can be mutated in **either** direction for
`delta > 300` and every test still passes: 46 in `binn-engine`, 266 in
`binn-lab`. Mutated inside the small property test's own `0u8..200` range it
**is** caught, which shows the parity test is live rather than vacuous.

The reason is structural, and worth writing down because it took four mutations
to establish: `schedule` computes the slot from `entry.at`, not from the level,
and `scan_earliest` visits **every** occupied bucket through the mask and
returns the minimum stored tick. Delivery order therefore comes from the tick an
entry carries and not from the bucket it sits in, so a mis-levelled entry is
delivered on time after one extra cascade. **`level_for` is a performance hint,
not a correctness invariant**, and the absence of a test pinning it is correct.

The one caveat, stated because the code says otherwise: the comment in
`schedule` warns that a wrapped delta would "file the event at the wrong level,
and reorder delivery". The reordering half of that is not what the mutations
show. The guard it defends — the `checked_sub` panic on a cursor that has
overtaken an entry — is still worth having, and the two-directional mutation
result does not weaken it.

---

## 3. What remains unswept

Narrower than §3 of the register, and stated so the next reader does not re-do
what is done:

| area | status after 2026-09-07 |
|---|---|
| `binn-engine/src/queue.rs`, `binn-core/src/sparse.rs` | **semantically swept.** No defect. `level_for` established as a non-invariant by mutation |
| `binn-engine/src/{cell,engine,resting,parallel,synapse,spikelog}.rs`, `binn-areas/` | **still only grep-swept** (2026-08-03, classes A/C/D clean). No semantic audit |
| `binn-core/src/metal_backend.rs` | not a gap. `METAL_GPU_DISPATCH_IMPLEMENTED = false` is a single source of truth, `available()` returns false through it, and callers are refused loudly. `find_weak_checks.py` flags the parity test's early return, correctly, as a check that would pass having measured nothing — it is a placeholder for unlanded work, not a live claim |
| ~20 `binn-lab/experiments/*` binaries (~8,000 lines) | **unswept**, unchanged. This is now the largest audit gap in the repository |
| `scripts/*.py` | unswept by instruction |

**The register's §3 risk column should be read with §1 above in hand.** These
crates are not future work whose risk is deferred to "any future BINN claim";
the published Gate G2 negative already runs on them.
