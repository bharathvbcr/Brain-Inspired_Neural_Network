# A Preregistered Instrument for Backpropagation-Free Learning: Four Negative Gates, One Conditional Positive, and Seven Withdrawals

**BINN — Brain-Inspired Neural Network Substrate**

*Whole-project manuscript, 2026-08-31. Companion to [`PAPER_DRAFT.md`](PAPER_DRAFT.md), which is the camera-ready treatment of the SHD read-out program alone. This paper covers the instrument and the full experimental record.*

---

> **Authority and provenance.** Every number in this paper is taken from
> [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) (the claim freeze) or from the
> on-disk result note cited at the point of use. Where the repository
> [`README.md`](../README.md) disagrees with the claim freeze, **the claim freeze
> wins and the README is stale** — see §8.1. Nothing in this paper is asserted
> beyond the freeze's ladder, and its explicit non-claims are reproduced in §9
> rather than summarised away.
>
> **Citation provenance.** References marked **[v]** were retrieved and checked
> against a primary index during the preparation of this manuscript. References
> marked **[r]** are inherited from the repository's own literature pass of
> 2026-08-27 and, per that pass's own warning, **have not been independently
> verified here**. The distinction is load-bearing and is preserved in the
> reference list.

---

## Abstract

BINN is a from-scratch, deterministic Rust research instrument built to answer one
falsifiable question: can a sparse-assembly, locally learned, event-driven network
learn competitively **without backpropagation**? It was built with preregistered
kill-gates, frozen configuration hashes, and a standing rule that a failed gate is
permanent. This paper reports what the instrument found.

The headline is a **negative**. Holding a dense leaky-integrate-and-fire forward
pass identical and swapping only the update rule, **broadcast ±1 three-factor
plasticity** — surrogate eligibility multiplied by a single scalar ±1 reward —
remains at chance on **both** forward graphs (feed-forward 0.5000, gap lower
confidence bound 0.0000; recurrent 0.5100, LCB −0.0192) against a SuperSpike
backpropagation-through-time reference at 1.0000, at n = 20 seeds. It is the only
rule tested that fails. Graded direct feedback alignment (0.9925 / 0.9875),
REINFORCE against frozen per-neuron feedback (0.9950 / 0.9812), broadcast *graded*
error (0.9975), and a discrete EventProp-style spike-adjoint (0.9450 / 0.8900) all
clear the gate. Because the reference sits at exactly 1.0000, **each pass reduces
to "the arm scored above 0.75", and no ordering among the passing arms may be
claimed**: the task isolates one failure from a field it can no longer rank.

