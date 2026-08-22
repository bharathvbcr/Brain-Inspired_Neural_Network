# Four registered thresholds were anchored to macOS reference values this campaign is not licensed to compare against

**Date:** 2026-08-20
**Affects:** W1-4, W5-2, W3-1, W3-3.
**Does not change any threshold or verdict.** Every one is reported as
registered. This records *why* several of them turned out to be less informative
than intended, so the next preregistration does not repeat it.

---

## 1. The pattern

| hypothesis | threshold anchored to | that value's origin | what the same-machine control actually did |
|---|---|---|---|
| **W1-4** | `tail_loss_improvement > −0.02` | the pilot's e20 window (macOS) | control reads −0.0368 at e400 — the bound rejects the *known-converged* reference |
| **W5-2** | final doubling < 0.01 | macOS **+0.000294** | control reads **+0.010233** — fails its own bound |
| **W3-1** | final width doubling ≥ 0.01 | macOS **+0.000883** | control reads +0.002871 |
| **W3-3** | contract spread > 0.02 | macOS **0.0034** | control reads **0.0273** — itself above the bound |

In each case the number that made the threshold *mean* something was measured on
the machine that produced the historical record, and
`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md` — written by me, before
these waves ran — says absolute comparison against that record is **unlicensed**
from this campaign.

I registered the campaign's paired contrasts correctly, so that the *treatment
versus control* comparisons are all same-machine and sound. Then I calibrated
several *absolute* thresholds against the record anyway.

## 2. Why it matters, concretely

W3-3 is the clearest case. It asks whether attention breaks the forward model's
resolution invariance, and 0.02 was chosen because the recorded control spread is
0.0034 — so exceeding 0.02 would be a six-fold departure.

Measured here: attention spreads **0.0435**, and the control spreads **0.0273**.
Attention passes the threshold, but it is only **1.6x** its own control rather
than the ~13x the framing implied. *"Attention breaks resolution invariance"* is
therefore **not distinguished** from *"this campaign's setup is less
resolution-invariant than the recorded one"*, and W3-3's SUPPORTED verdict cannot
carry the interpretation it was written to carry.

W5-2 is the same shape and I registered its consequence explicitly, so it at
least fails loudly: the control missing its bound makes W5-1 uninterpretable by
prior agreement.

## 3. What would have been right

Every threshold should have been expressed as a **contrast against the control
measured in the same campaign**, not against a historical absolute. For W3-3 that
is *"the attention arm's spread exceeds its own control's spread by at least
X"* — a quantity no cross-machine divergence can touch, because both terms move
together.

The paired hypotheses in this campaign — W1-1, W1-2, W1-3, W6-1..3, W7-1..3 —
are all of that form, and none of them has this problem. The four listed above
are the ones where I reached for the record instead.

## 4. What is being done

Nothing, to these waves. Re-anchoring a threshold after seeing its data is the
one repair that is never available, and the verdicts stand as registered and
reported.

For any successor protocol: **a threshold may reference a historical value for
context, but the criterion itself must be computable from cells measured in the
same campaign.** If a hypothesis cannot be written that way, it is a hypothesis
about the historical record and belongs in a protocol that re-measures it.
