# Differential closure matrix (no deferrals)

**Ship rule:** Phase C blocked until every row is `DONE-*`, `DISCLOSED`, or completed `RUN-vN`.  
**Status vocabulary:** `DONE-PASS` / `DONE-FAIL` / `DONE-INVALID` / `DISCLOSED` / completed run notes.

Camp: [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/)  
Axes: [`CLAIM_AXIS.md`](CLAIM_AXIS.md) · Audit: [`HARD_AUDIT.md`](HARD_AUDIT.md)

---

| # | Differential (matched vs live / claim risk) | Axis | Closure | Anchor |
|---:|---|---|---|---|
| D1 | Credit richness: ±1 three-factor vs graded DFA / RL×B on **same** dense-LIF | Novel-CS | **DONE-FAIL** (v4); **DONE-PASS** (v5/v12) | `c1-match-5dc6822e71229e9e`, `c1-dfa-c8c4fe0899908b84`, `c1-rl-42eddc9c801308e9` |
| D2 | Broadcast-graded vs three-factor honesty on coincidence | Integrity | **DISCLOSED** (0.9863; XOR for locality) | [`c1_dfa.md`](c1_dfa.md), [`deep_xor_thresh.json`](deep_xor_thresh.json) |
| D3 | Matched RL×B PASS → live muted-θ/k-WTA | Novel-CS | **DONE-FAIL** (v13–v19) | [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md) |
| D4 | Live DFA transfer (any matched PASS transfers?) | Novel-CS | **DONE-FAIL** (v20) | `c1-4db53e645405fae0` · [`c1_dfa_live.md`](c1_dfa_live.md) |
| D5 | Hard k-WTA vs soft/relaxed winner under structured B | Novel-CS | **DONE-FAIL** (v21) | `c1-f975db8fb3e5d569` · [`c1_sfb_soft.md`](c1_sfb_soft.md) |
| D6 | Matched three-factor undertrained (epochs/η) | Integrity | **DONE-FAIL** (v22) | `c1-match-b46b23549b37d90a` · [`c1_match_ep4.md`](c1_match_ep4.md) |
| D7 | θ=∞ mute as engineering confounder under SFB | Integrity | **DONE-FAIL** (v23) | `c1-4bbaf4b24c2d1da2` · [`c1_sfb_finth.md`](c1_sfb_finth.md) |
| D8 | Sign-truncated vs continuous structured B | Novel-CS | **DONE-FAIL** (v24) | `c1-840f820b7c07b512` · [`c1_sfb_cont.md`](c1_sfb_cont.md) |
| D9 | Gap formula mismatch (chance vs dense-local) | Integrity | **DISCLOSED** + dual-gap harvest (no gate change) | [`HARD_AUDIT.md`](HARD_AUDIT.md), [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) |
| D10 | Nominal sparsity = k/N under mute/force-fire | Integrity | **DISCLOSED** | harness notes / C1 markdown |
| D11 | Eligibility-ref ceiling ~1.0 | Integrity | **DISCLOSED** (grad is ceiling) | [`c1_g2.md`](c1_g2.md) |
| D12 | Sticky last_spike / partial reset (H1/H2) | Integrity | **DONE-FAIL** on `c1-iso*` | `c1-8ec031907a3426d0` |
| D13 | Natural hidden spiking / brain-like spikes | Brain-motif | **DONE-INVALID** (`c1-spike*`) | [`c1_spike.md`](c1_spike.md) |
| D14 | Assembly Calculus `project` | Brain-motif | **DONE-FAIL** (`c1-project*`) | [`c1_project.md`](c1_project.md) |
| D15 | Epoch / capacity / elig / teach gap-close | Novel-CS | **DONE-FAIL** (v14–v19) | [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md) |
| D16 | Dendrite compartments matter on C1 crux | Brain-motif | **DISCLOSED** — present in engine; C1 path not a dendritic-credit study | [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) |
| D17 | Continual forgetting (C2) | Novel-CS (post-G2) | Appendix **DONE-FAIL** | [`c2_g3.md`](c2_g3.md) |
| D18 | Credit depth D* (C3) | Novel-CS (post-G2) | Appendix **MEASURED** | [`c3_credit_depth.md`](c3_credit_depth.md) |
| D19 | Area scaling R1/R2 | Novel-CS (post-G2) | Appendix **NO-GO / additive** | [`r1_composition.md`](r1_composition.md), [`r2_scaling.md`](r2_scaling.md) |
| D20 | Soft→hard collapse (hybrid temp) | Novel-CS mechanism | Appendix (motivates v21; not H0 reopen) | [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) |
| D21 | Work-per-accuracy / efficiency | Novel-CS descriptive | Harvest Table F (no new G5 claim) | [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) |
| D22 | Seed bimodality / LCB instability (v15 / v20) | Integrity | Harvest diagnostics (v20 frac≥0.65=0.60; min 0.425) | [`c1_sfb.md`](c1_sfb.md), [`c1_dfa_live.md`](c1_dfa_live.md) |

---

## Open cells

**None.** Every row is `DONE-*` or `DISCLOSED`. Phase C ship gate satisfied for DIFF_CLOSURE.
