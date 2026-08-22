# Wave 8 — the headline's scope, measured instead of assumed

**Registered:** `PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md`, before any cell existed.
**Ran:** 2026-08-20 → 2026-08-21, 4 × `c7g.16xlarge` spot, **72/72 cells, 0 failures,
0 voided**. Pinned binary `22d97c51ab02` — the same one that produced waves 1–7,
which is what makes the reused controls legitimate.
**Archive:** [`shd_attention_campaign_v2/`](shd_attention_campaign_v2/) (cells,
manifest with per-file hashes, plan, verdicts).

**Two of six hypotheses supported. One of the failures costs the paper a claim it
was about to make.**

---

## 1. The measurements

144 cells, all passing every validity gate: 72 new, plus 72 reused controls from
waves 1/3 and the registered run.

| configuration | mean | min | max | seeds ≥ 0.80 |
|---|---:|---:|---:|---:|
| **d32/L4 anchor h128** *(registered run)* | **0.8320** | 0.8083 | 0.8472 | **12/12** |
| d32/L2 anchor h128 | 0.7897 | 0.7562 | 0.8211 | 4/12 |
| d32/L1 anchor h128 | 0.7483 | 0.7279 | 0.7774 | 0/12 |
| `ff+fixed` anchor h128 | 0.7062 | 0.7005 | 0.7164 | 0/12 |
| d32/L4 `channels-700` | 0.7864 | 0.6882 | 0.8202 | 6/12 |
| `ff+fixed` `channels-700` | 0.6774 | 0.6718 | 0.6873 | 0/12 |
| d32/L4 h512 | 0.8233 | 0.7933 | 0.8564 | 10/12 |
| `ff+fixed` h512 | 0.7357 | 0.7301 | 0.7420 | 0/12 |
| d32/L4 h1024 | **0.5768** | **0.3746** | 0.7412 | 0/12 |
| `ff+fixed` h1024 | 0.7386 | 0.7328 | 0.7407 | 0/12 |
| d32/L4 `published-10ms` | 0.8225 | 0.7915 | 0.8476 | 10/12 |
| `ff+fixed` `published-10ms` | 0.6734 | 0.6639 | 0.6829 | 0/12 |

## 2. Verdicts

| id | claim | result | verdict |
|---|---|---|---|
| **S-1** | clears 0.80 on `channels-700` | 0.7864, 6/12 (bars: 0.80, 9/12) | **NOT SUPPORTED** |
| **S-2** | the *gain* survives `channels-700` | **+0.1090**, positive **12/12** | **SUPPORTED** |
| **S-3** | depth rescues the width inversion | **−0.1618**, positive 1/12 | **NOT SUPPORTED** |
| **S-3b** | width trend *(descriptive, no verdict)* | +0.1258 → +0.0876 → −0.1618 | monotone decreasing |
| **S-4** | the gain survives `published-10ms` | **+0.1491**, positive **12/12** | **SUPPORTED** |
| **S-5** | *(mechanistic)* the gain shrinks with fewer timesteps | **+0.1491 at t=72** vs +0.1258 at t=358; bar was ≤ +0.1058 | **NOT SUPPORTED** |
| **S-6** | depth ladder monotone at convergence | 0.7483 → 0.7897 → 0.8320 | **SUPPORTED** |

## 3. What each one changes

### S-1 / S-2 — the effect generalises; the number does not

This is the cleanest result in the wave, and it separates two things the paper was
treating as one.

- **The 0.80 clearance is specific to `adjacent-sum-5`.** On the standard
  700-channel input, d32/L4 reaches 0.7864 with 6 of 12 seeds over the bar. The
  paper must state the geometry as part of the headline, not as a footnote.
- **The attention gain is not geometry-specific.** +0.1090 on `channels-700`,
  positive in **12 of 12** seeds — comparable to the +0.1258 on the anchor.

So: *attention buys the same amount everywhere tested; the anchor geometry is
simply the one where that is enough to clear the bar.* That is a stronger and
more honest statement than either "it clears 0.80" or "it is an artefact of the
downsample", and neither was available before this wave.

### S-3 — depth does not rescue width; it makes it much worse

