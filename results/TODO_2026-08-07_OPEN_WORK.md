# Open work — 2026-08-07

Supersedes the "Still open" register in `SUMMARY_2026-08-03.md` §6, which
predates the geometry closure, the H1-at-convergence extension, the surrogate
amendment and the scoring-path audit.

Ordering is by blocking relationship, not by interest. Items in §1 can
invalidate items in §3, so they come first.

Status key: `[ ]` not started · `[~]` in flight · `[x]` done · `[!]` blocked on a
human decision.

Amended 2026-09-05: §9 added (external reading — sparse reward subsystem in
LLMs — read against the record, with gaps and candidate experiments). §9
schedules nothing and is not evidence.

Amended 2026-09-07: **the two-manuscript split is closed — there is now one
paper.** `PAPER_BINN_INSTRUMENT_2026-08-31.md` was the superset and
`PAPER_DRAFT.md` its §7 expanded; the superset stopped at wave 23 and **no
verification script had ever read it**. Its four load-bearing sections were
merged into `PAPER_DRAFT.md` (§2.4 preregistration and kill-gates, §4.7 and
Appendix C the withdrawal ledger, §4.8 open problems, Appendix D the secondary
programme's non-claims, Appendix H the reproduction commands), which builds at
9 content pages of 9 and is swept by every check. The instrument document now
carries a SUPERSEDED banner naming the three claims in it that are no longer
true. This register never listed the split as open work, which is why it stood
for seven days; `scripts/test_paper_number_sweep.py` now fails if any
abstract-bearing document is neither swept nor bannered.

---

## 1. Record repair — blocking everything downstream

Source of truth: `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md`.

The pattern this section exists to close: **a fix that is not re-run is not a
fix.** Three binaries were hardened on 2026-07-25 and none of their reports were
regenerated. The corrections live in Rust doc comments; the uncorrected numbers
live in the paper.

- [x] **Re-run `track-b-rescue` at v131.** *(2026-08-19.)* Done — the v130 PASS
      is **withdrawn**; warning fired on 3/20 seeds, arm reports `INVALID_HARNESS`.
      See `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` and
      `track_b_results_v131.md`.
- [x] **Correct or withdraw the six citations of the v130 PASS.** Done — all
      six documents (`PAPER_DRAFT.md`, `PAPER_RESULTS_TABLE.md`,
      `PAPER_SKELETON.md`, `PUBLISHABLE_CLAIMS.md`,
      `CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`, `REPRO_ARTIFACT_CHECKLIST.md`)
      updated to reflect withdrawal.
- [x] **Re-run `deep-snn-scaling` at v134.** *(2026-08-20.)* Done, and the
      answer is worse than the defect it was meant to fix. Every depth-matched
      ceiling is at chance — 0.4880 / 0.5000 / 0.5000 / 0.5000 on a two-class
      task — including depth 1, on splits the treatment solves at 1.0000 in the
      same process. Modulator RMS is healthy and near-identical across depths
      (5.03e-1 to 5.04e-1), so the hypothesised mechanism is **ruled out**. The
      suite is `INVALID_HARNESS` on measured evidence.
      See `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`.
- [x] **Restate or withdraw the depth-collapse result.** *(2026-08-20 —
      **withdrawn**, on the v134 evidence above, not on the `N_IN = 2` argument.
      The argument would still have permitted a depth-1 comparison; the
      measurement does not.)* Original entry: Independently of the
      defect, `deep_snn_scaling.rs:22-26` records that `CoincidenceTask` has
      `N_IN = 2`, so a 256⁴ stack on two-dimensional near-noiseless input has no
      depth structure to exploit. The 1.0000-to-0.4525 collapse is weak evidence
      either way and must not be cited as local learning failing with depth.
- [!] **SUPERSEDED 2026-08-20 — do not port the guard.** `shd_scientific_sweep`
      **never loads SHD.** It fabricates 5 classes over 24 channels and 16
      timesteps, with each label firing only in its own three reserved channels —
      linearly separable from spike counts, with no temporal structure at all.
      Porting `ceiling_inverted` would put a correct label on a comparison that
      should not be made. The binary's self-description has been corrected
      (`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`); **retire-vs-rename is a
      maintainer call** and the report cannot be regenerated regardless, because
      the binary is refused by `authorize_campaign(LocalLearning)` while the
      instrument is `Uncalibrated`.
- [x] **Re-run `ei-inhibition-sweep` at v135.** Done — v135 report on disk
      (`results/ei_inhibition_results_v135.md`).

## 2. Ceiling health of the two arms still standing

This is the item most likely to move a headline result, and it is not covered by
§1 because both arms went through the clamped `runner.rs` path.

- [x] **Check whether the matched gradient references are undertrained.** *(2026-08-19.)*
      Done via `a6_ceiling_health` (`RESULT_2026-08-19_A6_CEILING_HEALTH.md`).
      The gradient reference climbs to 1.0000 by e640; at e80 (0.9013) it is still
      climbing. Therefore, `gap_closed` > 1 at e80 is an artefact of undertraining
      the reference, and the comparison reflects learning speed on a saturating task.
      Guards unified in `binn_lab::guards::assert_reference_learning`
      (`HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md`).

## 3. The transfer gap — the science worth doing next

A local rule passes on the matched dense substrate (0.9387 / LCB 0.6894 and
0.9200 / LCB 0.6846) and fails on the live event-driven engine across twelve
variants, best gap LCB 0.3127 against a 0.5 threshold, canonical protocol 0.4912
with LCB −0.0048. Nobody has isolated why.

- [ ] **Write the decomposition preregistration.** Four named suspects, never
      tested individually: sticky `last_spike`, partial membrane reset, θ=∞
      muting, hard k-WTA instead of soft competition. One factor per arm, plus
      the registered stopping rule and the named-outcomes table before any cell
      runs. *(§2 unblocked 2026-08-19; design drafted in
      `DESIGN_TRANSFER_GAP_DECOMPOSITION.md`.)*

## 4. SHD instrument — remaining scope and caveats

- [x] Budget axis closed (final doubling e400→e800 buys +0.000294)
- [x] Width axis closed (h512→h1024 buys +0.000883)
- [x] Geometry axis closed (`channels-700` 0.0283 *worse* at e400; the registered
      prediction that the gap would narrow was **refuted**)
- [x] H1 at converged budget (24 cells, 6 seeds, NOT SUPPORTED at both budgets;
      outcome 4 of 4 as named in advance)
- [~] **The count-destruction asymmetry needs its own bar — registered
      2026-09-07, not yet run.** H28-2 registered `|DiD| < 0.03`, a **two-sided**
      band on a one-sided question, and reported NOT MET at p90 on a DiD of
      **−0.0382** — the read-out lost 0.0698 where the substrate lost 0.1080, so
      its advantage *grew*. The band cannot tell that from the advantage
      falling, and the registered NOT-MET text asserts the opposite of the sign
      it printed against. Rule §9.3 forbids re-reading that bar on wave 28's
      cells.

      **Registered:** [`PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md`](PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md),
      with `scripts/aws/analyse_wave29.py` frozen and committed in the same
      change, before any cell exists. Two one-sided clauses — H29-1 refuted only
      by DiD > +0.03, H29-2 asking whether the growth clears −0.03 — on a fresh
      seed block `5290001`–`5290012` that the analyser enforces **by seed value**,
      so the replacement bar cannot be evaluated on the cells that motivated it.
      Ten tests, and restoring `abs()` to the two counts fails exactly the two
      that assert one-sidedness.

      **Blocked on compute, and only that.** 48 cells at a median 32,944 s each
      is roughly **440 core-hours** — a cloud campaign, not a local run. The
      paper reports H28-2 as NOT MET meanwhile, which is correct and stays.

- [ ] **Contract axis at convergence.** Six timing contracts are closed only at
      e100. This is the last scope qualifier on the 0.7378 ceiling. Resolution
      invariance held at the short budget, but the width and geometry axes both
      taught that short-budget behaviour does not transfer.
- [ ] **Synchrony at matched activity.** The 0.1336 increment is an upper bound:
      `channel-shuffled` moves `saturated_fraction` 0.0000 → ~0.032 and mean
      firing rate 0.21 → 0.28. Needs a **registered input-scale normalisation and
      a re-measurement**, not a re-analysis. H1 is unaffected (`bin-shuffled`
      saturates at exactly 0.0000 in every seed).
- [ ] **Sharpening direction at a third budget.** Order effect shrinks
      0.0189 → 0.0127 while synchrony grows 0.1248 → 0.1336 and the ratio goes
      6.6× → 10.5×. Descriptive, **not registered**, and must not be leaned on
      until checked at a third budget.
- [ ] **H2 relative question for the recurrent arm.** Ask whether recurrence
      degrades *less* under shuffling than feed-forward does. **The blocking
      dependency is now met:** this item required "a matched `ff+fixed` baseline
      at the same surrogate scale", and wave 14 produced exactly that — 12/12
      `ff+fixed` and 12/12 `ff+fixed+attn` at scale 0.4, with `ff+fixed` scoring
      0.7088 against 0.7062 archived at the default, so the scale is not
      distorting it (`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`).
      What remains is the shuffled arm itself. No absolute ceiling claim from
      this configuration.
- [ ] **Recurrent numerical marginality.** Still open, but the evidence here is
      superseded: this item cites 3 seeds at h512, and wave 13 measured it across
      **48 cells** at the anchor width and budget
      (`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`). What that adds is a
      completion rate rather than an anecdote — `rec+alif` 11/12 at scale 0.4 and
      8/12 at 1.0 — and a second failure mode: `rec+fixed` does not diverge at
      0.4 at all, it **saturates**, ten cells voided with up to 52% of hidden
      units pinned at maximum firing. Adaptation is what prevents that, so on
      this substrate adaptation is stabilising. Per the registered stopping rule,
      **no smaller surrogate scale will be tried**. If it matters enough to fix,
      the next interventions in evidence order are per-sample gradient clipping
      at a threshold from the recurrent arm's own distribution, then h64, then
      truncated BPTT.

## 5. Provenance and verification

- [x] **Commit the record.** *(2026-08-23.)* Done, and the premise this item
      rested on is no longer true. The record was first committed in `a3dafd1`
      and every preregistration, amendment, result and analyser since has been
      committed before the data it governs existed — waves 12, 13 and 14 each
      registered and had their analyser frozen in commits that precede their
      first cell. The ordering that carries the epistemic weight is attested by
      git history rather than by prose and mtimes.
- [x] **Gate F on the current binary.** *(Run 2026-09-07: **218/218
      bit-identical, 0 failures, PASS**.)* This entry asked for 13/13 and said
      the standing coverage was 7 cells. Both numbers were stale: the committed
      report already carried 14, and `--all` re-runs **218**.

      `scripts/gate_f_rust.py --all` against binary
      `8485b6f717b3a8ee74c915abb772c71f461fe5f459cac56b701101f762bbb89b`. The
      archived report was cut on `fe7904a48dd41609...`, a build that no longer
      exists, so this is the first Gate F pass on the binary in the tree.

      **The reference was not touched.** Gate F reads
      `shd_instrument_v4/cells/` and writes its observations to
      `shd_instrument_v4/gate-f-rust/`, unlinking each before the re-run so an
      instrument that exits 0 without writing cannot be scored BIT_IDENTICAL
      against a stale file. `git status` on `cells/` is empty.

      Every non-timestamp difference in the refreshed observations is a field
      the archived cells predate — `tau_m`, `surrogate_scale`, the clipping
      counters, `epoch_max_gradient_*`, and five `temporal_audit` sub-fields,
      all absent before and present now. **Not one field changed value.**
- [x] **`matrix_authorized`.** *(Closed 2026-08-23; this entry was stale until
      2026-09-07.)* It was false since 2026-08-03 alongside
      `historical_reference` and `clean_reference`, and this entry said the
      choice was between accepting on content evidence and re-running the six
      reference cells. **The cells were re-run**, and every one reproduced its
      archived value to every recorded digit. `results/shd_instrument_v4/gates.json`
      carries all three gates `true`; v1, v2 and v3 still carry the false ones,
      which is what an archive is for. `PAPER_DRAFT.md` §4.6 has said so since
      2026-08-23.

      **The entry outlived the decision by five weeks**, flagged `[!]` — human
      decision required — for something already decided by measurement. That is
      the same defect class as the record checks pointing at a corpus that had
      moved: a document nothing verifies against the artefacts it describes.
- [x] **Provenance flag.** *(Decided 2026-08-05; this entry was stale until
      2026-09-07.)* `PROVENANCE_DISCHARGE_ENABLED` is **`True`** at
      `scripts/shd_calibration/runner.py:334`, and the comment above it records
      why: it shipped default-off because discharging a provenance freeze is a
      judgement about what counts as evidence rather than an engineering call —
      "and that judgement was not the agent's to make. **It has now been
      made.**" Enabling it is not a bypass: `gate_f_discharge` still requires a
      PASS for *this* binary hash over at least `PROVENANCE_MIN_GATE_F_CELLS`
      cells spanning two geometries and two widths, and data-file changes stay
      undischargeable regardless.

      **This entry outlived its decision by 33 days**, flagged `[!]` — human
      decision required — for something a human had already decided. That is the
      **third** instance of one class found on 2026-09-07, after
      `matrix_authorized` (five weeks) and the defect register's claim that no
      result depends on `binn-engine`. All three are documents nothing verifies
      against the artefacts they describe.
      `scripts/test_open_work_is_current.py` now closes the two that are
      mechanically checkable.

## 6. Audit debt

- [~] **BINN proper is partly swept; ~8,000 lines of experiment binaries are
      not.** `binn-engine`, `binn-areas`, `binn-core` and ~20 experiment binaries
      have never had the treatment the SHD instrument got, which found ten
      defects, five of them the class *code reporting success while measuring
      nothing*. Clippy found none of them; they are semantic.

      **This entry's premise was right and the register beside it was wrong.**
      `AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md` §2b/§3 said no current result
      depends on those crates; Gate G2 executes `binn_engine::Engine` and
      `binn_areas` through `binn-lab/src/runner.rs`, so "whatever Gate 2 says is
      worth nothing until this is done" is the correct reading and the
      register's was not
      ([`AMENDMENT_2026-09-07_THE_ENGINE_IS_ON_THE_G2_PATH.md`](AMENDMENT_2026-09-07_THE_ENGINE_IS_ON_THE_G2_PATH.md)).

      **Swept 2026-09-07:** the three invariants the register named as the place
      a BINN-proper defect would live — timing-wheel ordering, event-queue
      correctness, `sparse.rs` CSR/CSC consistency. **No defect.** Both files
      carry reference-parity and property tests that execute; `level_for` was
      established by two-directional mutation to be a performance hint rather
      than a correctness invariant, so no test pins it and none should.

      **Swept 2026-09-07, second pass:** all 31 `binn-lab/experiments/*.rs`,
      **17,358 lines**, against the five defect classes that produced all ten
      defects in the register — each matcher calibrated against the register's
      own defects before its result was believed, and every hit read rather than
      counted. Classes A, B and E: **zero**. Class D: 16 production panics, all
      fail-loud on invariants or I/O. **Class C: one defect**, found and fixed at
      its owner — `read_event_cache` clamped a request to the file it found and
      returned quietly (defect #5's own shape, recorded as "FIXED at call site",
      which is a case fix), and `n_train` was absent from `PLAN_PINNED_FIELDS`
      so nothing compared a cell's realised sample count to the plan that asked
      for it ([`AUDIT_2026-09-07_THE_EXPERIMENT_BINARIES.md`](AUDIT_2026-09-07_THE_EXPERIMENT_BINARIES.md)).

      **Still open:** `binn-engine/src/{cell,engine,resting,parallel,synapse,
      spikelog}.rs` and `binn-areas/`, which have had the generic class greps
      (2026-08-03) but no semantic audit. Nothing in the repository is now
      unswept at the class level; what remains is semantic, and it is bounded to
      those files.
- [x] July campaign scoring path (`AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md`)
- [x] Rust instrument defect register (`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`)

## 7. Performance

- [x] Recurrent training cell 193.0 s → 30.6 s (6.3×), out of the
      forward/backward correctness fix
- [ ] **Profile before acting on `PERF_AUDIT_2026-08-02.md`.** The audit ran with
      no Rust toolchain; nothing in it was compiled, benchmarked or profiled, and
      its own opening says the ranking is a hypothesis and the first action
      should be to profile rather than to start at item #1.
- [ ] **Largest named candidate:** the plasticity step deep-copying its entire
      CSR *and* CSC on every update, ~30 MB of memcpy per step at nnz ≈ 2.5e6,
      purely to dodge a borrow conflict.
- [ ] **`ff+fixed` 4.5% regression.** Documented rather than hidden; removing it
      needs monomorphisation, a structural change with its own verification
      burden.
- [x] GPU throughput claims withdrawn. The backend selector was a bool the kernel
      never read; both columns ran byte-identical CPU code. No GPU code has ever
      executed in this repository.

## 8. Deferred by instruction

- [!] **Python arm.** Unswept by instruction; `matrix_verdict` reports `FAIL`
      with the cross-backend criterion explicitly unmet. Rerunning all 216 under
      the amended instrument is ≈4.4 days and cannot change any conclusion.
- [x] **Gate E / G7.** *(Measured 2026-09-07.)* There is a cross-backend
      recurrent fixture and it agrees. `scripts/gate_e_recurrent_parity.py`
      reports **GATE E: PASS**, 30 of 30 fields within 1e-09 absolute or 1e-05
      relative, across all four arms — and **16 of 16 on the recurrent arms**,
      which is the claim this entry called argued. `grad_w_rec` agrees at
      **exactly 0.000e+00** on both `rec+fixed` and `rec+alif`; spikes, rates
      and logits are exact on every arm. The script says so in its own closing
      line.

      **It was never unrunnable — it was unrun.** The blocker was `h5py`, which
      `requirements-shd-calibration.txt` has always declared and which was not
      installed, so the gate raised `ModuleNotFoundError` and two Python suites
      reported "could not run". `gate_e_recurrent_parity.py` needs only `h5py`
      and `numpy`; the torch stack in that requirements file belongs to the
      PyTorch reference, not to this gate. Provisioning those two also unblocked
      `test_provenance_discharge.py` (13 tests) and `test_shd_calibration.py`
      (8 tests), both of which pass.

---

## 9. External reading — the sparse reward subsystem in LLMs, read against the record (2026-09-05)

**Source.** Xu, Yuksekgonul & Zou, *Sparse Reward Subsystem in Large Language
Models*, arXiv:2602.00986 (v1 2026-02-01, v2 2026-05-11).
<https://arxiv.org/abs/2602.00986>

**Why it is in this register.** It is the closest published result to the
G2 / C1 question from the other side: it shows what a reward subsystem looks
like *after* backprop has built one. Nothing in it is BINN evidence. Every
item in §9.6 needs its own `PREREG_` document and protocol hash before a
single cell runs. **G2 stays closed** (`README.md` §7, `PAPER_DRAFT.md` §4.5).

### 9.1 What the paper claims, with the numbers that matter

- **Setup.** Frozen LLM (Qwen-2.5-7B/14B-SimpleRL-Zoo primary; Qwen2.5-7B-PPO-Zero,
  Qwen3.5-0.8B, Phi-3.5-mini, Llama-3.1-8B-Instruct, Gemma-3-4B). "Neuron" =
  one coordinate of the layer-*l* residual stream. Layers 2–5. Tasks GSM8K,
  MATH500, ARC, MBPP+, IFEval, Minerva.
- **Value neurons.** Two-layer MLP probe (hidden → 1024 → 1) trained by TD on
  token-level steps, γ = 1 − 1e-5, target binary correctness. Sparsity is
  measured by L1-pruning first-layer probe weights: AUROC at the *prompt
  position* s₀ is flat up to pruning ratio 0.99, so "< 1 % of neurons
  suffice" (≈ 51 of 5120 at 14B; absolute counts never stated).
- **Absolute value-neuron AUROC at s₀ is modest:** 0.59–0.73 (Table 3; avg
  0.67 vs 0.60 for a linear probe on the full hidden state). At the final
  position it is "mostly above 0.8". On Qwen3.5-0.8B, *question length* beats
  the value neurons (0.77 vs 0.71 MATH500).
- **Causal ablation (Table 2).** Zero the top-1 % value neurons in one layer
  of Qwen-2.5-7B-SimpleRL-Zoo, MATH500, base 75.2 %: layers 2/3/4/5 → 37.0 /
  13.6 / 29.4 / 1.2, avg −54.9. Random 1 % −0.6; next-token-probe neurons +1.2;
  magnitude −3.6; Wanda −9.4. Overlap of value and NTP neuron sets 0.7 %
  (random 0.5 %).
- **Appendix A — the item most relevant to us.** A probe trained on the
  *terminal reward only* (no TD) selects neurons whose ablation costs −8.8;
  the TD-trained probe's set costs −54.9. The TD target finds the
  load-bearing population; the terminal target does not.
- **Dopamine neurons.** Steps are *paragraphs*; V̂(s_t) from K Monte-Carlo
  rollouts (K unstated); δ_t = γV̂(s_t) − V̂(s_{t−1}); only |δ_t| > 0.3 used.
  Input is the per-neuron z-scored activation *mean-pooled over the paragraph*
  ("firing rate over a temporal window", their words). MLP → 32 → 1, MSE.
  Spearman vs pruning "largely invariant"; **no correlation values printed**.
  Used as a step scorer for K = 4 search on MATH500: greedy 72.2 / random
  72.2 / implicit PRM 75.0 / dopamine **77.8** (3 seeds).
- **Layered dependence (Appendix H).** Zeroing top-20 % value neurons in
  *earlier* layers destroys the RPE-like trajectory of dopamine neuron #1517
  in layer 5; 20 % random ablation does not.
- **Transfer.** Value-neuron set IoU across GSM8K / MATH500 / ARC, and across
  two RLVR descendants of the same base, exceeds the hypergeometric baseline.
  No IoU numbers printed.

### 9.2 Read against the record

| Their finding | Our record | What the pairing says |
|---|---|---|
| A sparse (~1 %) value population exists, and a localized RPE-tracking population sits *downstream* of it, in a backprop-trained model | G2 / C1: broadcast ±1 third factor stays at chance (0.5000, gap LCB 0.0000, `c1-match-5dc6822e71229e9e` FAIL); graded DFA 0.9387 and per-neuron RL 0.9200 pass on the identical forward | Their "dopamine" is not a global scalar — it is learned, layered, and addressed (Appendix H). That is the C1 negative stated from the interpretability side: backprop *produces* a reward subsystem; a broadcast scalar does not *train* one |
| TD-shaped target finds the causal population; terminal-only target does not (−54.9 vs −8.8) | `binn-learn/src/credit.rs` has `LearnedRpeCritic` (delta-rule V(s), δ = r − V) and a modulator with directional `B_i · rpe` plus unsigned `\|rpe\|`; the `credit-assignment` binary has an RPE arm; BINN-Hybrid H0 used a *terminal* teacher only | Independent evidence that terminal reward is the wrong *shape* of third factor, separate from whether it is addressed. Bears on any H-series retry, not on G2 |
| Value readout needs ~1 % of units | k-WTA target sparsity k/N, GC7 | Our activity sparsity is not what blocks a value representation. The block stays on delivery |
| RPE is measured as a windowed mean rate, z-scored per unit | Eligibility traces e_ij(t) with τ_e; per-cell firing counts already logged | The paper's measurement is exactly a trace-filtered rate. A per-cell RPE probe costs us nothing new in instrumentation |
| Value neurons transfer across tasks / sibling models | §3 transfer gap; live k-WTA transfer barrier (v13–v24) | Their transfer is of a *readout set*, not of a learning rule. Not the same gap; do not cite it as such |

### 9.3 What the paper does not address

No spiking, STDP, eligibility traces, three-factor rules, local learning,
neuromodulation as a *training* signal, or non-backprop learning appear
anywhere. Probes are AdamW-trained; the LLM is frozen. Whether the subsystem
emerges from pretraining or from RL is not tested (no base model probed;
instruct models show it too, which hints pretraining). The biology is naming
plus two borrowed measurement conventions (windowed rate; value → dopamine
ordering); no circuit mapping.

### 9.4 Gaps in the paper (cite it with these in hand)

- No absolute neuron counts for value neurons; only curves vs pruning ratio.
- No Spearman values for dopamine neurons; no IoU values for transfer.
- K for Monte-Carlo value estimation unstated. Whether TD training uses a
  stop-gradient / target network unstated.
- s₀ AUROC 0.59–0.73 is weak; a length baseline beats it on the smallest
  model. The causal ablation is the strong result, not the probe accuracy.
- Largest model 14B; Appendix K admits nothing > 32B was tried.
- The ablation is single-layer zeroing of 1 % of the residual — a large
  intervention on early layers of a residual network. Wanda at −9.4 shows
  some of the drop is generic importance; they control for it, but the
  random-1 % control is the weak one.

### 9.5 What can be inferred for BINN

1. **A framing sentence for §4 of the draft.** Models trained by backprop
   develop a sparse, layered value/RPE subsystem (Xu et al.); a broadcast RPE
   cannot build one on a matched forward (C1). The two results are converses,
   not competitors.
2. **A mechanistic signature we do not have.** We know *that* broadcast ±1
   fails and DFA/RL pass. We have never asked whether the passing arms
   *contain* a sparse value population and the failing arm does not. That is
   a one-probe question on trace exports we already produce.
3. **Third-factor shape.** Their Appendix A and our `LearnedRpeCritic` point
   the same way: a TD-shaped δ_t, not terminal r, is the candidate scalar.
   This is the only place the paper touches a *learning* choice we control,
   and it is confined to the hybrid successor line.
4. **A readout-sparsity number.** If a 1 %-sparse readout retains task
   accuracy on our SHD arm, the attention readout's advantage is not about
   reading *many* cells; if it does not, our substrate distributes value more
   than an LLM residual does. Either answer is a sentence in §3.7.

### 9.6 Candidate experiments — none preregistered, none scheduled

Each item names the arm, the metric, the falsifier, and what it cannot
authorize. Probes are offline, lab-side (Python under `scripts/` or
`binn-lab`), on exported traces — GC1 is untouched. All reuse frozen current
hashes; a retired hash anywhere in a prereg fails
`scripts/test_published_hashes_resolve.py`.

- [ ] **V1 — Value-population probe across the three matched arms.** Export
      hidden activity at the post-encoding / pre-decision position and at the
      terminal position for the broadcast ±1, DFA, and RL matched runs (same
      forward, current frozen hashes). Train the paper's L1-pruned probe
      (their recipe: MLP → 1024 → 1, AdamW 1e-4, wd 0.01) with (a) TD target,
      (b) terminal-only target. Metric: AUROC vs pruning ratio, per arm, per
      target. *Prediction:* a flat-to-0.99 curve under DFA/RL, not under
      broadcast. *Falsifier:* if the broadcast arm carries an equally sparse,
      equally accurate value population, the C1 failure is purely delivery and
      not representational — which is also worth knowing. Cannot authorize: any
      G2 reopening; any claim beyond the matched dense-LIF control.
- [ ] **V2 — Causal ablation of the value population.** Mute the top-1 %
      cells from V1 (θ = ∞, the existing k-WTA mute path) versus a random
      1 % and a firing-rate-magnitude 1 %, at test time only, on the DFA and
      RL arms. Metric: accuracy drop with LCB over the existing seed family.
      *Prediction:* value-cell mute ≫ random. *Falsifier:* value ≈ magnitude
      ≈ random ⇒ the probe found a correlate, not a mechanism. Depends on V1.
- [ ] **V3 — Terminal vs TD probe target (their Appendix A, on our
      substrate).** V2 run twice, with V1(a) and V1(b) cell sets. Metric:
      ablation cost ratio. This is the cheapest way to test whether the
      "TD finds the load-bearing set" result is substrate-independent.
- [ ] **D1 — Do any cells track RPE unasked?** On the SHD attention arm and
      on the RL matched arm, compute per-cell z-scored windowed rates (window
      = one input segment; matches their paragraph pooling) and regress on
      δ_t from `LearnedRpeCritic` and, separately, from MC rollouts (K
      stated in the prereg). Metric: Spearman vs pruning ratio. *Prediction:*
      RL arm > SHD arm > broadcast arm. *Falsifier:* nothing above the
      shuffled-δ control anywhere. Cannot authorize: any dopamine-cell
      *training* claim.
- [ ] **D2 — Layered dependence (their Appendix H).** Only if D1 finds
      RPE-tracking cells and only in a multi-area configuration: mute top-20 %
      value cells in the upstream area, measure the downstream RPE-tracking
      cells' Spearman. R1/R2 are opt-in and G4 is NO-GO; this item cannot
      become a scaling claim.
- [ ] **H4 — TD-shaped third factor in the hybrid line.** New protocol
      version (H0 v3 is frozen `HYBRID_NO_GO`): replace the terminal teacher's
      scalar with online δ_t from `LearnedRpeCritic` as M(t), with the
      existing postsynaptic, least-squares postsynaptic, and direct-terminal
      credit unchanged. Metric: C3 D\* against the D\* ≥ 6 gate on the
      smooth surrogate first, then the production event-engine diagnostic.
      *Prediction:* D\* rises above the direct-terminal 5. *Falsifier:* D\*
      unchanged ⇒ the third factor's temporal shape is not the limiting term,
      consistent with the identifiability reading in
      `BINN_HYBRID_PROTOCOL.md`. Cannot authorize: H1/H2/H3; anything about
      G2.
- [ ] **S1 — 1 %-sparse readout on SHD.** Attention readout restricted to
      the top-1 % cells by V1-style probe versus the full k-winner set versus
      a random 1 %. Metric: accuracy over the 12-seed family, plus the
      bin-shuffle collapse. *Prediction:* unknown — this is the one item
      where either answer is informative (§9.5 item 4). Depends on the
      SHD instrument being `Calibrated` for the readout arm.
- [ ] **T1 — Value-set transfer (their IoU test).** IoU of V1 cell sets
      across seeds and across the two passing arms, against the
      hypergeometric expectation. Metric: IoU vs pruning ratio. Belongs in
      §3 only as a *readout* transfer number, never as a rule-transfer
      number.
- [ ] **Draft edit.** One paragraph in `PAPER_DRAFT.md` §4.1 or §3.8 placing
      C1 as the converse of Xu et al.; cite with §9.4 caveats. No result may
      be cited from §9.6 until it exists under a hash.

**Ordering.** V1 → V2 → V3 are one export and three probe fits; they come
first because they can be done on the existing record without a new
campaign. D1 is next. H4 is a hybrid-line decision and waits on the
maintainer. S1 waits on §4. T1 is last.

---

## The one-line version

Repair the record (§1), find out whether the two surviving ceilings are real
(§2), and only then design the transfer-gap experiment (§3). Everything in §4
is real science that is already safe to continue. §9 is reading, not work,
until a prereg exists.
