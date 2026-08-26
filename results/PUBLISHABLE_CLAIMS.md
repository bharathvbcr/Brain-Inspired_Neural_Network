# BINN — publishable claim freeze

> ### SUPERSEDED IN PART — 2026-08-25 matched-architecture re-run
>
> Every matched-architecture number in this document was produced before the
> 2026-08-22 silent-initialisation repair, on a forward pass that emitted **zero
> spikes at any seed**, and none of them has been regenerated here. The re-run
> is [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)
> and [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 carries the current figures.
>
> **The lead negative survives** — broadcast ±1 three-factor is at chance on both
> forward graphs. **Three contrasts do not**: the discrete EventProp-style
> spike-adjoint goes 0.5000 FAIL to 0.9450 / 0.8900 PASS, and the two RL
> broadcast contrasts go 0.5250 to 0.9100 and 0.5113 to 0.7962. The gradient
> ceiling goes 0.8887-0.9150 to **1.0000**, so every `gap_closed` here divides by
> a different denominator than the instrument now produces.
>
> The `c1-*` config hashes cited below are **retired**: `MATCHED_INPUT_SCALE` was
> not part of them, so each named two experiments. They no longer resolve, by
> design. The retirement table is in the re-run document section 8.
>
> Rows concerning the SHD attention campaign, the live-transfer package and the
> XOR task are unaffected: none of them runs on the matched dense-LIF forward.

> **CITATION WARNING (added 2026-08-07).** This document cites the
> `track-b-rescue` **v130** row (`1.0000`, gap LCB `0.9988`, PASS matched) as a
> matched-substrate result. That report is stale: the source is **v131**, and the
> 130→131 bump is precisely the clamp-and-separation-gate fix for the defect the
> row exhibits. Under current code the arm **cannot be reported as PASS**. Do not
> cite it until `track-b-rescue` has been re-run. The DFA
> (`c1-dfa-c8c4fe0899908b84`) and RL (`c1-rl-42eddc9c801308e9`) matched PASSes are
> **not** affected by this defect; they ran through the clamped `runner.rs` path.
> See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` and
> `TODO_2026-08-07_OPEN_WORK.md` §1.
> **RESOLVED 2026-08-19.** The re-run landed. At v131 the arm reports
> **INVALID_HARNESS**, not PASS: the ceiling-inverted warning fires on 3 of 20
> learned-FB seeds and the code refuses to emit a PASS while it is present.
> The v130 PASS is **withdrawn**. See
> `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` and
> `track_b_results_v131.md`.



Authority: Rust sources + on-disk result notes. Do not widen beyond this sheet for research packaging.

**Thesis (locked):**

> **Broadcast ±1 three-factor** credit (surrogate eligibility × ±1 reward) fails a preregistered accuracy/gap bar on a matched dense-LIF coincidence forward, **on both forward graphs, at n=20** — and it is the **only** rule tested that does. Every other rule now clears the gate against a reference at 1.0000, so the task isolates one failure from a field it can no longer rank; disclose that, and do not read the passes as an ordering. Lead FAIL is ±1 three-factor, not “any broadcast.” Locality evidence is XOR. The event-driven C1 / live k-WTA path fails its operationalized gate under broadcast and under live REINFORCE transfer (including structured-`B` / capacity / eligibility gap-close arms)—report as a substrate/pipeline transfer negative with integrity disclosed, not as a biological or Assembly-Calculus result.

---

## Claim strength ladder

| Rank | Claim | Strength | Object under test |
|---:|---|---|---|
| **1 (lead)** | On an identical dense-LIF forward, **broadcast ±1 three-factor** (surrogate eligibility × ±1) does not clear the matched-arch gate vs SuperSpike BPTT | **Strongest / cleanest** | Rule topology only (±1 broadcast) |
| **1b (contrast)** | On the matched forward, graded DFA, REINFORCE×frozen-`B_i`, broadcast-graded **and a discrete spike-adjoint** all clear the gate — against a reference at 1.0000, so each pass reduces to “above 0.75” | **Weak — the task saturates; no ordering may be claimed** | Credit richness / locality |
| **1c (neuromorphic)** | On SHD, a **time-axis attention readout** reaches **0.8320** at d32/L4 (12/12 ≥ 0.80, gain **+0.1258** over `ff+fixed`); **temporal order is the mechanism** (+0.1337 shuffle drop, 12/12 seeds; 96% of benefit order-dependent) | **Strong / measured (n=12)** | Temporal attention readout on LIF |
| **2 (secondary)** | Under the coded C1 engine loop, local/dense three-factor miss `g2_min_*` while multi-epoch SurrogateLifReference on the same frozen splits succeeds | **Moderate — caveated** | Operationalized pipeline |
| **2b (transfer)** | Live C1 opt-in `ReinforceFeedback` (v13) and gap-close suite (v14–v19) fail G2; structured `B` / capacity clear accuracy floor but not gap LCB > 0.5 | **Moderate — new hashes** | Live k-WTA transfer |
| **3** | Exact-forward credit arms (`c1x-*`) fail G2-style bars on their separate hashes | Supporting methods notes | Exact-forward / hybrid credit |
| **4** | Checklist-closure protocols (`c1-spike-*`, `c1-spike-s-*`, `c1-project-*`, `c1x-eprop-true-*`, `c3-bptt-*`) | Supporting integrity / methods | New hash families only |
| **5** | 1-layer XOR / `xor_thresh`: broadcast ~chance, DFA solves (locality flip); mid-init depth does **not** show the same flip | Supporting task evidence | NumPy deep suite |
| — | Biology, neuromorphic HW, impossibility, brain model, live transfer from matched PASS, “any broadcast fails,” continuous textbook EventProp equivalence, depth collapse (v134 withdrawn), online learned FB v130 (v131 withdrawn), **the discrete spike-adjoint FAIL (withdrawn 2026-08-25: 0.5000 → 0.9450/0.8900 PASS)**, **the two RL broadcast contrasts as chance results (0.5250 → 0.9100, 0.5113 → 0.7962)**, **any ranking among the passing matched arms** | **Do not claim** | — |

---

## 1. Primary claim — matched-arch broadcast ±1 three-factor insufficiency

**May claim:** Holding the dense-LIF forward fixed and swapping only the update rule — **broadcast ±1 three-factor** vs SuperSpike BPTT — the local arm fails the preregistered matched gate on **both** the feed-forward and recurrent graphs (0.5000 and 0.5100, n=20, against a reference at 1.0000). Mechanism label: **broadcast ±1 three-factor**, not “spiking failed,” and not a bare “broadcast credit topology” ban that would misread the 0.9975 graded contrast.

**Must disclose alongside it:** every other rule tested clears this gate, and the reference sits at 1.0000 at the canonical budget, so `gap_closed` reduces each pass to “the arm scored above 0.75”. The claim that survives is *one rule fails a task the rest saturate* — which is narrower and better evidenced than a graded richness ordering, and the ordering **may not** be claimed.

**Caveat (A6 ceiling health), now the binding limitation:** on the archived instrument the canonical 80-epoch schedule undertrained the reference (0.8963 / 0.9013 at e80, climbing to 1.0000 by e640; `RESULT_2026-08-19_A6_CEILING_HEALTH.md`), so the e80 comparison read as *learning speed*. On the repaired instrument the reference reaches **1.0000 at e80 itself**, so there is no budget at which this task separates the arms and no ceiling comparison on it survives. Every matched claim in this sheet is bounded by that.

**Code:**

- `binn/binn-learn/src/matched_local_baseline.rs` — matched local + gradient arms; GC1-exempt baseline
- `binn/binn-lab/src/match_config.rs` — protocol v4, `c1-match-*` hashes; asserts distinct from `c1-118207fbc3eaba53`
- `binn/binn-lab/src/runner_match.rs` — harness; refuses to reopen protocol-v2
- `binn/binn-lab/experiments/c1.rs` — `--matched-arch` entry

**On-disk anchors (do not invent):**

- Scientific hash: `c1-match-5dc6822e71229e9e` — [`c1_match.md`](c1_match.md), [`MATCHED_ARCH_CONTROL.md`](MATCHED_ARCH_CONTROL.md)
- Verdict **FAIL**; matched-local mean **0.5000**; matched-gradient mean **0.8963**; gap LCB **0.0000**
- Quick hash (pilot only): `c1-match-85e9548f0615b85a`

**Does not reopen** `c1-118207fbc3eaba53`.

---

## 1b. Contrast claims — DFA / REINFORCE clear the matched gate

**May claim:** On the same matched dense-LIF coincidence forward, replacing **broadcast ±1 three-factor** with (a) graded DFA or (b) directional REINFORCE × frozen per-neuron `B_i` clears the preregistered gate at the 80-epoch budget.

| Arm | Hash | Verdict | Primary mean | Gap LCB | Note |
|---|---|---|---:|---:|---|
| Matched DFA (v5) | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | 0.6894 | [`c1_dfa.md`](c1_dfa.md) |
| Matched RL `rl_reinforce_fb` (v12) | `c1-rl-42eddc9c801308e9` | **PASS** | 0.9200 | 0.6846 | [`c1_rl.md`](c1_rl.md) |
| Matched RL Online Learned `B_i` (v130) | `track-b-rescue` | **WITHDRAWN** | — | — | [`RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`](RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md); v131 reports `INVALID_HARNESS` |
| Discrete EventProp-style H2H (v28) | `c1-eventprop-5bb083d5e88d0ad2` | **FAIL** | 0.5000 | 0.0000 | [`c1_eventprop.md`](c1_eventprop.md); ≠ continuous Wunderlich–Pehle |
| Matched RL graded primary (v11) | `c1-rl-ef504db58916720d` | **FAIL** | 0.5900 | 0.0182 | archived; do not retune |

**Honesty (required):** On the DFA schedule, **broadcast-graded** contrast is also high (**0.9863**). Do **not** claim coincidence alone proves credit locality is required. The lead FAIL is specifically **broadcast ±1 three-factor** (±1 × eligibility). Locality-flip evidence is **1-layer XOR** ([`deep_xor_thresh.json`](deep_xor_thresh.json)). Mechanism figure: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M.

**Falsifier:** A matched ±1 three-factor arm that clears accuracy floor and gap LCB under the same dense-LIF forward / Gate G2 thresholds overturns the lead claim.

**Non-claim:** do **not** claim that matched PASS transfers to live k-WTA C1 (see §2b). Do **not** widen the lead FAIL to “any broadcast rule.” Do **not** cite v131 / `live-transfer-rescue` as live-engine PASS (matched-only binary).

---

## 1c. SHD attention read-out & temporal order mechanism (Waves 1–9)

**May claim:**
1. **Headline accuracy:** On the SHD benchmark, a time-axis attention readout (`ff+fixed+attn`) reaches **0.8320** at `d32/L4` at `e400` (**12/12 seeds ≥ 0.80**, budget-stable |e400−e200|=0.0002, gain **+0.1258** over `ff+fixed` 0.7062; [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md)).
2. **Mechanism (temporal order):** Bin-shuffling causes a **+0.1337** accuracy collapse on the attention arm in **12 of 12 seeds** (seed range +0.0967 to +0.1568), versus **+0.0128** for the plain arm (10× factor). Under shuffling, the attention advantage disappears (+0.1258 → +0.0050; **96% of readout benefit is order-dependent**; [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md)).
3. **Sample efficiency:** Attention readout reaches 98.1% of e400 accuracy by 10 epochs (bracketed at `(5, 10]`; [`SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md`](SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md)).

**Scope limitations (must disclose):**
- Measured at **h128 / `published-2ms` / `adjacent-sum-5`**.
- Gain is positive across geometries (+0.1090 on `channels-700`, +0.1491 on `published-10ms`), but 0.80 clearance is geometry-specific (0.7864 on `channels-700`).
- Width scaling inverts by h1024 (−0.1618 at h1024/L4).
- Temporal-resolution mechanism (S-5) is **refuted** (fewer timesteps increased gain).
- **Not calibration:** Criterion 5 (Python mirror) remains unmet.

---

## 1d. Discrete EventProp H2H (matched; FAIL)

**May claim:** Under protocol v28 hash `c1-eventprop-5bb083d5e88d0ad2`, a **discrete** EventProp-style spike-triggered adjoint on the matched dense-LIF forward **FAIL**s the preregistered gate (mean **0.5000**, gap LCB **0.0000**) while SuperSpike BPTT reaches **0.9150**.

**Must disclose:** discrete hard spike-gate adjoint ≠ continuous Wunderlich–Pehle (2021) hybrid EventProp. See [`c1_eventprop.md`](c1_eventprop.md).

**Does not reopen** `c1-118207fbc3eaba53` / frozen match/dfa/rl hashes.

## 2. Secondary claim — engine C1 as operationalized pipeline negative

**May claim (with integrity appendix):** Under protocol-v2 hash `c1-118207fbc3eaba53`, Gate G2 FAIL — local/dense three-factor miss gap LCB and accuracy floor while gradient/eligibility references succeed on the same splits. Package as **this coded pipeline** fails its operationalized gate — not “local learning impossible.”

**Anchors:** local **0.4912** / gap LCB **−0.0048** — [`c1_g2.md`](c1_g2.md), [`U-NEG_protocol_v2.md`](U-NEG_protocol_v2.md).

### Integrity caveats (must disclose)

| ID | Finding | Status after hardening |
|---|---|---|
| **H1** | `ThreeFactor.last_spike` never cleared across trials (v2) | **Fixed under `c1-iso` / v5**; **still true on canonical v2** |
| **H2** | Incomplete membrane reset vs C3 v2 (v2) | **Fixed under `c1-iso` / v5**; **still true on canonical v2** |
| **θ=∞ mute** | Hidden thresholds set to `f32::INFINITY` during integrate | **Removed under `c1-spike-*` / v6 and `c1-spike-s-*` / v9**; **still true on canonical v2 / iso** |
| **`project` unused** | Assembly Calculus `project` not on C1 crux | **Wired under `c1-project-*` / v7** (scientific **FAIL**); **still unused on canonical v2** |
| **Hybrid e-prop naming** | Exact-forward “e-prop/DFA” ≠ textbook e-prop | **True σ′ e-prop under `c1x-eprop-true-*`**; frozen hybrid remains hybrid |

Isolation / capacity / temporal-PC scientific: all **FAIL** G2 under new hashes ([`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](CAMPAIGN_2026-07-23_CLAIM_FREEZE.md)).