Transferring the rules that *pass* the matched gate onto the live event-driven
k-WTA substrate fails across twelve preregistered gap-close variants (v13–v24),
best gap LCB 0.3127 against a 0.5 threshold. Two post-G2 gates also close negative:
local plasticity alone does not prevent catastrophic forgetting (forgetting 0.8948
against a replay baseline's 0.2725), and multi-area composition **degrades** rather
than compounds (capability ≈ −0.1924·ln(n) + 1.1673, R² = 0.985; NO-GO).

The one positive result is a **conditional**, and it is about a read-out rather
than about local learning. Adding a time-axis attention read-out over feed-forward
LIF features raises Spiking Heidelberg Digits accuracy from 0.7057 to **0.8332**
(gain +0.1275, positive in 32/32 seeds). That much is prior art. What is measured
here is **which component's marginal contribution is the order-dependent one**:
permuting time bins per sample in *both* the train and test splits — so the task
itself becomes rate-solvable — costs the attention read-out **+0.1347** of accuracy
against the rate read-out's **+0.0142** on the same seeds and splits, a **9.5×
ratio positive in 32/32 seeds**. This is a difference-in-differences on the *gain*,
not on accuracy. Three scope limits travel with it: the gain **inverts at h1024**
by a threshold that is located but unexplained, **0.8332 is not competitive**
against a 95–96.4% frontier, and the instrument is **uncalibrated**.

Finally, we report the record's own failures. **Seven results were withdrawn**
during two repair episodes, four of them PASSes. One defect — a matched forward
pass that emitted **zero spikes at every seed** — silently invalidated an entire
suite and was caught only by an instrument-health audit, not by any experiment's
own verdict. We argue that the withdrawal ledger is a first-class result.

---

## 1. Introduction

Backpropagation is the workhorse of deep learning and is also the part of it least
defensible as a model of biological learning: it requires a symmetric backward
weight transport, a global synchronisation of layer updates, and — for temporal
tasks — an unrolled computational graph whose memory grows with sequence length.
A large literature proposes alternatives that keep the credit signal local, among
them feedback alignment and its direct variant [1][v], eligibility-trace methods
such as e-prop [3][r][8][v], and three-factor rules in which a local synaptic
eligibility is gated by a neuromodulatory third factor [7][v].

The question that motivates BINN is narrower than "is local learning possible."
It is operational:

> Can a sparse-assembly, locally learned, event-driven network learn competitively
> without backpropagation — **on an instrument built to be able to say no**?

Most of the difficulty in answering this is not algorithmic but methodological. A
local rule that underperforms may be failing because locality is insufficient, or
because the forward pass it runs on differs from the reference's, or because the
training budget undertrains one arm, or because the harness is broken in a way no
arm's own accuracy would reveal. BINN was built to separate these. It is not a
product, not a neuromorphic deployment framework, and not a brain model. It is an
**exact, deterministic instrument with preregistered kill-gates**, and its design
premise is that the value of a negative result is entirely a function of how hard
the instrument tried to produce a positive one.

### 1.1 Contributions

1. **A matched-architecture kill gate** that isolates the update rule as the only
   varying term, and a clean negative on it: broadcast ±1 three-factor fails on
   both forward graphs while every other rule tested passes (§5).
2. **A substrate-transfer negative**: rules that pass the matched gate do not
   transfer to the live event-driven k-WTA engine, across twelve preregistered
   gap-close variants (§6).
3. **A difference-in-differences result on SHD** identifying the read-out's
   marginal contribution — not accuracy — as the order-dependent quantity, with
   its scope limits measured rather than asserted (§7).
4. **Two further negative gates** on continual learning and multi-area scaling (§8).
5. **A withdrawal ledger**: seven results retracted, the defects that caused them,
   and the instrument-health checks added in response (§10). We treat this as a
   contribution, not an erratum.

### 1.2 What this paper does not claim

We do not claim biological realism, cortical modelling, neuromorphic hardware
results, Assembly Calculus success, or impossibility of local learning in
principle. Every negative here is scoped to a named protocol on a named hash. The
full non-claim list is §9.

---

## 2. Related work

**Local and backprop-free credit assignment.** Direct feedback alignment shows that
a *fixed random* backward projection suffices to train deep networks, and scales
further than its early framing suggested [1][v]; meta-learned plasticity rules with
random feedback pathways probe the same axis from the rule side [2][v], as does
recent work meta-learning three-factor rules under sparse feedback [7][v].
Eligibility-trace methods make the temporal credit problem local in time: e-prop
and its STDP-augmented variants train recurrent spiking networks online without an
unrolled backward pass [3][r][8][v], and precise-spike-timing variants sharpen the
trace [9][v]. BINN's three-factor rule, Δw = η·e·M − λ·w, sits in this family; its
contribution is not a new rule but a **matched gate** that can tell one rule's
failure from a forward-pass difference.

**Gradient references for spiking networks.** SuperSpike provides a surrogate-
gradient reference for multi-layer spiking networks [4][v] and is the reference arm
in our matched gate. EventProp derives exact gradients from spike times via an
adjoint method [5][r]; our discrete spike-adjoint arm is *inspired by* it and is
explicitly **not** equivalent to the continuous formulation — a distinction that
became load-bearing when that arm's earlier result was withdrawn (§10).

**Assemblies.** The Assembly Calculus formalises computation over sparse
k-cap assemblies, with results on classification of well-separated distributions
[6][v] and on computing with sequences [10][v]. BINN implements `project` and
`associate` over k-WTA areas. We report that wiring `project` onto the crux task
**fails** the gate (§6.3); we do not report an Assembly Calculus result.

**The benchmark.** Spiking Heidelberg Digits was introduced for systematic
evaluation of spiking networks [11][v]. Its temporal-order dependence is
established prior art: spike-count-only variants could not exceed **60%** [11][r],
and later studies reach the same conclusion with model-side and spike-time
operators [12][r][13][r]. Time-axis attention in spiking networks is likewise prior
art — TA-SNN reports 91.08% [14][v] and STSC-SNN 92.36% [15][r]. **Neither "attention
helps on SHD" nor "SHD needs temporal order" is available to us as a claim**; §7
states precisely what is left, and it is narrower. The current frontier (95–96.4%)
is reached with learned delays [16][v], adaptation, and spiking transformers.

---

## 3. The instrument

BINN is a six-crate Rust workspace with a strict upward dependency order
`lab → data → learn → areas → engine → core`.

| Crate | Layer | Purpose |
|---|---|---|
| `binn-core` | L2 | SoA buffers, CSR/CSC sparse graphs, ChaCha12 RNG, SIMD, associative scans |
| `binn-engine` | L3 | 8-level hierarchical timing wheel, multi-compartment LIF cells, synapse tables |
| `binn-areas` | L4 | k-WTA competition, Assembly Calculus (`project`, `associate`), wiring priors |
| `binn-learn` | L5 | Three-factor plasticity, STDP eligibility, DFA / e-prop / BPTT reference baselines |
| `binn-data` | L6 | Fixed rate/latency/population encoders, SHD framing, work accounting |
| `binn-lab` | L7 | Experiment runners, multi-seed harnesses, config hashes, gates |

### 3.1 Neuron model

Each cell carries an adaptive soma v(t), an adaptive threshold θ(t), and K = 4
independent dendritic branches:

    τ_d · d v_dend[i]/dt = −v_dend[i] + I[i]
    τ_m · dv/dt          = −v + Σ_i g_c · (v_dend[i] − v)
    τ_θ · dθ/dt          = −(θ − θ_rest)

Sub-threshold dynamics are evaluated **analytically and lazily** when an event
touches the cell, so silent neurons cost nothing and total work scales with active
events rather than with wall-clock time. Somatic spikes reset only the soma
(v ← 0, θ ← θ + Δθ), **preserving dendritic potentials across emission**.
Supralinear dendritic coincidence is available as Σ max(0, v_dend[i])².

### 3.2 Areas and k-WTA

Neurons are partitioned into Areas; an Area admits at most k concurrent winners via
lateral inhibition, enforcing activity sparsity ≈ k/N. Hard k-WTA uses an O(N)
partial selection with deterministic tie-breaking (highest potential, then lowest
`CellId`); a soft/annealed variant selects via softmax(s_i/T). This hard
competition boundary — and the θ = ∞ muting of losers — is the substrate feature
that §6 shows to be the transfer barrier.

### 3.3 Plasticity

    Δw_ij = η · e_ij(t) · M_j(t) − λ · w_ij
    d e_ij/dt = −e_ij/τ_e + STDP(Δt)

Eligibility is local to the synapse; the third factor M_j is a global scalar
reward, a vector DFA signal, or a REINFORCE feedback. Reverse CSC indices give
O(fan-in) postsynaptic lookup. Weight decay is applied **only** to synapses with
active eligibility (|e_ij| > 1e−8), so quiescent synapses do not erode. Resident
memory is **O(1) in sequence length** — there is no unrolled graph.

### 3.4 Enforced global constraints

Seven constraints are checked in CI (`scripts/gc_checks.sh`), not merely intended:

| ID | Constraint |
|---|---|
| GC1 | No autograd or dense matmul on the production path (`engine`, `areas`, `learn`) |
| GC2 | Zero external ML frameworks (no torch, candle, tensorflow) |
| GC3 | Bit-determinism for identical seeds on identical platforms |
| GC4 | Fixed input encoders — no learned autodiff front-end |
| GC5 | Criterion benchmarks compile for all hot paths |
| GC6 | No undocumented `unsafe` |
| GC7 | Activity sparsity (≤ k/N) logged for every run |

GC1 is what makes the matched gate meaningful: the gradient reference arms are
explicitly GC1-**exempt** baselines in `binn-learn/src/matched_local_baseline.rs`,
segregated from the production substrate they are compared against.

---

## 4. Method: preregistration, hashes, and kill-gates

Three disciplines distinguish this record from an ordinary ablation sweep.

**Frozen configuration hashes.** Every scientific run is named by a hash over its
configuration. A protocol may not be reinterpreted under its old hash; a new
hypothesis requires a new hash. `c1-118207fbc3eaba53` (Gate G2) and
`r2-afafa0fa6f43e3fc` (Gate G4) are frozen and are not reopened by any downstream
result in this paper.

**Kill-gates are permanent.** G2 FAIL is terminal: every downstream experiment
(C2, C3, R1, R2) requires an explicit opt-in flag (`--enable-c2`, …) to run at all,
so that post-G2 work cannot be mistaken for a rescue of G2.

**Preregistered bars and falsifiers.** Each claim carries a numeric bar fixed
before the run and a stated falsifier. Where a preregistered prediction failed, we
report it as failed (§7.3) rather than dropping it.

**Machine-checked prose.** `scripts/record_checks.sh` recomputes **125 assertions**
from archived cells against the result documents, of which 13 are prose checks on
the camera-ready draft. Its coverage is the SHD attention campaign **only**; the
matched-architecture numbers are attested by hashed on-disk run records instead,
and we state that boundary rather than implying uniform verification.

---

## 5. The matched-architecture kill gate

### 5.1 Design

The gate holds the dense-LIF coincidence forward pass **identical** across arms and
swaps only the update rule, against a SuperSpike BPTT reference [4][v]. An arm
passes on an accuracy floor and a gap lower confidence bound. All figures below are
the 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20 seeds, on **both** the
feed-forward and recurrent graphs.

### 5.2 Result

| Arm | recurrent | feed-forward | verdict |
|---|---:|---:|---|
| **Broadcast ±1 three-factor** (lead) | **0.5100** | **0.5000** | **FAIL**, both |
| Graded DFA | 0.9875 | 0.9925 | PASS, both |
| REINFORCE × frozen per-neuron `B_i` | 0.9812 | 0.9950 | PASS, both |
| Broadcast *graded* (honesty contrast) | 0.9975 | 0.9975 | — contrast |
| RL graded-reward broadcast | 0.9100 | 0.8787 | — contrast |
| RL ±1 broadcast | 0.7962 | 0.7775 | — contrast |
| Discrete EventProp-style spike-adjoint | 0.8900 | 0.9450 | PASS, both |
| SuperSpike BPTT | **1.0000** | **1.0000** | reference |

The lead arm's gap LCB is **0.0000** (feed-forward) and **−0.0192** (recurrent)
against a 0.5 bar; its accuracy 0.51 against a 0.65 floor.

### 5.3 What may and may not be read from it

**May:** one rule — surrogate eligibility multiplied by a scalar ±1 reward — fails a
matched gate that the rest of the field saturates. That is a clean, mechanism-
labelled negative, and the label is **broadcast ±1 three-factor**, not "spiking
failed" and not "broadcast credit topology."

**May not, and this is the binding limitation:** because the reference sits at
exactly 1.0000, `gap_closed` reduces every pass to *"the arm scored above 0.75."*
**No ordering among the passing arms may be claimed.** Five of seven arms sit
between 0.88 and 1.00. The task separates one failure from a field it can no longer
rank, and that is both weaker and better evidenced than a graded richness ordering.

**Honesty contrast.** Broadcast-*graded* error reaches 0.9975 on the same schedule.
So coincidence alone does **not** prove credit locality is required, and any
phrasing that bans "broadcast" as a topology misreads this cell. The evidence for a
locality flip is one-layer XOR (broadcast 0.501, DFA 0.827, gradient 0.773); it does
**not** reappear at mid-depth (broadcast 0.816, DFA 0.825, rl_fb 0.803), so depth
help ≠ locality flip.

**The graphs are two comparisons, not one.** They disagree by more than the
registered 0.02 bar on the spike-adjoint arm (0.0550) and on RL graded-reward
(0.0313). "The same forward family" is false as written, and both are reported.

**Ceiling health.** On the archived instrument the canonical 80-epoch schedule
undertrained the reference (0.8963 / 0.9013 at e80, climbing to 1.0000 by e640), so
that comparison read as *learning speed*. On the repaired instrument the reference
reaches 1.0000 at e80 itself — hence there is **no budget at which this task
separates the arms**, and no ceiling comparison on it survives. Undertraining is
excluded as the cause of the lead FAIL: at 4× epochs (v22) the arm is still at
0.5000.

---

## 6. The substrate-transfer negative

### 6.1 The question

The matched gate runs on a dense-LIF forward. BINN's actual substrate is
event-driven, with hard k-WTA competition and θ = ∞ muting of losers. Do the rules
that pass the matched gate work *there*?

### 6.2 Result: they do not

Twelve preregistered gap-close variants, each on its own hash, against a G2
threshold of gap LCB > 0.5:

| Protocol | Hash | Local acc. | Gap LCB | Verdict |
|---|---|---:|---:|---|
| v13 live REINFORCE-FB | `c1-660401d74db3c88d` | 0.4900 | 0.0737 | FAIL |
| v14 epoch | `c1-714c115e14a3eeed` | 0.4838 | −0.0100 | FAIL |
| v15 structured `B` | `c1-493ddd56f8714fb6` | 0.7262 | 0.2567 | FAIL |
| v16 structured × epoch | `c1-677df7f7cbe4f8ec` | 0.5200 | 0.0844 | FAIL |
| v17 structured × capacity | `c1-983ee5303c00b147` | 0.6825 | **0.3127** | FAIL |
| v18 eligibility × REINFORCE | `c1-c7d2c86a2b1927f6` | 0.7125 | 0.2351 | FAIL |
| v19 structured × teach | `c1-dfab4a7ec19f17c2` | 0.6700 | 0.2238 | FAIL |
| v20 live DFA | `c1-4db53e645405fae0` | 0.7325 | 0.2601 | FAIL |
| v21 soft-WTA × SFB | `c1-f975db8fb3e5d569` | 0.5025 | 0.0406 | FAIL |
| v23 finite-θ SFB | `c1-4bbaf4b24c2d1da2` | 0.6638 | 0.2370 | FAIL |
| v24 continuous `B` | `c1-840f820b7c07b512` | 0.6437 | 0.1380 | FAIL |

**Reading.** Structured `B` is the accuracy lever — it clears the accuracy floor —
but capacity × structured gives the best gap LCB at 0.3127, still well short of 0.5.
Epochs under structured `B` *regress*. Eligibility co-design and restored target
teach do not beat v15. Softening the winner-take-all (v21) and removing the mute
(v23) both fail. Under the canonical protocol-v2 hash `c1-118207fbc3eaba53` the
local arm reads 0.4912 with gap LCB −0.0048 while a multi-epoch surrogate reference
on the same frozen splits succeeds.

This is packaged as a **substrate/pipeline transfer negative**, not as a biological
result and not as an Assembly Calculus result.

### 6.3 Integrity caveats on the canonical gate

The canonical v2 protocol carries known defects, each fixed under a *new* hash
rather than by reinterpreting v2:

| ID | Finding | Status |
|---|---|---|
| H1 | `ThreeFactor.last_spike` not cleared across trials | Fixed under `c1-iso`/v5; **still true on canonical v2** |
| H2 | Incomplete membrane reset | Fixed under `c1-iso`/v5; **still true on canonical v2** |
| θ=∞ mute | Hidden thresholds set to infinity during integrate | Removed under `c1-spike-*`/v6, `c1-spike-s-*`/v9; still true on v2 |
| `project` unused | Assembly Calculus `project` not on the crux | Wired under `c1-project-*`/v7 — **scientific FAIL** |
| e-prop naming | Exact-forward "e-prop" ≠ textbook e-prop | True σ′ e-prop under `c1x-eprop-true-*` (mean 0.7125) |

The natural-spiking variants (`c1-spike-*`) report **INVALID_HARNESS**, not a
verdict — the instrument refuses to score a run whose health checks fail. That
refusal is deliberate and is the mechanism by which §10's withdrawals were caught.

---

## 7. The conditional positive: what a time-axis read-out buys on SHD

### 7.1 Setup and the claim's exact shape

A causal self-attention read-out is placed over feed-forward LIF spiking features.
Anchor: h128 / `published-2ms` / `adjacent-sum-5` / e400 / `d32/L4`.

Accuracy rises from 0.7057 (`ff+fixed`) to **0.8332** (`ff+fixed+attn`), gain
**+0.1275**, positive in **32/32** seeds and at or above 0.80 in **32/32**.
Registered at n = 12 as 0.8320 vs 0.7062, gain +0.1258, 12/12 ≥ 0.80. *n = 12 is the
registered measurement; n = 32 is the confirmation, and both are reported —
twenty extra seeds moved the gain by +0.0017.*

**None of that is the claim.** Both "attention helps on SHD" and "SHD depends on
temporal order" are prior art [11][r][12][r][13][r][14][v][15][r]. What has not been
measured is **which component's marginal contribution is the order-dependent one**.

### 7.2 The difference-in-differences

Time bins are permuted **per sample, in both the training and the test split**,
with separate seed lineages — so the task itself becomes genuinely rate-solvable
rather than merely corrupted, which removes the distribution-shift confound of a
test-time-only probe. Every shuffled cell passes a temporal audit (spike counts
preserved, relocated fraction ≥ 0.5).

| Arm | intact | shuffled | cost |
|---|---:|---:|---:|
| `ff+fixed+attn` | 0.8332 | — | **+0.1347** |
| `ff+fixed` | 0.7057 | — | **+0.0142** |

A **9.5× ratio, positive in 32/32 seeds**. Equivalently the read-out's *advantage*
falls from +0.1275 to **+0.0070**: **94.5% of the read-out's marginal contribution
is contingent on temporal order.** Registered at n = 12: +0.1337 vs +0.0128, a 10×
factor, advantage +0.1258 → +0.0050, 96%.

**This is a statement about the gain, never about accuracy.** Under shuffling the
attention arm still scores 0.6983 against the rate arm's 0.6934. *"96% of accuracy
depends on temporal order" is false* and is an explicit non-claim.

### 7.3 It generalises — and its registered size prediction failed

The same operator, seeds and pinned binary at seven further operating points: **168
cells, zero divergences, zero voided.** The DiD clears its +0.03 bar at 12/12 seeds
at h256 (+0.0862), h384 (+0.0767) and h512 (+0.0968), and at both alternative
binnings — `channels-700` (+0.1122) and `published-10ms` (+0.0959). Coverage goes
from 2 of 21 operating points to **9 of 21**.

The same preregistration asked whether the shuffle cost *tracks* the gain across
width. **It does not.** Spearman ρ = **−0.1430** against a registered bar of
**+0.829** — a registered **NOT MET**. h768 carries the smallest gain on the ladder
(+0.0560) and the largest DiD in the wave (+0.1881). So *"the read-out's
contribution is order-dependent"* survives and is measured at nine points;
*"the gain is made of temporal order"* does **not** survive as a quantitative
account, and we do not assert it.

### 7.4 Scope limits, stated as results

**The width collapse is a threshold, located but unexplained.** Gains across six
rungs: +0.1258 (h128), +0.0966 (h256), +0.0760 (h384), +0.0876 (h512), +0.0560
(h768), **−0.1618** (h1024). The drop into h1024 is 0.2178 — **6.9×** the largest
gap below it — so it is a threshold between h768 and h1024, not the slope
continuing. Three preregistered rescue levers are all negative and all *worse* than
the arm they were meant to rescue: surrogate scale 0.5 → −0.2106, 0.25 → −0.2565,
gradient clipping at 1000.0 → −0.0904. Clipping moved the median epoch-mean
gradient norm from 55.494 to 11.660 — a real effect in the intended direction — and
accuracy did not follow. A registered prediction said that if the collapse and the
temporal-order account were one phenomenon, the shuffle cost should vanish where
the gain does. **It did not**: DiD(h1024) = +0.1122 in 10 of 12 seeds against a
+0.02 ceiling, while the gain over those same seeds is −0.1618. *The read-out
consumes temporal order while performing worse than no read-out at all.* We offer
**no mechanism**; overfitting on 8,156 training samples is neither excluded nor
supported. This is the package's leading open problem.

**The collapse is late, not intrinsic.** At the same width and depth, truncating to
e100 turns the gain from −0.1318 into +0.0827 in 12/12 seeds, and the arm retains
its fit in 12/12 where at e400 it loses it in 63 of 68. A `d32/L2` control moves
only +0.0149 over the same budget change, so this is specific to the collapsing arm.
Why the fit is lost remains unexplained.

**0.8332 is not competitive, and we concede it in the text.** The frontier is
95–96.4% via learned delays [16][v], adaptation and spiking transformers. This
instrument carries **no temporal kernel of any kind** and lands close to where the
dataset's own authors put a no-delay recurrent baseline (83.2 ± 1.3% at 1024
neurons with augmentation) [11][v]. Against a pinned third-party calibration
reference (0.9390 / 0.9368 / 0.9371) the 0.087 residual is attributed — by
elimination and code-reading, **not by an ablation that added the kernel** — to a
25-tap-per-synapse learned temporal kernel. **That is the weakest load-bearing
inference in this package**, and we label it as such.

