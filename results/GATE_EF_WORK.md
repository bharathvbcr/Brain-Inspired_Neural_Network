# Gate E / Gate F — work required before the rec+alif run

Companion to `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF.md` §2.
Harness: `scripts/gates_ef.py`.

```
claim_axis: Integrity
object_under_test: Whether the instrument is fit to measure a new architecture.
may_claim: Nothing. Gates E and F are preconditions, not results.
must_not_claim: Any accuracy, ceiling, or architecture statement.
```

**Neither gate is a result. Failing either blocks the campaign.**

---

## Gate F — regression (runnable today, blocking)

With recurrence and adaptation disabled, the instrument must reproduce the
already-completed ff+fixed cells **bit-identically**. Two things depend on this:
comparing new arms against the 216 completed rust cells, and the standing check
on the 2026-08-02 kernel vectorisation.

```
.venv-shd/bin/python scripts/gates_ef.py gate-f \
    --cell python__published-10ms__adjacent-sum-5__h128__e20__s5170001
.venv-shd/bin/python scripts/gates_ef.py gate-f --all-python
```

Compares every scientific field — `accuracy`, `mean_loss`, `mean_gradient_norm`,
`mean_update_rms`, `mean_firing_rate`, `classes_predicted`,
`majority_prediction`, `silent_fraction`, `saturated_fraction`,
`non_finite_events`, `scientific_status`, `n_train`, `n_test` — by `repr()`
against the recorded cell. `wall_secs` is excluded; timing is not a result.
Writes `results/shd_instrument_v4/gate-f/report.json`, exits non-zero on any
mismatch.

**Run this first.** It is the outstanding verification on the kernel patch, and
it must run on the Mac — the full 8156/2264 split cannot be framed in a 3 GB
sandbox (it was OOM-killed during development).

> **2026-08-03 — this gate only ever covered the python arm.** `gate_f_cell`
> raises *"gate-f regresses the python arm; the rust arm is checked by gate-e"*,
> and Gate E is unimplemented (G7 below), so the rust-arm obligation in
> `AMENDMENT_2026-08-02` §5.3 had nothing behind it — the 216 cells that *are*
> the scientific result were the ones with no runnable regression.
>
> `scripts/gate_f_rust.py` now implements it and passes on the cells run so far;
> see `AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md` §3-4.
>
> ```
> .venv-shd/bin/python scripts/gate_f_rust.py --cheapest 6
> .venv-shd/bin/python scripts/gate_f_rust.py --all
> ```
>
> Note also that `shd_matched_arms.rs` — the module every rust cell actually
> executes — is **outside** `SOURCE_PATHS`, so kernel changes there do not move
> the provenance fingerprint. See that amendment §5.

## Gate E — four-arm cross-backend parity (blocked, needs implementation)

Every arm must clear the existing registered tolerances before any matrix cell
runs: forward `<= 1e-6`, gradient `<= 1e-4`, update `<= 1e-5`. `gates_ef.py
gate-e` currently exits with the exact `shd-instrument parity --arm` invocation
it needs and the list of missing fixtures.

---

## Gaps, in dependency order

### G1 — Gate F unverified  ·  BLOCKING  ·  no code needed

The kernel vectorisation is verified at fixture level only: patched `model.py`
against the frozen `rust-parity.json` gives forward `1.095e-08`, gradient
`2.035e-06`, update `1.810e-07` — all passing, and identical to the pre-patch
values. Cell-level bit-reproduction has **not** been demonstrated. Run Gate F.

### G2 — coordinate cache memory  ·  CLOSED

`_frame_matrix` caches per-sample coordinates as `uint16 / uint16 / float32`
(8 bytes/event) rather than `int64 / int64 / float32` (20 bytes/event), with a
range assertion. Both axes stay under 700 for every registered contract, so the
narrowing is exact. Parity re-verified after the change: unchanged to four
significant figures.

### G3 — framing is now the bottleneck  ·  NOT BLOCKING  ·  do before scaling

With the kernel 2.7–12× faster, `frame_events` dominates. It loops in python
over every event with `np.float32` boxing — roughly 10⁸ interpreter iterations
per cell across the 8156/2264 split — and it is redone **per cell**, though the
result depends only on `(contract, geometry)`: 12 distinct framings recomputed
across 432 cells.

Two independent fixes, neither touching numerics:
1. Vectorise `frame_events` over the event arrays instead of looping per event.
2. Cache framed splits per `(contract, geometry)` across cells in one process.

Both must be validated by Gate F, since `FramedSample.fingerprint()` feeds the
cell manifests.

