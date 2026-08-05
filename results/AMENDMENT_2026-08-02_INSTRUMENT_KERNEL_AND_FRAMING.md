# Amendment — instrument kernel and framing change (python arm)

**Registered:** 2026-08-02, after the change, before any affected cell is re-run.
**Amends:** `shd_instrument_v4` calibration matrix, python backend only.
**Supersedes nothing in the rust arm.**

```
claim_axis: Integrity
object_under_test: Two performance changes to the python reference instrument,
  and whether they preserve the results already recorded under it.
may_claim: The framing change is exact. The kernel change is not, and the 79
  completed python cells are superseded rather than comparable.
must_not_claim: That any accuracy, ceiling, or verdict changed as a result.
```

Per `PREREG_2026-07-25_SHD_ARCH_ABLATION` §preamble, amendments go in a new
timestamped file. This is that file.

---

## 1. What changed

Three edits to the python arm, all performance-motivated, none scientific.

**A. Kernel vectorisation** (`model.py::loss_and_gradient`). Input drive carries
no recurrence, so the per-event python loop collapses into one matmul
(`frame_matrix @ w_in.T`); `grad_w_in` collapses the same way on the backward
side (`du_all.T @ frame_matrix`). The LIF recurrence stays sequential.

**B. Framing vectorisation** (`data.py::frame_events`, gap G3). The scalar
per-event binning loop — roughly 10⁸ interpreter iterations per cell across the
8156/2264 split — is replaced by array operations plus a single `np.unique`.

**C. Coordinate cache narrowing** (`model.py::_frame_matrix`). Per-sample frame
coordinates cache as `uint16/uint16/float32` (8 bytes/event) rather than
`int64/int64/float32` (20). Both axes are under 700 for every registered
contract, with a range assertion.

## 2. Evidence

**B and C are exact.** Framing was checked against a verbatim copy of the scalar
implementation on 60 real SHD samples (median 7,594 events) across all twelve
registered `(contract, geometry)` pairs. Fingerprint, per-frame tuples, tuple
dtypes, and the `retained`/`clipped` counters match in every case. Since
`FramedSample.fingerprint()` feeds the cell manifests, the provenance chain is
undisturbed. Speedup 8.7×–30.3×.

**A is not exact, and this is the substance of the amendment.**

Fixture-level parity against `rust-parity.json` is *unchanged* by the edit —
forward `1.095e-08`, gradient `2.035e-06`, update `1.810e-07`, frame hash
matching — which is why the change was initially believed safe. Gate F, run on a
full cell, disproved that:

```
[REGRESSION] python__published-10ms__adjacent-sum-5__h128__e20__s5170001
    accuracy:            recorded 0.6258833922261484  observed 0.6303003533568905
    mean_loss:           recorded 1.468234165933105   observed 1.4693553476781585
    mean_gradient_norm:  recorded 0.4101857901035054  observed 0.40992961116695037
    mean_update_rms:     recorded 0.0007372276877804885 observed 0.0007364838534164202
    mean_firing_rate:    recorded 0.30771182237536976 observed 0.3100559565418928
    majority_prediction: recorded 0.10247349823321555 observed 0.09938162544169611
```

**Mechanism.** BLAS matmul associates float32 additions differently from
sequential `+=`. On the parity fixture (131 steps, sparse frames) the orders
coincided and the result was bit-identical; at real training density (~70 events
per frame) they do not. A one-ulp difference then crosses the **hard spike
threshold**, which is discontinuous, and a single flipped spike compounds
through 20 epochs × 8,156 samples via Adam.

The divergence is amplification, not error: `mean_gradient_norm` differs by
0.06%. Both are valid computations of the same mathematics; they differ only in
summation order. The recorded methodological failure is that a fixture with
atypically sparse frames was accepted as evidence of general bit-identity.

## 3. Consequences

