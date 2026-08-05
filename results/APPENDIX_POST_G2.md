# Post-G2 appendix harvest (does **not** reopen Gate G2)

**Banner:** Every row below is post-G2 / exploratory / supporting. Canonical kill-gate `c1-118207fbc3eaba53` and matched primary claims are unchanged.  
**Camp:** [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/)  
**Axes:** [`CLAIM_AXIS.md`](CLAIM_AXIS.md) · Closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) D17–D20

---

## D17 — Continual forgetting (C2)

```
claim_axis: Novel-CS (post-G2)
object_under_test: Class-incremental forgetting under local three-factor vs replay baseline
may_claim: Under c2-* hash, local forgetting exceeds replay baseline (G3 FAIL)
must_not_claim: Reopening G2; biology; product continual-learning PASS
```

| Item | Value | Source |
|---|---|---|
| Hash | `c2-c45f08841f2f9df9` | [`c2_g3.md`](c2_g3.md) |
| Verdict | **DONE-FAIL** (G3) | mean forget local 0.8948 vs baseline 0.2725 |

---

## D18 — Credit depth D* (C3)

```
claim_axis: Novel-CS (post-G2)
object_under_test: Terminal ±1 three-factor depth sweep vs oracle teacher-forced reference
may_claim: Measured D*(local)=3 vs D*(oracle)=8 under c3-* 
must_not_claim: Reopening G2; proof of cortical hierarchy depth
```

| Item | Value | Source |
|---|---|---|
| Hash | `c3-445aa8de7761d4f4` | [`c3_credit_depth.md`](c3_credit_depth.md) |
| Verdict | **MEASURED** | D* local **3** |

Also: production C3 v2 / BPTT footnotes in [`c3_v2_production.md`](c3_v2_production.md), [`c3_bptt.md`](c3_bptt.md).

---

## D19 — Area scaling R1 / R2

```
claim_axis: Novel-CS (post-G2)
object_under_test: Multi-area composition (R1) and capability vs #areas (R2 / G4)
may_claim: R1 ADDITIVE (composed ≤ additive); R2 G4 NO-GO (degrade curve)
must_not_claim: Scaling to 10⁴–10⁶ areas; reopening G2
```

| Arm | Hash | Verdict | Source |
|---|---|---|---|
| R1 composition | `r1-5d30383e334b9cbe` | **ADDITIVE** | [`r1_composition.md`](r1_composition.md) |
| R2 scaling | `r2-afafa0fa6f43e3fc` | **NO-GO** (degrade) | [`r2_scaling.md`](r2_scaling.md) |

---

## D20 — Soft→hard collapse (hybrid temperature)

```
claim_axis: Novel-CS mechanism
object_under_test: Soft-to-hard winner-temperature ladder on hybrid teacher path
may_claim: Soft→hard collapse motivates live soft-WTA probe (v21); H0 remains HYBRID_NO_GO
must_not_claim: Reopening hybrid H0 as G2 rescue; temperature grid as paper primary
```

| Item | Value | Source |
|---|---|---|
| Hybrid H0 | `binn-hybrid-h0-v3-caedeec1a47475a5` | **HYBRID_NO_GO** (teacher D*=2) — hard-loop note |
| Mechanism | Soft / finite-T / hard ladder in `binn-hybrid-lab` | Motives **v21** only |

---

## Motif honesty checklist (Brain-motif-under-test)

| Motif | Present in engine | Exercised on C1 crux | Outcome |
|---|---|---|---|
| Event LIF + dendrites | yes | dendrites unused as credit study | **DISCLOSED** (D16) |
| Three-factor STDP algebra | yes | yes (canonical / matched) | Motif exercised; gate FAIL where claimed |
| Natural hidden spiking | yes under `c1-spike*` | scientific **INVALID_HARNESS** | **DONE-INVALID** (D13) |
| Assembly Calculus `project` | yes under `c1-project*` | G2 **FAIL** | **DONE-FAIL** (D14) |
| Neuromod / REINFORCE×B | yes | matched PASS; live FAIL | Novel-CS transfer |
| θ=∞ mute | yes on canonical / RFB / SFB | ablated in **v23** | Integrity |

**Must not claim:** cortex, digital brain, “biology failed.”

---

## Deep / NumPy supporting (locality)

| Exp | Finding | Axis | Source |
|---|---|---|---|
| `xor_thresh` | broadcast ~0.50 / DFA ~0.83 — locality flip | Novel-CS supporting | [`deep_xor_thresh.json`](deep_xor_thresh.json) |
| `depth_locality` mid | broadcast also solves — **not** a locality flip | Integrity | [`deep_depth_locality_mid.json`](deep_depth_locality_mid.json) |

---

## Efficiency / work-per-accuracy (D21)

Harvest from C1 budget rows / [`u20_efficiency.md`](u20_efficiency.md) into `PAPER_METRICS_FULL` Table F. **No new G5 claim** unless separately gated.
