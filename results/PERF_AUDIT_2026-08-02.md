# Performance audit — hot path, 2026-08-02

**Host of record:** Apple M5 Pro (64 GB), aarch64-apple-darwin.
**Method:** static reading of `binn-core`, `binn-engine`, `binn-areas`, `binn-learn`.
**Not done:** no profiling, no benchmarking, no compilation. Every cost figure
below is an operation count or a bandwidth estimate derived from the code, not a
measurement. Items are ranked by expected effect, and that ranking is a
hypothesis — the first action on this document should be to profile, not to
start at #1 and work down.

Notation, per tick: `E` = events, `D` = distinct target cells, `S` = spikes,
`F` = mean out-degree, `N` = pending events in queue, `n` = total cells,
`nnz` = edges.

---

## Applied in this pass (all bit-identical, no re-baseline needed)

| Location | Change | Why it is safe |
|---|---|---|
| `three_factor.rs:134` | CSR+CSC `clone()` → shared borrows | Same values, same order; only removes copies |
| `queue.rs` | Occupancy bitmasks replace the 2048-header `scan_earliest` sweep (§1) | Same minimum over the same entry set |
| `engine.rs:501` | Fan-out iterates slices; one bounds check instead of per-edge `get().expect()` | Same edges, same order |
| `engine.rs:358` | Split `cells` / `last_step_charge` borrow; `advance_to` hoisted out of the event loop | See equivalence argument below |
| `engine.rs:412` | Same `advance_to` hoist in the parallel path | Same |
| `engine.rs:411` | `sort_by_key` → `sort_unstable_by_key` | Keys are unique, so stability is unobservable |
| `inhibitory.rs` | Cache static `e_to_i` row sums in `new()` | Same `.sum()` over the same slice → same f32 |
| `scan.rs:119` | Runtime `i % chunk_size` → running counter | Identical boundaries, identical `combine` order |
| `sparse.rs:96` | Doc warning on O(nnz) `ncols()` | Comment only |

Two of these deserve their reasoning written down.

**The `advance_to` hoist** is only valid because the repeated calls were exact
no-ops. `advance_to` writes `self.last = t` solely on a non-zero `dt`
(cell.rs:195) and early-returns when `dt == 0` (cell.rs:139); the only other
write to `last` is `Cell::reset` (cell.rs:234), which neither `deposit` nor
`try_fire` calls. So after the first call at a given `tick`, every later call at
that tick already did nothing. `grouped` never holds an empty event vector — an
entry is only created by pushing to it — so hoisting cannot introduce a call
that did not previously happen.

**The `three_factor.rs` change is the largest single win found.** It was
deep-copying the entire CSR **and** CSC — ~`3·nnz + nrows + 2·ncols` u32s over 5
heap allocations — on every plasticity step, solely to avoid a borrow conflict.
At nnz ≈ 2.5e6 that is ~30 MB of memcpy per step, on the order of a millisecond
of pure waste at M5 Pro bandwidth.

### These are unverified — read before trusting them

There was no Rust toolchain in the environment this audit ran in (`rustup`
unreachable, no root for `apt`), so **nothing here was compiled or tested.** Run
`cargo test --workspace` plus the determinism gates first.

Most of the risk is loud rather than silent. The borrow-splitting changes either
compile or they don't; the `advance_to` hoist is argued above from every write
site of `last`. The exception is the queue rewrite, whose failure mode *is*
silent — a stale occupancy bit makes `scan_earliest` skip a bucket and the
simulation quietly drops events. That is why `queue.rs` now carries three tests
aimed squarely at it:

- `occupancy_mask_starts_clear` — trivial base case.
- `occupancy_mask_survives_cascade` — far-future ticks straddling the 2^8 /
  2^16 / 2^24 / 2^32 level boundaries, forcing the `mem::take` cascade path,
  asserting the invariant and the masked-vs-naive scan after every pop.
- `occupancy_mask_and_scan_match_brute_force` — proptest over randomised
  insert/pop sequences that asserts, **after every single operation**, that the
  mask agrees with the buckets and that `scan_earliest()` equals the brute-force
  `scan_earliest_naive()` it replaced.

`scan_earliest_naive` is retained under `#[cfg(test)]` precisely so the old
implementation stays available as an oracle. The invariant is maintained at
exactly three sites, each labelled `Mutation site N of 3` in the source:
`schedule` (push_back), `pop_earliest` (pop_front), `cascade` (`mem::take`).
A grep for `self.levels` confirms there are no others.

