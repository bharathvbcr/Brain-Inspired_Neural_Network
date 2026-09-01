# Defect — wave 22 cannot evaluate H22-3, and no cell that lands will change it

**Found:** 2026-08-31, with 393 of wave 22's 504 cells collected and 111 still
running.
**Status:** structural. It is **not** a shortage of cells and it is **not**
fixable by waiting.
**Scope:** H22-3 only. **H22-1, H22-2 and H22-4 are unaffected** — six of
H22-1's twelve points are already MET on complete 12/12 seed pairs.

---

## 1. What the analyser needs

[`analyse_wave22.py`](../scripts/aws/analyse_wave22.py) evaluates H22-3 by
comparing each depth point against a `d32l4` twin **at the same width, on the
anchor contract**:

```python
ANCHOR = ("published-2ms", "adjacent-sum-5")
twin, _, twin_pairs = did(cells, hidden, ANCHOR[0], ANCHOR[1], "d32l4")
```

`cells` is wave-isolated — `WAVE = "w22cov"` — so only this wave's own cells
can supply that twin.

## 2. What the wave contains

**No `w22cov` cell is `d32l4` at `published-2ms`/`adjacent-sum-5`.** Zero, at
any width. The wave's only `d32l4` cells are at `fixed-t100`, `fixed-t250` and
`fixed-t500`, where they exist to serve H22-4's resolution ladder.

So `twin` is `None` for every entry in `DEPTH_POINTS`, every row prints
`not evaluable`, and the hypothesis reports **NOT EVALUABLE** — as it already
does with `h128/d64l4` sitting at a complete 12/12 pairs and **+0.1331**.

## 3. Why the two halves disagree

Both decisions are individually correct and they were never checked against
each other.

The preregistration's H22-3 compares depth points "against their `d32/L4` twins
at the same width". At the anchor contract those twins exist **only in the
archive** — wave 21 and earlier produced them.

§3 of the same preregistration then forbids exactly that. Pairing new shuffled
halves against archived intact halves would build every difference-in-differences
out of two binaries, one predating the forward-finiteness guard, and
`bootstrap.sh` states the rule: *"a campaign whose cells came from more than one
binary is not one experiment."* The wave was made self-contained at 2.8x the
cost for that reason, and `analyse_wave22.py` admits only `w22cov` cells so an
archived cell "cannot be substituted by filename order".

**The isolation that makes every other verdict sound is the same isolation that
starves H22-3.** The 504-cell plan needed anchor-contract `d32l4` cells at
h128, h256, h512 and h768 to close the loop, and it has none.

A second, independent inconsistency points the same way: the preregistration
names `d32/L1`, `d32/L2` and `d32/L3` among the depth points, and those exist
only at h1024 — but `DEPTH_POINTS` contains no h1024 entry at all. Prose and
analyser disagree about which comparisons H22-3 even is.

## 4. What is deliberately NOT done

1. **The analyser is not edited.** It was frozen in `7fb7a70` before any cell
   existed, and it is the authority on every verdict in this wave. Changing it
   after seeing six MET results is indistinguishable from tuning it to the data,
   whatever the intent.
2. **Cells are not added.** The stopping rule is *504 cells, once*. Adding four
   new `d32l4` points now — after results are visible — is the precise move the
   rule exists to forbid.
3. **Archived twins are not substituted.** That is the two-binary contrast §3
   spent 2.8x to avoid, and it would corrupt a verdict rather than rescue one.

**H22-3 is therefore reported as NOT EVALUABLE, permanently, with this reason
attached.** It was registered as *a question, not a prediction*, so nothing is
refuted and no claim in the manuscript rests on it.

## 5. What this costs, and what it does not

It costs the answer to one registered question: whether the contrast depends on
read-out depth. That question is now open and would need its own wave, designed
with its own anchor twins inside it.

It costs nothing else. H22-1's twelve points are self-contained by construction;
six are MET on 12/12 pairs, including `h128/d64l4` at **+0.1331** and
`h128/fixed-t500/d32l4` at **+0.1161**. H22-2 counts covered points. H22-4's
three `fixed-tN` rungs are all in the plan.

## 6. The check that would have caught it

The analyser was verified before launch to report `NOT EVALUABLE` **against an
empty corpus**. That proves it fails closed on no data. It does not prove that
the *planned* corpus can satisfy it.

**A frozen analyser should be run against its own plan, not only against
nothing.** Feeding `plan_cells.py`'s output — ids alone, no results — and
asserting that every hypothesis finds the cells it references would have failed
here, at registration time, when adding four points was still free.

That check does not exist. It is the one worth building before the next wave is
registered.
