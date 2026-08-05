# Amendment — rust instrument kernel, recurrent arms

**Registered:** 2026-08-03, after the change, before any affected cell is run.
**Amends:** `shd_instrument_v4`, rust backend, `binn-learn/src/shd_matched_arms.rs`.
**Supersedes nothing.** No recurrent cell has ever been recorded, so there is no
prior result for this change to invalidate.
**Follows:** `AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md`, which did the same
thing for the feed-forward arms and explicitly deferred this one.

```
claim_axis: Integrity
object_under_test: A performance change to the recurrent and adaptive paths of
  the rust instrument kernel, and whether it leaves every arm's output unmoved.
may_claim: The change is bit-identical on all four arms at fixture level and at
  real training density, and leaves the 216 recorded ff+fixed cells untouched
  (Gate F, 13/13). Speedups as measured in §4.
must_not_claim: That any accuracy, verdict, or scientific conclusion changed.
  That bit-identity is proven for the 203 ff+fixed cells not re-run, or for
  widths and contracts outside those tested.
```

---

## 1. Why this was blocked until now

The predecessor amendment left the recurrent path alone and said why:

> Recurrent arms keep the original single-pass loop.

That was not a scheduling decision. The recurrent forward read the **live**
`previous_s` from inside a single fused loop over hidden units, so unit `h`'s
drive saw the *current* timestep's spike for every `j < h` and the previous
timestep's for every `j > h`. Splitting or reordering that loop would have
changed the result, because the result depended on the iteration order. The
aliasing was the defect recorded in
`DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`.

Fixing the defect — routing the drive through the `previous_spike_log` snapshot —
removed the order dependence. The loop can now be split, and splitting it is
what makes the aliasing *structurally impossible* rather than merely absent.

## 2. Why it was worth doing

Measured before the change, `temporal-sensitivity` over 128 test samples at
`h128 / published-2ms / adjacent-sum-5`:

| arm | wall |
|---|---:|
| `ff+fixed` | 0.32 s |
| `ff+alif` | 0.36 s |
| `rec+fixed` | 6.87 s |
| `rec+alif` | 6.93 s |

**The recurrent arms were 21x slower than feed-forward**, and the gap grows with
width: the recurrent work is `O(hidden^2)` per timestep against `O(hidden)` for
the input drive, so h128 → h512 costs 16x more recurrent and only 4x more
feed-forward. `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` is built on the
`rec+alif` arm at h512. At the pre-change rate that campaign was not affordable.

## 3. What changed

Four loops, all `O(hidden^2)` or strided, none scientific.

**A. Forward input drive, recurrent arms.** Now uses the same transposed
`[n_inputs, hidden]` copy the feed-forward arms already use, so each event is a
contiguous AXPY instead of a strided gather across rows.

**B. Forward recurrent drive, transposed.** `w_rec` is `[hidden, hidden]`
indexed `[h * hidden + j]`, so iterating `j` for fixed `h` walks a row but the
natural vectorised form wants `j` outermost — which gathers a column. The kernel
now transposes `w_rec` once per sample and loops `j` outermost over a contiguous
row.

**C. Forward recurrent drive, sparse.** Silent units are skipped. Spikes are
exactly `0.0` or `1.0` and firing is sparse (`mean_firing_rate` ~0.12 at the
arms measured here), so most of the `hidden^2` product is multiplication by
zero.

**D. Backward.** The same three fixes: the `ds` accumulation
`sum_j du_next[j] * w_rec[j * hidden + h]` was the same strided column gather and
is now `j`-outermost and contiguous; the `w_in` gradient scatter now uses the
transposed accumulator the feed-forward arms already used; and the `w_rec`
gradient accumulation skips silent units. No sparsity is available on the `ds`
term — `du_next` is dense.

## 4. Why each of these is bit-identical, not merely close

**Transposition (A, B, D-first-two) changes layout, not arithmetic.** For any
fixed hidden unit the addends still arrive in exactly the original order —
decay, then the frame's events in frame order, then the recurrent term in
ascending `j` — so no reassociation is introduced. rustc does not enable
fast-math.

**Sparsity (C, D-third) rests on an argument that has to be stated.** Skipping
`j` drops the term `w * 0.0` from the sum. That is an exact no-op *except* when
the accumulator holds `-0.0`, the one value for which `x + 0.0 != x`. The
accumulator can never hold `-0.0`:

- it starts at `alpha * previous_u[h] * (1 - previous_s[h])`, and `previous_u` is
  zero-initialised to `+0.0`;
- when `previous_s[h] == 1.0` the trailing factor is `0.0`, so the product would
  be `-0.0` only for negative `previous_u[h]` — but spiking requires
  `u >= MATCHED_THRESHOLD`, which is `1.0`, strictly positive, and adaptation
  only ever raises it. So `previous_u[h] > 0` in that branch;
- a sum reaches `-0.0` only from `-0.0 + -0.0`; exact cancellation `a + (-a)`
  gives `+0.0` under round-to-nearest.

