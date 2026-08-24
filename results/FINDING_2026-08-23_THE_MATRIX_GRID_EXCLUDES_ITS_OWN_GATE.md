# Finding — the instrument does not top out at 0.72, and the matrix grid excludes every configuration that passes

**Investigating:** why the instrument appeared to top out at 0.7182 against a
reference at 0.9390–0.9573.

**Corrects:** `RESULT_2026-08-23_CROSS_BACKEND_SLICE.md` §6, which I wrote
yesterday and which framed the question wrongly.

---

## 1. The premise was wrong

I recommended asking "why does the instrument top out at 0.72 when the reference
reaches 0.94". It does not top out at 0.72.

| scope | best accuracy |
|---|---:|
| the 432-cell **matrix** grid | 0.7182 |
| plain arm at converged budget (h1024/e400) | 0.7378 |
| **attention arm, d32/L4/e400** | **0.8821** (mean 0.8320) |
| pinned third-party reference | 0.9390–0.9573 |

**140 of 776 campaign cells sit at or above 0.80 and carry `scientific_status:
CELL_PASS`.** The instrument clears its own gate routinely. The 0.7182 figure is
the ceiling of one restricted grid, and I reported it as a property of the
instrument.

## 2. Why the matrix can never be CALIBRATED

The matrix grid, read from `all_cells()`:

```
epochs   : [20, 100]                      <- no e400
hidden   : [128, 256, 512]                <- no h1024
contracts: 6      geometries: 2
arm      : no arm field at all            <- every cell is plain ff+fixed
```

The three things that lift the instrument over 0.80 are **e400**, **h1024**, and
the **attention read-out**. The matrix grid contains none of them.

So `matrix_verdict` returning `FAIL` is not evidence about the instrument. **Its
grid excludes every configuration that clears its own `CELL_PASS` floor of 0.80.**
That is a mismatch between the grid and the gate, designed in, and no number of
additional cells changes it — which is a second and better reason not to run the
remaining 115 Python cells.

The recommendation in the previous document survives; the reasoning under it does
not.

## 3. The real gap, and what the record claims about it

Against the converged attention arm, the gap to the reference is **0.8320 → 0.9390
≈ 0.107**, not the 0.22 I quoted.

`HANDOFF_2026-08-02.md` §3 decomposes it:

> DFA 0.234 → BPTT 0.7151 = locality cost 0.481; 0.7151 → delays 0.951 =
> **architecture cost 0.236**

**That second half is a subtraction with a label on it, not a measurement.** It
attributes the entire residual to "delays" without testing delays. The reference
differs from the instrument in at least five ways, every one of which is known to
matter on SHD:

| | reference (`best_config_SHD.py`) | instrument (`ff+fixed`) |
|---|---|---|
| delays | **DCLS learnable, up to 250 ms** | none |
| hidden layers | **2** | 1 |
| dropout | **0.4** | none |
| augmentation | `augmentations.py` | none |
| synapses | **stateful**, `tau = 10 ms` | stateless |

Naming the residual "architecture cost" is fair. Naming it "delays" is an
attribution the record has not earned.

## 4. The decisive experiment is available and cheap to specify

`config.py:16` of the pinned reference:

```python
# model type could be set to : 'snn_delays' | 'snn_delays_lr0' | 'snn'
model_type = 'snn_delays'
```

**The reference repository ships a delay-free mode.** Running the same pinned
commit with `model_type = 'snn'` isolates delays from the other four differences
in a single run:

- if `'snn'` lands near 0.72–0.75, delays account for essentially the whole
  residual and the instrument sits at the top of its architecture class;
- if `'snn'` lands near 0.85, delays are a minority of it and the remainder is
  layers, dropout, augmentation and stateful synapses — four things the
  instrument could adopt without implementing delays at all.

Those two outcomes point at completely different next steps, which is what makes
the experiment worth ~5.5 hours.

**It must not write into the reference artifact directories.** It is a diagnostic,
not a calibration reference: a modified config produces a number that is not the
pinned baseline, and letting it near `references/` would corrupt the six cells
that just cost 33 CPU-hours.

## 5. What is established here, and what is not

**Established:** the instrument clears 0.80 in 140 recorded cells; the matrix grid
excludes every such configuration; the true gap to the reference is ~0.107 at the
converged attention arm; and the record's "delays" attribution rests on
subtraction rather than measurement.

**Not established:** what causes the remaining 0.107. Five candidates are named
above and none is tested. The `'snn'` ablation would settle one of them.

**Also unexplained, and worth flagging separately:** the recurrent arms are the
*worst* in the record — `rec+fixed` 0.2633, `rec+alif` 0.5141 — where the
literature has recurrence helping on SHD. That is the opposite of the expected
direction and is consistent with the numerical marginality recorded in
`FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md` §4. It is not the same question
as the reference gap and should not be folded into it.
