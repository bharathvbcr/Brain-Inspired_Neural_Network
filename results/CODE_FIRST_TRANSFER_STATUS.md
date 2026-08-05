# Code-first transfer status index

**Authoritative code-integrity status — 2026-07-26**

This index does not rewrite historical result artifacts. It supplies the status
banner that must be read before using them.

| Artifact family | Status | Required interpretation |
|---|---|---|
| Canonical C1 and v13–v24 | Preserved | Existing hashes, thresholds, and verdicts are unchanged. |
| Historical SHD claims, including p27/p29/v142 | **SUPERSEDED BY v143 RAW-RATE SHORTCUT** | Official paired confirmation found input-only accuracy above both hidden comparators. Stop SHD activity tuning, feedback parity changes, and further SHD ablations. |
| Historical `deep-snn-scaling` / `MatchedDeepGradient` | **INVALID_HARNESS — ceiling is not shared-forward BPTT** | Preserve for history; do not use it as a gradient ceiling or depth result. |
| Protocol v143 SHD input-rate control | **PASS — raw-rate shortcut confirmed** | Capped input-only `0.2618` exceeded hidden `0.2224`; full input-only `0.4428` exceeded SuperSpike `0.4157`. Controls, class diversity, no-test-update, and replay gates passed. |
| Protocol v144 temporal calibration | **INVALID_TASK** | No candidate qualified. RFB remained `0.2467–0.2733`; BPTT was `0.9433–1.0000`; raw-rate and time-shuffled controls remained at chance. |
| Protocol v145 shared-forward depth run | **BLOCKED — v144 produced no freeze** | Never run this protocol after a mechanism change. Any successor requires a new version, hash, and seed family. |
| Protocol v146 Rust/NumPy transfer falsifier | **BLOCKED — v144 produced no freeze** | Quick micro-conformance remains historical `PILOT`; no scientific v146 execution occurred. Never reuse v146 after a mechanism change. |
| Protocol v147 temporal eligibility diagnostic | **FAIL — STOP CURRENT LEARNED-FEEDBACK DESIGN** | Corrected RFB remained at `0.2400` while BPTT reached `1.0000`. The mandatory overfit gate failed (`0.2500`, 2/4 classes), although gradients/steps were finite and replay exact. Do not create v148-v150 or run v145/v146; reassess the treatment mechanism. |
| Protocol v148 shortcut-accessibility contrast | **PASS — MULTICLASS LOCAL POSITIVE CONTROL PASSED; RATE-IMMUNE CONTRAST AT CHANCE** | Same multiclass local path reached `1.0000` when class was accessible from channel counts and `0.2500` when counts were byte-identical; BPTT was `1.0000` on both. This is a fresh explicit contrast, not a v145/v146 revival or a transfer result. |

The resumable `scripts/run_code_transfer_campaign.sh` runner executes v143 and
v144, extends v143 to 20 seeds only when required, and starts v145/v146 only
when v144 writes the registered freeze artifact. Its execution summary keeps
process status separate from scientific verdicts.

The historical `0.51` matched-minus-live estimate is retired. It combines
incompatible harnesses, widths, exposures, seeds, and validity states and is
not an acceptance target.

Scientific status words are reserved as follows:

- `PILOT`: smoke/calibration planning evidence only.
- `INVALID_HARNESS` or `INVALID_TASK`: a validity gate failed; accuracy is not
  interpretable.
- `FAIL`: a valid frozen protocol missed its scientific gate.
- `PASS`: every mechanical, validity, and scientific gate passed.

No Phase 1 transfer attribution, 16-rung lattice, Brian2, Lava, or rescue-arm
work is authorized by the current implementation status.

The v147 stop remains authoritative for the failed learned-feedback mechanism.
V148 was minted only after an explicit override for a different, paired
positive-control question. It does not authorize v145/v146, a successor
calibration, or a transfer claim.

Tooling note: the DevCouncil checkout CLI regenerated the map and a targeted
sync reached healthy generation 6, but the subsequent compatibility export
still omitted the new untracked v147 binary. Source and Cargo wiring are
authoritative; this map omission remains a tooling limitation.
