# Audit — `binn-engine` and `binn-areas`, semantically

**Date:** 2026-09-07
**Scope:** `binn-engine/src/{cell,engine,resting,parallel,synapse,spikelog}.rs` and all of `binn-areas/src/`
**Closes:** the last row of [`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md) §3 that was still open at the class level
**Companion:** [`AMENDMENT_2026-09-07_THE_ENGINE_IS_ON_THE_G2_PATH.md`](AMENDMENT_2026-09-07_THE_ENGINE_IS_ON_THE_G2_PATH.md) swept `queue.rs` and `sparse.rs` the same day

**No defect found.** One dead-code candidate, reported and not acted on.

The register's method was generic defect-class greps. Those were run over these
crates on 2026-08-03 and came back clean. This pass asks the other question —
**would the test pass if the thing under test did nothing?** — and answers it by
breaking the code and watching, rather than by reading it and judging.

---

## 1. The file-level test counts are misleading, and that matters

Counting `#[test]` per file suggests five files in `binn-areas` have no coverage
at all:

| file | in-file tests |
|---|---:|
| `area.rs` | 0 |
| `wiring.rs` | 0 |
| `project.rs` | 0 |
| `assembly.rs` | 0 |
| `spikelog.rs` (engine) | 0 |

**All five are covered from outside.** `binn-areas/tests/determinism.rs` carries
**11 tests** over exactly this surface: the k-WTA winner cap and its measured
activity against `k/n`, `project` convergence including a disclosed seed sweep,
random-assembly overlap against `k²/n`, `associate` raising inter-assembly
overlap, and three wiring properties — determinism under a repeated seed, the
fan-out cap at many-area scale, and >90% intra-area events. It also carries a
**negative control**, `locality_falls_below_the_gate_at_a_denser_p_inter`, which
asserts the locality gate *can* fail. A suite whose gate cannot fail is the
defect class this repository is built around, and that test is the answer to it.

`spikelog.rs` is the return type of `Engine::step_until`, so all 46 engine tests
exercise it.

Recording this because a coverage report built on per-file counts would have
sent the next reader to write tests that already exist.

---

## 2. Three mutations, and what each proved

### k-WTA selects the right units, not just the right number

`binn-areas/src/wta.rs::k_wta` reversed so it takes the **lowest** scores:

```
test wta::tests::k_wta_selects_top_k ................. FAILED
test wta::tests::k_wta_at_most_k ..................... FAILED
test wta::tests::k_wta_with_margin_finds_boundary .... FAILED
test wta::tests::straight_through_k_wta_matches_hard_and_soft ... FAILED
```

Four failures. The invariant pinned is semantic — *which* units win — not merely
that `len() <= k`, which is the version of this test that would have passed.

### The cell dynamics are pinned analytically

`binn-engine/src/cell.rs::advance_to` with the dendritic leak deleted
(`decay_d = 1.0`):

```
test cell::tests::dendrite_analytic_when_uncoupled ....... FAILED
test cell::tests::compartment_coupling_drives_soma ....... FAILED
test cell::tests::impulse_does_not_create_permanent_branch_drive ... FAILED
test engine::tests::analytic_membrane_via_engine_touch ... FAILED
```

Four failures, and the shape is right: the tests compare against the closed-form
solution rather than against a recorded number, so they fail on the physics
rather than on a fingerprint.

### The G2 harness has an end-to-end net, and it is the positive controls

The same corrupted membrane, run against `binn-lab` — the crate that owns the
C1/Gate G2 harness:

```
test runner::tests::positive_control_floor_on_quick_seed ............ FAILED
test runner::tests::reinforce_fb_positive_control_uses_broadcast_and_clears_floor ... FAILED
test runner::tests::temporal_positive_control_floor_on_sensitivity_quick ... FAILED
263 passed; 3 failed
```

**All three failures are positive controls.** This is the most useful result in
the audit. The amendment written the same day establishes that Gate G2 executes
`binn-engine`; this establishes that a corrupted engine is **detectable at the
harness level**, not only in the engine's own unit tests — and that the thing
detecting it is the arm that is supposed to succeed. A suite of negatives cannot
tell a broken substrate from a real negative. The positive controls can, and
here they did.

### What does *not* catch it, stated so it is not mistaken for coverage

`binn-areas`' 28 tests **all pass** against the corrupted membrane. That is
correct scoping — they test selection, wiring and assembly geometry, not
dynamics — but it means the areas suite is not a safety net for the engine, and
nobody should read it as one.

All three mutations were reverted and the tree verified clean before this
document was written.

---

## 3. One dead-code candidate, not acted on

`binn-engine/src/spikelog.rs::extend_from` has **zero call sites** anywhere in
the workspace outside its own file, against 232 for `is_empty`, 53 for
`as_slice` and 10 for `clear`.

**Not removed.** The deletion discipline in this workspace requires two
independent signals, and there is one. `devmap dead --json` returned no rows at
all, which is absence of evidence rather than evidence of absence and does not
count as the second. It is also a documented two-line `pub` helper on a public
container type — a reasonable API for a `SpikeLog` to have whether or not this
workspace calls it today. Reported so the next reader has the fact, not deleted
on the strength of a grep.

---

## 4. What this establishes, and what it does not

**Does:** the load-bearing paths of `binn-engine` and `binn-areas` — cell
dynamics, k-WTA selection, wiring, assemblies, `project` — are pinned by tests
that fail when the behaviour changes, verified by changing it. Combined with
`queue.rs` and `sparse.rs` in the companion amendment, **the crates the register
listed as semantically unaudited are now audited**, and no defect was found in
any of them.

**Does not:** three mutations are three mutations. This is evidence that the
suites are live and that their invariants are the semantic ones, not a proof
that every path is covered. `resting.rs`, `parallel.rs`, `synapse.rs`,
`hub.rs`, `inhibitory.rs`, `multi_area.rs` and `predictive.rs` were read and
their tests inspected, but no mutation was run against each individually.

**The honest summary:** the register expected this to be where a BINN-proper
defect would live. It read as the riskiest surface because nothing had looked.
Having looked, it is better defended than the experiment binaries were — where
one defect *was* found the same day — and the reason is that these crates carry
analytic tests and a negative control, while the binaries carried neither.