**Benchmark caveat.** SHD ships no validation set; the same model validated on test
reads 95.81 ± 0.56 against 93.79 ± 0.76 with a proper held-out split. Differences
below ~1.5 points between published SHD numbers are not reliably meaningful — this
work's included.

**Further limits.** The 0.80 clearance is geometry-specific (0.7864, 6/12, on
`channels-700`). Temporal *resolution* points the other way: on `fixed-tN`, which
holds a 1400 ms window fixed, the gain **shrinks** with finer bins (+0.1927 at
14.0 ms → +0.1751 at 5.6 ms → +0.1474 at 2.8 ms, difference −0.0453); the earlier
`published-Nms` result is **refuted and withdrawn** because it moved bin width and
sequence length together. The read-out does **not** substitute for substrate
temporal state: adaptation is inert at the anchor (+0.1258 vs +0.1285, difference
+0.0027, 6/12), and on a recurrent substrate the gain roughly doubles (+0.2612 vs
+0.1201, difference +0.1411, 10/10) — substitution is refuted on both axes.
`rec+alif` nonetheless does **not** win absolutely (0.7874 vs 0.8289), is measured
from a lower base (1.34× headroom-normalised), and rests on ten pairs at the
registered minimum. **The instrument is uncalibrated**: criterion 5 (a Python
mirror) is unmet, so every number here is a within-instrument, same-machine
comparison.

