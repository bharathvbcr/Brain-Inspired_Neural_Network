# Result — the readout is exonerated, and the enumeration is still incomplete

**Prereg:** `PREREG_2026-08-23_READOUT_ABLATION.md`, committed `68053ac` before
the run.
**Artifact:** `results/diagnostics/snn_a3_readout_s5170001.json`.

**Registered outcome 3 fires: the readout is exonerated.**

---

## 1. The measurement

| config | accuracy | effect |
|---|---:|---:|
| delay-free baseline, `loss = 'sum'` (non-spiking, `v_threshold = 1e9`) | 0.9041602 | — |
| **A-3, `loss = 'spike_count'`** (output layer spikes, `v_threshold = 2.0`) | **0.9029385** | **−0.0012** |

The three-seed spread of the clean reference is 0.0022. **The effect is half the
seed spread**, well under the registered 0.005 floor, and is reported as *not
resolvable at n=1* rather than as zero.

## 2. What that means

Replacing the reference's non-spiking summed readout with a **spiking spike-count
readout — the instrument's own readout style — costs essentially nothing.**

This was the difference where the reference was doing something the instrument
architecturally cannot, and the one I expected to be large. It is not a source of
the reference's advantage.

## 3. My prediction was wrong for the third time

| run | predicted | measured |
|---|---|---|
| delay-free | 0.75–0.85 | **0.9042** — wrong on magnitude |
| A-1 layers | cost 0.02–0.05 | **+0.0145** — wrong on **sign** |
| A-3 readout | cost 0.02–0.05 | **−0.0012** — wrong on magnitude |

Three predictions, three failures, in three different ways. The fourth outcome
branch I added after the sign error did not fire either — the readout was not
*better*, it was simply the same.

I do not have a working model of this reference. That is worth stating plainly:
each of these was registered in advance, and the value of the series is entirely
in the measurements, not in my expectations about them.

## 4. The decomposition, complete as far as it goes

Marginal effects from the delay-free baseline, which **must not be summed** —
registered in advance and load-bearing here:

| feature | effect | resolvable? |
|---|---:|---|
| learning the delay positions | **+0.0348** | yes, 16× spread |
| second hidden layer | **−0.0145** | yes, 6.6× — *hurts* |
| dropout 0.4 | **+0.0128** | yes, 5.8× |
| non-spiking summed readout | **−0.0012** | **no**, 0.55× |

```
best delay-free config seen (1 layer, dropout 0.4)   0.9187
instrument, converged attention arm                  0.8320
                                                     ------
                                                     0.0867
```

**Of everything tested, only dropout is a positive contributor the instrument
lacks — and it is worth 0.0128 of 0.0867.** Roughly 0.074 is unaccounted for by
any measured cause.

## 5. The honest conclusion

The prereg named this outcome and its default: *"the gap is in binning, or in
something not yet on the list — and after three enumeration errors today, the
second is the honest default."*

That default now stands. Of the seven differences I enumerated, four are measured
and none explains the gap; one (binning) is confounded with kernel extent and
cannot be cleanly tested on this config; two (width, budget) are confounded with
the instrument's own tuning rather than the reference's.

**The most likely remaining explanation is a difference I have not enumerated.**
Three times today I have been wrong about what this config contains — claiming two
features that were switched off, and twice claiming knobs were not knobs. A fourth
omission is the prior, not the exception.

What I would do next is stop ablating the reference and instead **diff the two
forward passes directly** — the instrument's kernel against `SnnDelays.forward`,
term by term — rather than continuing to guess at the config surface. Every result
in this series came from reading code that contradicted an assumption, and none
came from a prediction.

## 6. What this does not establish

- **One seed.** The effect is below the resolvability floor, so "no effect" is not
  claimed either; only "not resolvable at n=1".
- **It varies the reference, not the instrument.**
- **The run was re-done after the repository moved mid-flight.** The first attempt
  died on a `FileNotFoundError` when the dataset path vanished; that attempt is
  void and produced no number. The re-run used the same cache, dated Jul 27 and
  moved intact, so it is comparable with the baseline and the other ablations.
- **No gate moved.** Diagnostic outside the instrument tree; `references/`,
  `reference-manifests/`, `reference-states/` untouched; `gates.json` never opened.
