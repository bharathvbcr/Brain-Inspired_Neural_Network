# Result — a depth instrument that works, and a hypothesis anchored in the wrong place

**Prereg:** `PREREG_2026-08-23_DEPTH_ON_A_TASK_WITH_HEADROOM.md`, registered
before the suite was built.
**Artifact:** `results/credit_depth_scaling_v1.md`.

**Registered outcome fired: V-1 fails at depth 1. No D-1 or D-2 verdict is
issued.**

---

## 1. What the suite reports

n = 12, width 64, 40 epochs, `CreditDepthTask` at `n_states = 8`, task depth 4,
chance 0.1250.

| network depth | treatment | SE | ceiling | SE | gap | headroom | ceiling health |
|---:|---:|---:|---:|---:|---:|---|---|
| 1 | 0.4742 | 0.0088 | 0.4692 | 0.0080 | **+0.0050** | ok | **INVERTED** |
| 2 | 0.5708 | 0.0135 | 0.5822 | 0.0125 | −0.0114 | ok | ok |
| 3 | 0.5336 | 0.0116 | 0.5794 | 0.0085 | −0.0458 | ok | ok |
| 4 | 0.4647 | 0.0122 | 0.5119 | 0.0084 | −0.0472 | ok | ok |

**The headroom held.** Every ceiling sits between 0.4692 and 0.5822 — far above
chance and nowhere near the 0.95 saturation gate. This is the first depth
instrument in the workspace whose reference has somewhere to fall, which was the
entire point of moving off `CoincidenceTask`.

**No readout is degenerate.** Both arms predict 6.9–8.0 of 8 classes at every
depth, with the most-predicted class taking 0.21–0.27. The two suspicious cells
in the feasibility sweep did not recur here.

## 2. Why no verdict, and it is my design error not the data's

Depth 1 is **INVERTED**: the treatment (0.4742) sits above the ceiling (0.4692).
`CeilingHealth` refused it and the report printed the harness banner, exactly as
designed — the instrument declined to report a comparison it could not support.

The inversion is 0.0050 against standard errors of 0.0088 and 0.0080. It is
noise. But it is **structurally guaranteed to be noise**, and that is the
problem:

> At network depth 1 there is nothing between the hidden layer and the readout,
> so the learned feedback matrix aligns to the readout itself and the treatment
> *is* the true gradient. The two arms are the same computation. Which one lands
> higher is a coin flip.

v136 recorded exactly this — *"at depth 1 the learned feedback aligns to the
readout, so the treatment is the true gradient there… a consistency check, not a
finding"* — and I anchored **D-1 on `gap(4) − gap(1)`** anyway. A hypothesis whose
baseline is a depth where the two arms are identical by construction was badly
posed before any data existed.

So the printed `D-1 = 0.0522, SUPPORTED` **must not be read**. It is a difference
taken against a depth the harness invalidated, and the report says so.

## 3. What can be said descriptively

Depths 2, 3 and 4 are individually healthy, unsaturated, and non-degenerate.
Their gaps:

```
depth 2   -0.0114
depth 3   -0.0458
depth 4   -0.0472
```

The treatment falls further below its ceiling as the network deepens. **This is
descriptive and carries no verdict** — it was not the registered contrast, and
reporting it as one after the registered contrast failed is the substitution this
record exists to prevent. It is stated because it is what the next registration
should be about.

Note also that both arms *rise* from depth 1 to 2 and then fall: the ceiling goes
0.4692 → 0.5822 → 0.5794 → 0.5119. Depth helps this task up to a point, then
hurts. That is a statement about the architecture on this task, and D-2 (which
would have measured it) is likewise unreadable at 0.0428 against a 0.05 bar,
anchored on the same invalid depth.

## 4. What happens next, and what must not

**Next:** a re-registration anchoring the depth contrast on **depth 2** — the
shallowest network where the arms are genuinely different computations — run on a
**disjoint seed block**, in the pattern
`PREREG_2026-08-23_TRACK_B_REREAD.md` established. The seeds used here are now
seen and must not decide it.

**Must not:** re-read D-1 against depth 2 using this data. The anchor would be
chosen after seeing that depth 1 failed and that the depth-2 anchor gives a
larger effect. That is selecting an analysis on the outcome, and it is precisely
what the registered response to a V-1 failure forbids.

## 5. What this may not claim

- **No depth verdict is established.** Not a penalty, not its absence.
- **It is a compositional symbolic task**, not an input-rich sensory one. The SHD
  suite remains the thing this is a substitute for, and it remains refused at the
  calibration gate.
- **Task depth 4 is one point.** At task depth 8 the ceiling falls to 0.2750 and
  that regime is untested.
- **v136 is not superseded.** It stands as the `CoincidenceTask` result with its
  saturation caveat; this is a second task.

## 6. What did work

The instrument refused itself. `CeilingHealth` caught an inversion, the
saturation gate was live and passed on its own terms, the readout audit ran on
both arms at every depth, and the report declined to issue a verdict rather than
printing one. Every one of those guards was added this week in response to a case
where its absence let a bad number through, and here they cost a verdict I wanted.
That is what they are for.