### 7.5 Falsifier

A bin-shuffle contrast on the same instrument, anchor, seed lineage and both-splits
operator in which the attention read-out's shuffle cost **fails to exceed** the rate
read-out's — or in which intact-minus-shuffled falls below the registered +0.05 bar,
or is positive in fewer than 10 of 12 seeds — overturns the lead. So does a
destruction operator that leaves temporal order intact while destroying something
else (rate, synchrony, channel identity) and reproduces the same collapse. Widening
the seed pool, switching anchor geometry, or substituting a test-time-only shuffle
does **not** count.

---

## 8. Post-G2 harvest: two further negative gates

These ran only under explicit opt-in flags and **do not reopen G2**.

**G3 — continual learning (C2).** Under class-incremental training, local plasticity
alone yields mean forgetting **0.8948** against a replay baseline's **0.2725**.
Plasticity does not prevent catastrophic forgetting without replay.

**G4 — multi-area scaling (R2), hash `r2-afafa0fa6f43e3fc`, NO-GO.** Capability
against area count fits **capability ≈ −0.1924·ln(n) + 1.1673** (R² = 0.985) — a
*degrading* curve. Area composition does not compound capability without hierarchy.
The frozen G4 NO-GO is not remassaged by the separate directed-credit hypothesis
(`r2-credit-*`), which runs on its own hashes.

