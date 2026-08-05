# Preregistration — is the 0.7378 ceiling geometry-bound?

**Registered:** 2026-08-04, before any `channels-700` cell at a budget above e100.
**Instrument:** `shd_instrument_v4` matched BPTT, `ff+fixed` arm, rust backend.
**Extends:** `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` and
`RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md`, whose
`must_not_claim` names geometry as the binding scope limit.
**Claim axis:** architecture-ceiling.

```
claim_axis: architecture-ceiling
object_under_test: Whether the converged 0.7378 ceiling for ff+fixed is a
  property of the forward, or of the `adjacent-sum-5` input geometry it was
  measured under.
may_claim: Under this protocol, at published-2ms / channels-700, the ff+fixed
  forward reached the measured accuracy at the named budget and width, and
  cleared or failed the registered 0.80 gate.
must_not_claim: That any figure produced by Stage 1 alone is a CEILING — that
  requires Stage 2 (§3.2). That geometry generalises beyond the two tested.
  SOTA; Gate G2; biology; neuromorphic hardware; anything about BINN.
```

---

## 0. Authorization posture — stated up front, not buried

`train-cell` is `CampaignKind::Calibration`
(`binn-lab/experiments/shd_instrument.rs:46`), which
`authorize_campaign` permits while `SHD_INSTRUMENT_STATE` is `Uncalibrated`
(`binn-lab/src/instrument_status.rs:34`). **The binary gate authorizes this
campaign.** It is not an architecture ablation and not a local-learning
campaign: the arm, contract, width and seeds are unchanged, and only the input
geometry — an axis of the already-registered 216-cell matrix — moves.

**But `matrix_authorized` is false.** The 216 matrix cells ran while it was
true; the budget, width and temporal extensions of 2026-08-03/04 ran after it
went false, and this campaign would too. That is a real weakening and it is not
being papered over: any result from this design **inherits threat §7.8 of
`SHD_BPTT_CEILING_NEGATIVE_RESULT.md`** and must not be cited as resting on a
currently-`VALID` harness. The blocker is
`AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md`, which established that it
is **not closeable by code** and needs a human decision or a reference re-run.

If that decision lands before this campaign runs, this section is superseded and
the result is stronger. If it does not, the campaign is still worth running — the
scope limit it closes is scientific, and it does not depend on the reference
gates — but the write-up must carry the caveat.

## 1. Why this, and why it is the cheapest remaining test

The ceiling claim has had two qualifiers stripped from it in the last two days,
each by measurement:

| axis | status | evidence |
|---|---|---|
| budget | **CLOSED** | final doubling e400→e800 buys +0.000294 |
| width | **CLOSED** | final doubling h512→h1024 buys +0.000883 |
| **geometry** | **OPEN** | `channels-700` unrun above e100 |
| contract | partially closed | resolution invariance across 6 contracts, but at e100 only |

Geometry is now the only axis on which "0.7378 is what this forward reaches"
could still be wrong in the same way the width claim was wrong. It is also the
cheapest to close — three cells for a verdict.

**There is a specific reason to expect a surprise.** At e100, `channels-700` is
*worse* than `adjacent-sum-5`, and the gap grows with width:

| hidden | `adjacent-sum-5` | `channels-700` | gap | n per cell |
|---:|---:|---:|---:|---:|
| 128 | 0.667624 | 0.650054 | 0.017570 | 18 |
| 256 | 0.684236 | 0.665955 | 0.018281 | 18 |
| 512 | 0.704186 | 0.681463 | 0.022723 | 18 |

At the anchor specifically (`published-2ms / h512 / e100`, 3 seeds):
`adjacent-sum-5` 0.715106 ± 0.003926 versus `channels-700` 0.689193 ± 0.007099 —
a gap of **0.025913**.

That pattern is the same shape as the width curve that turned out to be a budget
artefact. `channels-700` presents **700 input channels where `adjacent-sum-5`
presents 140**, so it has 5× the input parameters to fit from the same 8156
training samples. A model with more parameters to fit reaches a given loss in
more epochs, so **at a fixed short budget it looks worse for a reason that has
nothing to do with the geometry's information content.**

This is the identical confound, running in the opposite direction, and the
project has already been caught by it once. That is the argument for measuring
rather than assuming.

## 2. Registered prediction, recorded before any cell runs

**The e100 geometry gap is substantially a budget artefact, and will narrow at
convergence.**

Mechanism as above: 5× input parameters, same data, same budget. If the gap is
information-bearing it should be stable or grow with training; if it is a fitting
artefact it should shrink.

This is registered as a **prediction with a mechanism**, not a hypothesis with a
decision attached, and it is recorded so that a narrowing gap cannot later be
presented as something anticipated after the fact — and so that a *stable* gap
counts as evidence against the mechanism.

## 3. Design

### 3.1 Stage 1 — the verdict (blocking, 3 cells)

`ff+fixed` × `published-2ms` × `channels-700` × `h512` × `e400` × seeds
5170001–3, full 8156/2264 split, rust backend.

