# Hard Audit — All Claims and the Plan (v7)

## An adversarial pass over v1–v6. What survives, what's overstated, what's wrong.

**Date:** 22 July 2026
**Scope:** every load-bearing claim in `Foundation_Reinvention_Plan.md` (v1), `_v2`, `_Scaling_Plan_v3`, `_Gap_Analysis_v4`, `BINN_Build_From_Scratch_v5`, `BINN_Project_Plan_v6`.
**Method:** re-derived the math, fact-checked against 2025–26 literature, and looked specifically for places where I argued *toward* the conclusion I wanted.

**One-line verdict up front:** the *plan* is sound because it is gated to fail cheaply; several *claims* used to motivate it are overstated, one is technically wrong, and one is an unresolved contradiction. None of the corrections change the go/no-go structure, but three of them should change what you expect and how you build. The honest prior remains: **this most likely fails at the crux gate — and that is still worth the ~1–2 months to find out.**

---

## 1. Findings by severity

### CRITICAL — a claim that is technically wrong

**F1. The "parallelize membrane recurrence with an associative scan" claim (v5 §L5, v6 §4.1/4.4; the headline mitigation for gap B1) is wrong as stated.**

I claimed the membrane recurrence can be expressed as a linear/associative scan so time computes in parallel during training. Reality: **only the *sub-threshold* dynamics are linear and scan-parallelizable. The hard reset after a spike introduces a sequential, data-dependent dependency that provably blocks full parallelization** — it is *the* fundamental barrier to parallel SNN training. Workarounds exist but each has a cost: process in *chunks* (parallel within a chunk, sequential across chunks — partial, not full), or *remove the reset entirely* (the "Parallel Spiking Neuron" trick, which turns the neuron into a linear filter and discards the spike-reset dynamics that are half the point), or specialized discretizations.

**Correction:** B1 is *mitigated in part*, not solved. Rewrite the claim as: "sub-threshold dynamics parallelize via chunked associative scans; the reset remains a sequential barrier, so training-time parallelism is partial and comes with a modeling trade-off." This does not sink the project (the primary learner is online-local, which sidesteps BPTT anyway), but the plan must stop presenting scan-parallelism as a clean escape.

---

### CRITICAL — overstated dodges and an unresolved contradiction

**F2. "Online local learning is O(1) in time and *dodges* B1" (v4 §3, v5, v6) is overstated.**

Online local learning dodges the *backward unroll* and the *activation-storage memory* of BPTT — that part is true and real (O(1) memory in sequence length). But it does **not** dodge the *sequential forward simulation*: events still have to be processed in time order, so single-stream throughput is still time-sequential. You parallelize across **neurons, areas, and independent streams**, not across time within a stream.

**Correction:** B1 is *halved*, not dodged — memory/credit yes, forward-time-throughput no. Every "dodges B1" in v4–v6 should read "removes the backward-unroll half of B1; the sequential-forward half remains, mitigated by neuron/area/stream parallelism."

**F3. The encoder/decoder co-training story contradicts the "no autodiff, no backward graph" purity rule (v5 §L6, v6 §4.5 vs. v5 §5 / v6 guardrails). Unresolved.**

I said encoders/decoders are "learned, co-trained, first-class model components," and separately I forbade any backward graph in the production path. These conflict: if the encoder is a learned neural map, how is it trained without the very autodiff the plan bans? I hand-waved this.

**Correction (must be a real design decision, not prose):** pick one — (a) encoders/decoders learn via the *same* local three-factor rule (intellectually consistent but unproven for input encoders, and probably weaker); (b) allow a small, explicitly-labeled *autodiff island* just at the I/O boundary, accepting it is not part of the "pure" substrate; or (c) use fixed, hand-designed, information-preserving encoders and put all learning inside the substrate. Option (c) is the cleanest for the crux experiments and should be the default for P0–P3; revisit (a)/(b) only later. Until this is chosen, v5/v6 contain a live contradiction.

---

### CRITICAL — theory cited beyond what it proves