That argument depends on a constant defined in a **different file**
(`shd_matched::MATCHED_THRESHOLD`), which is exactly the kind of coupling that
rots silently. `sparse_recurrent_skip_requires_a_positive_threshold` pins it, and
says in its failure message that the correct response to a threshold change is to
drop the skip, not to relax the test.

Reasoning is not evidence, which is the standing lesson of the 2026-08-02
amendment. §5 is the evidence.

## 5. Evidence

Reasoning about float behaviour is how the *python* kernel change went wrong on
2026-08-02, so the change is pinned by measurement at two levels.

### 5.1 A new bit-pin for the three arms no gate covered

Gate F regresses recorded cells, and **all 216 recorded cells are `ff+fixed`**.
`ff+alif`, `rec+fixed` and `rec+alif` had no recorded output of any kind: a
shared-kernel change could have moved them with every gate in the repository
still green. That is the same shape of hole that let the recurrent aliasing
defect survive.

`every_arm_forward_and_backward_is_bit_pinned` closes it. It hashes the raw bit
patterns of `membrane`, `spikes`, `logits`, `grad_w_in`, `grad_w_out` and
`grad_w_rec` for all four arms, at a fixture deliberately wider and denser than
the module's default. The constants were captured from the post-defect-fix
kernel, *before* this optimisation, and were unchanged by it.

The test also asserts the fixture's spike density is in `[0.05, 0.95]`. Without
that, a fixture that drifted to all-silent or all-firing would hash a constant
and catch nothing — and in particular would not exercise a sparse path at all.

### 5.2 Density-level regression, which fixture parity does not reach

The binding lesson of `AMENDMENT_2026-08-02_INSTRUMENT_KERNEL_AND_FRAMING.md` is
that fixture parity is not evidence, because the parity fixture has atypically
sparse frames. So each non-`ff+fixed` arm was trained for 3 epochs at
`h128 / published-2ms / adjacent-sum-5` before the change and again after, and
every scientific field of the cell compared with `scripts/compare_cells.py`.

| arm | before | after | speedup | every scientific field |
|---|---:|---:|---:|---|
| `ff+alif` | 11.8 s | 10.3 s | 1.15x | bit-identical |
| `rec+fixed` | 193.0 s | 30.6 s | **6.30x** | bit-identical |
| `rec+alif` | 192.8 s | 30.9 s | **6.24x** | bit-identical |

"Every scientific field" means every key the cell schema carries except
`wall_secs`, compared by `repr` so that `1.0` and `1` cannot pass as equal.
`accuracy`, `mean_loss`, `mean_gradient_norm`, `mean_update_rms`, and both
per-epoch traces are included.

Each timing is a single run. The 6.3x is far too large to be anything else, but
`ff+alif`'s 1.15x is well inside the run-to-run spread §6.4 measures and should
not be quoted as a speedup — the feed-forward arms were not the target and the
honest expectation for them is "unchanged". Bit-identity, unlike the timings, is
exact and not a matter of degree.

### 5.3 Gate F, for the arm that has recorded cells

`ff+fixed` shares every one of these loops. The 13-cell suite was re-run against
every rebuilt binary — **13/13 bit-identical, PASS, five times**:

| binary | when |
|---|---|
| `a544d4a215ed` | before this change |
| `bc7da3a8d3c0` | after the kernel change |
| `3187fe6c2120` | after removing the §6.2 buffer |
| `3187fe6c2120` | same-binary replicate (§6.4) |
| `5ee479f9bc66` | shipping — after hoisting the §6.5 branch |

All appended to `gate-f-rust/runs.jsonl` keyed by binary hash. The recorded
`ff+fixed` cells did not move a bit through four rebuilds of the kernel they
share with the recurrent arms.

The three arms Gate F does not cover were re-checked at training density against
the shipping binary too, not only against the first optimised build: `ff+alif`,
`rec+fixed` and `rec+alif` are bit-identical to their pre-change cells
(6.49x and 6.16x on the recurrent pair).

## 6. Results

### 6.1 Speedup

`temporal-sensitivity`, 128 test samples, `h128 / published-2ms / adjacent-sum-5`:

| arm | before | after kernel | after probe fix |
|---|---:|---:|---:|
| `ff+fixed` | 0.32 s | 0.32 s | 0.32 s |
| `ff+alif` | 0.36 s | 0.40 s | 0.30 s |
| `rec+fixed` | 6.87 s | 1.08 s | **0.74 s** |
| `rec+alif` | 6.93 s | 1.04 s | **0.71 s** |

The third column includes a change to the probe rather than the kernel: it was
recomputing the `intact` forward once per condition, so 8 forward passes per
sample where 5 suffice. That is now hoisted, and the probe's condition means are
bit-identical across the change. It does not affect `train-cell`, so the honest
kernel number is §5.2's **6.3x**, not 9.3x.

