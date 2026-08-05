# Campaign claim freeze

Numbers only from this campaign's on-disk notes. Do not invent.  
Updated 2026-07-24: EventProp FAIL packed; v131 matched-only honesty; track-b hash demotion; SHD/mac appendix labels.

| Claim object | Hash / ID | Verdict | Local / primary | Gap LCB |
|---|---|---|---:|---:|
| Matched broadcast (v4) | `c1-match-5dc6822e71229e9e` | **FAIL** | 0.5000 | 0.0000 |
| Matched DFA (v5) | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | 0.6894 |
| Matched RL reinforce_fb (v12) | `c1-rl-42eddc9c801308e9` | **PASS** | 0.9200 | 0.6846 |
| Online Learned `B_i` (v130) | `track-b-rescue` *(schedule ID; not `c1-*-<hex>`)* | **PASS (matched)** | 1.0000 | 0.9988 |
| Discrete EventProp H2H (v28) | `c1-eventprop-5bb083d5e88d0ad2` | **FAIL** | 0.5000 | 0.0000 |
| Live ReinforceFeedback (v13) | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 |
| RFB × epoch (v14) | `c1-714c115e14a3eeed` | **FAIL** | 0.4838 | −0.0100 |
| Structured B (v15) | `c1-493ddd56f8714fb6` | **FAIL** | **0.7262** (floor ✓) | 0.2567 |
| Structured × epoch (v16) | `c1-677df7f7cbe4f8ec` | **FAIL** | 0.5200 | 0.0844 |
| Structured × capacity (v17) | `c1-983ee5303c00b147` | **FAIL** | **0.6825** (floor ✓) | **0.3127** |
| Eligibility × REINFORCE (v18) | `c1-c7d2c86a2b1927f6` | **FAIL** | **0.7125** (floor ✓) | 0.2351 |
| Structured B × teach (v19) | `c1-dfab4a7ec19f17c2` | **FAIL** | **0.6700** (floor ✓) | 0.2238 |
| Live DFA (v20) | `c1-4db53e645405fae0` | **FAIL** | 0.7325 | 0.2601 |
| Soft-WTA×SFB (v21) | `c1-f975db8fb3e5d569` | **FAIL** | 0.5025 | 0.0406 |
| Match 4×ep (v22) | `c1-match-b46b23549b37d90a` | **FAIL** | 0.5000 | 0.0000 |
| Finite-θ SFB (v23) | `c1-4bbaf4b24c2d1da2` | **FAIL** | 0.6638 | 0.2370 |
| Continuous B (v24) | `c1-840f820b7c07b512` | **FAIL** | 0.6437 | 0.1380 |
| Capacity sensitivity | `c1-d38d7644d8afc84b` | **FAIL** | 0.6775 (≥0.65 floor) | 0.0000 |
| Temporal-PC sensitivity | `c1-a49deeaedb495a09` | **FAIL** | 0.5263 | (gap mean 0.0947) |
| Trial isolation | `c1-8ec031907a3426d0` | **FAIL** | 0.5188 | (gap mean 0.2109) |
| Canonical C1 / G2 | `c1-118207fbc3eaba53` | **FAIL** | 0.4912 | −0.0048 |
| True e-prop (methods) | `c1x-eprop-true-true-surrogate-eprop-0e2aeb90d68ac5f9` | note | 0.7125 | — |

## Honesty footnotes (v130 / v131)

| ID | Status |
|---|---|
| v130 `track-b-rescue` | Matched online-FB PASS on dense-LIF; **demoted** from config-hash class (schedule/experiment name only). |
| v131 `live-transfer-rescue` | **Matched-only** schedule contrast (misnamed). **Not** live Engine / k-WTA. **Not** camera-ready live-transfer PASS. Live transfer = v13–v24 FAIL. |
| EventProp | Discrete H2H **FAIL**; ≠ continuous Wunderlich–Pehle. |

## Appendix science (non-MUST; protocol labels — do not mix)

| Family | Protocol label | Hash / note | Chance / task | Paper status |
|---|---|---|---|---|
| SHD calibration overnight | **p27** C1-SHD-CAL | `c1-shd-cal-eb3cb5d93417a638` (h128); `c1-shd-cal-bafa6835d8de7eb8` (h256) | 20-way; chance **0.05**; capped 2000/500; e-prop ceiling | Appendix only; ≠ G2; **do not remassage** |
| SHD full-corpus + SuperSpike | **p29** C1-SHD-FULL | `c1-shd-full-2c93117075740ed0` (full); smoke `c1-shd-full-a9542a730cb22c74` | 20-way; chance **0.05**; official 8156/2264; SuperSpike BPTT ceiling | Appendix; ≠ G2; ≠ p27 e-prop; ≠ proto-135 |
| SHD scientific sweep | **proto-135** | schedule ID `shd-scientific-sweep` | **5-class**; chance **0.20** | Exploratory; do **not** mix with p27 / p29 |
| Mac-probe size science | overnight `c1-mac-probe-*` | e.g. H1–H3 hashes in [`runs/2026-07-24-overnight-scale/OVERNIGHT_NOTE.md`](runs/2026-07-24-overnight-scale/OVERNIGHT_NOTE.md) | syn-matched / fan sweeps | Appendix; ≠ G2; ≠ Foundation Micro |
| Exploratory bins | proto 132–134 | deep-snn / EI / multi-channel | various | Exploratory PASS language demoted; not claim-axis MUST |

## NumPy deep (this campaign)

| Exp | Finding |
|---|---|
| `xor_thresh` | broadcast 0.5008 / DFA 0.8267 / grad 0.7733 — **locality flip** |
| `depth_locality` mid | broadcast 0.8158 / DFA 0.8250 / rl_fb 0.8033 — depth help ≠ locality flip |

## Locked reading

1. **Primary publishable:** broadcast three-factor fails matched dense-LIF gate.
2. DFA / REINFORCE×`B_i` / online learned-`B_i` (matched) clear matched gate; live k-WTA transfer fails (v13–v24). Structured `B` (v15), structured×capacity (v17), eligibility×REINFORCE (v18), and structured×teach (v19) clear the accuracy floor but not gap LCB > 0.5.
3. Discrete EventProp-style H2H **FAIL**s (`c1-eventprop-5bb083d5e88d0ad2`); disclose discrete ≠ continuous EventProp.
4. Capacity alone raises local above 0.65 but gap vs dense/grad stays closed — descriptive footnote only.
5. Temporal-PC and isolation do not rescue G2.
6. Canonical v2 kill-gate replay FAIL — integrity intact.
7. Gap-close suite (v14–v19) + break-it (v20–v24): all still FAIL G2.
8. Do **not** claim biology, Assembly Calculus PASS, impossibility, or v131 live rescue.

## Stop decision

Gap-close experimental package **closed** (v14–v19 / v20–v24). Camera-ready paper package honesty-synced:

- [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) — cite-every-number (incl. EventProp FAIL)
- [`PAPER_DRAFT.md`](PAPER_DRAFT.md) — prose draft (§3.1 aligned with §4.2)
- [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) — figures (**artwork present:** figM/fig1/fig3/GA in camp `figures/`)
- [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) — includes EventProp FAIL + v131 honesty
- [`REPRO_ARTIFACT_CHECKLIST.md`](REPRO_ARTIFACT_CHECKLIST.md) — fig boxes checked when camp artwork exists
- [`VENUE_FORMATTING.md`](VENUE_FORMATTING.md) · [`references.bib`](references.bib) — venue skeleton

No remassage of P4/P5/P9, frozen G2, or v13–v24.
