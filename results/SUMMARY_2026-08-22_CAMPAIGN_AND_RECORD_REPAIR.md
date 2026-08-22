# Campaign and record repair, 2026-08-19 → 2026-08-22

Supersedes [`SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md`](SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md),
which covered waves 1–7 only. This is the whole of it: what was measured, what was
withdrawn, what was hardened, and what is still open.

**One-line state:** the attention read-out is a real, mechanistically explained,
tightly scoped result; four other results were withdrawn; the instrument now
cannot certify a broken comparison; and the paper is blocked on writing and on
one external reference reproduction, not on compute.

---

## 1. The result the paper leads with

**d32/L4 at e400 on the anchor** (h128, `published-2ms`, `adjacent-sum-5`):

| | value |
|---|---:|
| accuracy | **0.8320** |
| seeds ≥ 0.80 | **12 / 12** |
| gain over `ff+fixed` (0.7062) | **+0.1258** |
| budget stability, \|e400 − e200\| | 0.0002 |

Registered before the data existed
([`PREREG_2026-08-20_…_D32L4_AT_E400.md`](PREREG_2026-08-20_SHD_ATTENTION_D32L4_AT_E400.md)),
reported in [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md).

### The mechanism, measured at that exact configuration

| | intact | bin-shuffled |
|---|---:|---:|
| d32/L4 | 0.8320 | 0.6983 |
| `ff+fixed` | 0.7062 | 0.6934 |
| **gain** | **+0.1258** | **+0.0049** |

**96% of the advantage is contingent on temporal order**, 12/12 seeds, every
per-seed delta between +0.0967 and +0.1568. Shuffling costs the attention arm
**10×** what it costs the plain arm, so the order sensitivity lives in the
read-out, not the spiking layer
([`RESULT_2026-08-21_W9_…`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md)).

