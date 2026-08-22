# Finding — the bit-identical replay property is macOS-specific

**Date:** 2026-08-19
**Status:** verified by direct measurement on two hosts
**Bears on:** `REPRO_ARTIFACT_CHECKLIST.md`, every `--config-hash` replay claim, and
`WEEK_PLAN_2026-08-19.md` §0.4

---

## Claim

`f32` transcendental functions are **not** bit-identical between
`aarch64-apple-darwin` (Apple libm) and `aarch64-unknown-linux-gnu` (glibc). The
`--config-hash` replay property therefore holds **within a platform**, not across
platforms. A reviewer or artifact evaluator replaying on Linux will not reproduce
the gradient-reference numbers, on identical source, identical `Cargo.lock`,
identical seeds, and identical frozen splits.

Both hosts are **aarch64**. This is not an architecture difference. It is a libm
difference.

## How it surfaced

`a6-ceiling-health` builds its self-check row from the canonical config and
compares it against the published number. Running the same binary from the same
source snapshot on both hosts:

| quantity | M5 Pro (aarch64-apple-darwin) | c7g.8xlarge (aarch64-unknown-linux-gnu) |
|---|---|---|
| `c1-dfa` `MatchedDfa` arm | 0.9387 | 0.9387 |
| `c1-dfa` `MatchedBroadcastErr` control | 0.9863 | 0.9863 |
| `c1-dfa` **gradient reference** | **0.8963** | **0.9013** |
| `c1-rl` `MatchedRlReinforceFb` arm | 0.9200 | 0.9200 |
| `c1-rl` `MatchedRlGraded` | 0.5250 | 0.5250 |
| `c1-rl` `MatchedRlFlat` | 0.5113 | 0.5113 |
| `c1-rl` **gradient reference** | **0.8887** | **0.9188** |

Every local-rule arm reproduced exactly. Only the BPTT gradient reference moved —
by 0.0050 on the DFA schedule and **0.0301** on the RL schedule. The RL drift is
of the same order as that arm's own standard error (0.0326), so it is not a
rounding cosmetic; it is a materially different reference.

## Direct confirmation, independent of BINN

A standalone probe (no BINN code) hashes 400,000 `f32` results per function over
`z ∈ [-20, 20]` and prints an FNV-1a fingerprint of the raw bit patterns:

| function | aarch64-apple-darwin | aarch64-unknown-linux-gnu | identical? |
|---|---|---|---|
| `sqrt` | `9aaf2459b2d754a6` | `9aaf2459b2d754a6` | **yes** |
| `exp` | `0c7232359109084c` | `f3cdd09421bc6bf4` | no |
| `ln` | `0af02664adb7aa6e` | `944a88891524fb1a` | no |
| `tanh` | `360d26676921d5b3` | `a80692a7e6885c3b` | no |
| `powf` | `77e3ff7b919676d8` | `ebc4511e2711dcf0` | no |
| `1/(1+exp(-z))` as written in the source | `804fcb3c24970740` | `00d8fc4f1f235f18` | no |

`sqrt` agrees because IEEE-754 **requires** it to be correctly rounded, and it
compiles to a single hardware instruction. Nothing else in that list is required
to be correctly rounded, and every one of them disagrees.

Splitting `exp` into eight sub-ranges of 25,000 samples each, **all eight** ranges
disagree. This is pervasive across the domain, not an edge case at the extremes.

## Why only the gradient reference moved

`MatchedGradient` runs full BPTT: it evaluates the surrogate sigmoid
(`matched_local_baseline.rs:426`, `1.0 / (1.0 + (-z).exp())`) on every unit, at
every timestep, on every forward *and* backward pass, for 80 epochs. Sub-ULP
disagreements compound along that trajectory into a different weight vector, and
eventually into different `argmax` decisions on test examples that sit near a
decision boundary.

The local-rule arms call the same sigmoid (`matched_dfa_baseline.rs:264`), so they
are **not** structurally immune. They reproduced here because their coarser
updates left the test-set decisions far enough from the boundary that no example
flipped at these 20 seeds.

**Do not report the arms as "portable".** The correct statement is: at these
seeds, on this task, the arms' decisions were insensitive to libm and the
reference's were not. The mechanism threatens both.

## What this does and does not invalidate

- **Does not** invalidate any published number. The macOS numbers were produced on
  macOS and reproduce on macOS.
- **Does** invalidate the unqualified form of the replay claim. Any wording that
  says a `--config-hash` replays from a clean checkout must say **on the same
  platform**, and the artifact should state the platform it was produced on.
- **Confirms** `WEEK_PLAN_2026-08-19.md` §0.4: canonical runs must not move to a
  Linux box. The plan reasoned this from `ensure_manifest` hashing `rust_binary`.
  That reasoning was right, and this is the stronger, measured version of it — the
  numbers themselves move, not just the binary hash.
- **Bounds A6.** Reference-vs-arm *ordering* measured on one host is a valid
  within-platform comparison, because reference and arms are computed on the same
  box. Absolute reference values from the Linux sweep must **not** be compared to
  the published macOS 0.8963 / 0.8887.

## Recommended actions

1. State the production platform in `REPRO_ARTIFACT_CHECKLIST.md` and in the
   paper's reproducibility section, and scope the replay claim to it.
2. Decide explicitly whether to keep libm-dependent bit-identity as a headline
   property. If it is to survive cross-platform review, the surrogate's
   transcendental calls need a vendored, correctly-rounded implementation rather
   than the platform's — that is a real piece of work, not a wording fix.
3. Treat "reproduces on the reviewer's machine" as an open risk in the artifact
   evaluation, not a settled one.

## Reproducing this finding

```bash
# Same source snapshot on both hosts, then:
cargo build --locked --release -p binn-lab --bin a6-ceiling-health
./target/release/a6-ceiling-health --suite both --epochs 80
# Compare the "Harness check" line on each host.
```

The standalone libm probe used for the table above is not part of the workspace;
it is 20 lines of `std`-only Rust that hashes `f32::exp/ln/sqrt/tanh/powf` over a
fixed grid and prints the fingerprint.
