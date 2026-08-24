# Open work — 2026-08-07

Supersedes the "Still open" register in `SUMMARY_2026-08-03.md` §6, which
predates the geometry closure, the H1-at-convergence extension, the surrogate
amendment and the scoring-path audit.

Ordering is by blocking relationship, not by interest. Items in §1 can
invalidate items in §3, so they come first.

Status key: `[ ]` not started · `[~]` in flight · `[x]` done · `[!]` blocked on a
human decision.

---

## 1. Record repair — blocking everything downstream

Source of truth: `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md`.

The pattern this section exists to close: **a fix that is not re-run is not a
fix.** Three binaries were hardened on 2026-07-25 and none of their reports were
regenerated. The corrections live in Rust doc comments; the uncorrected numbers
live in the paper.

- [x] **Re-run `track-b-rescue` at v131.** *(2026-08-19.)* Done — the v130 PASS
      is **withdrawn**; warning fired on 3/20 seeds, arm reports `INVALID_HARNESS`.
      See `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` and
      `track_b_results_v131.md`.
- [x] **Correct or withdraw the six citations of the v130 PASS.** Done — all
      six documents (`PAPER_DRAFT.md`, `PAPER_RESULTS_TABLE.md`,
      `PAPER_SKELETON.md`, `PUBLISHABLE_CLAIMS.md`,
      `CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`, `REPRO_ARTIFACT_CHECKLIST.md`)
      updated to reflect withdrawal.
- [x] **Re-run `deep-snn-scaling` at v134.** *(2026-08-20.)* Done, and the
      answer is worse than the defect it was meant to fix. Every depth-matched
      ceiling is at chance — 0.4880 / 0.5000 / 0.5000 / 0.5000 on a two-class
      task — including depth 1, on splits the treatment solves at 1.0000 in the
      same process. Modulator RMS is healthy and near-identical across depths
      (5.03e-1 to 5.04e-1), so the hypothesised mechanism is **ruled out**. The
      suite is `INVALID_HARNESS` on measured evidence.
      See `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`.
- [x] **Restate or withdraw the depth-collapse result.** *(2026-08-20 —
      **withdrawn**, on the v134 evidence above, not on the `N_IN = 2` argument.
      The argument would still have permitted a depth-1 comparison; the
      measurement does not.)* Original entry: Independently of the
      defect, `deep_snn_scaling.rs:22-26` records that `CoincidenceTask` has
      `N_IN = 2`, so a 256⁴ stack on two-dimensional near-noiseless input has no
      depth structure to exploit. The 1.0000-to-0.4525 collapse is weak evidence
      either way and must not be cited as local learning failing with depth.
- [!] **SUPERSEDED 2026-08-20 — do not port the guard.** `shd_scientific_sweep`
      **never loads SHD.** It fabricates 5 classes over 24 channels and 16
      timesteps, with each label firing only in its own three reserved channels —
      linearly separable from spike counts, with no temporal structure at all.
      Porting `ceiling_inverted` would put a correct label on a comparison that
      should not be made. The binary's self-description has been corrected
      (`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`); **retire-vs-rename is a
      maintainer call** and the report cannot be regenerated regardless, because
      the binary is refused by `authorize_campaign(LocalLearning)` while the
      instrument is `Uncalibrated`.
- [x] **Re-run `ei-inhibition-sweep` at v135.** Done — v135 report on disk
      (`results/ei_inhibition_results_v135.md`).

## 2. Ceiling health of the two arms still standing

This is the item most likely to move a headline result, and it is not covered by
§1 because both arms went through the clamped `runner.rs` path.

- [x] **Check whether the matched gradient references are undertrained.** *(2026-08-19.)*
      Done via `a6_ceiling_health` (`RESULT_2026-08-19_A6_CEILING_HEALTH.md`).
      The gradient reference climbs to 1.0000 by e640; at e80 (0.9013) it is still
      climbing. Therefore, `gap_closed` > 1 at e80 is an artefact of undertraining
      the reference, and the comparison reflects learning speed on a saturating task.
      Guards unified in `binn_lab::guards::assert_reference_learning`
      (`HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md`).

## 3. The transfer gap — the science worth doing next

A local rule passes on the matched dense substrate (0.9387 / LCB 0.6894 and
0.9200 / LCB 0.6846) and fails on the live event-driven engine across twelve
variants, best gap LCB 0.3127 against a 0.5 threshold, canonical protocol 0.4912
with LCB −0.0048. Nobody has isolated why.

- [ ] **Write the decomposition preregistration.** Four named suspects, never
      tested individually: sticky `last_spike`, partial membrane reset, θ=∞
      muting, hard k-WTA instead of soft competition. One factor per arm, plus
      the registered stopping rule and the named-outcomes table before any cell
      runs. *(§2 unblocked 2026-08-19; design drafted in
      `DESIGN_TRANSFER_GAP_DECOMPOSITION.md`.)*

## 4. SHD instrument — remaining scope and caveats

- [x] Budget axis closed (final doubling e400→e800 buys +0.000294)
- [x] Width axis closed (h512→h1024 buys +0.000883)
- [x] Geometry axis closed (`channels-700` 0.0283 *worse* at e400; the registered
      prediction that the gap would narrow was **refuted**)
- [x] H1 at converged budget (24 cells, 6 seeds, NOT SUPPORTED at both budgets;
      outcome 4 of 4 as named in advance)
