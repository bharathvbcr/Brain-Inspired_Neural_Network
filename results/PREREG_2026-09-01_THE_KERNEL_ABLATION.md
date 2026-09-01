# Preregistration — remove the temporal kernel and measure what it was worth

**Written before any cell of this series exists.** Attested by git history.
**Closes:** the item `PAPER_GAPS_2026-08-29.md` §2.3 calls **the paper's weakest
load-bearing inference** — §3.8's attribution of the 0.087 residual to the
reference's 25-tap temporal kernel, which rests on *elimination and
code-reading* and on no ablation that moved the kernel.

---

## 1. Why elimination was not enough

§3.8 measures a **0.087** gap between the instrument's converged attention arm
(**0.8320**) and the pinned third-party reference (**0.9390 / 0.9368 / 0.9371**).
Four preregistered ablations turned depth, dropout, read-out style and
normalisation. **Together they do not explain it and two of them subtract** —
removing the second hidden layer *gains* 0.0145, removing batchnorm appears to
gain 0.0058, and only dropout contributes, at 0.0128 of the 0.087.

Reading both forward passes term by term attributed the remainder to a
`Dcls1d` kernel of **25 taps per synapse on every layer**, spanning 250 ms
([`FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md`](FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md)).
`ReferenceKernelInvariantTests` makes the two structural claims executable: the
kernel is built in **every** `model_type`, and `time_step` cannot move without
moving the kernel width. Both are still true. **Neither is a measurement of what
the kernel is worth**, and this series is.

## 2. The manipulation, and why it is minimal

All three `Dcls1d` constructions in `snn_delays.py` take
`dilated_kernel_size = self.config.max_delay`, and `config.py` sets
`max_delay = 250 // time_step`, forced odd — **25** at the pinned
`time_step = 10`.

**The ablation sets `max_delay = 1` and changes nothing else.** A one-tap
dilated convolution is pointwise: the temporal kernel is gone while every other
tensor, layer, neuron, optimiser and schedule is identical. This is the smallest
edit that removes the mechanism, and it is why no `model_type` reaches it —
`sigInit` follows `max_delay` and collapses to 0 with it.

Two arms, both on the pinned checkout, clean protocol, 150 epochs:

| arm | `max_delay` | seeds |
|---|---:|---:|
| **A — reference as pinned** | 25 | 3 |
| **B — kernel removed** | 1 | 3 |

**n = 3, not n = 1.** The existing §3.8 ablations ran at n = 1 with a three-seed
spread of 0.0022, which is why the batchnorm effect is quoted as suggestive and
not resolved. Arm A also **re-measures the reference on this machine**, so the
kernel's cost is a within-series contrast and never a subtraction across
hardware.

## 3. Hypotheses, with bars fixed now

Let **K = mean(A) − mean(B)**, the accuracy the kernel is worth.

**HK-1 — the kernel carries the majority of the residual.** `K ≥ 0.050`, more
than half of the 0.087. *If met*, §3.8's attribution stops resting on
elimination.

**HK-2 — removing it lands the reference near the instrument.** `|mean(B) −
0.8320| ≤ 0.030`. *This is the strong form and it can fail while HK-1 holds*:
the kernel may be worth a lot and still leave a second unexplained difference.
Failing HK-2 while meeting HK-1 is the **most informative** outcome — it says
the kernel is real and it is not the whole story, and §3.8 would have to say so.

**HK-3 — the refutation, and it is not a formality.** If `K < 0.020`, the
attribution is **refuted**: the residual does not live in the kernel, §3.8's
term-by-term reading is wrong, and the paragraph is withdrawn rather than
softened. The instrument's own result does not depend on it — §3.8 is
contextual — so this series is genuinely free to fail.

**HK-4 — Arm A reproduces the pinned reference.** `|mean(A) − 0.9376| ≤ 0.010`
against the three recorded values. **If this fails, nothing else in this series
is read**: an Arm A that does not reproduce means the harness moved, and a
kernel cost measured against a broken baseline is worse than no measurement.

## 4. What this series may NOT do

1. **No arm is re-run to improve its verdict** and no seed is added beyond three.
   Six runs, once.
2. **`max_delay` is the only variable.** If any other config value must change to
   make Arm B run, the series is **abandoned and reported as abandoned**, because
   then it is no longer one manipulation.
3. **It varies the reference, never the instrument.** Every statement about the
   instrument stays an inference from a different model, exactly as §3.8 already
   discloses. This series does not change that limit and must not be written as
   if it had.
4. **No accuracy claim for the instrument follows from any outcome.** The paper
   claims no competitive accuracy and this cannot supply one.

## 5. The analyser is frozen with this document

`scripts/analyse_kernel_ablation.py`, committed here, is the authority on all
four verdicts. It reports `NOT EVALUABLE` against an absent or short corpus
rather than computing a verdict from fewer runs than registered.

---

## 6. Amendment, before any run — `max_delay` is not one value

§2 said the ablation "sets `max_delay = 1` and changes nothing else". Reading
`config.py` before launching shows that is **not achievable as written**, and
the gap is large enough to change what the series measures. Recorded here, with
no data in existence.

**Seven values derive from `max_delay`:**

```
max_delay      = 250//time_step            -> 25
sigInit        = max_delay // 2            -> 12
left_padding   = max_delay-1               -> 24
right_padding  = (max_delay-1) // 2        -> 12
init_pos_a     = -max_delay//2             -> -13
init_pos_b     = max_delay//2              -> 12
time_mask_size = max_delay//3              -> 8
```

Setting `max_delay = 1` and letting every one follow gives `time_mask_size = 0`,
which **switches off the time-masking augmentation**. That is a second mechanism,
unrelated to the kernel, moving in the same edit. An ablation that removes a
temporal convolution *and* a temporal augmentation cannot attribute its effect
to either.

**So the derivations split, and the split is registered now:**

- **Follow `max_delay`** — `sigInit`, `left_padding`, `right_padding`,
  `init_pos_a`, `init_pos_b`. Every one is **mechanically required** by a
  one-tap kernel: padding and delay-positions for a 25-tap convolution are
  undefined for a 1-tap one, and letting them follow is what "the kernel is
  gone" *means*.
- **HELD at its 25-tap value** — `time_mask_size = 8`. It is an **augmentation**
  applied to the input, not a property of the kernel, and holding it is what
  isolates the manipulation.

**This is a deliberate departure from "config.py at `max_delay=1`", and it is
the whole point.** The registration's rule 2 forbids changing another config
value *to make Arm B run*; this holds one fixed **so that Arm B measures one
thing**. The two are opposite moves and only the second is compatible with
attributing K to the kernel.

**Arm C, added now, is what makes the split checkable.** A third arm at
`max_delay = 1` with `time_mask_size` **allowed to fall to 0**, n = 3.
`mean(B) − mean(C)` is then the augmentation's contribution, measured rather
than assumed away.

**HK-5 — the augmentation is not doing the work.** `|mean(B) − mean(C)| ≤ 0.020`.
If it is exceeded, the time-masking augmentation is a material term at this
operating point, K is reported **with that caveat attached**, and the naive
`max_delay=1` ablation that a later reader would reach for first is on record as
confounded.

Nine runs, not six. Still once, still no arm re-run to improve a verdict.