### 8.1 A note on the repository README

The README's results table is **stale** against the claim freeze and should not be
cited: it reports the matched arms at DFA 0.9387 and RL 0.9200 (pre-repair figures
produced on a zero-spike forward pass), and it cites `c1-*` config hashes that are
**retired by design** — `MATCHED_INPUT_SCALE` was never mixed into them, so each
named two different experiments either side of the repair. The current figures are
those in §5.2. We record this here because a reader arriving via the README would
otherwise cite withdrawn numbers.

---

## 9. Explicit non-claims

Reproduced from the claim freeze, because summarising them away is how a claim
ladder degrades:

**On the SHD program.** No novelty for temporal attention on SHD [14][v][15][r], nor
for SHD's dependence on temporal order [11][r][12][r][13][r]. Not "96% of *accuracy*
depends on temporal order" — the fraction is of the **gain**. Not competitive
accuracy, and the 0.087 calibration residual is not a tuning gap. **No mechanism for
the h1024 collapse**, and no dip claimed at h384 (−0.0116, sd 0.0253 — inside its own
noise). No claim of instrument calibration. No temporal-*resolution* mechanism (S-5
withdrawn; `fixed-tN` moves the opposite way). No recurrent-substrate win.

**On the matched and engine programs.** No biology or cortex. No Assembly Calculus
PASS (`project` wired under `c1-project-*` and FAILs). No natural-spiking G2 verdict
(`c1-spike-*` are INVALID_HARNESS). No neuromorphic-hardware claim. **No impossibility
in principle** — these are scoped operationalised negatives. No reopening of frozen
hashes by threshold massage. No widening of the lead FAIL to "any broadcast."
Undertraining is not the cause (v22 at 4× epochs is still at chance). **No ranking
among the passing matched arms.** No live-engine rescue from a matched PASS. No
claim that the discrete spike-adjoint is a negative result, and none that it is
equivalent to continuous EventProp [5][r]. No digital-brain or brain-equivalence
claim.

