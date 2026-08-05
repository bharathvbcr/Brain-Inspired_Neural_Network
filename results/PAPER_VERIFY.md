# Paper scientific verify — 2026-07-23

Exact `--config-hash` replays of primary camera-ready arms.  
Camp: `results/runs/2026-07-23-paper-verify/`

**Result: ALL MATCH freeze** (bit-stable means / verdicts / LCBs).

| Arm | Hash | Verdict | Primary | Gap LCB | Replay log |
|---|---|---|---:|---:|---|
| Matched broadcast 3F | `c1-match-5dc6822e71229e9e` | **FAIL** | 0.5000 | 0.0000 | `match_replay.log` |
| Matched DFA | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | 0.6894 | `dfa_replay.log` |
| Matched RL reinforce_fb | `c1-rl-42eddc9c801308e9` | **PASS** | 0.9200 | 0.6846 | `rl_replay.log` |
| Canonical C1 / G2 | `c1-118207fbc3eaba53` | **FAIL** | 0.4912 | −0.0048 | `g2_replay.log` |
| Live RFB v13 | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 | `rfb_replay.log` |
| Structured B v15 | `c1-493ddd56f8714fb6` | **FAIL** | 0.7262 | 0.2567 | `sfb_replay.log` |

Notes also written under this camp as `c1_*_replay.md`.

## Code integrity (same day)

- Unit tests lock paper scientific hashes (`paper_scientific_hashes_are_frozen`, match/dfa/rl hash freezes).
- Gap-close markdown disclosures tested (`gap_close_protocols_render_discloses_and_freeze_hashes`).
- Structured `B` sign test after readout boost (`structured_feedback_signs_follow_readout_columns_after_boost`).
- v17 markdown now cites protocol-v2 non-reopen (disclosure fix).

Reproduce:

```bash
cd binn
cargo test -p binn-lab --lib --locked paper_
cargo test -p binn-lab --lib --locked gap_close_protocols_render
cargo test -p binn-lab --lib --locked structured_feedback_signs
# then the six hash replays in REPRO_ARTIFACT_CHECKLIST.md §A–C
```
