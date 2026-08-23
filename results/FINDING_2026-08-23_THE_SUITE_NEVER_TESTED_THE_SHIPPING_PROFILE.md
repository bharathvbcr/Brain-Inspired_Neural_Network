# The suite has never tested the profile that produces the results

**Found:** 2026-08-23, while re-checking a claim I had made wrongly the day before.

**Bears on:** every gate in the repository, and on why the attention-pin
confusion of 2026-08-22 was possible at all.

---

## 1. The gap

Every place the test suite runs invokes `cargo test` with **no `--release`**:

| caller | line |
|---|---|
| `.github/workflows/ci.yml` | `cargo test --locked --workspace` |
| `scripts/run_all.sh` | `cargo test --locked --workspace` |
| `scripts/overnight.sh` | `cargo test --workspace` |
| `scripts/run_code_transfer_campaign.sh` | `cargo test --locked --workspace` |
| `scripts/check_gc3.sh`, `check_gc7.sh` | `cargo test --locked -p …` |

Every scientific cell is produced by a **release** build:

| caller | line |
|---|---|
| `scripts/aws/bootstrap.sh` | `cargo build --locked --release -p binn-lab --bin shd-instrument` |
| `scripts/gate_f_rust.py` | `DEFAULT_BINARY = target/release/shd-instrument` |

So the profile that produces every number in the paper was validated by nothing
but Gate F — which regresses *recorded* cells and is therefore blind to any arm
that has none — and the profile the suite validates has never produced a result.

## 2. It is not a hypothetical distinction for this kernel

`shd_attention::positional_code` calls `.sin()` and `.cos()` on the same
argument. At opt-level ≥ 2, LLVM's libcall simplification merges the pair into
Darwin's combined `__sincosf_stret`; at 0 and 1 it emits separate `sinf` and
`cosf`. Those do not agree to the last ulp, the spike threshold is hard, and the
difference compounds through Adam.

The attention arms therefore hash **differently between profiles**, which is why
`shd_matched_arms.rs` now pins both sides (`PIN_*_OPTIMISED` /
`PIN_*_UNOPTIMISED`). Established in `ca92bee` from the emitted assembly, and
extended in `45bd6fb`: glibc is a third set, so the split is Darwin-only.

A kernel change that moved only the optimised side would pass `cargo test`, pass
GC1–GC7, pass CI, pass `run_all.sh` — and silently alter every cell produced
afterwards.

## 3. How this produced a wrong claim

On 2026-08-22 I measured `every_attention_arm_forward_and_backward_is_bit_pinned`
failing at four commits and concluded the pin "had never passed" and that its
constants "match no committed state". I ran `cargo test --release` throughout and
never said so. The constants match exactly at opt-level 0, which is what the
default `cargo test` — and therefore the commit messages I contradicted — was
running.

Both measurements were correct. The conclusion drawn from mine was not, and it
was uncharitable about three commits that were reporting a suite that really did
pass where they ran it. Corrected in
`HARDENING_2026-08-22_THE_ATTENTION_KERNEL_HAD_NO_GATE.md` §1.

The useful residue: **a bit-pin that does not record the profile it was taken
under is not reproducible**, and neither is a test report that does not say which
profile it ran.

## 4. The fix

`scripts/check_kernel_profiles.sh` runs `binn-learn` and `binn-lab` — the two
crates a recorded number must pass through — in **both** profiles and requires
both to pass. Wired into `scripts/run_all.sh` and into CI immediately after the
existing debug test step.

Measured on the current tree: **80 targets, 996 tests, 0 failures** across the
two profiles.

Scoped to two crates rather than the workspace so the cost is one extra optimised
build of the kernel, not of everything.

Its own first run failed on `"${flags[@]}"` under `set -u` in macOS's bash 3.2 —
recorded here because it is the kind of failure that would otherwise have been
quietly edited away, and because it failed loudly rather than skipping the check,
which is the behaviour the script exists to have.

## 5. What is still not covered

* **Only two crates.** A change in `binn-core` or `binn-data` that altered a
  recorded number would have to pass through `binn-learn` to do it, but the
  profile check does not run their own suites twice.
* **Only two profiles.** opt-level 1 sits on the unoptimised side and 3 on the
  optimised side, so `debug` and `release` cover both, but nothing pins that
  mapping — it is measured in `ca92bee`, not asserted by a test.
* **Nothing here changes the record.** Every recorded cell came from a release
  build, on Linux under glibc, where the Darwin sincos split does not arise at
  all. The paper's numbers are unaffected in either direction.