---

## 2b. Live REINFORCE transfer + gap-close (v13–v19)

**May claim:** The neuromodulator family that PASSes matched dense-LIF (`rl_reinforce_fb`) **FAIL**s Gate G2 on live muted-θ / k-WTA C1 under an honest opt-in mapping. Separable gap-close hypotheses (epochs, structured `B`, capacity, eligibility timing, restored target teach) were run under **new hashes**; none clear gap LCB > 0.5.

| Protocol | Hash | Verdict | Local | Gap LCB |
|---|---|---|---:|---:|
| v13 live RFB | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 |
| v14 epoch | `c1-714c115e14a3eeed` | **FAIL** | 0.4838 | −0.0100 |
| v15 structured B | `c1-493ddd56f8714fb6` | **FAIL** | **0.7262** | 0.2567 |
| v16 structured×epoch | `c1-677df7f7cbe4f8ec` | **FAIL** | 0.5200 | 0.0844 |
| v17 structured×capacity | `c1-983ee5303c00b147` | **FAIL** | **0.6825** | **0.3127** |
| v18 elig×REINFORCE | `c1-c7d2c86a2b1927f6` | **FAIL** | **0.7125** | 0.2351 |
| v19 structured×teach | `c1-dfab4a7ec19f17c2` | **FAIL** | **0.6700** | 0.2238 |

