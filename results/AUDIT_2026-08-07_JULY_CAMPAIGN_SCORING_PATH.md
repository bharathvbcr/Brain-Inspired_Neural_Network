# Audit — the July campaign's scoring path, starting from the arms that score 1.0000

> ## OUTCOME: the defect class was **already found and fixed in code, twice**, and
> ## **never propagated to the shipped reports or the paper documents**. One
> ## instance remains unfixed in its own file while being cited by name as the
> ## example in another. Six documents, including the paper abstract, still cite a
> ## PASS that the current harness explicitly forbids.

**Run:** 2026-08-07.
**Trigger:** three arms report accuracy exactly `1.0000` with `SE 0.0000`, and in
each case equal or beat the exact-gradient ceiling they are measured against.
**Method:** static reading of the experiment binaries and `binn-lab/src/guards.rs`,
plus a source-versus-report protocol-version comparison. Nothing was compiled or
re-run.

```
claim_axis: harness-integrity
object_under_test: the scoring and verdict path of the July matched-substrate
  campaign, not the learning rules themselves.
may_claim: That specific shipped reports are stale with respect to the source
  that produced their protocol version, and that named guards are absent from
  named binaries.
must_not_claim: That any learning rule is or is not effective. That the arms
  are wrong — they are unverified, which is a different statement. Nothing was
  re-run, so no number here supersedes a number there.
```

---

## 1. Summary of the version skew

| experiment | source | shipped report | skew |
|---|---:|---:|---|
| `track-b-rescue` | **v131** | v130 | **1 behind, and the bump *is* the fix** |
| `deep-snn-scaling` | **v134** | v132 | 2 behind |
| `ei-inhibition-sweep` | **v135** | v133 | 2 behind |
| `shd-scientific-sweep` | v135 | **v135** | **current — this one is genuinely unfixed** |

The first three reports were produced by code that no longer exists. The fourth
was produced by the code that is there now, which has no ceiling guard at all.

## 2. `track-b-rescue` — the defect is documented in the fix that nobody applied

`binn-lab/experiments/track_b_rescue.rs:39-49` carries this doc comment, on the
function that replaced the defective one:

> **The bug this replaces.** Both rescue harnesses computed
> `(acc − 0.5) / (grad − 0.5).max(1e-4)` with no clamp and no separation gate,
> which is how gap-closed values of **1.0155** and **1.0244** — i.e. "closed 102%
> of the gap to the ceiling" — reached shipped reports. `runner.rs` has always
> clamped to `[0, 1]`; these two did not. A value above 1 means the arm beat the
> reference it is meant to be bounded by, which is a harness warning (saturated
> task / undertrained ceiling), not a result.

`results/track_b_results.md` is protocol **v130** and reports, for E1.3 Online
Learned FB: accuracy **1.0000**, **Gap Closed Mean 1.0155**, gap LCB **0.9988**,
verdict **PASS (matched)**.

`1.0155` is the exact value named in the bug comment. The shipped report is the
artefact the fix was written against.

Under the current code that PASS cannot be emitted. `track_b_rescue.rs:311-319`
constructs a `HARNESS WARNING — ceiling inverted` note whenever any seed exceeds
the ceiling, and states: *"no PASS is permitted while this warning is present."*

**The fix was made on 2026-07-25 (`HARDENING_v12`, protocol bump 130→131) and the
experiment was never re-run.** The stale PASS is still cited in:

- `PAPER_DRAFT.md:9` — **the abstract**, as "primary **1.0000**, gap LCB **0.9988** PASS matched"
- `PAPER_RESULTS_TABLE.md:17` — Table A, bolded
- `PAPER_SKELETON.md:95`
- `PUBLISHABLE_CLAIMS.md:55`
- `CAMPAIGN_2026-07-23_CLAIM_FREEZE.md:11`
- `REPRO_ARTIFACT_CHECKLIST.md:196`

### 2.1 What this does *not* touch

The doc comment is specific: *"`runner.rs` has always clamped to `[0, 1]`; these
two did not."* "These two" are the two rescue harnesses, `track_b_rescue` and
`live_transfer_rescue`. The DFA and RL matched PASSes ran through `runner.rs`:

- `c1-dfa-c8c4fe0899908b84`: 0.9387, gap LCB 0.6894
- `c1-rl-42eddc9c801308e9`: 0.9200, gap LCB 0.6846

Both went through the clamped path, so whatever their ceiling health, their
reported bounds are bounded rather than unbounded. **The matched-versus-live
transfer gap does not depend on the compromised arm.** Both do, however, exceed
their own gradient reference, which is the same smell in milder form and is
taken up in §5.

## 3. `deep-snn-scaling` — fixed in source, and the fix names two further problems

`binn-lab/experiments/deep_snn_scaling.rs:7-26` records three fixes made on
2026-07-25, none of which are reflected in the v132 report on disk:

1. **The Verdict column was the literal string `PASS`** for all four depth arms
   regardless of measurement. The shipped report contained rows reading
   `FAIL | PASS`, and a summary derived from it claimed "PASS across all depths"
   while 2L, 3L and 4L scored 0.4525, 0.5130 and 0.4500 against a 0.65 floor.
2. **The ceiling was 1-hidden-layer for every depth arm**, so a depth-related
   collapse "could not be attributed to feedback alignment rather than to the
   optimiser or to the task lacking depth structure."