- [ ] **Contract axis at convergence.** Six timing contracts are closed only at
      e100. This is the last scope qualifier on the 0.7378 ceiling. Resolution
      invariance held at the short budget, but the width and geometry axes both
      taught that short-budget behaviour does not transfer.
- [ ] **Synchrony at matched activity.** The 0.1336 increment is an upper bound:
      `channel-shuffled` moves `saturated_fraction` 0.0000 → ~0.032 and mean
      firing rate 0.21 → 0.28. Needs a **registered input-scale normalisation and
      a re-measurement**, not a re-analysis. H1 is unaffected (`bin-shuffled`
      saturates at exactly 0.0000 in every seed).
- [ ] **Sharpening direction at a third budget.** Order effect shrinks
      0.0189 → 0.0127 while synchrony grows 0.1248 → 0.1336 and the ratio goes
      6.6× → 10.5×. Descriptive, **not registered**, and must not be leaned on
      until checked at a third budget.
- [ ] **H2 relative question for the recurrent arm.** Ask whether recurrence
      degrades *less* under shuffling than feed-forward does. **The blocking
      dependency is now met:** this item required "a matched `ff+fixed` baseline
      at the same surrogate scale", and wave 14 produced exactly that — 12/12
      `ff+fixed` and 12/12 `ff+fixed+attn` at scale 0.4, with `ff+fixed` scoring
      0.7088 against 0.7062 archived at the default, so the scale is not
      distorting it (`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`).
      What remains is the shuffled arm itself. No absolute ceiling claim from
      this configuration.
- [ ] **Recurrent numerical marginality.** Still open, but the evidence here is
      superseded: this item cites 3 seeds at h512, and wave 13 measured it across
      **48 cells** at the anchor width and budget
      (`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`). What that adds is a
      completion rate rather than an anecdote — `rec+alif` 11/12 at scale 0.4 and
      8/12 at 1.0 — and a second failure mode: `rec+fixed` does not diverge at
      0.4 at all, it **saturates**, ten cells voided with up to 52% of hidden
      units pinned at maximum firing. Adaptation is what prevents that, so on
      this substrate adaptation is stabilising. Per the registered stopping rule,
      **no smaller surrogate scale will be tried**. If it matters enough to fix,
      the next interventions in evidence order are per-sample gradient clipping
      at a threshold from the recurrent arm's own distribution, then h64, then
      truncated BPTT.

## 5. Provenance and verification

- [x] **Commit the record.** *(2026-08-23.)* Done, and the premise this item
      rested on is no longer true. The record was first committed in `a3dafd1`
      and every preregistration, amendment, result and analyser since has been
      committed before the data it governs existed — waves 12, 13 and 14 each
      registered and had their analyser frozen in commits that precede their
      first cell. The ordering that carries the epistemic weight is attested by
      git history rather than by prose and mtimes.
- [~] **Gate F to 13/13 on the current binary.** Currently 7 cells, 0 failures.
- [!] **`matrix_authorized`.** False since 2026-08-03, with
      `historical_reference` and `clean_reference` also false. **Not closeable by
      code** — `AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` withdrew
      itself. Either accept on content evidence (human decision) or re-run the
      six reference cells.
- [!] **Provenance flag.** Default-off, awaiting a human decision.

## 6. Audit debt

- [ ] **~8,000 lines of BINN proper are unswept.** `binn-engine`, `binn-areas`,
      `binn-core` and ~20 experiment binaries have never had the treatment the
      SHD instrument got, which found ten defects, five of them the class *code
      reporting success while measuring nothing*. Clippy found none of them; they
      are semantic. Whatever Gate 2 eventually says is worth nothing until this
      is done.
- [x] July campaign scoring path (`AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md`)
- [x] Rust instrument defect register (`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`)

## 7. Performance

- [x] Recurrent training cell 193.0 s → 30.6 s (6.3×), out of the
      forward/backward correctness fix
- [ ] **Profile before acting on `PERF_AUDIT_2026-08-02.md`.** The audit ran with
      no Rust toolchain; nothing in it was compiled, benchmarked or profiled, and
      its own opening says the ranking is a hypothesis and the first action
      should be to profile rather than to start at item #1.
- [ ] **Largest named candidate:** the plasticity step deep-copying its entire
      CSR *and* CSC on every update, ~30 MB of memcpy per step at nnz ≈ 2.5e6,
      purely to dodge a borrow conflict.
- [ ] **`ff+fixed` 4.5% regression.** Documented rather than hidden; removing it
      needs monomorphisation, a structural change with its own verification
      burden.
- [x] GPU throughput claims withdrawn. The backend selector was a bool the kernel
      never read; both columns ran byte-identical CPU code. No GPU code has ever
      executed in this repository.

## 8. Deferred by instruction

- [!] **Python arm.** Unswept by instruction; `matrix_verdict` reports `FAIL`
      with the cross-backend criterion explicitly unmet. Rerunning all 216 under
      the amended instrument is ≈4.4 days and cannot change any conclusion.
- [ ] **Gate E / G7.** No cross-backend recurrent fixture; agreement is argued,
      not measured.

---

## The one-line version

Repair the record (§1), find out whether the two surviving ceilings are real
(§2), and only then design the transfer-gap experiment (§3). Everything in §4
is real science that is already safe to continue.