### G4 — weight file format  ·  BLOCKING  ·  the subtle one

`WEIGHT_MAGIC = b"SHDWGT1\0"` has no slot for `W_rec` or adaptation parameters.
Adding them changes the format, and the existing `initialization/*.weights`
files must keep loading **bit-identically** or Gate F fails by construction.

Required: a `SHDWGT2` magic carrying `w_rec` and the adaptation block, with
`SHDWGT1` retained as a reader that yields `w_rec = 0`, `adaptive = false`. The
ff+fixed arm must continue to consume the existing files unchanged. Do not
migrate the existing files.

### G5 — instrument extension, rust  ·  BLOCKING

- `binn-learn/src/shd_matched.rs` — `MatchedWeights` grows `w_rec`
  (`hidden × hidden`, zero diagonal enforced at construction and after every
  update) and the adaptation parameters. `loss_and_gradient` grows the recurrent
  term and the adaptation trace in both directions. `MatchedGradient`,
  `zeros_like`, `add_assign`, `scale`, `l2_norm`, `all_finite` all extend.
- `binn-lab/experiments/shd_instrument.rs` — `--arm {ff+fixed,ff+alif,rec+fixed,rec+alif}`
  on `parity`, `init` and `train-cell`. Default must remain `ff+fixed`.

BPTT, not e-prop: `∂a/∂s` is differentiated, not truncated. `binn-learn/src/shd_alif.rs`
has the forward algebra and the `τ_a = 20`, `β_a = 0.18` defaults, but only
`Dfa`, `EpropCeiling`, `BroadcastPm1` rules — there is no BPTT path to reuse.

### G6 — instrument extension, python  ·  BLOCKING

Same surface in `scripts/shd_calibration/model.py`, kept structurally parallel
so parity failures are debuggable. The recurrent term cannot be collapsed into
the pre-loop matmul the way input drive was — it depends on the previous step's
spikes, so it stays inside the sequential loop as a `hidden × hidden` gemv. This
is the source of the ≈6.5× cost multiplier in the prereg budget.

### G7 — per-arm parity fixtures  ·  BLOCKING  ·  after G4–G6

Four fixtures, `rust-parity-{ff-fixed,ff-alif,rec-fixed,rec-alif}.json`, plus
per-arm weights. `rust-parity-ff-fixed.json` must match the existing
`rust-parity.json` exactly — that is Gate F at fixture level.

Then extend `runner.preflight` to iterate arms, so `preflight` is the single
entry point rather than a second parallel harness.

### G8 — `W_rec` initialization scale  ·  BLOCKING  ·  pilot, then freeze

Not fixed by the calibration lineage and the most likely cause of a
`SATURATED` `rec+alif` — which, per prereg §5, means H1 has not been tested at
all. Set once by pilot to keep the ff+fixed firing-rate band (0.108–0.331),
freeze, and record the value. The pilot is not reportable.

### G9 — authorization  ·  BLOCKING  ·  not a code gap

`SHD_INSTRUMENT_STATUS.md` blocks new architecture campaigns while the
instrument is `UNCALIBRATED`, and the rust entry points enforce it. Calibration
must close first. Because the verdict is already determined to be `FAIL`, an
authorization amendment is required, and it must state on what basis a
`FAIL`-verdict instrument is fit to measure a new architecture. See prereg §0.

---

## Order of execution

| # | Step | Blocking | Needs Mac |
|---|---|---|---|
| 1 | Gate F on completed python cells (G1) | yes | yes |
| 2 | `preflight` + `replay-smoke` on patched kernel | yes | yes |
| 3 | Finish the 137 remaining python cells | yes | yes |
| 4 | Calibration verdict + authorization amendment (G9) | yes | no |
| 5 | Framing optimisation, re-check Gate F (G3) | no | yes |
| 6 | Weight format `SHDWGT2` (G4) | yes | no |
| 7 | Rust + python arm implementations (G5, G6) | yes | no |
| 8 | Per-arm fixtures, `preflight --arm` (G7) | yes | yes |
| 9 | `W_rec` scale pilot (G8) | yes | yes |
| 10 | Timing pilot — replaces prereg §9 estimates | yes | yes |
| 11 | Confirmatory run | — | yes |

Steps 6 and 7 are the bulk of the engineering and need no Mac, so they can
proceed in parallel with steps 1–3.

## Housekeeping

`scripts/gates_ef.py` is canonical. A duplicate was created at
`scripts/shd_calibration/gates_ef.py` during development and has a stale import;
delete it — the sandbox lacks permission to.
