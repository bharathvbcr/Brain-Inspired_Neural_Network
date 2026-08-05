# Amendment — provenance freeze may be discharged by proven bit-identity

**Date:** 2026-08-03
**Amends:** `results/shd_instrument_v4/manifest.json` freeze semantics, as
enforced by `scripts/shd_calibration/runner.py::ensure_manifest`.
**Registered before the change**, per `PREREG_2026-07-25_SHD_ARCH_ABLATION`
§preamble and `HANDOFF_2026-08-02.md` §6 pitfall 1.

**Requires human sign-off before it takes effect.** It changes authorization
semantics, so §6 makes it opt-in and inert until a human enables it.

---

## claim_axis

```
axis: instrument-provenance
claim: A frozen provenance manifest may be advanced to a new source/binary state
  when, and only when, that new state is proven to reproduce recorded cells
  bit-for-bit.
may_claim: That the cells recorded under the old state remain valid evidence
  under the new state, because their bytes were reproduced by it.
must_not_claim: That any scientific threshold moved. None did. The accuracy gate
  remains `>= 0.80` and no recorded accuracy changes.
```

## 1. What the freeze currently does

`ensure_manifest()` hashes the calibration sources into `source_fingerprint`, and
separately hashes six input files including `rust_binary` and `cargo_lock`. Any
mismatch raises and every downstream path — `run_cell`, `recover_references` —
hard-fails.

## 2. Why that is the wrong mechanism, demonstrated rather than argued

As of today the instrument is in exactly the state the freeze was built to catch,
and it is provably a false positive:

| | frozen | current |
|---|---|---|
| `source_fingerprint` | `64923d64655d86ee…` | `4b85606d11fb3d52…` |
| `rust_binary` sha256 | `4c8b3414a7a1dca1…` | `10df998c491ca5ab…` |
| 13 recorded cells reproduced | — | **bit-for-bit, 13/13** |

The source changed, the binary changed twice, and no recorded number moved. See
`AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md` and
`shd_instrument_v4/gate-f-rust/runs.jsonl`, which records the same 13-cell suite
passing under two distinct binary hashes.

The mechanism is wrong in **both** directions:

- **Too strict.** A byte change to source that provably does not change the
  computation blocks the entire campaign. Today that is a comment, a test, and a
  loop transposition whose output identity is verified on 13 cells.
- **Too weak.** Byte-identity of source does not imply identity of output. A
  different rustc, target-cpu, or LLVM version reproduces the fingerprint exactly
  and can still reassociate a float and flip a spike — which is the documented
  failure mode that broke the original Gate F
  (`HANDOFF_2026-08-02.md` §6 pitfall 2). The freeze would not catch it.

The freeze is a hash of the *inputs* standing in for a property of the *outputs*.
Now that the output property can be measured directly, the proxy should defer to
the measurement.

## 3. The forced choice this creates

With no discharge path, an agent facing a tripped freeze has two options, and
both are bad:

1. Never run another cell.
2. Delete `manifest.json` and re-freeze — which silently destroys the record of
   what produced the 296 completed cells.

Option 2 is what an agent under task pressure will actually do, and it is
unrecoverable. A gate whose only escape is to destroy the evidence is a gate that
will eventually be escaped that way. That is the strongest argument for this
amendment: it replaces an undocumented destructive workaround with a recorded,
evidence-bearing transition.

## 4. The amended rule

On fingerprint or binary mismatch, `ensure_manifest()` no longer raises
immediately. It raises **unless** all of the following hold:

1. A Gate F report exists at `shd-instrument`'s current sha256, with
   `status == "PASS"` and `failures == 0`.
2. That report covers at least `PROVENANCE_MIN_GATE_F_CELLS` (= 8) recorded
   cells, spanning **more than one geometry and more than one hidden width** —
   a narrow suite is not evidence, per the parity-fixture lesson in
   `HANDOFF_2026-08-02.md` §6 pitfall 2.
3. The transition is appended to `manifest.provenance_chain`, recording both
   states, the Gate F report sha256, and the cell ids covered.

Failing any of these, the original hard error stands.

## 5. What this does not do

- It does not move any scientific threshold. Accuracy remains `>= 0.80`.
- It does not revalidate cells. Cells recorded under a prior state stay attached
  to that state through `core_manifest_sha256` in their cell manifests.
- It does not weaken `matrix_authorized`, whose seven gates are untouched. The
  matrix stays blocked on `historical_reference` and `clean_reference`.
- It does not apply to data. `train_events` / `test_events` / `train_h5` /
  `test_h5` mismatches remain unconditionally fatal: bit-identity of a training
  kernel says nothing about whether the inputs changed.

## 6. Default-off

`PROVENANCE_DISCHARGE_ENABLED` defaults to **False**. Until a human sets it, the
behaviour is exactly as before and this document is a proposal. The
implementation and its tests land now so the decision is reviewable against
working code rather than against a description of it.
