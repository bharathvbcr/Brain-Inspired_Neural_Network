# The recurrent arm produces zero usable cells, with or without attention — 24 of 24 diverged

**Protocol:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` §4 (wave 4),
registered before any campaign cell ran. Verdicts computed once.
**Cells:** 24 planned, **0 completed, 24 diverged**, 0 voided.
**Backend:** rust on Linux/aarch64, binary `22d97c51`, h256, e100, gradient
clipping at 1.0, surrogate scales 1.0 and 0.4, 6 seeds per configuration.

---

## 1. Result

| arm | surrogate scale | completed | diverged |
|---|---:|---:|---:|
| `rec+alif` | 1.0 | **0** | 6 |
| `rec+alif` | 0.4 | **0** | 6 |
| `rec+alif+attn` | 1.0 | **0** | 6 |
| `rec+alif+attn` | 0.4 | **0** | 6 |

Every cell aborted on a non-finite training value, which is the instrument
refusing to emit a cell rather than a harness fault. **Divergence is the
measurement here, not a failure to measure.**

## 2. The registered verdicts

| ID | statement | verdict |
|---|---|---|
| **W4-1** | Attention does not inherit the recurrent instability | **NOT SUPPORTED** — 12 of 12 attention cells diverged |
| **W4-2** | (descriptive) usable-cell counts, side by side | `rec+alif` **0/12**, `rec+alif+attn` **0/12** |

W4-2 was registered as descriptive with the reason stated in advance: *"the honest
prior is that it will not help: the explosion is in the recurrent BPTT path, which
attention sits beside rather than inside. Registering it as a hypothesis would be
inventing a prediction to be right about."*

## 3. The comparison is what makes this reportable

For several hours the only divergences were `rec+alif+attn`, because the base arm
had not yet been scheduled. On that evidence alone the reading "attention
destabilises the recurrent arm" was available and wrong — the base arm was absent,
not stable.

**Both arms now sit at 0/12.** Attention neither causes nor cures the instability.
It is a property of the recurrent BPTT path.

## 4. This reproduces the record under strictly easier conditions

`TODO_2026-08-07_OPEN_WORK.md` §4 records `rec+alif` producing *zero usable cells*
at h512, with activation peaks from 3.08e10 to 3.93e33. This wave reproduces that
at **half the width**, **with gradient clipping active at 1.0**, and **at a reduced
surrogate scale of 0.4** — the two levers the record names as what was needed to
get any recurrent cell to complete at all.

Neither lever helps. The registered hypothesis
`PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` H1, which needs a `rec+alif` number,
therefore remains **unmeasured rather than refuted**, and this wave is evidence
that it is not cheaply measurable on this instrument.

## 5. What must not be claimed

- **Not** that recurrence cannot work on SHD. Published recurrent SNNs reach far
  higher; this is a statement about *this* forward model and *this* BPTT path.
- **Not** that attention is stabilising or destabilising. Both arms are at zero;
  there is no contrast to read either way.
- **Not** that clipping or surrogate scaling are ineffective in general — only
  that at these two settings, at h256/e100, they did not produce one usable cell.

## 6. What would move it

The record's own next suggestion, untried here: **truncated BPTT**. That bounds
the depth of the recurrent gradient path directly, where clipping only bounds its
magnitude after the fact and surrogate scaling only shrinks its per-step gain.
That is a separate protocol and is not authorised by this campaign.
