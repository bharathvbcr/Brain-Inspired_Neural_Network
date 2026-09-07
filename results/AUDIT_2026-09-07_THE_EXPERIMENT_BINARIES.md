# Audit — the experiment binaries

**Date:** 2026-09-07
**Scope:** all 31 `binn-lab/experiments/*.rs`, **17,358 lines**
**Closes:** the largest entry in [`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md) §3 — "other ~20 `binn-lab/experiments/*` binaries, ~8000 lines, **NO**, several produced results in `results/` that nothing here re-verified"

**One defect found and fixed, at the owner rather than the call site.**

The register's own scope note said the 2026-08-03 sweep of the unaudited crates
"applied generic defect-class greps, not a semantic audit." This audit does the
same thing to the experiment binaries, with one difference: **every matcher is
calibrated against the register's own defects before its result is believed**,
and every hit is read rather than counted.

---

## 1. Method

Five classes, from §1 of the register. For each, a matcher, a calibration, and a
reading of every hit.

The calibration matters more than the matcher. A scan that reports zero is
worthless until it has been shown to fire, so each was run against a probe file
carrying the register's actual defect shapes before being run against the tree.

---

## 2. Results by class

| class | sites | outcome |
|---|---:|---|
| **A — a counter read by a predicate, never incremented** (defects #1–#4, four of ten) | **0** | matcher calibrated 2/2 against `non_finite_events` and `"trained": false`, and correctly ignored a counter that *is* written |
| **B — f32 sum-of-squares** (defect #6) | **0** | no accumulation of squares in f32 anywhere in the binaries |
| **C — silent clamping** (defect #5) | 5 | **1 defect** (§3); 3 are the same chain, 1 is a diagnostic probe, 1 is a test assertion |
| **D — panics on data paths** | 16 production, 84 test | no defect (§4) |
| **E — a literal emitted under a result key** (defect #2) | **0** | no `"field": true/false` literal in any emission |

---

## 3. The defect: a short cache reads short, and nothing said so

**`binn-data/src/shd_contract.rs::read_event_cache` clamped a request to the
file it found** — `max_samples.unwrap_or(n_file).min(n_file)` — and returned
quietly. This is defect #5's own shape, and the register records defect #5 as
**"FIXED at call site — loud error"**.

A call-site fix is a case fix. Every call site added after it inherited the
original behaviour, and the chain that matters is:

```
shd_depth_scaling.rs  probe()            asks load_splits for `samples` examples
  -> load_splits                          asks load_shd_dense_examples for n_train
    -> binn-lab/src/shd_dense.rs:120      checks only `raw.is_empty()`
      -> read_event_cache                 returns min(requested, n_file), quietly
```

`shd_instrument.rs:733-734`, the main train-cell path, has the same gap: it
checks `train_raw.is_empty() || test_raw.is_empty()` and never that the count
matches the request.

**What limited the damage, and what did not.** The cell is honest — `n_train` is
written from `train.len()`, the *realised* count, so no cell can claim to have
trained on more samples than it did, and `shd_instrument.rs:839` refuses an
order file whose length disagrees with the loaded set. The evidence that a cell
had trained on a short cache was therefore already on disk, in every cell.

**Nothing read it.** `scripts/cell_validity.py::PLAN_PINNED_FIELDS` compared
`arm`, `attn_readout`, `attn_dim`, `attn_layers`, `tau_m` and `val_speakers`
against the plan — and not `n_train`, which **every plan in the campaign
carries**. A cell that quietly trained on 5,000 of a requested 8,156 would have
passed every check in that file. That is the register's dominant class exactly:
not a wrong number, but a guard that could not fire over a number already
written down.

### Both halves fixed

1. **The owner.** `read_event_cache` now returns an error naming both counts
   when `max_samples` exceeds the cache, and the doc comment records why. All
   six call sites pass either `None` ("all of it") or a count they require
   exactly — a prefix to reach one indexed sample, or a registered
   `--max-train` / `--max-test` — so no caller wanted the clamp.
   Three tests pin it: the over-request errors and names both numbers, the
   exact/prefix/`None` requests still read, and one-past-the-end is refused.
   **Mutation-proven:** restoring `.min(n_file)` fails two of the three.

2. **The record.** `n_train` is now in `PLAN_PINNED_FIELDS`, so a cell that
   trained on a different number of samples than its plan registered is voided
   with the two counts in the message. Verified against the corpus: plans carry
   `n_train` 3,824 times at 8,156 and 24 times at 6,987, the cells agree on both,
   and **no archived cell is voided by the addition**.

The remaining three clamping sites are the same `load_splits` chain and are
closed by the owner fix. `shd_instrument.rs:1462` clamps a *diagnostic*
attention probe to the loaded test split, which is now provably the requested
split. `shd_depth_scaling.rs:1110` is an assertion bound inside a test — a false
positive of the matcher, recorded so the next run of it is not re-read.

---

## 4. Class D, split rather than counted

100 `unwrap`/`expect` across the binaries, which is the number a grep gives and
is not an answer. Split at each file's test module: **84 are in tests**, where a
panic is the assertion, and **16 are in production code**.

All 16 were read. They fall into three groups, none of them the dangerous shape:

- **Structural invariants** — `c1.rs:1159,1180` `.expect("config has ≥1 seed")`,
  `deep_snn_scaling.rs:333` `.expect("the ladder is non-empty")`,
  `shd_depth_scaling.rs:136` on a registered contract literal,
  `extensions.rs:384,388` on an edge index the line above computed.
- **File I/O** — `multi_area_scaling.rs:480,482`, `c1_enhanced.rs:319,321`.
  A `Result` would read better; a panic on an unwritable report is not a
  silent success.
- **Computed results** — `a6_ceiling_health.rs:402,410,433`,
  `efficiency.rs:400`, `extensions.rs:437`, `paper_figures.rs:16` (a CLI
  argument).

Every one **fails loud**. The class the register warns about is the opposite —
code that reports success while measuring nothing — and none of these does that.

---

## 5. What this does and does not establish

**Does:** the five defect classes that produced all ten defects in the register
are now scanned across every experiment binary, with calibrated matchers and
every hit read. One defect found, fixed at its owner, and pinned by tests that
fail without the fix.

**Does not:** this is a defect-class sweep, not a line-by-line semantic audit of
each binary's science. A protocol that computes the wrong quantity correctly
would not appear here. The register's §1 clustering is the reason to expect this
sweep to be worth more than its cost — five of ten defects were one class, and
that class is now scanned — but it is not a proof of correctness.

**A note on the register's §2 table.** It records `shd_instrument.rs` as swept at
**875 lines**. The file is now **2,726**. Roughly two thirds of it postdates its
own sweep, which is why it was re-scanned here rather than trusted.
