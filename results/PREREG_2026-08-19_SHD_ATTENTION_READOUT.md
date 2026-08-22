# Preregistration — time-axis attention read-out on the matched SHD instrument

**Registered:** 2026-08-19, **before any comparison cell was run.** The only
prior executions of the new code are unit tests and two one-epoch mechanical
timing probes (`published-2ms/adjacent-sum-5/h128/e1/s5170001`, one per arm),
run to size the schedule below. Those probes are **not evidence** and are not
cited as a result anywhere.

**Binary:** `cargo run --locked --release -p binn-lab --bin shd-instrument`
**New arm axis:** `--arm <base-arm>+attn`, `--attn-dim`, `--attn-layers`
**Claim axis:** whether the 0.7378 `ff+fixed` ceiling is a *memory* limit.

> **CLOSED 2026-08-19.** All fifteen cells ran, the verdict was computed once
> and is reported in `RESULT_2026-08-19_SHD_ATTENTION_READOUT_PILOT.md`.
> H-A1 SUPPORTED (+0.1702), H-A2 NOT A CAPACITY ARTEFACT (+0.1527),
> H-A3 MEMORY (+0.0869), H-A4 STABLE. No seed was added.

---

## 1. Why this arm exists — the diagnosis it acts on

Three properties of the instrument's forward model, read off the source, not
inferred:

1. **The only cross-bin state is the LIF membrane.**
   `shd_matched_arms::loss_and_gradient_arm_scaled` carries `previous_u` with
   `alpha = exp(-dt / 10.05 ms)`. At the anchor contract (`published-2ms`)
   `alpha = 0.8195`, so a drive is down to 1% of itself after **23 bins =
   46 ms**. Mean SHD utterance duration is **716 ms**
   (`results/shd_instrument_v4/data_summary.json`). The horizon is ~6% of a
   word, and the hard reset `alpha * u * (1 - s)` truncates it further.
2. **The read-out is permutation-invariant over bins.** `rates[h]` is the
   unweighted mean of the hidden spike train over `t`. Order can reach the loss
   only through that 46 ms window.
3. **The hidden error signal is constant in `t`.** `direct_spike[h]` is
   `sum_c w_out[c,h] * p_c / T` — identical at every timestep. No timestep can
   be credited differently from any other, so the layer cannot be taught *when*
   to fire, only how often.

The measured consequence is already on record and is what makes this worth
running rather than asserting:

| manipulation | cost to accuracy | source |
|---|---:|---|
| destroy bin **order** (train + test) | **0.0189** | `RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md` |
| additionally destroy within-bin **synchrony** | **+0.1248** (6.6x more) | same |
| **shortfall of the converged ceiling to the 0.80 gate** | **0.0622** | `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` |

The shortfall is **3.3x the entire order effect the architecture can express.**
Both scaling axes are closed against it — the final budget doubling buys
+0.000294 and the final width doubling +0.000883 — so the missing 0.0622 is not
budget and not capacity.

The registered architectural fix for this is recurrence plus threshold
adaptation, and it remains **unmeasured rather than refuted**: `rec+alif` at
h512 yields zero usable cells, with activation peaks from 3.08e10 to 3.93e33
against ~0.15 for a healthy `ff+alif` (`TODO_2026-08-07_OPEN_WORK.md` §4). That
failure mode is specific to backpropagating through `T` sequential steps.

**This arm takes the other route to the same missing capability** — the one
recent small language models take when a fixed-size recurrent state cannot
recall across a long context: keep the cheap state-passing layers and add a
small number of **full-attention** layers beside them. Here the spiking layer is
the state-passing part. The attention gradient path is constant-depth in `T`, so
it cannot exhibit the `rec+alif` explosion.

## 2. What the arm computes

On the hidden spike train `S` of shape `[t_steps, hidden]`, **after** the
spiking forward, which is untouched:

```
z_0(t)  = W_e s(t) + pos(t)                        W_e: [hidden, d_model]
repeat `layers` times:
    q,k,v = W_q z(t), W_k z(t), W_v z(t)
    A     = softmax_row( q k^T / sqrt(d_model) )   [t_steps, t_steps]
    z(t) <- z(t) + W_o (A v)(t)
pooled  = mean_t z_L(t)
logits += W_a pooled
```

`pos` is a **fixed** sinusoidal code over normalised position, `d_model/2`
geometric frequencies from 1 to 64 cycles per utterance. It is load bearing:
attention without position is permutation-equivariant and mean-pooling on top of
it is permutation-**invariant**, so a position-free block would add pairwise
interaction and stay order-blind — the exact failure under test. Pinned by
`shd_attention::tests::position_is_what_makes_the_read_out_order_sensitive`.

Two structural properties are asserted in tests rather than asserted in prose:

- **Additive, never a replacement.** At `W_a = 0` an attention arm reproduces
  its base arm's membrane, spikes, rates, logits, loss, prediction and every
  base gradient block. `a_zero_read_out_reduces_every_attention_arm_to_its_base_arm`.
- **`W_o` starts at zero**, so every block starts as an exact identity residual;
  `dL/dW_o` is non-zero at the first step, so it does not stay there.

**Correctness of the gradient is checked, not argued.** Everything downstream of
the spike threshold is smooth, so — unlike `w_in` and `w_rec`, whose analytic
values are deliberately surrogates — every attention parameter *and* the spike
gradient `ds_attn` are finite-difference checkable, and all of them are checked
(`every_attention_parameter_matches_finite_difference`,
`spike_gradient_matches_finite_difference`,
`attention_parameters_match_finite_difference_through_the_arm`).

Gate F is preserved by construction: the four base arms are bit-identical
(`every_arm_forward_and_backward_is_bit_pinned` still passes on its 2026-08-03
constants), `SHDWGT1`/`SHDWGT2` are byte-unchanged, and attention arms write a
new `SHDWGT3` container.

