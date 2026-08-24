# Preregistration — the readout, which is the difference that is actually clean

**Registered:** 2026-08-23, before the run.

**Follows:** `RESULT_2026-08-23_DECOMPOSING_THE_RESIDUAL.md`, where neither tested
cause explained the gap and the enumeration was flagged as probably incomplete.

---

## 1. I said these two were not knobs. I was wrong about one and right about the other, for the wrong reason.

`RESULT_2026-08-23_DECOMPOSING_THE_RESIDUAL.md` §4 said differences 6 (binning)
and 7 (readout) "are not knobs on this config". **Both are knobs.** That is the
third thing I have got wrong about this config today. Read, not inferred:

```python
loss = 'sum'   # 'mean', 'max', 'spike_count', 'sum'          config.py:48
output_v_threshold = 2.0 if loss == 'spike_count' else 1e9    config.py:50
time_step = 10                                                config.py:20
```

**The readout is a clean single variable and is run.** With `loss = 'sum'` the
output layer has a threshold of 1e9 and therefore *cannot spike*: `output` is
membrane potential, summed through a softmax. With `loss = 'spike_count'` the
threshold becomes 2.0, the output layer **spikes**, and `m = torch.sum(output, 0)`
counts those spikes — which is what the instrument's rate readout does. One config
line, with the threshold derived by the author's own code.

**The binning is not clean and is not run.** `time_step` also drives
`max_delay = 250 // time_step`, which is `dilated_kernel_size` on every `Dcls1d`
layer. Going 10 → 2 ms would take the kernel from 25 taps to 125 and rescale
`init_tau` at the same time. That is three changes wearing one name, and a number
from it would not attribute to binning. Saying so is the finding; running it would
manufacture a result.

## 2. The ablation

| id | change from the delay-free baseline (0.9042) | isolates |
|---|---|---|
| **A-3** | `loss: 'sum' → 'spike_count'` | difference 7, the readout |

Everything else held: `model_type = 'snn'`, 2 hidden layers, dropout 0.4, seed
5170001, 150 epochs, clean protocol.

## 3. Why this one is worth a run when the last two were not decisive

A summed non-spiking readout is not a small variation on a spike-count readout —
it is a different output computation, and it is the one difference where the
reference is doing something the instrument architecturally cannot. If it is
worth little, the readout is exonerated and the remaining gap is in binning or in
something still unenumerated. If it is worth a lot, it is the first identified
cause that is both large and specific to what the reference *is*.

## 4. Registered thresholds and outcomes

Same resolvability bar as before, from the 0.0022 three-seed spread: **≥ 0.010
real, < 0.005 not resolvable at n=1, between suggestive.**

**Signs are named explicitly this time.** My last outcome set assumed every effect
was positive and had no branch for the one that was not.

- **A-3 much lower than 0.9042 (≥ 0.03)** → the non-spiking summed readout is a
  major part of the reference's advantage, and it is something the instrument's
  rate readout structurally lacks. The first large identified cause.
- **A-3 lower by 0.010–0.03** → a real but partial contribution; the residual is
  still mostly elsewhere.
- **A-3 within 0.010 either way** → the readout is exonerated. The gap is in
  binning, or in something not yet on the list — and after three enumeration
  errors today, the second is the honest default.
- **A-3 *higher* than 0.9042** → the spiking readout is *better* for the
  reference, as removing a hidden layer was. That would mean the reference is
  carrying two suboptimal choices, and the "reference is stronger" framing needs
  re-examining rather than patching.

## 5. Prediction

I expect **A-3 lower by 0.02–0.05**. I have now predicted wrongly twice today —
once on magnitude, once on sign — so this is recorded with low confidence and the
fourth outcome above exists because of it.

## 6. Constraints

Diagnostic, not a reference. Scratch copy outside `results/`; nothing written to
`references/`, `reference-manifests/`, `reference-states/`; `runner.reference()`
not called; `gates.json` not opened.