**Reading to cite:** structured `B` is the accuracy lever (floor cleared); capacity×structured is the best gap LCB still short of G2; epochs under structured B regress; eligibility co-design and restored target teach do not beat v15.

**Sources:** [`MATCHED_ARCH_LIVE_REINFORCE.md`](MATCHED_ARCH_LIVE_REINFORCE.md), [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md).

**Stop:** no remassage of v13–v19 knobs without a new hypothesis + hash.

---

## 2c. Break-it wave (v20–v24) — no deferrals

```
claim_axis: see row
object_under_test: live / matched differentials D4–D8
may_claim: Named protocol FAIL under fixed G2 bars (dual gap reported on live)
must_not_claim: Transfer PASS; remassage v13–v19; biology
```

| Protocol | Hash | Axis | Verdict | Local | Gate LCB | Chance LCB |
|---|---|---|---|---:|---:|---:|
| v20 live DFA | `c1-4db53e645405fae0` | Novel-CS | **FAIL** | 0.7325 | 0.2601 | 0.3321 |
| v21 soft-WTA×SFB | `c1-f975db8fb3e5d569` | Novel-CS | **FAIL** | 0.5025 | 0.0406 | 0.0122 |
| v22 match 4×ep | `c1-match-b46b23549b37d90a` | Integrity | **FAIL** | 0.5000 | 0.0000 | — |
| v23 finite-θ SFB | `c1-4bbaf4b24c2d1da2` | Integrity | **FAIL** | 0.6638 | 0.2370 | 0.2370 |
| v24 continuous B | `c1-840f820b7c07b512` | Novel-CS | **FAIL** | 0.6437 | 0.1380 | 0.1163 |

