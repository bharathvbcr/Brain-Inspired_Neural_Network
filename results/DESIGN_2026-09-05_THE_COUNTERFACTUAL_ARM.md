# The counterfactual arm — instrument built, no wave registered

**Written:** 2026-09-05, **before any cell of any wave that uses it exists.**
This document registers what protocol 29 *is*. It registers **no bar, no
prediction and no verdict**: those belong to a preregistration written when a
wave is planned, and nothing here may be cited as a result.

**Status:** instrument only. Four preset hashes exist and the arm runs
end-to-end. No scientific run has been performed.

---

## 1. Why: the arm the twelve variants could not have been

Open problem 4 of
[`PAPER_BINN_INSTRUMENT_2026-08-31.md`](PAPER_BINN_INSTRUMENT_2026-08-31.md)
§11.1 asks *whether any local rule crosses the k-WTA transfer barrier, given
that structured `B` moves accuracy but not the gap*. Protocols v13–v24 answer
one half of that question twelve times over. Every one of them varies the
feedback vector (v13, v15, v19, v20, v24, v25), the capacity (v17), the exposure
(v14, v16), the eligibility *timing* (v18), or the forward competition itself
(v21 soft-WTA, v23 finite-θ). None varies what the credit signal has to
multiply.

That matters because of four consecutive facts in the live path, all of which
are code and not inference:

1. [`runner.rs:1873`](../binn-lab/src/runner.rs) — every hidden cell is set to
   `theta = f32::INFINITY` for the integrate window, so **no hidden cell spikes
   at all**.
2. `runner.rs:1935-1940` — k-WTA scores residual membrane `Cell::v`. That score
   exists for every cell, winner and loser.
3. **`runner.rs:1984` — `eng.cell_mut(cell).v = 0.0;` for every hidden cell.**
   The pre-competition activation is discarded one line before the winners fire.
4. `runner.rs:1989` — only `active_cells` are `force_spike`d, and eligibility is
   absorbed exclusively from the emitted spike log
   ([`three_factor.rs:120`](../binn-learn/src/three_factor.rs)).

So a cell that loses selection ends every trial with **identically zero**
afferent eligibility, and `Δw = η · e · B` is zero for it *whatever* `B` says.
Twelve arms reshaped `B`. On the losers, all twelve were multiplying by zero.

This is stated as a test rather than as prose:
`three_factor::counterfactual_tests::a_loser_cannot_be_taught_by_credit_alone`
gives a losing cell a credit of 1000 and asserts its afferent weights do not
move by a single bit, alongside a positive control on the winner that does move.

### 1.1 What this rules out, before it was built

Scaling *credit* by margin proximity — the obvious reading of "focus plasticity
near the decision boundary" — cannot work here, for the same reason. `binn-learn`
already contained `MarginScaledCredit` and `k_wta_straight_through`, both
unwired and both credit-side. Wiring either alone would have produced a clean
null that said nothing about the hypothesis. `MarginScaledCredit` is therefore
**deliberately not used by this arm**; §6 records what was done with it instead.

---

## 2. What protocol 29 changes, and what it does not

**The forward pass is v15's, bit-for-bit.** Same muted-θ integrate, same hard
k-WTA, same winners, same spikes, same single pass. Nothing is softened,
unmuted, annealed or resampled.

**The trace changes.** For the nearest-miss losers only, the afferent synapses
receive

    Δe_ij = λ_c · φ(v_i) · STDP(t_winner − t_pre)

where `φ(v) = exp(−(v − v_boundary)² / 2σ²)` is Gaussian proximity to the k-WTA
boundary. `v_boundary` is the highest score that did not win
(`binn_areas::boundary_below`), and `φ` is `binn_learn::margin_weight`.

Three restrictions are load-bearing, and each is a test:

| restriction | why | test |
|---|---|---|
| **afferent edges only** | An efferent write would credit cells downstream of a spike that never arrived — a change to the network, not to credit assignment. | `a_candidate_writes_neither_efferent_edges_nor_the_pairing_table` |
| **no `last_spike` entry** | A phantom pairing entry would re-time the STDP of every *real* spike that pairs against this cell afterwards, so the arm would stop sharing a forward pass with v15. | same |
| **deposits interleave chronologically** | `Eligibility::decay_to` rewinds a synapse's decay clock to whatever time it is handed. Appending a deposit at `winner_at` after a later action spike would re-decay that synapse from an earlier instant. | `a_candidate_does_not_disturb_a_later_real_spike` |

