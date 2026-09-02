# Result — the membrane is 81.5% of the term the kernel ablation could not name

**Registered:** [`PREREG_2026-09-02_THE_MEMBRANE_ABLATION.md`](PREREG_2026-09-02_THE_MEMBRANE_ABLATION.md)
at `f9ac1e3`, **before any run of this series existed**.
**Analyser:** `scripts/analyse_membrane_ablation.py`, frozen in that commit and
the authority on every verdict below.
**Runs:** 6 of 6, two arms × three seeds, 150 epochs, `CLEAN_TEST_ONCE`.

---

## 1. The question

[`RESULT_2026-09-02_THE_KERNEL_ABLATION.md`](RESULT_2026-09-02_THE_KERNEL_ABLATION.md)
measured the reference's 25-tap kernel at **K = +0.3111** and found that removing
it drops the reference to **0.6276** — **0.2044 below** the instrument's 0.8320,
not down to it. The gap between the two systems is therefore a **net** of terms
of opposite sign, and the negative one was unidentified.

§3.8 names the candidate: the reference's temporal integration lives in its
convolution and its neuron is close to pointwise, while the instrument has no
kernel and a membrane retaining 0.82 per step. Take the kernel away and the
reference has *no temporal integration at all*.

## 2. Result

| arm | `max_delay` | `init_tau` | retention | accuracy | mean |
|---|---:|---:|---:|---|---:|
| **B′** — kernel removed, membrane as pinned | 1 | 10.05 ms | 0.0050 | 0.6250, 0.6322, 0.6256 | **0.6276** |
| **E** — kernel removed, membrane matched | 1 | 55.5556 ms | 0.8200 | 0.7989, 0.7879, 0.7959 | **0.7942** |

**HM-1 — MET, and more exactly than the bar required.** Arm B′ reproduces the
kernel ablation's Arm B at **0.6276 against 0.6276**, a drift of **0.0000**
against a bar of 0.020. The three per-seed values are **identical to every
recorded digit** — 0.6250, 0.6322, 0.6256 — on a **different spot instance**,
five hours later. A 150-epoch stochastic PyTorch training run reproduced exactly
across machines under the pinned environment. That is the gate, and it passed as
completely as a gate can.

**M = +0.1666** — what the membrane is worth once the kernel is gone.

**HM-2 — MET.** M = +0.1666 against a bar of 0.050. **The membrane is 81.5% of
the 0.2044** that separated the kernel-free reference from the instrument.

**HM-3 — NOT MET.** Arm E reaches 0.7942 against the instrument's 0.8320, a
distance of **0.0378** against a bar of 0.030. Matching the membrane does not
close the gap; it closes four fifths of it.

**HM-4 — not refuted.** The membrane hypothesis stands as measured.

## 3. The decomposition, which now closes

Every term below is a difference of means from this series or the kernel
ablation, on one pinned environment:

| term | value |
|---|---:|
| reference as pinned (Arm A) | 0.9387 |
| − the 25-tap temporal kernel | **−0.3111** |
| + a membrane matched to the instrument's 0.82 | **+0.1666** |
| = kernel-free, slow-membrane reference (Arm E) | 0.7942 |
| unexplained remainder to the instrument | **+0.0378** |
| = instrument's converged attention arm | 0.8320 |

So the reference's lead over the instrument, **0.1067** on the re-measured Arm A,
is **+0.3111 − 0.1666 − 0.0378**. Two measured terms of opposite sign and a
remainder a third the size of either.

**This is what §3.8 lacked.** The section previously read a 0.087 gap as the
kernel's value; the kernel ablation showed that was wrong by a factor of 3.6 and
could not say what offset it. It is now named and measured, and what is left
unexplained has fallen from **0.2044 to 0.0378** — below the 0.05 bar this
programme has used for "material" throughout.

## 4. What the manipulation was, and how it was fixed before the runs

The reference neuron is `LIFNode(tau=init_tau)`, and what matters is per-step
membrane retention, so it was **measured before registration** — build the
neuron, inject one unit, step once with no input:

| `init_tau` | normalised τ | measured retention |
|---:|---:|---:|
| 10.05 ms (as pinned) | 1.0050 | **0.0050** |
| **55.5556 ms** | 5.5556 | **0.8200** |

The pinned reference retains **half a percent** per step — §3.8's "close to a
pointwise nonlinearity" confirmed to three decimals rather than restated.
55.5556 ms reproduces the instrument's 0.82 exactly, and it is the only value
this series introduced.

**The prediction the registration made and this run did not vindicate.**
§3 recorded, before any run, that Arm E's *untrained* loss is 19.49 against Arm
B's 4.61, and registered the outcome where a reference tuned around a pointwise
neuron gets *worse* with a slow one. That was the risk; it did not materialise.
M is strongly positive. Recording the risk in advance is what makes the positive
result readable as a result rather than as the only outcome that was ever
imagined.

## 5. Limits

- **n = 3 per arm**, as registered. Seed spreads are 0.0072 (B′) and 0.0110 (E),
  small next to M = 0.1666, but three seeds is three seeds.
- **Arm E is not the instrument.** It is the reference with two values changed.
  Every statement here is about the *reference's* sensitivity to its own
  membrane; anything about the instrument is an inference from a difference of
  differences.
- **0.0378 is still unexplained**, and the candidates are untouched: the
  synaptic filter (`stateful_synapse` is `False` in the pinned config and stayed
  false), the optimiser, the schedule, and the read-out.
- **One operating point** — the pinned SHD recipe at `time_step = 10`.
- **The two terms are not independently additive by construction.** They were
  measured sequentially — kernel removed first, then membrane matched on top —
  so +0.1666 is the membrane's value *in the absence of the kernel*, not in
  general. The reverse order was not run.