## 3. Registered schedule

**This is a PILOT.** It is confirmatory for a budget-matched paired contrast and
for nothing else. It is **not** a ceiling measurement and cannot become one.

| axis | value |
|---|---|
| contract / geometry | `published-2ms` / `adjacent-sum-5` (the anchor) |
| splits | full, 8156 train / 2264 test |
| hidden | 128 |
| epochs | **20** |
| seeds | **exactly three**: 5170001, 5170002, 5170003 |
| attention | `d_model = 32`, `layers = 1` |

Arms D and E use a **fixed** `--temporal-seed 5170001`, so the shuffled corpus
is identical across seeds and identical between the control and the treatment;
only the initialisation varies with seed.

Arms and conditions — 15 cells:

| # | arm | hidden | temporal | seeds | role |
|---|---|---:|---|---:|---|
| A | `ff+fixed` | 128 | intact | 3 | control |
| B | `ff+fixed+attn` | 128 | intact | 3 | treatment |
| C | `ff+fixed` | 192 | intact | 3 | **matched-parameter control** |
| D | `ff+fixed` | 128 | bin-shuffled | 3 | mechanism control |
| E | `ff+fixed+attn` | 128 | bin-shuffled | 3 | mechanism treatment |

Arms A and B share **bit-identical base weights and bit-identical epoch orders**
at each seed, by construction — the attention block draws from its own
`PortableRng` lineage. Verified before registration by byte comparison of the
initialisation files. Any A/B difference is therefore the read-out and nothing
else.

Arm C exists because attention adds 8,832 parameters to a 20,500-parameter
network (+43%). At h192 the *control* carries 30,740 against the treatment's
29,332 — **more** parameters than the treatment — on the same forward model. If
B beats C, parameter count is not the explanation.

Cells may be run concurrently. Each is single-threaded and deterministic, so
concurrency changes `wall_secs` and nothing else; `wall_secs` is not a gate here.

## 4. Hypotheses and thresholds

| ID | statement | threshold | status if met |
|---|---|---|---|
| **H-A1** (primary) | The attention read-out lifts accuracy at a matched budget | mean(B) − mean(A) **≥ 0.05** *and* all three per-seed differences positive | SUPPORTED |
| **H-A2** (capacity confound) | The lift is not explained by parameter count | mean(B) − mean(C) **≥ 0.02** and positive in at least 2 of 3 seeds | NOT A CAPACITY ARTEFACT |
| **H-A3** (mechanism) | The lift is temporal-order-derived | `[mean(B) − mean(A)] − [mean(E) − mean(D)]` **≥ 0.02** | MEMORY, not just capacity |
| **H-A4** (numerical) | Attention avoids the `rec+alif` failure mode | `non_finite_events == 0` and `epoch_max_gradient_norm < 1e3` in all 6 attention cells | STABLE |

**H-A1 and H-A3 are the reason this exists.** H-A1 alone is compatible with
"more parameters, more accuracy". H-A3 is the one that can distinguish memory
from capacity, and it is the one registered to be reported first if the two
disagree.

**Not tested, and must not be claimed:** that any arm clears the registered 0.80
gate; that the 0.7378 ceiling moves; anything about `channels-700`, other
contracts, other widths, converged budgets, or the four base arms' recorded
cells. A short budget was refuted as a proxy for a converged one on the width
axis (`SHD_BPTT_CEILING_NEGATIVE_RESULT.md` erratum E4) and nothing here escapes
that lesson.

## 5. Validity gates — a cell that fails any of these is void, not a result

1. **Non-finite accounting.** `non_finite_events == 0` in every reported cell.
2. **Degeneracy.** `classes_predicted == 20` and `majority_prediction < 0.30`
   in every reported cell; `silent_fraction <= 0.95`, `saturated_fraction <= 0.05`.
3. **Manipulation.** For arms D and E, `counts_preserved == true` and
   `relocated_fraction >= 0.5` — enforced inside `apply_temporal`, which errors
   rather than returning.
4. **Reduction.** The `w_a = 0` reduction test and the Gate F bit-pins must pass
   on the binary that produced the cells.
5. **Initialisation identity.** A and B (and D and E) must load base weights and
   epoch orders that compare byte-equal at each seed.

If gate 2 fails on the *control* arms it is a statement about the 20-epoch
budget, not about attention, and the whole pilot is reported as void rather than
as a negative.

## 6. Stopping rule

**Exactly three seeds. The verdict is computed once, from the fifteen cells, and
reported whichever way it falls.** No fourth seed, no additional width, no
learning-rate sweep, no attention-dimension sweep. If the result is interesting,
the follow-up is a *separately registered* converged-budget run, not more cells
appended to this one.

If the pilot supports H-A1 and H-A3, the registered next step is a converged
h128 run at e400 against the 0.7032 reference this document does not measure.
If it does not, the arm is reported as not supported at this budget and the
attention axis is not carried into any paper claim.

## 7. Named outcomes, before the run

| outcome | reading |
|---|---|
| H-A1 and H-A3 both met | The ceiling is a memory limit and attention reaches the missing structure. Register the converged run. |
| H-A1 met, H-A3 not | Attention helps, but not by using temporal order. Most likely a capacity or optimisation effect; H-A2 decides which. Do **not** describe it as memory. |
| H-A1 met, H-A2 not | The lift is parameter count. Report as a capacity result and stop. |
| H-A1 not met | Attention does not help at this budget. Report the negative; do not extend the budget to look again. |
| H-A4 not met | The arm inherited the recurrent instability after all. That is a finding about the substrate, and it is reported even though the other three are then uninterpretable. |