Nothing is emitted into the engine. `active_cells` remains the complete set of
hidden spikes.

---

## 3. The ladder, and why its control rung is not a separate arm

`cf_lambda` is a hashed `Config` field, so every rung of the λ_c ladder mints
its own hash and no rung's result can be cited for another's
(`every_lambda_rung_mints_its_own_hash`).

**λ_c = 0 is v15's update rule, bit-for-bit** — not "v15 plus a mechanism that
happens to do nothing". `Config::counterfactual_credit()` returns `None` at λ_c
= 0, so the control rung takes the *off* code path. This is deliberate: a
zero-scale deposit adds no eligibility but would still advance its afferents'
lazy-decay clock, splitting one decay into two and moving the last bits of every
afferent trace. The control would then sit a few ulps away from the only arm it
exists to be compared against.

Verified at two levels, not asserted:

* `three_factor::counterfactual_tests::no_candidates_is_bit_identical_to_the_mechanism_being_absent`
  — degenerate scales (`0.0`, `NaN`, negative) are refused at the queue, and
  every weight and trace bit matches the mechanism-absent run.
* `runner::tests::counterfactual_at_lambda_zero_is_bit_identical_to_v15` — a
  full local-assembly condition at λ_c = 0 matches `c1-sfb` on accuracy bits,
  activity-sparsity bits, the whole weight trace, and the plasticity-update
  count.

Confirmed end-to-end on the quick preset: λ_c = 0 gives `local=0.6125`,
`|local−dense|=0.2250`, identical to v15's `local=0.6125`, `|local−dense|=0.2250`
under its own distinct hash. **This is a smoke test on 5 pilot seeds and is not
a result.**

---

## 4. Disclosed constants

| constant | value | rationale |
|---|---:|---|
| `C1_CF_LAMBDA` | `1.0` | A loser *at* the boundary is credited with the full eligibility a real spike would have written. Above 1 would credit a cell that lost more than one that won, which this arm does not test. |
| `C1_CF_SIGMA_FRAC` | `0.5` | `σ = 0.5 · (v_top − v_boundary)`. Scale-free: membrane scores carry engine units that move with `init_w`, `readout_boost` and width, so a σ in volts would silently mean a different experiment at every rung of a capacity sweep. |
| `C1_CF_CANDIDATE_MULTIPLE` | `2` | Deposits capped at the top `2k` losers by score, under the same total order `k_wta` selects with. |

**Registered hashes** (`Config::known_presets`, so `--config-hash` resolves each):

| hash | preset |
|---|---|
| `c1-0c20d1afd24410d4` | scientific, λ_c = 1.0 |
| `c1-8bcadeadd9c410d4` | scientific, λ_c = 0 (control) |
| `c1-c319e70d6d36fc00` | quick, λ_c = 1.0 |
| `c1-435746896ab6fc00` | quick, λ_c = 0 |

The knobs are mixed into the hash **only under protocol 29**, following the
`is_mac_probe_geometry` precedent, so every archived hash is unchanged. Checked
rather than assumed: `counterfactual_knobs_do_not_disturb_any_other_protocol_hash`
sets all three fields on a v15 config and on canonical C1 and asserts the hashes
are identical, including the literal `c1-118207fbc3eaba53`.

`c1-sfb-cf` also starts with `c1-sfb`, so protocol 29 is excluded from
`is_structured_fb_protocol()`. Without that exclusion it would run under v15's
version number and print v15's disclosure text
(`counterfactual_is_its_own_protocol_not_v15`).

---

## 5. The coverage number, and why two are carried

The candidate budget is a **cap**, not a proof that the remaining losers are
irrelevant. So the runner counts, and the results note prints, both the losers
considered and the near-misses actually credited. On the quick preset: 120
training trials, **4664** cells lost a competition, **182** were credited — 1.52
per trial against a cap of 2, or **3.9%** of the losers.