The proptest also asserts `peek_earliest_tick() == scan_earliest_naive()`. That
one checks pre-existing cache logic, not the mask — if only that line fails,
look at `insert`/`pop_earliest`, not at `occupied`.

---

## 1. `TimingWheel::scan_earliest` — O(2048 + N), once per tick

`binn-engine/src/queue.rs:213-220`

```rust
fn scan_earliest(&self) -> Option<Tick> {
    self.levels.iter()
        .flat_map(|level| level.iter())
        .flat_map(|bucket| bucket.iter())
        .map(|entry| entry.at)
        .min()
}
```

Reached from `pop_earliest` (queue.rs:168-173) whenever the last event of the
current tick is drained — i.e. **once per tick**. Each call walks all
`LEVELS × SLOTS = 8 × 256 = 2048` `VecDeque` headers (32 B each → **64 KB of
pointer-chased header traffic per tick, regardless of occupancy**) and then
visits every one of the `N` pending entries to take a min.

This is the finding that matters most, because it directly contradicts the
premise stated at `engine.rs:5` — "work scales with events, not with the idle
cell population". A wheel holding one event costs the same 64 KB sweep as a full
one, every tick.

**APPLIED.** Per-level 256-bit occupancy masks (`[[u64; 4]; LEVELS]`), maintained
at the three mutation sites, so `scan_earliest` visits only non-empty buckets via
`trailing_zeros`. `next_occupied_level0` and `skip_empty_and_cascade` now consult
the mask too, skipping the `VecDeque` header deref for empty slots — that deref
was the entire cost of both loops in a sparse wheel.

Bit-identical: it computes the same minimum over the same entry set. See the
test coverage note above — this is the one change here whose failure mode is
silent, so it is also the one with a brute-force oracle wired into a proptest.

## 2. `last_step_charge.fill(0.0)` — O(n) per `step_until` *call*

`binn-engine/src/engine.rs:241` and `:324`

A `4n`-byte memset (400 KB at n = 1e5) that also evicts the working set.
`step_until` is not called once per run — `resting.rs:58` calls it **once per
tick**, and `runner.rs` has 11 call sites, several per trial. Same
event-driven-premise violation as #1.

**Fix:** a dirty-index list of cells touched this step, or a generation-tagged
`(gen: u32, charge: f32)` pair, making the clear O(D) instead of O(n).
Bit-identical.

## 3. Three scalar `expf` per cell-update, with provably constant arguments

`binn-engine/src/cell.rs:145`, `:168`, `:192`

```rust
let decay_d  = (-dtf / self.tau_d).exp();
let e_a      = (-alpha * dtf).exp();
let decay_th = (-dtf / TAU_THETA).exp();
```

Plus ~5 scalar f32 divides (lines 145, 163, 164, 172, 177, 184). arm64 has no
hardware `exp`; each is an opaque libm call that clobbers registers and blocks
vectorisation. Frequency: `D` per tick — this is the true per-cell-update cost.

The useful observation: **cell parameters are homogeneous in every production
run.** `Engine::with_cells` (engine.rs:79) builds every cell from
`Cell::default_params()`, and the only non-test write to `tau_m` / `tau_d` /
`g_c` anywhere in the workspace is
`binn-hybrid-lab/src/production_diagnostics.rs:846-848`, which copies them from
another cell. So `alpha`, `scale` and `k_g` (cell.rs:162-164, three divides) are
loop-invariant and can be computed once per parameter change.

Hoisting those three is bit-identical. Going further — a `dt`-keyed memo, or a
4-wide NEON polynomial `exp` evaluating all three together — is **not**, and
also needs a `dt` histogram first: under lazy integration `dt` is the gap since
a cell was last touched, and I have no evidence about its distribution. Measure
before building the memo.

## 4. Per-tick `BTreeMap` grouping with a `Vec` per cell

`binn-engine/src/engine.rs:336-345`

A fresh `BTreeMap<CellId, Vec<Decoded>>` every tick: heap-scattered internal
nodes, an O(log D) descent per event, and — worse — `or_default()` allocating a
**separate `Vec` per distinct cell**, so **D malloc/free pairs per tick**.

The sorted iteration order is not observable: `fired` is re-sorted by unique
`ordinal` at engine.rs:412. So this can become a reusable scratch `Vec<Decoded>`
bucketed by cell id (head-index array + intrusive next-link), zero allocation
after warmup. Bit-identical.

