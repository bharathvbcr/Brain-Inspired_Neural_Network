# Paper scientific verify — 2026-07-23

> **SUPERSEDED IN PART — 2026-08-25 matched-architecture re-run, banner added
> 2026-08-29.** The three **matched** rows below are archived. Their hashes are
> **retired** and `from_hash` no longer resolves any of them —
> `MATCHED_INPUT_SCALE` was not mixed in, so each named two different
> experiments either side of the silent-initialisation repair — and their
> numbers are the pre-repair block, from a forward pass that emitted **zero
> spikes at any seed**
> ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).
> The current matched figures are in **Table A** of
> [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md).
>
> **What this document still establishes, and it is not nothing.** On
> 2026-07-23 six `--config-hash` replays reproduced their arms bit-stably. That
> is a claim about the replay machinery rather than about the arms, and it holds
> for all six. The three non-matched rows — canonical C1, live RFB v13,
> structured B v15 — run on the event-driven engine, are unaffected by the
> matched re-run, and their hashes still resolve.
>
> The banner is late. It was added when
> `scripts/test_published_hashes_resolve.py` was written and found this
> document publishing three retired hashes with pre-repair numbers, under the
> heading **"ALL MATCH freeze"**, with nothing anywhere to say so. It sits in
> `PAPER_SIDE` — the paper's own downstream artefacts, which
> `check_every_number.py` deliberately does not sweep — so no number check
> would ever have reached it.

Exact `--config-hash` replays of primary camera-ready arms.  
Camp: `results/runs/2026-07-23-paper-verify/`

**Result as recorded on 2026-07-23: ALL MATCH freeze** (bit-stable means /
verdicts / LCBs), against the freeze *as it stood that day*.

| Arm | Hash | Status | Verdict | Primary | Gap LCB | Replay log |
|---|---|---|---|---:|---:|---|
| Matched broadcast 3F | `c1-match-5dc6822e71229e9e` | **retired / pre-repair** | FAIL | 0.5000 | 0.0000 | `match_replay.log` |
| Matched DFA | `c1-dfa-c8c4fe0899908b84` | **retired / pre-repair** | PASS | 0.9387 | 0.6894 | `dfa_replay.log` |
| Matched RL reinforce_fb | `c1-rl-42eddc9c801308e9` | **retired / pre-repair** | PASS | 0.9200 | 0.6846 | `rl_replay.log` |
| Canonical C1 / G2 | `c1-118207fbc3eaba53` | current | **FAIL** | 0.4912 | −0.0048 | `g2_replay.log` |
| Live RFB v13 | `c1-660401d74db3c88d` | current | **FAIL** | 0.4900 | 0.0737 | `rfb_replay.log` |
| Structured B v15 | `c1-493ddd56f8714fb6` | current | **FAIL** | 0.7262 | 0.2567 | `sfb_replay.log` |

**Read the three retired rows as history only.** Broadcast 3F is the one whose
*verdict* survived the re-run — it is FAIL at 0.5000 ff / 0.5100 rec on the
repaired forward too — but the row above is not that measurement. DFA and RL
still PASS and read 0.9925 / 0.9875 and 0.9950 / 0.9812.

Notes also written under this camp as `c1_*_replay.md`.

## Code integrity (same day)

- Unit tests lock paper scientific hashes (`paper_scientific_hashes_are_frozen`, match/dfa/rl hash freezes).
- Gap-close markdown disclosures tested (`gap_close_protocols_render_discloses_and_freeze_hashes`).
- Structured `B` sign test after readout boost (`structured_feedback_signs_follow_readout_columns_after_boost`).
- v17 markdown now cites protocol-v2 non-reopen (disclosure fix).

Each matched suite's frozen-hash test now records its **retired** value beside
its current one, so the break is visible in the source rather than inferred;
`scripts/test_published_hashes_resolve.py` asserts those comments agree with the
hash each test actually freezes, and that no published `--config-hash` anywhere
in the record names a retired one.

Reproduce — the matched three at their **current** hashes, since the archived
three cannot be replayed by any binary in this repository:

```bash
cd binn
cargo test -p binn-lab --lib --locked paper_
cargo test -p binn-lab --lib --locked gap_close_protocols_render
cargo test -p binn-lab --lib --locked structured_feedback_signs
python3 scripts/test_published_hashes_resolve.py
# then the replays in REPRO_ARTIFACT_CHECKLIST.md §A–C
```