`deposited == 0` under a non-zero `considered` prints an explicit paragraph
saying the run's numbers are v15's numbers reached by v15's rule, and are a fact
about the margin gate rather than a negative result for the arm.

The distinction between *not recorded* and *recorded zero* is carried in the
type (`Option<CounterfactualDiagnostics>`) and printed differently. This was a
live defect during construction: the parent runs each condition in a child
process and reads one JSON line back, and the first working version printed
"not recorded" on **every real run** — the message was true and useless. The
counts now cross that boundary
(`counterfactual_counts_survive_the_isolate_child_boundary`), and a protocol
that does not run the mechanism still comes back as `None`.

---

## 6. Bugs and gaps found while building this

1. **`k_wta_with_margin` claimed a caller it never had.** Its doc comment read
   *"Used by `MarginScaledCredit`"*. `rg -uu` across the tree finds zero call
   sites for either outside their own definitions, tests and re-exports. Comment
   corrected; the boundary definition is now owned by `boundary_below`, which
   takes the winner set instead of recomputing `k_wta` — so it is also correct
   for `soft_k_wta`, whose winners are not the top `k`.
2. **`MarginScaledCredit::update_margins` attenuated cells outside the
   competition.** It took a dense `&[f32]` over *all* cells and wrote a computed
   weight into every slot, so a read-out post — membrane typically `0.0`, not in
   the competition — was scaled by `φ(0 − v_boundary)`. At the test's own
   parameters that is `exp(−2)`: an 86% cut to read-out credit, silently.
   Replaced by `update_margins_for`, which takes only the competitors and leaves
   everything else at a neutral `1.0`
   (`margin_scaling_leaves_cells_outside_the_competition_alone`).
3. **`MarginScaledCredit` remains unwired, now deliberately.** §1.1 gives the
   reason. Its doc comment now states it, so the next person to reach for it
   learns why it cannot move a near-miss before running it, not after.
   `k_wta_straight_through` is likewise still unwired — it is a different
   mechanism (soft credit weights for *all* neurons) and was left alone rather
   than deleted.
4. **Trial-boundary resets now drop unabsorbed candidates.** A near-miss belongs
   to the selection event that produced it; surviving into the next trial would
   credit a cell for losing a competition already resolved. Evaluation trials
   queue nothing at all, for the same reason.

### Pre-existing, not touched

* `cargo fmt --all -- --check` **fails at HEAD** under rustfmt 1.9.0-stable
  (2026-08-18), in `shd_attention.rs`, `shd_matched_arms.rs`, `shd_temporal.rs`
  and `shd_instrument.rs` — none of which this change edits. Reformatting them
  was reverted rather than shipped as a drive-by; the eight files this change
  does touch are fmt-clean.
* `./scripts/record_checks.sh` **fails at HEAD**, on three findings in
  `temporal_order.rs`, `credit.rs::broadcast_modulators_preserve_legacy_scalar`
  and `temperature_ablation.rs`. Its output under this change is identical to
  its output at HEAD: no new findings, and none of the tests added here were
  flagged.

---

## 7. What a wave using this would still have to register

Nothing below is decided here.

* **Bars.** No accuracy floor, gap-LCB threshold or seed count is registered for
  protocol 29. Gate G2's existing numeric thresholds are untouched.
* **The ladder's rungs.** Which λ_c values, and how many seeds each.
* **Whether `2k` is the right budget.** At the scientific preset `k_wta = 2`, so
  the cap is 4 of 126 losers. `cf_candidate_multiple` is a hashed field
  precisely so this can be swept, and a wave that does not sweep it should say
  why the nearest 4 are the right 4.
* **Whether afferent-only is the right restriction.** It is the
  credit-assignment-relevant direction and it is what keeps the forward pass
  shared, but a wave could register the efferent variant as a separate arm with
  its own hash. It would no longer be a pure credit-side change, and should not
  be described as one.
* **What a positive result would and would not license.** Moving the gap on the
  coincidence task is not moving it on SHD, and this arm carries no temporal
  mechanism of any kind.

## 8. Reproduction

```bash
cargo test --locked --workspace
```

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --counterfactual --quick --out results/c1_sfb_cf_quick.md
```

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --cf-lambda 0 --quick --out results/c1_sfb_cf_control_quick.md
```