**Reading:** Matched DFA does not transfer (v20 clears floor, misses gap). Soft winners under SFB regress. Undertraining does not rescue matched three-factor. Mute-off under SFB still fails gap. Continuous B does not beat v15. Closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md). Camp: [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/).

---

## 3. Supporting notes (optional paper material)

- Exact-forward credit suite: [`credit_assignment.md`](credit_assignment.md) — separate `c1x-*` hashes; all listed arms FAIL.
- Spiking-path true DFA (P4): `c1x-dfa-spike-true-dfa-a911e793e590b0ed` **FAIL** (gap LCB 0.0733) — one honest attempt; stop.
- True e-prop (`c1x-eprop-true-*`): true-surrogate mean **0.7125** — methods contrast only.
- NumPy `xor_thresh`: broadcast **0.501** / DFA **0.827** / grad **0.773** — **locality flip** (cite 1-layer XOR, not depth).
- NumPy `depth_locality` mid: broadcast **0.816** / DFA **0.825** / rl_fb **0.803** — depth help ≠ locality flip (P7).
- Engine isolation / spike / project: [`RESEARCH_HARDENING.md`](RESEARCH_HARDENING.md).

---

## Explicit non-claims

Do **not** claim:

1. **Biology / cortex** — motif-level substrate only.
2. **Assembly Calculus PASS** — `project` wired under `c1-project-*` and **FAIL**s G2.
3. **Natural-spiking G2 verdict** — scientific `c1-spike-*` / `c1-spike-s-*` are **INVALID_HARNESS** (PC below floor).
4. **True e-prop rescues broadcast insufficiency** — supporting methods note only.
5. **Neuromorphic hardware** — software harness only.
6. **Impossibility in principle** — scoped operationalized negatives only.
7. **Reopening protocol-v2 by threshold massage** — G2 thresholds and `c1-118207fbc3eaba53` stay frozen.
8. **Digital brain / brain equivalence**.
9. **Live-engine rescue from matched DFA / RL PASS** — transfer FAIL (v13–v24). Do **not** cite v131 / `live-transfer-rescue` as live PASS (matched-only).
10. **Structured `B` / capacity / eligibility / soft-WTA / continuous-B as G2 PASS** — floor cleared ≠ gate cleared.
11. **Coincidence-only proof that locality is required** — broadcast-graded also learns coincidence on the DFA schedule (**0.9863**); cite XOR for locality flip. Do not replace “broadcast ±1 three-factor” with bare “broadcast credit topology” in a way that would ban graded broadcast.
12. **Undertraining as the matched ±1 three-factor FAIL cause** — v22 4× epochs still at chance.
13. **Continuous / textbook EventProp equivalence** — discrete H2H `c1-eventprop-5bb083d5e88d0ad2` is **FAIL** and ≠ Wunderlich–Pehle continuous adjoint; do not claim EventProp “absent” or EventProp rescue.
14. **Appendix G3 / G4 / hybrid H0 as reopening G2** — post-G2 harvest only; G4 NO-GO does not license remassaging the kill-gate.
15. **Hybrid T=2.0 collapse ≡ live v21 (T=1)** — separate protocols; do not equate.
16. **Overnight SHD p27 / mac-probe ≡ proto-135 SHD sweep ≡ p29 full SuperSpike** — distinct protocols (20-way chance 0.05 capped e-prop vs 5-class chance 0.20 vs full-corpus SuperSpike BPTT `c1-shd-full-*`); do not mix.
17. **Online learned feedback alignment v130 PASS** — withdrawn under v131 (`INVALID_HARNESS` due to ceiling-inverted warnings; `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`).
18. **Depth collapse / deep SNN scaling** — withdrawn under v134 (`INVALID_HARNESS`; all depth-matched gradient ceilings at chance; `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`).
19. **`shd-scientific-sweep` claims** — withdrawn (synthetic 24-channel / 16-timestep data; never loaded SHD; `DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`).
20. **Temporal-resolution mechanism for SHD attention** — refuted by S-5 (fewer timesteps increased rather than decreased gain; only temporal *order* is supported via shuffle inversion M-1/M-2).
21. **SHD instrument calibration for attention** — criterion 5 (Python mirror) unmet; uncalibrated.