Wave 3 found the d32/**L1** gain at −0.0159 at h1024 and the paper carried that
forward as "scoped to h128". The measured d32/**L4** gain at h1024 is **−0.1618**,
an order of magnitude worse, positive in only 1 of 12 seeds.

The spread is the tell: h512/L4 ranges 0.7933–0.8564, while h1024/L4 ranges
**0.3746–0.7412**. Every validity gate passes — no non-finite events, all 20
classes predicted — so this is not divergence. It is a configuration that
sometimes trains and sometimes does not.

**Named next step, and it is not registered here:** whether h1024/L4 is
optimisation instability (rather than a capacity or credit effect) is a new
question with its own controls. It must not be answered by picking a learning rate
after seeing these numbers.

### S-4 — the gain survives the other literature contract, and nearly clears the bar

0.8225 with **10 of 12** seeds ≥ 0.80 on `published-10ms`, gain **+0.1491**,
positive 12/12. Combined with S-1, the binding scope limit is **geometry, not
contract**.

### S-5 — the registered mechanistic prediction failed

This is the one that matters most, and it is the reason S-5 was registered.

The temporal-order reading predicted that the gain would **shrink** when the
utterance is framed into 72 timesteps instead of 358 — a fifth as many positions
for a `t × t` attention matrix to order. The registered bar was ≤ +0.1058.

**The gain grew: +0.1491 at t=72 versus +0.1258 at t=358.**

Per the prereg's named outcomes, the consequence is fixed in advance: *the
mechanism claim is incomplete; the paper reports the shuffle result **without**
the resolution story.* The 12/12 shuffle inversion still shows temporal order
**matters**. It does not show order is the **whole** mechanism, and S-5 is now
positive evidence that it is not.

#### A confound in S-5's design, stated rather than buried

S-5 compares *gains* across two contracts with different baselines, so it mixes
the effect with headroom. The `ff+fixed` control falls from 0.7062 to 0.6734 under
coarser framing while the attention arm barely moves (0.8320 → 0.8225). A gain
computed against a lower baseline is larger for that reason alone.

**That confound was in the registered design and it is not a reason to discount
the failure** — the prediction was directional and the direction reversed. But it
means "the gain grew" should not be read as "attention likes coarse time".

#### The post-hoc observation, explicitly labelled

The attention arm is nearly **invariant** to temporal framing resolution
(0.8320 vs 0.8225) while the plain arm loses 0.033. That is a different and
arguably more interesting temporal story than the registered one. **It is
post-hoc.** It came from looking at these numbers, it has no registered
threshold, and it may not be claimed until it is registered and tested on its own
terms.

### S-6 — the depth ladder holds at convergence

L1 → L2 → L4 at e400: 0.7483 → 0.7897 → 0.8320, steps +0.0414 and +0.0423. Wave 2
measured +0.0357 and +0.0299 at e100, so depth helps slightly **more** at
convergence, not less. The depth claim is not budget-scoped.

## 4. The scope paragraph the paper can now write

Every clause below is measured at the configuration reported, not extrapolated
from d32/L1:

> The attention read-out raises SHD accuracy by **+0.10 to +0.15** across two
> geometries and two contracts, positive in **12 of 12 seeds in every case**. It
> clears the 0.80 gate on `adjacent-sum-5` at h128 (**0.8320**, 12/12) and comes
> within one seed of it on `published-10ms` (**0.8225**, 10/12), but **not** on
> the full 700-channel geometry (**0.7864**, 6/12). The gain falls monotonically
> with hidden width — **+0.1258 / +0.0876 / −0.1618** at h128 / h512 / h1024 —
> and at h1024 the configuration becomes unstable across seeds. Depth in the
> read-out helps monotonically at convergence (**+0.041**, **+0.042** for
> L1→L2→L4).

## 5. What this wave may not claim

Unchanged from prereg §7, and now with one addition:

- **Not calibration.** Criterion 5 is unmet; no compute changes that.
- **No comparison to macOS-recorded numbers.** Cross-machine Gate F FAILs on all
  four instances, as expected and as recorded in the manifest.
- **Not optimality.** d32 remains the only dimension tested at convergence.
- **No resolution-based mechanism claim.** S-5 removed it. What survives is the
  shuffle result — and that is measured at d32/**L1**, not at the headline
  configuration, which is what wave 9 (`M-1`, registered 2026-08-20) now exists
  to settle.

## 6. Cost

~$23 of AWS credit, ~19 h wall, 0 cells lost.