`e400` is the budget demonstrated SUFFICIENT for `adjacent-sum-5`
(`AMENDMENT_2026-08-03_CONVERGENCE_RULE_FINAL_DOUBLING.md`). **It is not
demonstrated sufficient for `channels-700`, and Stage 1 does not assume it is** —
see §3.2 and the naming restriction in §6.

**Data order is paired with the existing `adjacent-sum-5` cells.** The `.orders`
files depend only on `(n_train, epochs, seed)`, so the *same*
`n8156-e400-s{seed}.orders` files already used by the e400 anchor are reused
rather than regenerated. The geometry contrast is therefore paired on epoch
shuffle order, and any difference cannot be a data-ordering difference.

### 3.2 Stage 2 — axis closure, required only for a ceiling claim (6 cells)

Stage 1 answers *"does `channels-700` clear 0.80?"*. It does **not** answer
*"what does `channels-700` converge to?"* — that needs both axes closed for this
geometry, exactly as they were for `adjacent-sum-5`.

| cells | purpose | bound |
|---|---|---|
| `h512 / e800 / 3 seeds` | budget axis | `mean(e800) − mean(e400) ≤ 0.01` |
| `h1024 / e400 / 3 seeds` | width axis | `mean(h1024) − mean(h512) ≤ 0.01` |

Same 0.01 constant as the registered rule; no threshold is introduced here.

**Stage 2 is mandatory before quoting any converged figure for `channels-700`,
and optional otherwise.** The escalation rule below fixes in advance when it is
worth the compute, so that skipping it is a registered decision rather than a
budget excuse:

> **Escalation rule.** If Stage 1's 95% CI upper bound is below **0.78**, Stage 2
> is **not required for the verdict** and may be skipped, because no single
> doubling on either axis has ever bought more than **+0.018551** in this
> instrument (h128→h256 at e400, the largest per-doubling gain on record), so
> two doublings cannot bridge a shortfall greater than 0.02. If the upper bound
> reaches 0.78 or above, Stage 2 is **blocking** and must run before any verdict
> is reported.

The 0.78 constant and the +0.018551 justification are fixed here, before Stage 1
runs.

### 3.3 What is deliberately not in this design

- **Other contracts.** `published-2ms` only. Resolution invariance at e100 makes
  contract the weaker of the two remaining axes, and mixing them would confound.
- **Other arms.** `rec+alif` is blocked on the h512 instability
  (`MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`) and is not
  touched here.
- **Temporal manipulations.** `--temporal intact` throughout. The
  order/synchrony question at `channels-700` is a separate registered design.

## 4. Hypotheses and thresholds

| ID | Statement | Threshold |
|---|---|---|
| **G1** *(primary)* | `channels-700` does not clear the registered gate | mean accuracy < 0.80 with the 95% CI (t, df=2) entirely below 0.80 |
| **G2** | geometry does not rescue the shortfall | `channels-700` ≤ `adjacent-sum-5` + 0.02 at the matched budget and width |
| **G3** | the e100 geometry gap is substantially a budget artefact | gap at e400 < **0.025913**, the measured e100 anchor gap |
| **G0** | geometry was load-bearing | `channels-700` exceeds `adjacent-sum-5` by ≥ 0.02 at e400 |

G1 and G2 are confirmatory. G3 is the registered prediction from §2 and is
**exploratory-confirmatory**: its threshold is fixed in advance, but it is a
directional test on a single pre-existing number and carries less weight than G1.
G0 is the complement of G2 and is stated so the surprising outcome has a name
before it can happen.

**G1 is a one-sided bound, not an equivalence test.** It is the same gate the
216 cells failed, applied unchanged, at a geometry those cells only reached at
e100.

## 5. Validity gates

1. **Registered per-cell gates (blocking).** All five non-accuracy gates —
   `classes_predicted == 20`, `majority_prediction < 0.30`,
   `silent_fraction <= 0.95`, `saturated_fraction <= 0.05`,
   `non_finite_events == 0`. A cell failing any of them is excluded and the
   exclusion reported; **more than one exclusion voids the stage.**
2. **Activity-regime disclosure (reporting, non-blocking).** Report
   `mean_firing_rate`, `silent_fraction` and `saturated_fraction` per cell
   alongside accuracy, and compare against the `adjacent-sum-5` cells at the same
   budget. *Rationale: §6.7 of `RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`
   records a case where a manipulation stayed inside the saturation gate while
   still shifting the operating regime enough to qualify the result. Passing the
   gate is not the same as being in the same regime, and a geometry that changes
   the input dimension 5× is exactly where that could recur.*
3. **Determinism.** Gate F must return 13/13 `BIT_IDENTICAL` on the binary used,
   before the stage runs. Gate F's fixture set already includes
   `rust__published-2ms__channels-700__h512__e20__s5170002`, so this geometry is covered by
   the existing suite at no extra cost.
4. **Provenance.** Cells written under `results/shd_instrument_v4/
   geometry-converged/`, with the binary sha256 recorded in the stage log.
