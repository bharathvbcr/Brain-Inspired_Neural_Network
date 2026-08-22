# The calibration gap is provenance, not accuracy — and criterion 4 is not about a BINN arm

**Found:** 2026-08-21, checking a question I had posed wrongly.
**Corrects:** `PAPER_STATUS_2026-08-20.md` §3, which framed criterion 4 as a
choice about whether `ff+fixed+attn` counts as reference architecture, and
described the Python mirror as the single blocker "compute cannot buy". **Both
were wrong.**

---

## 1. Criterion 4 does not refer to any BINN arm

`SHD_INSTRUMENT_STATUS.md` criterion 4 reads *"three clean reference seeds, each
at least `0.80`"*. That is mechanised as the `clean_reference` gate, and
`scripts/shd_calibration/reference.py:17` shows what a "reference seed" is:

```python
REFERENCE_URL = "https://github.com/Thvnvtos/SNN-delays.git"
REFERENCE_COMMIT = "d169b4e3049a3d5bff56c84a8b2f0c4e835aafda"
```

It clones a **third-party PyTorch baseline** at a pinned commit and runs it. It
executes no line of the Rust instrument. `ff+fixed`, `ff+fixed+attn`, and every
other BINN arm are irrelevant to it — the question of which one "counts" does not
arise, under any reading.

## 2. Both reference gates are already satisfied on the numbers

All six artifacts exist in `results/shd_instrument_v4/references/`:

| mode | seeds | recorded | requirement | meets it? |
|---|---|---|---|---|
| clean | 5170001–3 | **0.9390 / 0.9368 / 0.9371** | each ≥ `CALIBRATION_FLOOR` 0.80 | **yes** |
| historical | 5170001–3 | **0.9447 / 0.9573 / 0.9475** (mean 0.9498) | within 0.05 of `PUBLISHED_ACCURACY` 0.951, exposure-tainted | **yes** |

**The accuracies are not the problem and never were.** The gates are false for a
provenance reason.

## 3. The provenance reason, and why it is not fixable by code

`valid_reference_payload` (`runner.py:834`) requires the artifact's recorded
`source_fingerprint` to equal `relevant_source_fingerprint()`. The artifacts
record `64923d64…`, computed over the **broad** `SOURCE_PATHS` set — which
includes `Cargo.lock` and the Rust kernel — at freeze time on 2026-07-27. Every
kernel edit since has moved it.

**This was already diagnosed, attempted, and withdrawn.**
`AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` narrowed the fingerprint to
the three files that can actually affect a reference run, recorded a falsifiable
expectation in §5, ran it, and **the expectation failed**: narrowing produces
`50c0fe76…`, which is not a fresh value of the recorded quantity but *the output
of a different function over different inputs*. A historical broad fingerprint
cannot be retroactively compared to a narrow one. Per its own §5 the code was
reverted.

`reference_source_fingerprint()` still exists at `runner.py:183` with **zero call
sites** — the withdrawn attempt's residue, correctly left unwired.

Its §6 names the only three routes, and rules out one of them:

1. **Accept the artifacts on content evidence** — all six match on
   `result_sha256`, `log_sha256`, `manifest_sha256`, mode, seed and
   `mechanical_status`. This is a **human judgement call**, the same one pending
   on `AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md`.
2. **Re-run the three clean and three historical reference cells** under current
   source, regenerating manifests with a current fingerprint. Honest, and the
   only route that closes the gap without a judgement call.
3. **Migrate the fingerprints** — *not available*; it needs the source state at
   freeze time, which is not recoverable.

## 4. What this corrects

> **I stated on 2026-08-20 that the Python mirror was "the one thing between this
> and a calibrated instrument" and that "it was never a compute problem".**

That is wrong. The chain is:

```
criteria 3 + 4  (external PyTorch reference)  ──►  matrix_authorized  ──►  criterion 5 (matched Python/Rust config)
        ^ FALSE, and blocking                              ^ FALSE            ^ where the attention mirror lives
```

`matrix_authorized` requires all seven conjuncts (`runner.py:436-448`).
`historical_reference` and `clean_reference` are two of them and are false, so
**criterion 5 is not currently reachable and the attention mirror is not the
binding constraint.** Route 2 above *is* a compute job — a third-party PyTorch
model, six runs — and there are $7,500 of unused Lambda Cloud GPU credits.

## 5. One prerequisite discharged today

`recover-references` and `ensure_manifest` were additionally blocked by a
manifest-freeze violation: `rust_binary` and `cargo_lock` had both moved since
the freeze. The error names its own remedy — Gate F over ≥8 recorded cells
spanning two geometries and two widths.

Ran it: **10/10 bit-identical, PASS**, spanning `adjacent-sum-5` and
`channels-700`, h128 and h256, contracts `fixed-t100` and `published-10ms`.
Binary `4de5c977151e24c6…`. Report at
`results/shd_instrument_v4/gate-f-rust/report.json`. The manifest guard no longer
fires; what remains is the reference fingerprint in §3.

## 6. What was deliberately not done

**The gates were not flipped.** Setting `clean_reference` / `historical_reference`
to true is route 1 — a human decision, explicitly pending, and exactly the class
of action the standing rules forbid an agent from taking on its own ("never move
a threshold to make a gate pass"). The numbers passing is not authorisation.

**`reference_source_fingerprint()` was not wired in.** Its own amendment was
withdrawn after its registered expectation failed, and re-applying a withdrawn
change because it looks right is the pattern this record exists to prevent.

## 7. Scope

- **Verified:** every fingerprint value, gate state, accuracy, and call-site
  count above, from the files and by running the harness.
- **Verified:** the Gate F PASS, this session, this machine.
- **Not verified:** that re-running the six reference cells succeeds. `dcls` is
  **not installed** in `.venv-shd-cal` (torch 2.13.0 and MPS are available), and
  the run has not been attempted.
- **Not claimed:** that the instrument should be calibrated. That is a decision
  about evidence, not a computation.