## 5. Fan-out strides 32-byte AoS for 12 bytes of payload

`binn-engine/src/engine.rs:501-511`, executed `S × F` per tick — the hottest
loop in the crate alongside `queue.insert`.

The loop reads only `weight` and `delay` but strides a 32-byte `Synapse` whose
other three fields are cold plasticity state: **2 synapses per cache line
instead of 5**. A full sweep of a 1e6-edge network streams 32 MB instead of 12.

The engine **already owns the SoA weight array** — `pub edge_w: Vec<f32>` at
engine.rs:42, kept in lockstep — and the hot loop reads the AoS copy anyway.
Reading `edge_w` plus a split-out `Vec<u32>` delay array would make this stream
two dense arrays. Also `self.syn.get(edge).expect(...)` is a bounds check plus
`Option` branch per edge despite the `assert_eq!` two lines above, and that
assert itself re-verifies a global structural invariant once per spike (it
belongs in `set_connectivity`).

Bit-identical. Note it duplicates the weight array in memory today, so this also
reclaims 4 bytes/edge.

## 6. `collect()` + stable sort of 128-byte jobs, for an unobservable ordering

`binn-engine/src/engine.rs:375-386`

Two allocations per tick: `D × 128 B` for the jobs, plus driftsort's temporary
of up to `D/2 × 128 B`. The sort key `(partition, cell_id)` is unique, so
stability buys nothing — `sort_unstable_by_key` is strictly better. And since
`par_iter_mut` jobs touch only their own state, write-back touches disjoint
cells, and `fired` is re-sorted afterwards, the sort exists **only** for rayon
partition locality. With `partition < n_partitions` (typically 4) it should be a
counting sort, O(D) in one pass, or dropped if locality isn't measurably helping.

**Correction to a hypothesis I raised earlier:** the `self.cells[...].clone()`
at engine.rs:380 is **not** a problem. `Cell` (cell.rs:53-71) owns no heap data
— `v_dend` and `branches` are `[f32; 4]` — so the derived `Clone` is a
non-allocating 64-byte memcpy, single-digit cycles on M5 Pro. The cost at that
line is the scattered random access into `cells`, not the copy. Likewise
`advance_to` being called per-event rather than per-cell (engine.rs:389) costs
~5-10 cycles per redundant call because of the `dt == 0` early return at
cell.rs:139 — worth hoisting as a one-liner, but not the win it looked like.

## 7. `assoc_scan`'s parallel path cannot beat its own sequential path

`binn-core/src/scan.rs:112-150`

Phase 1 is a **full sequential left-fold over all `n` elements** to record exact
chunk-boundary prefixes. Phase 2 then re-folds every chunk in parallel. Total
work is `n` sequential + `n` parallel, so the serial phase alone already costs
what `assoc_scan_sequential` costs end-to-end — and then the function does the
whole thing again across threads, plus rayon fork-join overhead.

This is a deliberate trade, and the comment says so: element-wise left-fold
"keeps f32 results identical to a pure sequential scan." The exactness goal was
met. But the consequence is that **the rayon in Phase 2 buys nothing and the
parallel path is strictly slower than the sequential one.** Either accept that
and call `assoc_scan_sequential` unconditionally, or adopt a genuine two-pass
scan (parallel chunk-total reduction, then parallel re-fold) and accept
re-association — which changes f32 results and needs a re-baseline.

Worth confirming against a bench before acting; I may be missing a caller whose
`combine` is expensive enough to change the arithmetic.

## 8. Everything else, briefly

**Bit-identical, lower value:**

- `queue.rs:74` — `Vec<Vec<VecDeque<Entry>>>` gives triple indirection on
  `insert`, the highest-frequency operation in the simulator (`S·F` per tick).
- `engine.rs:150`, `:447` — `self.queue = TimingWheel::new()` used as a clear,
  rebuilding 2048 `VecDeque`s and dropping 2048 per trial. An in-place `clear()`
  retaining capacity is free.
- `engine.rs:355`, `queue.rs:190` — `Vec::new()` / `vec![first]` per tick.
- `engine.rs:113` — `conn_rev` (CSC, 12 B/edge) is built for `binn-learn` only
  and never read by the step loop, but competes for cache during stepping.
- `binn-areas/src/wta.rs:18`, `multi_area.rs:107,142` — 3-4 area-sized
  allocations per step per area; `k_wta` copies all N pairs even when nothing is
  filtered.
