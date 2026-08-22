# The instrument's bit-exactness does not survive a change of libm, and the gap is 0.005 accuracy

**Date:** 2026-08-19
**Backend:** rust, both sides.
**Recorded side:** macOS 27 / aarch64 (Apple libm), the machine that produced all
216 cells and every `gate-f-rust` baseline.
**Observed side:** Amazon Linux 2023 / aarch64, kernel 6.18.41, glibc, rustc
1.97.1. Instance `i-053f8151c8fa94444`, binary sha256 `5ddece4e…`.
**Command:** `scripts/gate_f_rust.py --cheapest 3`, run automatically by
`scripts/aws/bootstrap.sh` before any campaign work.

```
claim_axis: instrument-validity
claim: Re-running recorded rust cells on a different libm reproduces them to
  about three decimal places, not bit-exactly. The residual is ~0.005 in
  accuracy, which is 26% of the order effect the instrument is used to measure.
may_claim: That cross-machine absolute comparison against the recorded record is
  unlicensed at the resolution this instrument works at, and that same-machine
  paired contrasts are unaffected.
must_not_claim: That either side is wrong. Neither is a reference implementation
  for the other; this is a divergence, not an error. That the cause is
  *specifically* libm — that is the leading explanation and is consistent with
  the magnitude and the sign pattern, but it is **inferred**, not isolated. The
  ISA is the same on both sides and Rust emits no fused multiply-add without an
  explicit `mul_add`, which is what leaves the transcendentals as the suspect.
```

---

## 1. Why this was measured at all

`gate_f_rust.py` exists because "a one-ulp difference flips a spike and compounds
through Adam over epochs". That statement was made about a kernel change on one
machine. Moving the campaign to EC2 asked the same question across machines, and
the honest position before measuring was that nobody knew.

The campaign was therefore designed so that **no result depends on the answer**:
every wave in `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` carries its own
control arm, run on the same instance, from bit-identical base weights and
bit-identical epoch orders. The gate runs first on every instance and its verdict
is written to `gates/<instance>.json` whether it passes or fails, so a wave can
never be analysed without its verdict in hand.

## 2. Result — 0/3 bit-identical

| cell | field | recorded (macOS) | observed (Linux) | delta |
|---|---|---:|---:|---:|
| `fixed-t100 h128 e20 s5170001` | accuracy | 0.609982332 | 0.614840989 | **+0.004859** |
| | mean_loss | 1.726072573 | 1.726156692 | +0.000084 |
| | mean_firing_rate | 0.183328549 | 0.183598398 | +0.000270 |
| `…s5170002` | accuracy | 0.600265018 | 0.602473498 | **+0.002208** |
| | majority_prediction | 0.104681979 | 0.110865724 | +0.006184 |
| `…s5170003` | mean_loss | 1.730262679 | 1.727921302 | −0.002341 |
| | mean_gradient_norm | 0.274524836 | 0.271970501 | −0.002554 |

Accuracy on the third cell matched to all nine printed digits while its loss and
gradient norm did not — which is what a spike-threshold model does: the
divergence is continuous underneath and discretised at the readout, so it
sometimes cancels and sometimes does not.

Deltas are **not** systematically signed. Two accuracies moved up, one did not
move; loss moved up on two cells and down on one. That is the signature of
rounding divergence, not of a version skew or a configuration difference, both of
which would bias one way.

## 3. Why libm is the leading explanation

The two sides share an ISA (aarch64), the same source, the same `Cargo.lock`, and
a compiler that does not contract `a*b + c` into an FMA without `mul_add`. What
they do not share is the implementation of the transcendentals the forward calls:
`exp` for the leak factor `alpha` and the adaptation factor `rho`, `exp` again in
every softmax, `sin`/`cos`/`powf` in the attention position code, and `ln` in the
loss. None of these is required by IEEE-754 to be correctly rounded, and vendors
differ in the last place.

**This is inferred.** Isolating it would mean pinning the transcendentals to a
portable implementation and re-running — a real change to the kernel, with its
own Gate F consequences, and it is not done here.

## 4. What this costs, in the units the instrument works in

| quantity | value |
|---|---:|
| cross-machine accuracy divergence | **~0.005** |
| the order effect this instrument measures | 0.0189 |
| the ceiling's shortfall to the 0.80 gate | 0.0622 |
| the attention pilot's lift | 0.1702 |

The divergence is **26% of the order effect** and 8% of the ceiling shortfall.
Any cross-machine comparison at the 0.02 resolution the temporal-information
campaign works at is therefore inadmissible. At the 0.17 resolution of the
attention contrast it would be a rounding detail — but the campaign does not rely
on that, because it never compares across machines in the first place.

## 4b. Cross-*instance* determinism holds, and it had to be checked

The campaign's work queue hands cells to whichever worker is free, so a control
arm and its treatment can land on **different instances**. If two instances did
not agree, the paired design — the thing that makes §5 survivable — would be
worthless.

Two instances, `i-053f8151c8fa94444` and `i-06620abd20a856251`, built binaries
with **different** sha256 (`5ddece4e…` and `22d97c51…`), which by itself proves
nothing either way: cargo embeds absolute paths and build ids, so a hash
difference is expected and does not imply different codegen.

Their Gate F runs give the comparison for free. On all three cells and all six
compared fields, the two instances produced **identical** observed values to
every printed digit — 18 of 18. Cross-instance determinism holds.

This is a *measured* result on this AMI and instance type. It is not a guarantee
about a future host with a different glibc, and the gate that produced it runs on
every instance for exactly that reason.

## 5. Consequences, stated so they cannot be forgotten later

1. **The recorded macOS references — 0.7032 at h128/e400 and the 0.7378 converged
   ceiling — must not be quoted beside an AWS number.** Wave 1 re-runs `ff+fixed`
   on the instance precisely so the comparison has a same-machine control.
2. **Gate F remains a within-machine gate.** It did its job here: it detected a
   real divergence and refused to call it a pass. Nothing about the change to the
   parallel reduction is implicated — that change was separately verified
   bit-identical on the recorded machine, 4/4.
3. **A future instance reporting PASS would not retire this note.** It would mean
   that instance's libm happens to agree, which is a property of that host.
4. **The python arm is unaffected and remains deferred.** This is a rust-to-rust
   divergence.

## 6. What would close it

Pin the transcendentals — a vendored, bit-specified `exp`/`ln`/`sin`/`cos` — and
re-pin every recorded cell against the new kernel. That is a large, deliberate
change to the scientific record and is not proposed here. Short of it, the
correct discipline is the one the campaign already uses: **compare within a
machine, and carry the control arm everywhere.**
