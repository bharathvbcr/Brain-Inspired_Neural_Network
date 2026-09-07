# Preregistration — wave 29: the asymmetry gets the bar it should have had

**Date:** 2026-09-07
**Status:** registered before any wave-29 cell exists. The git history is the attestation, not this line.
**Analyser:** `scripts/aws/analyse_wave29.py`, frozen with this document and committed in the same change.
**Supersedes nothing.** [`PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md`](PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md) stands, its verdicts stand, and H28-2 stays **NOT MET**.

---

## 1. Why this wave exists

Wave 28 registered, for H28-2:

> at every sensitive rate carrying an attention arm, **|DiD(`spike-dropout-pN`)| < 0.03**

and the analyser reported **NOT MET**: at `p90`, `|DiD| > 0.03` in 9 of 12 seed
quadruples.

**The bar was two-sided and the question was not.** With

```
DiD = (attn_intact - attn_X) - (rate_intact - rate_X)
```

a **positive** DiD means the read-out lost *more* than the substrate — the
reading H28-2 was written to detect. The measured values are **negative**:
**−0.0132** at p70 and **−0.0382** at p90. The read-out lost **0.0698** where
the substrate lost **0.1080**. Its advantage did not fall under count
destruction; it **grew**.

An absolute-value band cannot tell those apart, so it fired on the outcome
nobody had named. The registered NOT-MET reading — "destroying counts DOES cost
the read-out its advantage" — describes the opposite of the sign it was printed
against. The analyser is not at fault: it printed the text this project
registered for that branch.

**Per rule §9.3, that bar is not re-read here.** A bar that turns out to be
wrong is reported as a wrong bar together with the result it produced, and a
replacement governs the *next* wave, on cells that do not yet exist. Wave 28's
cells are not re-analysed by this document, and no clause below may be evaluated
against them.

---

## 2. What is disclosed

Everything below is informed by wave 28. The direction of H29-2 is **the
direction wave 28 measured**, and registering it as a prediction anyway is the
same move wave 28 made with p30 and disclosed in its §4.

This is legitimate only because the clauses are evaluated on **new seeds**. It
would be HARKing on wave 28's, and the analyser refuses them by construction
(§5).

---

## 3. Design

**Anchor, unchanged:** h128, feed-forward, `published-2ms`, `adjacent-sum-5`,
e400, `d32/L4` on the attention arm. Same pinned binary.

**Rate:** `p90` only. It is the rung where wave 28's band fired and where the
effect is largest. `p70` is measured as a secondary direction if its cells
land, and is not required by any primary clause.

**Seeds:** a fresh block, `5290001`–`5290012`, **disjoint from wave 28's**
`5170001`–`5170012`.

**Cells:** 48 — `{intact, spike-dropout-p90} × {ff+fixed, ff+fixed+attn} × 12
seeds`. The intact arms are re-run rather than reused from `w26pos`, because a
new seed block has no intact twin and pairing across seed blocks is not pairing.

**Cost, stated because it is the reason this wave is registered before it is
run:** the campaign's median cell is **32,944 s** of wall time — about 9.2
hours. 48 cells is roughly **440 core-hours**, which is a cloud campaign and not
a local run.

---

## 4. Clauses

### H29-1 — primary, one-sided: does destroying counts cost the read-out?

This is wave 28's question, asked with a bar that can only be failed by the
answer it was written to detect.

**Registered prediction:** at `p90`, **DiD < +0.03** in **at least 9 of 12**
seed quadruples.

**Refuted only by DiD > +0.03.** A negative DiD — the read-out losing less than
the substrate — **satisfies** this clause. It is not a failure, and it is not
silently a success either: it is the subject of H29-2.

### H29-2 — primary, one-sided: is the growth real?

**Registered prediction:** at `p90`, **DiD < −0.03** in **at least 9 of 12**
seed quadruples — the read-out's advantage *grows* under count destruction, by
more than the campaign's own bar.

**Disclosed:** wave 28 measured −0.0382 at p90 on its own seed block. See §2.

**H29-2 is conditional on the instrument being sensitive, in that order, and the
analyser enforces it.** The rate arm must lose more than **0.03** from intact in
at least 9 of 12 — wave 28's H28-1 bar, restated. If it does not, H29-2 reads
**NOT EVALUABLE**, never NOT MET. A null measured with an instrument that cannot
see is the ambiguity this ladder exists to remove.

### H29-3 — secondary, descriptive: is it monotone in rate?

If `p70` cells land, report `DiD(p90) < DiD(p70)` with its per-seed count. **No
bar, no verdict.** A direction over two rungs is not a curve, and this clause
may not be cited as one.

---

## 5. What the analyser refuses

Written down because each is a way this wave could produce a number that means
something other than what it says.

1. **Wave 28's seeds.** `5170001`–`5170012` are rejected by seed value, not by
   wave tag, so a re-tagged wave-28 cell cannot enter this analysis.
2. **Cross-block pairing.** Every clause is over seeds present in all four arms
   of its own block. A quadruple missing any arm is dropped, not imputed.
3. **Index collisions.** Two cells on one (key, seed) is fatal, as in wave 28 —
   a collision resolved by sort order is a wrong number with no symptom.
4. **Off-anchor cells.** Anything not at h128 is dropped and counted aloud.
5. **Voided cells.** `cell_validity.validity_problems` decides, and the count is
   printed. This wave inherits the `n_train` pin added on 2026-09-07.

---

## 6. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **H29-1 MET, H29-2 MET** | Destroying counts does not cost the read-out its advantage — it **increases** it, by more than the bar, on a second seed block. The paper's mechanism claim stands as an order claim, and the asymmetry becomes a result with a bar behind it rather than a NOT MET clause with a footnote. |
| **H29-1 MET, H29-2 NOT MET** | The read-out is not hurt by count destruction, but the growth wave 28 saw is not reproduced at this size on new seeds. The honest reading is that the advantage is **insensitive** to counts, and wave 28's −0.0382 was a seed-block effect. This retires the asymmetry rather than establishing it. |
| **H29-1 NOT MET** | DiD is above +0.03: the read-out loses *more* than the substrate. That is the reading H28-2's text asserted, now with a bar that actually tests it, and it would make the paper's mechanism claim broader than order. Wave 28's sign would then be the seed-block artefact. |
| **Instrument not sensitive at p90** | **NOT EVALUABLE.** p90 cost the rate arm 0.1080 in wave 28; if that does not reproduce, the ladder itself is in question and no reading is taken about the read-out. |
| **Cells void below the pairing floor** | Reported as INCOMPLETE with the void counts. **Not** re-run under a loosened validity gate. |

---

## 7. What this wave cannot settle

- **One rate, one anchor.** p90 at h128 / feed-forward / `published-2ms` /
  `d32/L4` / e400. Nothing here transfers to h1024 or to a recurrent substrate,
  and the decomposition of §3.5 already carries that limit.
- **Not a mechanism.** If the advantage grows under count destruction, this wave
  measures *that it does*, not why. No account is offered and none may be read
  in.
- **Not a curve.** Two rungs at most, and H29-3 is descriptive.
- **The paper's H28-2 verdict does not change.** It stays NOT MET, reported as
  NOT MET, with the wrong bar named. This wave adds a clause; it does not
  rewrite one.