---

## 10. The withdrawal ledger

Seven results have been withdrawn from this package. We report them because a
record that only shows surviving results tells the reader nothing about how hard
the instrument was tried.

**Episode 1 — the 2026-08-19→22 record repair.** Four results withdrawn, three of
them PASSes:

- **`track-b-rescue` v130 online learned feedback alignment, PASS at 1.0000 (gap LCB
  0.9988) — withdrawn.** At v131 the arm reports `INVALID_HARNESS`: a
  ceiling-inverted warning fires on 3 of 20 learned-feedback seeds and the code
  **refuses to emit a PASS while it is present**. Re-read under both repairs it is
  1.0000 against a ceiling of 1.0000 with zero variance — a *saturation* result, not
  a credit-assignment one.
- **The depth-collapse / deep-SNN scaling result — withdrawn** (v134
  `INVALID_HARNESS`: every depth-matched gradient ceiling was at chance).
- **`shd-scientific-sweep` — withdrawn.** It ran on synthetic 24-channel /
  16-timestep data and **never loaded SHD**.
- **The `live-transfer-rescue` arms — `INVALID_HARNESS`**, and the protocol was
  misnamed: it is matched-only, not live-engine.

Separately, three gradient *references* were found at or near chance on tasks their
own treatments solve; two are diagnosed (`MatchedDeepGradient` collapses to silence;
`ShdEpropCeiling` is a constant predictor by a different mechanism). None is used.