3. Modulator scale is now recorded per depth.

And a limitation stated in the source, which materially weakens the depth result
independently of any defect:

> `CoincidenceTask` has `N_IN = 2`. A 256⁴ stack on a 2-dimensional,
> near-noiseless input has no depth structure to exploit, so a depth result on
> this task is weak evidence either way.

**Consequence.** The 1.0000-at-one-layer versus 0.4525-at-two-layers collapse
should not be presented as evidence that local learning fails with depth. Its own
source file says the task cannot support that inference, and the ceiling it was
measured against was the wrong depth.

## 4. `shd-scientific-sweep` — diagnosed in another file, unfixed in its own

This is the one that is genuinely open.

`results/shd_scientific_results.md` (protocol **v135**, matching source) reports
Graded DFA at **1.0000 ± 0.0000** and True E-prop Ceiling at **0.2140 ± 0.0294**
against a chance level of **0.2000**. The reference arm is at chance while the
treatment is perfect.

`deep_snn_scaling.rs:18-20` names this suite explicitly as the worked example of
the pathology it was fixing:

> Modulator scale is now recorded per depth. A ceiling whose credit signal is
> orders of magnitude weaker than the treatment's is not a ceiling — **that is
> exactly how the SHD suite produced a "ceiling" below its own treatment.**

`HARDENING_v12` added `dfa_modulator_rms`, `eprop_modulator_rms`,
`modulator_rms_ratio` and `ceiling_inverted` to `ShdCalReport`, which is the SHD
**calibration** path (`c1-shd-cal-*`, 20-way). `shd_scientific_sweep.rs` is a
different binary, the 5-class proto-135 sweep, and it received none of it.

Its entire verdict logic is a per-arm comparison against `chance + 0.05`
(`shd_scientific_sweep.rs:202-214`). There is no ceiling-inversion check, no
gap-closed computation, no modulator-scale recording, and no cross-arm
consistency check of any kind. An arm at 1.0000 and a reference at 0.2140 both
report "beats chance ✓" and the report is emitted without comment.

**The mechanism is already hypothesised in the codebase**: the e-prop ceiling's
credit signal is orders of magnitude weaker than the DFA treatment's, so the
"ceiling" is not a ceiling, it is an undertrained arm. That hypothesis is
untested for this binary because the instrumentation that would test it was
added to a different one.

## 5. What survives

The transfer gap, which is the result worth chasing, does **not** rest on any
arm audited here.

Matched substrate, through the clamped `runner.rs` path: DFA graded with fixed
feedback at 0.9387 (gap LCB 0.6894) and directional REINFORCE with per-neuron
feedback at 0.9200 (gap LCB 0.6846), both PASS.

Live engine: twelve variants, all FAIL, best gap LCB 0.3127 against a 0.5
threshold, canonical protocol at 0.4912 with a bound of −0.0048.

Removing the `track-b-rescue` arm entirely leaves that contrast intact. The gap
is between two PASSes and twelve FAILs, not between one PASS and twelve FAILs.

**One check is still owed** before even that is safe. Both surviving matched
PASSes exceed their own gradient reference: DFA 0.9387 against 0.8963, RL 0.9200
against 0.8887. On the DFA schedule the broadcast-graded contrast reaches 0.9863
against the same 0.8963, so on that schedule **two arms beat the ceiling and one
of them is a control**.

These are milder inversions than 1.0000-versus-0.9930, and they went through the
clamping path, so the reported bounds are bounded rather than unbounded. But
"local rule beats exact gradient" is the same smell, and a gradient reference at
0.8963 on a binary task where the treatment reaches 0.9863 looks undertrained.
The ceiling-health question should be asked of `c1-dfa-*` and `c1-rl-*` before
the transfer-gap preregistration is written. **This audit did not answer it, and
it is now the single thing most likely to move the transfer-gap result.**

## 6. Actions, in order

1. **Re-run `track-b-rescue` at v131.** Three outcomes are possible and all are
   publishable: the warning fires and the PASS is withdrawn; the warning does not
   fire and the arm stands on a clamped bound; or the arm no longer reaches
   1.0000 at all. Until then the six citing documents carry a verdict the harness
   forbids.
2. **Re-run `deep-snn-scaling` at v134** with the depth-matched ceilings, and
   restate the depth result with the `N_IN = 2` limitation attached, or withdraw
   it.
3. **Port the ceiling-inversion guard into `shd_scientific_sweep`.** The fields
   already exist on `ShdCalReport`; this is instrumentation that has been written
   once and needs applying to a second call site. Then re-run, and report the
   modulator RMS ratio alongside the accuracies.
4. **Ask the ceiling-health question of `c1-dfa-*` and `c1-rl-*`** before writing
   the transfer-gap preregistration (§5).
5. **Re-run `ei-inhibition-sweep` at v135** for completeness; it is exploratory
   and nothing depends on it, but it is two versions stale.

## 7. The pattern, stated plainly

Three separate binaries produced a treatment arm that met or beat its own
reference. In two of them the defect was found, diagnosed precisely, and fixed in
code. In none of them was the report regenerated. The corrections live in Rust
doc comments; the uncorrected numbers live in the paper.

This is the same class as the five defects in
`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md` — code reporting success while
measuring nothing — with one addition specific to a research repository: **a fix
that is not re-run is not a fix.** The version number is the tell, and it was
sitting in the report header the whole time.
