# Result — the kernel is worth 0.311, which is 3.6× the residual it was supposed to explain

**Registered:** [`PREREG_2026-09-01_THE_KERNEL_ABLATION.md`](PREREG_2026-09-01_THE_KERNEL_ABLATION.md)
at `ce2ec42`, amended at `c292c4a` and again at `d6108ef`, all **before any run
existed**.
**Analyser:** `scripts/analyse_kernel_ablation.py`, frozen with the
registration and the authority on every verdict below.
**Runs:** 6 of 6, two arms × three seeds, 150 epochs each, `CLEAN_TEST_ONCE`.

---

## 1. What was measured

§3.8 of the manuscript attributes a **0.087** gap — between the instrument's
converged attention arm at **0.8320** and the pinned third-party reference at
**0.9376** — to that reference's `Dcls1d` temporal kernel of 25 taps per synapse
on every layer. `PAPER_GAPS_2026-08-29.md` §2.3 calls it **the paper's weakest
load-bearing inference**: it rested on elimination and code-reading, and on no
ablation that moved the kernel.

The manipulation sets `max_delay = 1` and lets the five values mechanically
derived from it follow. A one-tap dilated convolution is pointwise: the temporal
kernel is gone while every other tensor, layer, neuron, optimiser and schedule
is identical.

| arm | `max_delay` | accuracy | mean | wall |
|---|---:|---|---:|---:|
| **A** — reference as pinned | 25 | 0.9392, 0.9366, 0.9402 | **0.9387** | 5.06 h |
| **B** — kernel removed | 1 | 0.6250, 0.6322, 0.6256 | **0.6276** | 1.57 h |

## 2. Verdicts

**HK-4 — MET, and it is the gate.** Arm A is **0.9387** against the pinned
**0.9376**, a drift of **0.0011** against a bar of 0.010.

This matters more than it looks. The series ran on Graviton3 / aarch64 Linux,
not on the macOS arm64 machine that produced the pinned 0.9376, so Arm A is a
**re-measured** baseline rather than a continuation of that series. HK-4 existed
to test exactly that move, and the environment was pinned to make it survivable:
torch **2.13.0** — the same version, from its `manylinux_2_28_aarch64` wheel —
spikingjelly at commit `6dca147`, and the reference checkout verified to
`d169b4e` on the instance. It reproduced to about one part in a thousand. The
platform move is a measured fact, not an assumption.

**HK-1 — MET.** **K = +0.3111** against a bar of 0.050. The kernel carries the
majority of the residual.

**HK-3 — not refuted.** K is far above the 0.020 that would have withdrawn
§3.8's attribution. **The kernel is load-bearing, measured rather than
inferred.**

**HK-2 — NOT MET, and this is the informative outcome.** Arm B lands at
**0.6276** against the instrument's **0.8320** — a distance of **0.2044**
against a bar of 0.030.

## 3. What HK-2's failure means, stated plainly

§3.8's inference was an accounting one: *the reference beats the instrument by
0.087, the kernel is the structural difference, therefore the kernel is worth
about 0.087.* The registration named the outcome that would break it, and that
is the outcome that occurred.

**The kernel is worth 0.3111 — 3.6× the gap it was invoked to explain.** Remove
it and the reference does not descend to the instrument; it falls **0.2044
below** it.

So the two architectures are **not** "the instrument plus a temporal kernel."
They differ by at least two large terms of opposite sign: a kernel worth +0.31
to the reference, and something else worth roughly −0.20 to it relative to the
instrument. The observed 0.087 is the **net** of offsetting differences, not the
size of any one of them.

**This strengthens the paper and narrows it at the same time.** The kernel is no
longer a plausible story — it is a measured 0.31, larger than anything else in
the ablation series, and it is the single biggest structural fact about the
reference. What must go is the *arithmetic* attribution: §3.8 may no longer say
the residual "is" the kernel. It must say the kernel is worth three and a half
times the residual, that the residual is therefore a net, and that the
compensating term is **not identified by this series**.

Beside the four earlier ablations — depth (**−0.0145**, it *gains*), batchnorm
(**−0.0058**, it gains), read-out style, and dropout (**+0.0128**, the only
positive contributor) — the picture is now: one dominant positive term, two
small negative ones, and an unexplained negative remainder.

## 4. A hypothesis this does not test, and should not be read as testing

The obvious reading of "Arm B falls to 0.6276, below the instrument's 0.8320" is
that the instrument's attention read-out recovers something the one-tap
reference cannot. **This series cannot say that.** The two systems differ in
substrate, training loop, optimiser and schedule, and the wave measured one
manipulation inside one of them. The comparison of Arm B against 0.8320 is a
distance, reported because HK-2 registered it as a bar, and it is not a
controlled contrast.

## 5. Corroboration from wall time

Arm A takes **5.06 h**; Arm B takes **1.57 h** on identical hardware, threads
and data. A 3.2× speedup is what removing a 25-tap convolution and its 24 + 12
padding should cost, and it is independent evidence that the manipulation
reached the mechanism rather than a config value nothing reads — which is
exactly the failure the third amendment caught in Arm C before it ran.

## 6. What was withdrawn before any run, and why it is in the record

The registration reached nine runs and shipped six. **Arm C and HK-5 were
withdrawn** at `d6108ef` after tracing every consumer of `time_mask_size`: it is
read only at `datasets.py:45-46`, reachable only under `if self.config.augment:`
— and `best_config_SHD.py:125` sets `augment = False`, while the clean protocol
never constructs the augmentation at all. Arms B and C were the **same
configuration**, and HK-5 would have compared an arm with itself **and passed**,
printing "the augmentation is not carrying K" as though something had been
measured. Full record:
[`DEFECT_2026-09-01_HK_5_CANNOT_FIRE.md`](DEFECT_2026-09-01_HK_5_CANNOT_FIRE.md).

## 7. Limits

- **n = 3 per arm**, as registered. The seed spread is 0.0036 in Arm A and
  0.0072 in Arm B, so K's uncertainty is small next to its size, but three seeds
  is three seeds.
- **One operating point** — the pinned SHD recipe at `time_step = 10`, 150
  epochs, clean protocol.
- **The compensating −0.20 term is not identified.** Naming it is the next
  ablation, not a conclusion of this one.
- **Arm A is a re-measurement on new hardware.** HK-4 says it reproduced; it
  does not make the two series one series.