Taking the ratio from the training cells rather than the probe, which is less
noisy at this duration: `rec+fixed` 30.6 s against `ff+alif` 10.3 s is **~3x**,
where it was ~21x. The `O(hidden^2)` scaling is unchanged — this moves the
constant, not the asymptotics, so the gap will still widen with width.

### 6.2 A feed-forward regression, found and removed

The first optimised build made `ff+fixed` **slower**. Total wall over the 13-cell
Gate F suite:

| binary | total | vs. pre-change |
|---|---:|---:|
| `a544d4a215ed` (pre-change) | 291.4 s | — |
| `bc7da3a8d3c0` (kernel change) | 319.0 s | +9.5% |

§6.4 and §6.5 follow this through two further builds to the shipping one.

The cause was mine: unifying the backward staged `direct_spike` through a shared
`ds_all` buffer, which the feed-forward arms then read straight back out — a
redundant write-and-read pass per timestep on the path that carries all 216
recorded cells. `ff+fixed` now reads `direct_spike` directly.

Worth recording because of how nearly it was missed. Every gate passed,
bit-identity held on all four arms, and the headline was a 6.3x win; a 9.5%
regression on the *other* arm is exactly the kind of thing that survives that.
It was visible only because Gate F prints per-cell wall times and the previous
run's numbers were still on disk in `runs.jsonl` — the run history added earlier
the same day for an unrelated reason.

A first pass at this section quoted "about 5%", eyeballed from rounded per-cell
seconds. The recorded totals say 9.5%.

Removing the buffer recovered only 7.2 s of the 27.6 s, leaving `ff+fixed`
apparently 7.0% slower than before the change. §6.4 is about whether that
residual is real.

### 6.4 How much of that is measurement noise

The three totals above are single runs of three different binaries, and **the
same binary had never been run twice**, so there was no estimate of run-to-run
spread to compare a 7% difference against. Attributing it to the code change
would have been an unsupported claim, so a same-binary replicate of
`3187fe6c2120` was run.

| run | total |
|---|---:|
| `3187fe6c2120` | 311.8 s |
| `3187fe6c2120` again | 316.2 s |

**Spread on the same binary is 1.4%, so the 7% residual is real.** The
`if arm.recurrent` test on the `ds` source sat inside the innermost `h` loop of
the backward — evaluated once per `(timestep, hidden unit)` on a branch whose
value is constant for the entire call, and enough to stop the loop vectorising.

Fixed by selecting the source slice once per timestep instead:

```rust
let ds_source: &[f32] = if arm.recurrent { &ds_all } else { &direct_spike };
```

### 6.5 It recovered about half, and the rest is being accepted

| binary | total | vs. pre-change |
|---|---:|---:|
| `a544d4a215ed` (pre-change) | 291.4 s | — |
| `bc7da3a8d3c0` (kernel change) | 319.0 s | +9.5% |
| `3187fe6c2120` (buffer removed) | 311.8 s | +7.0% |
| `3187fe6c2120` (replicate) | 316.2 s | +8.5% |
| `5ee479f9bc66` (branch hoisted) | **304.6 s** | **+4.5%** |

Hoisting the branch recovered roughly half the residual. `ff+fixed` remains about
4.5% slower than before this amendment, which is outside the 1.4% same-binary
spread.

**A limitation in that comparison, stated because it bounds the conclusion.**
The 291.4 s baseline is itself a *single* run — the replicate was done on
`3187fe6c2120`, a different build, and the pre-change binary was not kept. So
"+4.5%" rests on one measurement of the baseline against one of the current
build, with a spread estimate borrowed from a third. The residual is very likely
real, since every post-change total exceeds every pre-change one; its magnitude
is uncertain to roughly the spread.

**This is being accepted rather than chased.** Removing it properly means
monomorphising the kernel on the arm — separate specialised loops, or const
generics — which is a structural change with its own verification burden and
does not belong in an amendment about transposition and sparsity. The trade as
it stands: `ff+fixed` costs ~4.5%, on cells of 17-41 s, against 6.3x on
recurrent cells that were 193 s. For the temporal campaign's feed-forward half
(~1.7 h) that is a few minutes.

What would change the calculus is width. The feed-forward tax is a constant
factor; the recurrent saving grows as `O(hidden^2)`. At h512 the recurrent arms
are where all the time goes, and 4.5% on the cheap arm will matter less, not
more.

### 6.3 What this unblocks

`PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` runs `rec+alif` at h512. The
recurrent work is `O(hidden^2)` per timestep, so the saving grows with width —
h512 has 16x the recurrent fan-in of the h128 measured here, which is where the
old kernel was worst.

It also made the shakedown in `MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md`
cheap enough to bother running, and that found `rec+fixed` diverging at the
default initialisation with an epoch-1 gradient norm of 9.8e12. At 193 s per cell
that sweep would have been an hour; at 31 s it was worth doing on a whim. The
optimisation's real value may turn out to be that rather than the campaign hours.