Wave 9 existed because the original shuffle control was measured at d32/**L1** and
carried onto a d32/**L4** headline — the same extrapolation wave 8 was built to
fix, applied to the claim rather than its scope.

### The scope, measured rather than assumed

| axis | finding |
|---|---|
| **width** | gain **inverts** by h1024 (−0.1618), and **depth does not rescue it** — that was wave 8's question, answered no |
| **geometry** | on the standard 700-channel input the effect survives (+0.1090) but **the 0.80 gate does not** (0.7864) |
| **depth** | monotone at convergence: L1 0.7483 → L2 0.7897 → L4 0.8320 |
| **dimension** | d32 is the *tested* configuration, not the chosen one (d64/L4 = 0.8441, **no verdict**) |

## 2. What was withdrawn

Four results left the record this week. **All four by measurement, not by
argument.**

| result | fate |
|---|---|
| `track-b-rescue` v130 PASS (1.0000, LCB 0.9988) | **withdrawn**; both arms `INVALID_HARNESS`. Six citing documents corrected |
| depth collapse (1.0000 → 0.4525) | **withdrawn** — every depth-matched ceiling is at **chance** |
| `shd-scientific-sweep` | **withdrawn** — the binary never loaded SHD |
| S-5, the resolution prediction | **NOT SUPPORTED** — the gain grew where it was predicted to shrink |

### The pattern underneath three of them

Three independent suites were checked and **three gradient references turned out
to be at or near chance** on tasks their own treatments solve:

| suite | treatment | reference | chance |
|---|---:|---:|---:|
| `deep-snn-scaling` v134 | 1.0000 | 0.4880 | 0.5000 |
| `shd-scientific-sweep` v135 | 1.0000 | 0.2140 | 0.2000 |
| C1 matched arms (A6) | 0.9387 | 0.9013 → **1.0000 by e640** | 0.5000 |

The first two do not learn. The third learns but was stopped early. **In all three
the treatment beat its own ceiling**, and the guard that should have caught it
either did not exist, was not ported, or was ported to a different binary. This is
a stronger methodological contribution than any single arm result in the package.

## 3. What was hardened

**`shd-scientific-sweep` never loaded SHD.** It fabricated 5 classes over 24
channels and 16 timesteps, each label firing only in its own three reserved
channels — linearly separable from spike counts, no temporal structure — and
printed `PASS — Learns multi-class SHD audio digits!`. Self-description corrected;
retire-vs-rename left to the maintainer
([`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`](DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md)).

**Ceiling health had five owners and one hole.** Every site tested
`ceiling < treatment`, which is silent when the reference never learned *and* the
treatment is below it. `deep-snn-scaling` v134's depth-4 row printed **`ok`** for a
constant predictor, and its 1-layer arm printed **PASS** against a reference at
chance. `guards::CeilingHealth` is now the single owner, testing against chance
first; all five sites migrated; a class guard fails the build on any bypass and
**refuses to pass vacuously**
([`HARDENING_2026-08-21_…`](HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md)).

v135 re-run: **every accuracy identical, every ceiling flagged, every arm
`INVALID_HARNESS`.** The numbers were never wrong; the instrument was doing the
misreading.

**Checks that cannot fail.** `scripts/find_weak_checks.py` scans for tests whose
assertions a degenerate result satisfies. It is **calibrated against the known
instance and refuses to report if it stops detecting it** — which caught its own
regex breaking mid-session. It found a real gap in `shd_alif`'s only architecture
coverage, where `AlifEval::defects()` already knew how to say it and nothing
called it.

**Reused-control integrity.** Waves 8–9 reuse 96 archived control cells. The
analysis now refuses to report unless the pinned binary matches and every reused
cell matches its recorded hash. **96/96 verified, 0 drift**, negative-tested by
corrupting one field.

**Parallelism.** 27 of 29 experiment binaries were single-threaded, and
`binn-learn` — where every paper number comes from — has no `rayon` dependency at
all. `deep_snn_scaling` parallelised: **9.1×**, byte-identical across ten thread
counts including primes
([`MEASUREMENT_2026-08-20_EXPERIMENT_PARALLELISM.md`](MEASUREMENT_2026-08-20_EXPERIMENT_PARALLELISM.md)).

## 4. Reproducibility — materially stronger than the record said

The Azure campaign stopped at 95/252 when its credit ran out. Its surviving cells
were the expensive h1024/h512 arms, which AWS had also run:

| | aarch64 (Graviton3) | x86-64 (EPYC, AVX-512) |
|---|---|---|
| binary | `22d97c51ab02…` | `666a73420a63…` |
| wall time, same cell | 12,220 s | 38,063 s |
| **36 cell pairs, 57,960 float values** | | **0 differing** |

Including complete 400-epoch loss and gradient trajectories.

- **Superseded:** *"results are not reproducible across machines."*
- **Supported:** *"reproducible across CPU architectures under the same libm; the
  only divergence is Apple libm vs glibc."*

Caveat carried in the finding: cell JSON serialises at ~9 decimals, so this is
agreement across 57,960 values at that precision, not proven bit-identity
([`FINDING_2026-08-22_…`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md)).

**Two near-misses worth recording.** The first check was on `accuracy`, which is
`correct/2264` and quantised — identical accuracy proves nothing, and the claim
only became a result once it rested on continuous fields. And the truncation was
first attributed to the harness watchdog because the predicted and observed times
matched to under a minute; **the operator supplied the actual cause**, and the
record was corrected.

## 5. Campaigns

| campaign | cells | outcome |
|---|---:|---|
| AWS waves 1–7 | 552 | complete, 24 diverged (all `rec+alif`), 0 voided |
| AWS wave 8 — scope | 72 | complete. S-1 ✗, S-2 ✓, S-3 ✗, S-4 ✓, S-5 ✗, S-6 ✓ |
| AWS wave 9 — mechanism | 24 | complete. **M-1 ✓, M-2 ✓**, M-3 descriptive |
| Azure — AZ8 | 95 / 252 | **stopped, credit exhausted, not relaunchable.** AZ8-2 ✗, AZ8-6 **VOIDED** (6/12 degenerate), rest no data |
| **AWS wave 10 — resolution ladder** | 72 | **in flight**, registered 2026-08-22 |

Total AWS spend across ten waves ≈ **$77**. One pinned binary throughout.

**Wave 10** closes the gap Azure left, on the family that isolates the axis S-5
botched: `fixed-tN` divides one fixed 1400 ms window into exactly N frames, so
every sample has the same `t` and only resolution moves — 14.0 / 5.6 / 2.8 ms bins
across a 5× span. C-2 is registered **two-sided**, because registering a direction
after watching S-5 fail in the opposite direction would be choosing with knowledge
of related data
([`PREREG_2026-08-22_…`](PREREG_2026-08-22_SHD_ATTENTION_RESOLUTION_LADDER.md)).

## 6. Open — and none of it is compute

| item | state |
|---|---|
| **`PAPER_DRAFT.md` abstract** | still reasons from the withdrawn v130 PASS. Authorial |
| **A6 caveat in the body** | the 80-epoch schedule undertrains *everything*, so `gap_closed` is not ceiling-normalised. Must not be a footnote |
| **A1 — commit + remote** | tree dirty, no git remote. Several days of preregs exist only as local files, and their value is their ordering |
| **arXiv endorsement** | flagged "start Day 0" on 08-19, still open |
| **Calibration** | criteria 3 + 4 are an external PyTorch reference reproduction whose **numbers already pass** (clean 0.9390/0.9368/0.9371 vs a 0.80 floor). The gap is **provenance**, and the only clean route is re-running those six seeds ([`FINDING_2026-08-21_…`](FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md)) |
| **Two gated binaries** | `shd-arch-ablation` and `temporal-deep-campaign` carry the `CeilingHealth` fix but **cannot be re-run** while the instrument is `Uncalibrated`; their on-disk reports still come from the old logic |

`MatchedDeepGradient` and `ShdEpropCeiling` are **not fixed**. This week made their
failure impossible to mistake for a result; diagnosing them is separate work.

## 7. Verification

Two gates, and they check different things.

```bash
bash scripts/gc_checks.sh       # the source:   GC1-GC7
bash scripts/record_checks.sh   # the evidence: tooling, numbers, weak checks
```

**Source:** `cargo fmt --check` clean · `cargo clippy --workspace --all-targets
-D warnings` clean · GC1–GC7 all executed and passed · **56 `test result: ok`,
0 failures**.

**Evidence:** 15 campaign-tooling tests · **12/12 published numbers reproduce from
the cells** via an implementation sharing no code with the analyser · weak-check
scanner calibrated and green.

**Reproducibility:** Gate F 10/10 bit-identical over two geometries, two widths and
two contracts, and 3/3 at each of `RAYON_NUM_THREADS` ∈ {1, 3, 8, 16} against
**recorded** values. Cross-architecture: 57,960/57,960 float values identical.

Both `record_checks.sh` components were **negative-tested**: perturbing one digit
in a result document fails the number check with exit 1; corrupting one archived
control fails the reuse guard. See
[`HARDENING_2026-08-22_THE_EVIDENCE_LAYER_HAD_NO_TESTS.md`](HARDENING_2026-08-22_THE_EVIDENCE_LAYER_HAD_NO_TESTS.md).