**Episode 2 — the 2026-08-25 matched re-run.** Every previously published
matched-architecture number was produced on a forward pass that emitted **zero
spikes at any seed**. Three more results were withdrawn:

- **The discrete EventProp-style spike-adjoint FAIL** (0.5000 → 0.9450 / 0.8900
  PASS). The failure mode is instructive: *a method whose entire mechanism is the
  spike had no spikes to differentiate through*, while every other arm could still
  separate classes by sub-threshold membrane rate. The defect was maximally
  misleading precisely on the arm that most depended on the broken quantity — and
  the prior explanation offered for that number ("discrete hard spike-gate adjoint ≠
  continuous Wunderlich–Pehle") was an explanation for an artefact, and is retired
  with it.
- **Both RL broadcast contrasts** (0.5250 → 0.9100; 0.5113 → 0.7962). The reading
  they supported — "continuous magnitude without spatial directionality is
  insufficient on this gate" — no longer has evidence behind it.

**The lead negative survived both episodes** on both forward graphs, which is the
only reason it is reported at all.

**What the record repair changed structurally.** Configuration hashes that did not
mix in a semantically load-bearing constant (`MATCHED_INPUT_SCALE`) were found to
name *two* experiments each; they are now retired rather than silently
reinterpreted. Ceiling-health checks were added such that a reference at chance
**fails the harness** instead of scoring the treatment against it. Machine-checked
prose assertions (125 of them) were added over the SHD campaign.

**The generalisable lesson.** Every one of these defects produced *plausible*
numbers. None was caught by an arm's own accuracy; all were caught by
instrument-health checks that ask a different question — "could this run have
produced this number for the wrong reason?" A harness that scores whatever it is
given will eventually score an artefact, and a check that cannot run must never
report the same result as a check that ran and passed.

---

## 11. Discussion

The instrument was built to test whether local, sparse, event-driven learning is
competitive without backpropagation. On its own terms it answered: **not here, not
by these rules, not on this substrate.** The matched gate isolates a genuine
failure — broadcast ±1 three-factor cannot assign credit where a graded or
spatially addressed signal can — but it also saturates, which means it can name one
failure and cannot rank the survivors. The transfer suite shows that passing on a
dense forward buys nothing on a hard k-WTA substrate: twelve variants, best gap LCB
0.3127 against 0.5. The composition gate points the wrong way entirely.

The one thing that worked is a **read-out**, not a learning rule — and its
contribution is measured, honestly, as a conditional about the *gain* rather than a
headline about accuracy that would not have been ours to claim. That result's own
scope limits are the more interesting part: an unexplained threshold collapse at
h1024 where the read-out still consumes temporal order while performing worse than
no read-out at all, and a registered prediction about the effect's *size* that
failed.

We think the most transferable output is methodological. A research programme that
can withdraw four PASSes — including its own most spectacular result, a 1.0000 —
and still report a surviving negative, is one whose surviving results mean
something. The zero-spike forward pass is the cautionary case: it ran for weeks,
produced publishable-looking numbers across an entire suite, and was invisible to
every experiment's own verdict.

### 11.1 Open problems

1. **The h1024 threshold.** Located between h768 and h1024, unexplained, three
   rescue levers negative, order-dependence persisting through the collapse.
2. **Calibration.** Criterion 5 (Python mirror) is unmet; until it is met, no number
   here is comparable to an externally recorded one.
3. **The kernel attribution.** The 0.087 calibration residual is attributed to a
   25-tap learned temporal kernel by elimination, not by an ablation that added it.
4. **Whether any local rule crosses the k-WTA transfer barrier**, given that
   structured `B` moves accuracy but not the gap.

---

## 12. Reproduction

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
./scripts/gc_checks.sh          # GC1-GC7
./scripts/record_checks.sh      # 125 SHD-campaign assertions
```

Canonical Gate G2 replay:

```bash
cargo run --locked --release -p binn-lab --bin c1 -- \
  --config-hash c1-118207fbc3eaba53 --out results/c1_g2_replay.md
```

Post-G2 gates require explicit opt-in (`--enable-c2`, `--enable-c3`, `--enable-r1`,
`--enable-r2`). **The `c1-match-*` / `c1-dfa-*` / `c1-rl-*` hashes printed in the
README are retired and will not resolve**; the current matched figures come from the
2026-08-25 re-run recorded in `RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` and
`matched_rerun_2026-08-25/`.

---

## References

**[v] = retrieved and checked against a primary index during preparation of this
manuscript. [r] = inherited from the repository's 2026-08-27 literature pass and
NOT independently verified here — check against the primary source before
submission.**

1. **[v]** J. Launay, I. Poli, F. Boniface, F. Krzakala. *Direct Feedback Alignment
   Scales to Modern Deep Learning Tasks and Architectures.* arXiv:2006.12878, 2020.
2. **[v]** N. Shervani-Tabar, R. Rosenbaum. *Meta-Learning Biologically Plausible
   Plasticity Rules with Random Feedback Pathways.* arXiv:2210.16414, 2022.
3. **[r]** G. Bellec et al. *A solution to the learning dilemma for recurrent
   networks of spiking neurons* (e-prop). Nature Communications, 2020.
4. **[v]** F. Zenke, S. Ganguli. *SuperSpike: Supervised learning in multi-layer
   spiking neural networks.* arXiv:1705.11146, 2017.
5. **[r]** T. Wunderlich, C. Pehle. *EventProp: Backpropagation for Exact Gradients
   in Spiking Neural Networks.* 2021. *(Retrieval attempt during preparation timed
   out; cited as inherited.)*
6. **[v]** M. Dabagia, C. H. Papadimitriou, S. S. Vempala. *Assemblies of neurons
   learn to classify well-separated distributions.* arXiv:2110.03171, 2021.
7. **[v]** D. Maoutsa. *Meta-learning three-factor plasticity rules for structured
   credit assignment with sparse feedback.* arXiv:2512.09366, 2025.
8. **[v]** W. van der Veen. *Including STDP to eligibility propagation in
   multi-layer recurrent spiking neural networks.* arXiv:2201.07602, 2022.
9. **[v]** M. Traub, M. V. Butz, R. H. Baayen, S. Otte. *Learning Precise Spike
   Timings with Eligibility Traces.* arXiv:2006.09988, 2020.
10. **[v]** M. Dabagia, C. H. Papadimitriou, S. S. Vempala. *Computation with
    Sequences in a Model of the Brain.* arXiv:2306.03812, 2023.
11. **[v]** B. Cramer, Y. Stradmann, J. Schemmel, F. Zenke. *The Heidelberg spiking
    datasets for the systematic evaluation of spiking neural networks.*
    arXiv:1910.07407, 2019; **[r]** published as IEEE TNNLS 33(7), 2022 — the 60%
    spike-count-only figure and the 83.2 ± 1.3% no-delay recurrent baseline are
    cited from the repository's pass.
12. **[r]** *Neuromorphic Sequential Arena.* IJCAI 2025. (SHD 86.48 → 68.51 with
    temporal processing removed model-side.)
13. **[r]** Yu et al. arXiv:2507.16043, 2025.
14. **[v]** M. Yao, H. Gao, G. Zhao, D. Wang, Y. Lin, Z. Yang, G. Li.
    *Temporal-wise Attention Spiking Neural Networks for Event Streams
    Classification* (TA-SNN). arXiv:2107.11711, ICCV 2021. (91.08% on SHD **[r]**.)
15. **[r]** *STSC-SNN: Spatio-Temporal Synaptic Connection with Temporal
    Convolution and Attention.* 2022. (92.36% on SHD.)
16. **[v]** B. Mészáros, J. C. Knight, T. Nowotny. *Efficient Event-based Delay
    Learning in Spiking Neural Networks.* arXiv:2501.07331, 2025.

---

## Appendix A — primary on-disk anchors

| Result | Document |
|---|---|
| Claim freeze (authority) | `PUBLISHABLE_CLAIMS.md` |
| SHD camera-ready prose | `PAPER_DRAFT.md` |
| Cross-referenced numbers | `PAPER_RESULTS_TABLE.md` |
| Matched re-run (current figures) | `RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`, `matched_rerun_2026-08-25/` |
| Registered DiD (n=12) | `RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md` |
| Confirmation + threshold (n=32) | `RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md` |
| Generalisation wave | `RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md` |
| Late-collapse wave | `RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md` |
| Ceiling health | `RESULT_2026-08-19_A6_CEILING_HEALTH.md` |
| v130 withdrawal | `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` |
| Depth-collapse withdrawal | `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md` |
| Synthetic-sweep defect | `DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md` |
| Forward-pass difference | `FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md` |
| Record repair summary | `SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md` |
| G3 continual | `c2_g3.md` |
| G4 scaling | `r2_scaling.md` |
| Live transfer | `MATCHED_ARCH_LIVE_REINFORCE.md`, `GAP_CLOSE_RFB_TRANSFER.md`, `DIFF_CLOSURE.md` |
