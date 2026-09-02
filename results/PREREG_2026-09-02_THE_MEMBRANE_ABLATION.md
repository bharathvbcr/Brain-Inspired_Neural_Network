# Preregistration — is the reference's fast membrane the term that offsets its kernel?

**Registered:** 2026-09-02, **before any run of this series exists.** Attested by
git history: this file and `scripts/analyse_membrane_ablation.py` are committed
together, and the first run is produced afterwards.

**Analyser:** `scripts/analyse_membrane_ablation.py`, frozen in the same commit
and the authority on every verdict below.

**Closes, or fails to close:** the open item
[`PAPER_GAPS_2026-08-29.md`](PAPER_GAPS_2026-08-29.md) §2.4 opened on 2026-09-02
— *the reference's compensating −0.20 term is unidentified*.

---

## 1. What the kernel ablation left behind

[`RESULT_2026-09-02_THE_KERNEL_ABLATION.md`](RESULT_2026-09-02_THE_KERNEL_ABLATION.md)
measured the reference's 25-tap `Dcls1d` kernel at **K = +0.3111** — against a
gap to the instrument of **0.087**. Removing the kernel does not bring the
reference down to the instrument's **0.8320**; it drops it to **0.6276**,
**0.2044 below**. So the two systems differ by at least two large terms of
opposite sign and 0.087 is their net. **The negative term was not identified**,
and §3.8 now says so rather than implying an additive decomposition.

§3.8 already names the candidate. The reference's temporal integration lives in
the convolution and its neuron is close to a pointwise nonlinearity; the
instrument is the mirror image — no kernel, and a membrane that retains **0.82**
per step. Take the kernel away from the reference and it has *no temporal
integration at all*, which is a plausible reason it lands below a system that
has one.

**That is a hypothesis, and this series is the test of it.**

## 2. The manipulation, measured rather than derived

`best_config_SHD.py` sets `init_tau = 10.05` ms and normalises it by
`time_step = 10`. The reference neuron is `neuron.LIFNode(tau=init_tau)`, whose
membrane retention per step is what actually matters, so it was **measured**
before registration by constructing the neuron, injecting one unit and stepping
once with no input:

| `init_tau` | normalised τ | measured retention |
|---:|---:|---:|
| **10.05 ms** (as pinned) | 1.0050 | **0.0050** |
| 50.0 ms | 5.0000 | 0.8000 |
| **55.5556 ms** | 5.5556 | **0.8200** |
| 60.0 ms | 6.0000 | 0.8333 |

The pinned reference retains **half a percent** of its membrane per step — it is
pointwise to three decimal places, which is §3.8's claim confirmed rather than
restated. **`init_tau = 55.5556` ms reproduces the instrument's 0.82 exactly**,
and it is the only value this series introduces.

## 3. Design

Two arms, both at `max_delay = 1`, on the pinned checkout and the pinned
environment, clean protocol, 150 epochs, **3 seeds each — six runs**.

| arm | `max_delay` | `init_tau` | retention | seeds |
|---|---:|---:|---:|---:|
| **B′** — kernel removed, membrane as pinned | 1 | 10.05 ms | 0.0050 | 3 |
| **E** — kernel removed, membrane matched | 1 | 55.5556 ms | 0.8200 | 3 |

**Arm B′ is a repeat of the kernel ablation's Arm B and is not optional.** Those
runs were produced on a different instance; comparing Arm E against a number
from another machine would put a platform difference inside the manipulation.
Running B′ beside E makes the contrast within-fleet, and its agreement with
0.6276 is a reproducibility check the series gets for free.

**M = mean(E) − mean(B′)** is what the membrane is worth once the kernel is gone.

**Both arms were built and driven through a forward and a backward pass before
registration**, which is a no-training measurement and is reported here rather
than withheld. Arm E's derived values are `max_delay = 1`, `sigInit = 0`,
normalised τ **5.5556**, retention **0.8200** — the instrument's, matched
exactly. Registration rule 2 does not fire.

One observation from that build is worth recording *now*, because it is the kind
of thing that looks like a discovery afterwards: **Arm E's untrained loss is
19.49 against Arm B's 4.61** on identical synthetic input. A membrane retaining
0.82 accumulates far more than one retaining 0.005, so the initial scale of the
network is much larger. That is the manipulation, not a defect — and it is
exactly why the fourth row of §5 is registered. If the reference's optimiser,
schedule and thresholds are tuned around a pointwise neuron, M can come back
negative, and this series will report that rather than call it a null.

## 4. Hypotheses, with bars fixed now

**HM-1 — Arm B′ reproduces Arm B. This is the gate.**
`|mean(B′) − 0.6276| ≤ 0.020`. Arm B's own three-seed spread was 0.0072, and
this bar is 2.8× it — the same ratio HK-4 used over Arm A's 0.0036 spread, so
the bar is derived and not chosen. **If HM-1 fails nothing below is read**: a
membrane effect measured against a baseline that moved is not a measurement.

**HM-2 — the membrane is a material part of the missing term.**
`M ≥ +0.050`. HK-1's bar, unchanged, on the same scale of quantity.

**HM-3 — does matching the membrane close the gap to the instrument?**
`|mean(E) − 0.8320| ≤ 0.030`. HK-2's bar, unchanged.

**HM-4 — refutation.** `M < +0.020` means the membrane is **not** the
compensating term, the §2.4 register item stays open, and the search moves to
the remaining differences. HK-3's bar, unchanged.

## 5. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **HM-2 MET and HM-3 MET** | The two architectures are accounted for by **two** terms: a kernel worth +0.31 to the reference and a membrane worth roughly +0.20 to the instrument. §3.8 gains a decomposition it has never had, and the register item closes. |
| **HM-2 MET, HM-3 NOT MET** | The membrane carries part of the term and something else carries the rest. The register item is **narrowed with a number**, not closed, and §3.8 says how much is left. |
| **HM-4 refuted (M < 0.020)** | The membrane is not it. This is a real possibility — the reference's synaptic filter, its normalisation, its optimiser and its schedule are all untouched — and the register item stays open with one candidate eliminated, which is worth more than a guess. |
| **M strongly negative** | Slowing the membrane makes the kernel-free reference *worse*. That would say the reference's other machinery is tuned around a pointwise neuron, and it is reported as such rather than as a null. |

## 6. What this series may not do

1. **No arm is re-run to improve its verdict**, and no seed is added beyond
   three. Six runs, once.
2. **`init_tau` is the only value that moves.** If Arm E cannot run at
   55.5556 ms, the series is **abandoned and reported as abandoned** rather than
   rescued by changing a second value — the rule that caught Arm C of the kernel
   ablation before it ran.
3. **Arm E is not "the instrument".** It is the reference with two values
   changed. Every statement here is about the *reference's* sensitivity to its
   own membrane, and any claim about the instrument remains an inference from a
   difference of differences.
4. **One operating point**, the pinned SHD recipe at `time_step = 10`.
5. **`stateful_synapse` is `False` in the pinned config and stays false.** The
   synaptic filter is a separate candidate term and is not tested here.

## 7. Cost

Six runs at `max_delay = 1` on one `c7g.8xlarge`. Arm B measured **1.57 h**;
Arm E adds no parameters and no timesteps, so the series is expected to finish
in about **2 h** for roughly **$1**. The environment is the one
`scripts/aws/kernel_ablation/requirements.lock` pins — torch 2.13.0,
spikingjelly `6dca147`, checkout `d169b4e` — verified on the instance before any
epoch runs.