---

## Object-under-test checklist (honesty ledger)

| Item | Status |
|---|---|
| Event LIF + dendrites exist | yes |
| STDP eligibility + three-factor algebra live | yes |
| Natural hidden spiking during C1 integrate | **yes under `c1-spike-*` / `c1-spike-s-*`** (INVALID_HARNESS on PC); **no** under canonical v2 |
| Assembly Calculus `project` on C1 | **yes under `c1-project-*` (G2 FAIL)**; **no** under canonical v2 |
| Trial-isolated `last_spike` | **yes under `c1-iso` / spike/project**; **no** under canonical v2 |
| Matched DFA / RL reinforce_fb PASS | **yes** (`c1-dfa-*`, `c1-rl-*` v12, subject to A6 schedule context) |
| Matched RL online-learned FB PASS | **no** (v130 withdrawn; v131 INVALID_HARNESS) |
| Live RFB / gap-close G2 PASS | **no** (v13–v19 FAIL) |
| Live DFA / soft-WTA / finth / cont-B G2 PASS | **no** (v20–v24 FAIL) |
| v131 live-transfer-rescue = live Engine | **no** (matched-only; misnamed) |
| Discrete EventProp matched H2H PASS | **no** (`c1-eventprop-5bb083d5e88d0ad2` FAIL) |
| Matched undertrain rescues 3F | **no** (v22 FAIL @ chance) |
| SHD attention read-out headline | **yes** (d32/L4 reaches 0.8320 @ e400; 12/12 seeds ≥ 0.80) |
| SHD attention temporal-order mechanism | **yes** (M-1/M-2: +0.1337 shuffle drop; 12/12 seeds) |
| Brain model | **no** |

New-protocol rows are closed with **new hashes**; they do not reinterpret v2 FAIL.
