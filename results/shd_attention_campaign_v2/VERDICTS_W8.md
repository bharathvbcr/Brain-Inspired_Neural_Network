# Wave 8 — registered verdicts

Prereg: `PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md` §3. All 72 wave-8 cells present, plus 72 reused control cells from waves 1/3 and the registered run, same pinned binary `22d97c51ab02`.

**Validity gates: all 144 cells pass.**

## Measurements

| configuration | mean | min | max | seeds >= 0.80 |
|---|---:|---:|---:|---:|
| d32/L4 anchor h128 *(registered run)* | 0.8320 | 0.8083 | 0.8472 | 12/12 |
| d32/L2 anchor h128 | 0.7897 | 0.7562 | 0.8211 | 4/12 |
| d32/L1 anchor h128 *(wave 1)* | 0.7483 | 0.7279 | 0.7774 | 0/12 |
| `ff+fixed` anchor h128 *(wave 1)* | 0.7062 | 0.7005 | 0.7164 | 0/12 |
| d32/L4 `channels-700` | 0.7864 | 0.6882 | 0.8202 | 6/12 |
| `ff+fixed` `channels-700` *(wave 3)* | 0.6774 | 0.6718 | 0.6873 | 0/12 |
| d32/L4 h512 | 0.8233 | 0.7933 | 0.8564 | 10/12 |
| `ff+fixed` h512 *(wave 3)* | 0.7357 | 0.7301 | 0.7420 | 0/12 |
| d32/L4 h1024 | 0.5768 | 0.3746 | 0.7412 | 0/12 |
| `ff+fixed` h1024 *(wave 3)* | 0.7386 | 0.7328 | 0.7407 | 0/12 |
| d32/L4 `published-10ms` | 0.8225 | 0.7915 | 0.8476 | 10/12 |
| `ff+fixed` `published-10ms` | 0.6734 | 0.6639 | 0.6829 | 0/12 |

## Verdicts

**S-1** d32/L4 clears the gate on `channels-700`: mean **0.7864** (bar 0.80), **6/12** seeds >= 0.80 (bar 9). -> **NOT SUPPORTED**

**S-2** gain on `channels-700`: **+0.1090** (bar +0.05), positive in **12/12** seeds (bar 10). -> **SUPPORTED**

**S-3** gain at h1024 with L4: **-0.1618** (bar +0.05), positive in **1/12** seeds (bar 10). Wave 3 measured **-0.0159** at d32/L1. -> **NOT SUPPORTED**

**S-3b** *(reported, no threshold)* gain vs width at L4: h128 **+0.1258**, h512 **+0.0876**, h1024 **-0.1618**. Monotone decreasing: **True**

**S-4** gain on `published-10ms`: **+0.1491** (bar +0.05), positive in **12/12** seeds (bar 10). -> **SUPPORTED**

**S-5** *(mechanistic)* the gain shrinks with fewer timesteps: `published-10ms` (t=72) **+0.1491** vs `published-2ms` (t=358) **+0.1258**; bar is <= **+0.1058**. -> **NOT SUPPORTED**

**S-6** depth ladder monotone at e400: L1 **0.7483** -> L2 **0.7897** (+0.0414) -> L4 **0.8320** (+0.0423). -> **SUPPORTED**