**F4. Assembly Calculus is invoked as theoretical support for the "local credit is sufficient" bet (v2, v4 gap E2, v6 §4.3), but its proven results cover a weaker regime than the bet needs.**

What Assembly Calculus actually proves: (i) projection converges to stable assemblies; (ii) it can, in principle, carry out arbitrary bounded computation (a few hundred parallel steps / registers); (iii) it can form class assemblies that are reliably recalled **as long as the classes are "reasonably separated."** What it does *not* establish: competitive learning on **not**-well-separated, deep, compositional tasks — which is exactly the hard regime real ML lives in and exactly where my crux bet is placed. There are also documented expressiveness limits (an Assembly-Calculus parser is *weaker than a finite automaton* — it can't handle Kleene closures, and same-type assemblies collide).

**Correction:** the theory is real and valuable, but it supports "assemblies can represent and separate well-separated classes with local plasticity," not "local plasticity is sufficient for hard compositional credit assignment." Stop letting it *de-risk* the crux. The crux (G2/C1) remains a genuinely open empirical question, and F4 means the prior should be *less* optimistic than v2 implied, not more.

---

### MODERATE — overclaims needing qualification

**F5. "1–2% activity → 1–2% of the work" (v2, v3, v6 metric) ignores per-event overhead.** Each event costs queue operations and cache-missing pointer chases; the realized speedup is materially less than the naive activity ratio, and on CPU/GPU the honest figure is the ~65–75% band v4 already cited — with the large multipliers only on neuromorphic hardware. The v6 success metric should say "work-per-accuracy including per-event overhead," not imply linear-in-activity savings.

**F6. "R2 answers the whole scaling question cheaply" (v3 §4, v6 G4) overstates extrapolation.** A healthy capability-vs-#areas curve over a few hundred areas is a *strong first signal*, not proof it continues to 10⁴–10⁶ areas. Scaling laws can bend. Correction: G4 yields "justification to invest in the next order of magnitude," not "the trillion-node question is settled."

**F7. The 18-week timeline (v6 §6) is optimistic for 1–2 engineers building everything from scratch.** A deterministic event engine + areas + a novel learner + a data/encoder layer + a harness, *plus* meaningful experiments, is more realistically a 6–9 month effort to reach a trustworthy G2/G4, and P0→P2 alone is likely 2–3 months for one engineer. Treat v6's weeks as an aggressive best case; plan in months.

**F8. Brain-scale synapse gap understated.** v3 says brain-scale is "~1–2 orders" from today's hardware. By *neurons* (86B vs Hala Point's 1.15B) that's ~2 orders; by *synapses* (~100T vs 128B) it's closer to **~3 orders**. Synapses, not neurons, are the binding resource. Minor, but state it accurately.

**F9. "Both outcomes are wins; the negative is publishable" (v2–v6) is motivated optimism.** A clean negative is scientifically valuable, but a null result about an obscure substrate is not guaranteed to be publishable or career-rewarding. The *decision-theoretic* framing ("cheap information either way") is sound; the "both are wins" gloss oversells the downside.

---

### MINOR — nuances, not errors

- **F10. "A single neuron ≈ a deep network"** (v2, v4) is fair but should be stated precisely: a *temporal-convolutional network of moderate depth is needed to predict a cortical neuron's input-output mapping* — a complexity measure, not a claim that neurons are trained like DNNs. As used, fine.
- **F11. The `k²/N` overlap math is correct** (expected overlap of two random k-subsets of N). No issue.
- **F12. Novelty is incremental synthesis, not a paradigm shift** — v2's novelty ledger already said this honestly; keep that framing and don't let v5/v6's confident tone drift away from it.

---

## 2. Two structural findings about the whole body of work

**S1. Everything after the crux is contingent, and the later documents don't wear that contingency visibly enough.** v3 (scaling), the v4 mitigations, v5 (build), and v6 (plan) are *all* downstream of G2 passing. If the sufficiency bet fails at the crux, roughly 80% of v3–v6 is moot. The plan is correctly *gated* so this costs little — but the documents are written with a forward confidence that can read as if the foundation were settled. Every post-G2 artifact should carry a one-line "contingent on G2" banner so no one mistakes a detailed plan for a validated one.