- **`source_fingerprint` changes.** Every cell manifest written after this
  amendment carries a different fingerprint from the 79 completed python cells.
  **Addendum 2026-08-03:** this has a second consequence not recorded at the
  time. `valid_reference_payload` requires fingerprint equality, so the change
  also dropped `historical_reference` and `clean_reference` to false, and with
  them `matrix_authorized`; `harness_status` fell from `VALID` to
  `PENDING_PREREQUISITES`. The reference artifacts are intact — all six state
  hashes match and every accuracy and exposure check passes — but `SOURCE_PATHS`
  folds the instrument kernel into the fingerprint that guards a third-party
  reference whose result cannot depend on `model.py`. See
  `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md` §3.
- **The 79 completed python cells are superseded**, not comparable. They may be
  cited only as pre-amendment measurements, and may not be pooled with
  post-amendment cells.
- **The rust arm is unaffected.** `binn_data::frame_events` was not touched, and
  `shd_instrument.rs` now routes through `shd_matched_arms`, whose
  `ff_fixed_matches_shipped_reference` test asserts bit-identity against the
  shipped kernel. All 216 rust cells stand.
- **No scientific conclusion changes.** `matrix_verdict` was already determined
  to return `FAIL` from the completed rust arm alone
  (`SHD_BPTT_CEILING_NEGATIVE_RESULT.md`): all 216 rust cells fail
  `accuracy >= 0.80`, best 0.7151 ± 0.0032. A 0.0044 python difference sits far
  inside the 0.05 cross-backend band and far outside the 0.0849 shortfall to the
  gate. This amendment changes reproducibility, not findings.

## 4. Why the change was kept rather than reverted

Measured decomposition of the h128/e20/adjacent-sum-5 cell:

| | before G3 | after G3 |
|---|---:|---:|
| framing | 80 s | 5 s |
| training | 141 s | 141 s |
| framing share of cell | 36% | 4% |

Mapped onto the observed Mac timings (504 s original, 163 s after the kernel
change), framing is ~59 s of the cell. So:

- revert kernel, keep framing → **~449 s** (11% better than original)
- keep both → **~108 s** (4.7× better)

Essentially all the speed is in the kernel. Reverting for exactness would
discard a 4.7× improvement to preserve 79 cells that a `FAIL` verdict already
made non-load-bearing, and would leave the remaining python arm at ~13 days
instead of ~4.4 days for a full from-scratch rerun.

## 5. Obligations

1. **Re-run all 216 python cells** under the amended instrument. Partial reuse
   of pre-amendment cells is not permitted.
2. **Gate F is retired for the python arm** — with no comparable history there is
   nothing to regress against. It is replaced by the determinism check:
   `scripts/gates_ef.py determinism --cell <id>` runs a cell twice in **separate
   processes** and demands byte-identical output. That is the property
   `fresh_process_replay` already assumes, and it remains testable.
3. **Gate F stays in force for the rust arm**, where the 216 cells are the
   scientific result and bit-reproducibility is still meaningful.
4. **Re-run `preflight`** to refresh `parity_report.json` under the amended
   kernel before any new cell is written.
5. This amendment must be cited by any document reporting post-amendment python
   numbers.

## 6. What this amendment may not claim

- Not that the instrument became more accurate. It became faster; the numerical
  result moved by an amount consistent with float reassociation under a
  discontinuous nonlinearity, in an unknown direction.
- Not that pre- and post-amendment python cells can be compared.
- Not that the rust arm needs re-running.
- Not that the calibration verdict changed.

---

**Artifacts.**
`scripts/shd_calibration/model.py` — kernel, coordinate cache.
`scripts/shd_calibration/data.py` — framing.
`scripts/gates_ef.py` — Gate F, Gate E, determinism.
`binn-learn/src/shd_matched_arms.rs`, `binn-lab/experiments/shd_instrument.rs` — arm surface (G4/G5), unrelated to this amendment except that they preserve rust bit-identity.
`results/shd_instrument_v4/gate-f/report.json` — the failing Gate F run quoted in §2.