5. **Convergence rule (Stage 2 only).** The amended final-doubling rule, 0.01
   bound, applied per axis. If either axis returns UNDERTRAINED the ladder
   extends by one doubling and the test repeats, to a hard stop at e1600 / h2048.
   Failing to converge within that is reported as *"not converged within the
   budget explored"* — **not** as a ceiling. No threshold is adjusted to force a
   verdict.

## 6. Decision rules

**G1 holds, G2 holds** — the expected outcome. The ceiling claim loses its last
scale-related qualifier at `published-2ms`. `SHD_BPTT_CEILING_NEGATIVE_RESULT.md`
may then state the ceiling for the contract rather than for one geometry, and the
remaining scope limit narrows to *contract*, which resolution invariance already
makes the weakest of the four.

**G1 holds, G0 holds** — `channels-700` is better by ≥ 0.02 but still short of
0.80. The gap decomposition in §5 of the negative result must be restated against
the better geometry, and the "cost of this architecture" row moves. The ceiling
claim survives but its *value* changes, and the write-up must say which geometry
each figure belongs to.

**G1 fails** — `channels-700` clears 0.80 at a sufficient budget. This is the
outcome that would overturn the negative result, and the obligations are heavier
than the others combined:
- the ceiling claim is **withdrawn**, not softened, and the word removed from the
  title as it was on 2026-08-03;
- `adjacent-sum-5` becomes a **confound in the 216-cell matrix**, not a neutral
  preprocessing choice, and every conclusion drawn from the matrix is re-scoped;
- the width and budget closures are re-opened for the new geometry — Stage 2
  becomes blocking regardless of the escalation rule;
- the gap decomposition is rebuilt from scratch.

**Naming restriction, binding on all outcomes.** If Stage 2 does not run, no
figure from this campaign may be described as a **ceiling**, a **converged**
value, or *"what `channels-700` reaches"*. The only permitted form is *"X at
e400/h512"*. This restriction exists because the project has already published a
budget-limited figure as a ceiling once, and the correction cost two withdrawals.

## 7. Cost

Estimated from measured wall time, and flagged as an estimate.

`channels-700 / h512 / e100` measured **3071 s** against `adjacent-sum-5 /
h512 / e100` at **1405 s** — a **2.19×** geometry factor. Post-transpose,
`adjacent-sum-5 / h512 / e800` measures **5850 s**, so e400 is ≈ 2925 s.

| stage | cells | est. per cell | est. total |
|---|---:|---:|---:|
| Stage 1 — h512 / e400 | 3 | ≈ 1.8 h | **≈ 5.3 h** |
| Stage 2a — h512 / e800 | 3 | ≈ 3.6 h | ≈ 10.7 h |
| Stage 2b — h1024 / e400 | 3 | ≈ 2.8 h | ≈ 8.5 h |
| | | | **≈ 24.5 h if all run** |

**The 2.19× factor was measured before the transposed kernel**
(`AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md`), which changes the input-matmul
cost structure, so the true factor may differ in either direction. Treat these as
order-of-magnitude figures; the stage log should record actuals.

## 8. Commands

Initialisations are generated fresh — `n_inputs` is 700 here, so the registered
`adjacent-sum-5` weights do not apply — while `.orders` files are **reused** from
the existing e400 runs, per §3.1.

```bash
BIN=target/release/shd-instrument
OUT=results/shd_instrument_v4/geometry-converged
mkdir -p "$OUT/init"

for S in 5170001 5170002 5170003; do
  "$BIN" init \
    --n-inputs 700 --hidden 512 --classes 20 --seed "$S" \
    --epochs 400 --n-train 8156 \
    --weights "$OUT/init/n700-h512-s$S.weights" \
    --orders  "$OUT/init/n8156-e400-s$S.orders"

  "$BIN" train-cell \
    --train-events data/shd/events/train.events \
    --test-events  data/shd/events/test.events \
    --contract published-2ms --geometry channels-700 \
    --arm ff+fixed \
    --weights "$OUT/init/n700-h512-s$S.weights" \
    --orders  "results/shd_instrument_v4/probe/orders/n8156-e400-s$S.orders" \
    --epochs 400 \
    --out "$OUT/ff-fixed__channels-700__h512__e400__s$S.json"
done
```

The `init` call still writes an `.orders` file; `train-cell` is pointed at the
**existing** one so the pairing in §3.1 holds. The freshly written orders file is
retained only as a determinism check — it must be byte-identical to the reused
one, since both are functions of `(n_train, epochs, seed)` alone. **If it is
not, the pairing assumption is false and this design is void until that is
explained.**

## 9. Amendment discipline

Per the `PREREG_2026-07-25_SHD_ARCH_ABLATION` §preamble rule, this document is
amended by a new file with a new timestamp, never edited in place. Errata against
its stated facts are recorded as banners here; hypotheses, thresholds, conditions
and decision rules are changed only in an amendment registered **before** the
affected cells run.

**Commit before running.** This file, and any amendment to it, is committed
before the first cell of the stage it governs. That ordering is the evidence that
the rule preceded the result, and as of 2026-08-04 it is version-controlled
rather than attested by file mtimes.