**S2. The body of work is confirmation-friendly.** Across v1→v6 I repeatedly surfaced reasons the bet *could* work and then gated the risks. But the base rate matters: replacing backprop with a local rule that scales has been attempted for decades and has not succeeded. The honest Bayesian prior is that **G2 probably returns a negative.** That is not a reason to abandon the program — the whole design is "learn the answer cheaply" — but the *expected* result should be stated as likely-negative, and the enthusiasm in v2/v5 should be read against that prior. This audit exists partly to counterweight that lean.

---

## 3. What actually survives

Stripped of overstatement, here is the defensible core:

1. **The diagnosis holds.** Credit assignment, not the neuron model, is the real foundation; the field's local rules plateau; leading with the substrate (v1) was the wrong emphasis. (Solid.)
2. **The representation-first bet is coherent and worth testing.** Sparse assemblies → low interference → possibly-sufficient local credit is a legitimate, falsifiable hypothesis — but supported by *less* theory than v2 implied (F4), so it is a genuine bet, not a near-thing.
3. **The from-scratch decision is correct** for the stated goal, because the disadvantaging gaps are ecosystem inheritances — with the caveat that the I/O boundary can't be fully autodiff-free without a real decision (F3).
4. **The gate structure is the strongest part of the whole body of work.** G2 (kill) and G4 (decision) reached early, cheap, and decisive-enough. This is what makes the program rational even under a likely-negative prior.
5. **The efficiency case is real but modest in software** (F5), large only on hardware that exists (Hala Point) but is a separate program.

What does *not* survive unqualified: clean scan-parallelism (F1), "dodges B1" (F2), theory de-risking the crux (F4), linear-in-activity efficiency (F5), R2 settling the scaling question (F6), and the 18-week timeline (F7).

---

## 4. Required edits (punch list)

| # | Document | Change |
|---|---|---|
| F1 | v5 §L5, v6 §4.1/4.4 | Reword scan-parallelism as *partial* (reset is a sequential barrier); cite the chunked-scan / reset-removal trade-offs |
| F2 | v4 §3, v5, v6 | Replace "dodges B1" with "removes backward-unroll half; forward-time remains, parallelized across neurons/areas/streams" |
| F3 | v5 §L6, v6 §4.5 | Resolve encoder training: default to fixed information-preserving encoders for P0–P3; label any autodiff island explicitly |
| F4 | v2, v4 E2, v6 | Downgrade Assembly-Calculus support to "well-separated classes / bounded computation"; stop de-risking the crux with it |
| F5 | v2, v3, v6 metric | Efficiency = work-per-accuracy *including per-event overhead*; software win modest, big win HW-only |
| F6 | v3 §4, v6 G4 | G4 = "justifies next order of magnitude," not "settles trillion-node question" |
| F7 | v6 §6 | Reframe timeline in months; 18 weeks = aggressive best case |
| F8 | v3 §2 | Synapse gap to brain ≈ 3 orders, not 1–2 |
| S1 | v3–v6 | Add "contingent on G2" banner to every post-crux document |
| S2 | v2, v5 | State the likely-negative prior explicitly |

None of these touch the build order or the gates. They correct the *claims*, tighten the *expectations*, and fix *one real technical error* (F1) and *one contradiction* (F3) before either can cost engineering time.

---

## 5. Recalibrated bottom line

The plan is good and should proceed **as gated** — but proceed with three corrections front-of-mind: (1) do not count on parallel-in-time training to rescue throughput; the reset is a hard sequential barrier, so plan for neuron/area/stream parallelism instead. (2) Decide the encoder-training question before P2 or it will silently contaminate the crux result. (3) Carry a likely-negative prior — the value of this program is *cheap, decisive information*, not a probable breakthrough. Build P0, run to G2, and let the crux experiment — not the elegance of the story — decide whether any of v3–v6 matters.