- `wta.rs:193`, `:140`, `:173` — `winners.contains(id)` linear-scans a k-vector
  inside an O(N) loop. In `k_wta_with_margin` this is doubly wasteful:
  `items[take]` after `select_nth_unstable_by` **is** the boundary value already.
- `multi_area.rs:145` — dense O(N·K) `dendritic_coincidence_score` sweep over
  every destination cell bolted onto an otherwise correctly sparse step. This is
  what makes the step dense.
- `multi_area.rs:110` — linear scan over projections per step; O(M) on exactly
  the axis `multi_area_scaling.rs` varies.
- `resting.rs:92`, `:102`, `:175` — `Vec::contains` on vectors that are already
  sorted and deduped; `binary_search` or a bitset is a direct substitution.
- `inhibitory.rs:96` — a release-mode `assert!` scanning all `e_activities`
  every tick. Left alone deliberately: weakening a guard in a scientific harness
  is your call, not mine.
- `three_factor.rs:149-156` — `apply_weights` iterates **all** nnz synapses per
  call regardless of how few spiked, inside a system whose premise is sparsity.

**Numerics- or RNG-visible — these need a re-baseline, do not slip them in:**

- `wta.rs:82-93` — `soft_k_wta` rebuilds the full softmax `k` times: `k` full
  O(N) max scans, `k` allocations, and `k·N` scalar `exp()` calls. The
  mathematically-redundant recompute could be `N` exps and 1 allocation. But it
  changes f32 summation order in `total` and in the `draw -= *w` accumulation,
  which can flip a boundary pick. Plausibly the dominant term in any run using
  soft WTA — worth doing, worth doing *deliberately*.
- `wiring.rs:215-228` — `sample_unique` is O(fan²) rejection sampling
  (`~3.3e8` comparisons at N=1e4, fan=256), and each draw pays a runtime 64-bit
  modulo in `rng.rs:58-61`. A bitset membership mask or Floyd's algorithm fixes
  it, but changes the RNG consumption pattern and therefore every seeded result.
  Setup-only, so lower priority than its cost suggests.

## Hypotheses that did not survive

- **k-WTA is not doing a full sort.** `wta.rs:34-40` already uses
  `select_nth_unstable_by`; the subsequent `sort_unstable_by_key` runs on `k`
  survivors, not N. The win is already taken. Its `cmp` is a *total* order
  (score desc, then CellId asc), which is what makes the unstable select
  determinism-safe — do not remove that tie-break.
- **The CSR is clean.** No binary search, no sorted-insert, no per-edge
  indirection; `edges()` hoists the `row_ptr` loads per row. `Csc::from_csr` is
  setup-only.
- **No SipHash in the hot path.** Zero `HashMap`/`HashSet` in `binn-areas/src`
  or `binn-core/src`; the engine's grouping is a `BTreeMap`.
- **`Buffer<T>` costs nothing** — and is also essentially unused. The engine
  stores raw `Vec<f32>`; the SoA abstraction was never wired in.

---

## Suggested order of work

1. **Compile and test.** `cargo test --workspace`, then the determinism gates
   (`scripts/gc_checks.sh`), then a replay of one frozen hash — e.g.
   `c1-match-5dc6822e71229e9e` — to confirm the applied changes really are
   bit-identical in practice and not just in argument.
2. **Then profile.** `cargo bench -p binn-core` plus a sampling profile of one
   representative `c1` run. Everything below is a hypothesis until then, and the
   ranking in this document is the least trustworthy part of it.
3. Remaining bit-identical structural work, roughly in value order: §2
   (dirty-index charge clear), §4 (BTreeMap → bucketed scratch), §5 (SoA
   fan-out), §6 (counting sort), then the §8 list.
4. Treat §3's `exp` memo, §7's re-association, `soft_k_wta`, and `sample_unique`
   as a separate, deliberately re-baselined batch. Freeze new hashes and note
   the change in the protocol record. Do not let these ride along with a
   bit-identical batch — the whole value of separating them is that a hash
   change becomes evidence of a mistake rather than expected noise.

## Changelog

- **2026-08-02, pass 1** — audit written; `three_factor.rs`, `inhibitory.rs`,
  `scan.rs`, `sparse.rs` doc.
- **2026-08-02, pass 2** — §1 queue occupancy masks implemented with proptest
  oracle; `engine.rs` fan-out slicing, borrow split, `advance_to` hoist (both
  paths), `sort_unstable_by_key`.
