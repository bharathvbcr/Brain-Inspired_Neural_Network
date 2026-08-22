# The attention read-out's converged gain is 0.0421 — below the registered 0.05, and W1-1 is NOT SUPPORTED

**Protocol:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` §1, registered before
any campaign cell ran. Verdicts computed once, at the registered n=12, by
`scripts/aws/analyse_campaign.py` — written before the data existed.
**Cells:** 60 of 60, **0 voided**, `published-2ms` / `adjacent-sum-5`, h128, e400,
full splits, seeds 5170001–5170012.
**Backend:** rust on Linux/aarch64, binary `22d97c51`, single binary throughout.

```
claim_axis: architecture
may_claim: A same-machine paired contrast at e400/h128 between ff+fixed,
  ff+fixed+attn, a parameter-matched ff+fixed at h192, and the bin-shuffled
  counterparts of the first two.
must_not_claim: That the pilot's +0.1702 replicates - it does not. Any comparison
  with the macOS-recorded 0.7032 / 0.7378: every instance FAILED the
  cross-machine gate, so those comparisons are unlicensed. That the arm is
  converged - W1-4 says otherwise and W1-4 is itself defective (§4).
```

---

## 1. The registered verdicts

| ID | measured | threshold | verdict |
|---|---:|---|---|
| **W1-1** | **+0.0421** (t=8.21 vs crit 3.106) | ≥ 0.05 **and** paired t at α=0.01 | **NOT SUPPORTED** |
| **W1-2** | **+0.0301** vs h192 | ≥ 0.02 | NOT A CAPACITY ARTEFACT |
| **W1-3** | **+0.0912** | ≥ 0.02 | **MEMORY, not just capacity** |
| **W1-4** | worst −0.5009 | > −0.02 | UNDERTRAINED → **W1-1 reported UNTESTED** |

**W1-1 required both criteria and met only one.** The effect is about as
statistically reliable as this instrument produces — t=8.21 against a critical
3.106, all twelve seeds positive — and it is **below the effect size registered as
worth having**. Registering both was the right call and the disagreement is the
finding, exactly as it was when H1 split the same way on 2026-08-03.

## 2. The measurements

| arm | n | mean | sd | tail_loss_improvement |
|---|---:|---:|---:|---:|
| `ff+fixed` h128 | 12 | 0.7062 | 0.0045 | −0.0368 |
| `ff+fixed+attn` h128 | 12 | **0.7483** | 0.0163 | −0.2456 |
| `ff+fixed` h192 (more parameters) | 12 | 0.7181 | 0.0069 | −0.0492 |
| `ff+fixed` bin-shuffled | 12 | 0.6934 | 0.0063 | −0.0336 |
| `ff+fixed+attn` bin-shuffled | 12 | 0.6442 | 0.0105 | −0.2675 |

Per-seed `attn − ff+fixed`: +0.0115 to +0.0707, **all twelve positive**.

## 3. The pilot does not replicate, and that is the headline

The pilot reported **+0.1702** at e20. At e400 the same contrast is **+0.0421** —
**a quarter of it.** The pilot is not wrong about what it measured; it measured a
different thing. Its scope section said so in advance: *"this is a 20-epoch pilot
and the attention arms are visibly undertrained… 0.7509 must not be quoted against
the 0.7378 converged ceiling."*

It was right to say so, and the campaign is what turned that caution into a
number. **Anyone citing +0.1702 as an architecture result is citing a budget
effect.**

Why the gap closed: between the pilot and here the *control* gained roughly 0.12
from twenty times the budget while the attention arm gained little. Quantifying
that properly needs a same-machine ladder, which is what wave 6 exists for —
comparing the pilot's macOS 0.7509 to this Linux 0.7483 crosses machines and is
not licensed (`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md`).

## 4. W1-4 fails, and its failure means the opposite of what it says

W1-4 reports **UNDERTRAINED**, so W1-1 is additionally reported **UNTESTED**, per
the prereg's named outcomes. That consequence is applied, not softened.

But the *signature* is not undertraining. The attention arm's
`tail_loss_improvement` is −0.2456 against the control's −0.0368: loss still
falling fast. Test accuracy meanwhile is flat. Falling loss with flat accuracy is
**overfitting**, and the instrument's own registered convergence rule reads that
branch as *"the budget is sufficient"*.

This was disclosed before any attention cell existed
(`DEFECT_2026-08-19_W1_4_THRESHOLD_IS_NOT_BUDGET_INVARIANT.md`): the statistic's
window scales with the budget, and the −0.02 bound — calibrated on a 2-epoch
window at e20 — also rejects the known-converged `ff+fixed` reference here
(−0.0368). **Wave 5's e800 ladder settles convergence with the budget-invariant
rule.** Until it lands, no claim here describes this arm as converged.

## 5. What survived, and it is the more interesting half

**W1-3 is emphatic.** On bin-shuffled data the attention arm is not merely
unhelpful — it is **worse than the control**, by −0.0492, in **all twelve seeds**.
The same read-out that adds +0.0421 when temporal order is intact *subtracts*
almost as much when order is destroyed and nothing else is.

That is a far stronger mechanism result than the pilot's "about half the gain
survives shuffling". A read-out that only helps when order is present, and hurts
when it is absent, is using order. The base arm's own order sensitivity here is
0.0128 (0.7062 vs 0.6934), so attention's order-derived component is **7×** what
the architecture could previously express.

**W1-2 holds too**: the h192 control carries more parameters (30,740 vs 29,332)
and buys +0.0119 against attention's +0.0421.

## 6. Where this leaves the paper

The registered reading of "W1-1 not met" is: *report the negative; the attention
axis does not enter any paper claim.* Applied literally, that ends it.

Applied honestly, three things are true at once and all three belong in the
record:

1. **The architecture claim is dead.** +0.0421 is not the 0.05 that was
   registered as worth having, and the pilot's +0.1702 was a budget artefact.
2. **The mechanism claim is alive and stronger than before.** W1-3 at +0.0912,
   twelve seeds out of twelve, with the shuffled arm going negative.
3. **A different claim is untested.** If wave 6 shows the arm reaching its
   accuracy at e20 and holding it, the result is *sample efficiency*, not
   ceiling — a claim this campaign has not yet earned and which W6-1/W6-3 were
   registered in advance to decide.

Waves 5 and 6 are already queued. Nothing here should be written up as a paper
claim until they land.
