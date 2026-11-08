# Preregistration: does the attention read-out add anything to a substrate that already has a temporal kernel?

**Status: DRAFT. Not yet attested.** This repository's preregistrations are attested by git history. This document does not count as a registration until two things are committed to BINN before any cell of the series exists: this file, and the frozen analyser named in §7. Before that commit, any edit is free. After it, only dated amendments are allowed, and only while no run exists.

**Drafted:** 2026-11-08, outside the BINN tree in `binn-wsp-study/prereg/`, because another session holds BINN's `results/`.

---

## 1. The question, and why it is the next one

The manuscript's SHD contribution is a measurement. On a substrate with no temporal kernel, the attention read-out raises accuracy by +0.1258, and destroying time order removes 94.5% of that gain (§3.5). §3.8 then attributes most of the gap to the pinned DCLS reference (0.9376) to that reference's 25-tap learned-delay kernel, and an ablation measured the kernel at K = +0.3111 (`RESULT_2026-09-02_THE_KERNEL_ABLATION.md`). The paper's reading is that the read-out recovers part of what a temporal kernel supplies. That claim is about the kind of contribution, not its quantity.

Both the kind claim and any architecture claim depend on one experiment that has not been run: **put the same read-out on the substrate that has the kernel.**
- If the read-out adds accuracy on top of learned delays, and that addition depends on order, the read-out is doing something the kernel does not. This is the first result that could support calling the combination an architecture.
- If it adds nothing, the kind claim is strengthened. The read-out and the kernel then compete for the same temporal information, and no architecture claim follows. That outcome is equally publishable, and the series is designed so that it can happen.

The secondary series (§3, arm set B) asks the converse on the kernel-removed reference: does the read-out buy back the kernel's 0.31?

## 2. Substrate and protocol (pinned, unchanged from the kernel ablation)

- **Reference:** `SNN-delays` at `d169b4e`, `best_config_SHD.py` as pinned:
  - 2 hidden layers of 256 LIF units, `time_step = 10` ms, `n_bins = 5`, dropout 0.4, batchnorm on.
  - Output layer: a non-spiking membrane (`output_v_threshold = 1e9`). Loss `sum`, meaning Σₜ softmax(v(t)), with cross-entropy.
  - Adam with one-cycle schedule, 150 epochs, `augment = False`.
- **Environment:** torch 2.13.0, spikingjelly `6dca147`, the clean protocol (`reference_clean_main.py`, `CLEAN_TEST_ONCE`).
- **Device: AWS Graviton CPU, one run per instance, every arm at one configuration.** The environment is the kernel ablation's, from `scripts/aws/kernel_ablation/requirements.lock` (Python 3.12, torch 2.13.0 aarch64 wheel), on Amazon Linux 2023. Every run in every contrast uses the same instance type, thread count and oneDNN setting. See §4 for why, and for what it costs.
  - **The configuration is chosen for speed, before any run, by a rule that reads only wall time.** With fresh controls, settings that change the arithmetic are allowed, provided every arm uses them and they are fixed before any outcome exists. The configuration probe (`binn-wsp-study/aws_attn/body_optimise.sh`) times 2 epochs of seed 5170001 at 25 taps, with the memo on, for threads {16, 32} × oneDNN {on, off, off for the backward pass only}, on c7g (Graviton3) and c8g (Graviton4). `aws_attn/pick_config.py` then picks:
    - the fastest epoch-2 wall at 16 threads;
    - 32 threads only if it is at least 1.25× faster, because it needs a whole 32-vCPU instance per run, and 16 of those use the account's entire 512-vCPU spot quota.
    - **Across the two boxes:** each box picks by the rule above. The family whose pick has the lower epoch-2 wall wins, unless it is less than 5% faster than the other, in which case c7g wins (the archived controls' platform). The campaign then runs on that family's .4xlarge (16 threads) or .8xlarge (32 threads). The memo verdict used is the winning box's.
    - **If 32 threads is picked,** the 16 runs need exactly the 512-vCPU spot quota. The campaign launcher refuses to start while any other instance runs in the account.
  - **The pick (probe run 2026-11-08 on i-04a4607f611c8b630 and i-04e6150f35faa5e61; `aws_attn/fresh_config_probe/RESULT.md`): c8g.8xlarge (Graviton4), 32 torch threads, oneDNN off, memo on, one run per instance.**
    - **Epoch-2 wall:** 24.0 s.
      - c8g's best at 16 threads was 33.4 s, so 32 threads is 1.39× faster and passes the 1.25× test.
      - c7g's pick was 16 threads with oneDNN off for the backward pass only, at 36.6 s. c8g is 34% faster.
    - **Memo check at the pick:** A/A determinism, data stream and memo all PASS, for `max_delay` 25 and 1. The A/A ran on c8g.8xlarge itself.
    - **The cells also run with `PYTHONHASHSEED=5170001`,** the constant every probe gate run used. So the series runs under the condition the A/A measured.
    - **Timing is n = 1 per configuration.** The three 32-thread settings on c8g lie within 6% of each other (24.0–25.4 s), so the choice among them is not resolved by noise. It is made by the rule, and it barely matters for time.
  - **Why oneDNN is a candidate.** On this wheel, oneDNN's convolution backward is slower than ATen's own: backward 44.9 s against 29.8 s per 20 batches at 5 threads (`aws_attn/graviton_profile/RESULT.md`). About 80% of the compute is the 3 DCLS convolutions per batch, two hidden layers and the output (backward 61%, forward 18% of self-CPU time; `ops_t5_alone.txt`, 9 calls over 3 profiled batches), so the convolution backend is where the time is.
  - **This configuration does not reproduce the archived numbers bit for bit.** The archived controls ran 5 threads with oneDNN on, 6 runs per c7g.8xlarge. So no archived number enters any contrast. The archived macOS CPU and Graviton means are used only as the reference for the platform-move check in HA-0, the role HK-4 played when the kernel ablation moved from macOS to Graviton.
  - **Determinism is measured, not assumed.** The probe's memo check includes an A/A pair: two unpatched 2-epoch runs on one box at the pick, compared on the batch stream, each epoch's loss and accuracy bits, and the final state hash. Across boxes it is measured by stage 0's same-seed rerun (§4), run on a different instance from A00 seed 5170001.
  - **The `__getitem__` memo** (`binn-wsp-study/aws_attn/harness_patch.py`) caches decoded samples and takes about 11% off every epoch after the first (epoch 2: 105.5 s against 118.2 s at 5 threads). It was verified bit-exact on Graviton3 at 5 threads over 2 epochs. It is used only if the probe's memo check passes at the pick, for `max_delay` 25 and 1. Otherwise it is not used. Either way every record carries `memo`.
  - The worker refuses a checkout that is not `d169b4e`.

## 3. The manipulation

**The attention read-out (R1) is the instrument's read-out, ported with an unchanged definition** (`binn-learn/src/shd_attention.rs`):

```text
s(t)     = post-dropout spike train of the LAST hidden layer      [T_h, 256]
           (the tensor `x` after blocks[n_hidden_layers-1][1][1]; stateful_synapse = False
            in the pinned config, so this is exactly what the output Dcls1d reads)
z_0(t)   = W_e s(t) + pos(t)                                       d_model = 32
4 blocks: z(t) <- z(t) + W_o softmax_row(q kᵀ/√d) v                (q,k,v = W_q z, W_k z, W_v z)
pooled   = mean_t z_L(t)
a        = W_a pooled                                              [20]
```

- **Lengths.** The pinned forward pads each `Dcls1d` input by 24 + 12. So T_h = 124 at `max_delay = 25` and T_h = 100 at `max_delay = 1`, and the output membrane sequence has 136 and 100 steps respectively. These lengths are inferred from the padding arithmetic. The parity test below asserts them.
- `pos` is the instrument's fixed sinusoidal code over normalised time u = t/(T_h−1).
- **Initialisation** copies `AttentionParams::deterministic`: Glorot for W_e, W_q, W_k, W_v and W_a; **W_o = 0**. At initialisation, R1 is therefore *not* the reference: W_a is non-zero.
- **Where `a` enters.** The reference's read-out is Σₜ softmax(v(t)). R1 replaces it with Σₜ softmax(v(t) + a), with `a` added at every output step. This puts `a` on the logits, where the instrument adds it. The model reduces exactly to the reference *when W_a = 0*. That is a property of the construction, not the initial state.
- **Optimiser.** The read-out's parameters join the reference's weight group, with the same Adam and one-cycle schedule. They get no separate learning rate (§6, limit 2).
- **Implementation parity gate, before any run.** The PyTorch module's forward must match the Rust forward (`shd_attention.rs`) on fixed inputs and weights at T_h ∈ {124, 100}, to within 1e-5 max-abs on `pooled` and `a`. The test is committed with this registration. If it fails, there is no series.

**The order factor is `hidden-shuffled`, the operator of §3.5.** It is an independent per-sample permutation of the time axis *that the read-out sees*. That is s(t) only, after the substrate has produced it. The input, the substrate and the reference's own read-out are untouched.
- The permutation is applied in training and test alike. Each sample's seed is a hash of (series seed, sample index).
- The reference arm pays nothing for it by construction. Its hidden-shuffled arm is identical to A00, so it is not run.
- **Why not shuffle the input.** On this substrate, permuting the input also destroys what the delay kernel reads. Attention's gain would then be measured on a collapsed substrate, a different regime from the intact gain, and the difference-in-differences would not isolate the read-out. Input shuffling is not run in this series.

## 4. Arms: every arm re-measured at one configuration

**The controls are re-run, not reused.** The kernel ablation's Arms A and B (Graviton: 0.9392, 0.9366, 0.9402 and 0.6250, 0.6322, 0.6256) are exactly reproducible on their own harness. They are not paired with this series' runs, because §2's configuration changes the arithmetic. So A00 and B00 are run again at that configuration, and the archived values serve only as HA-0's platform-move reference.

| run | substrate | read-out | order | seeds | runs |
|---|---|---|---|---|---:|
| A00 | full reference | reference | intact | 5170001–3 | 3 |
| A00′ (rerun) | full reference | reference | intact | 5170001 | 1 |
| B00 | kernel removed | reference | intact | 5170001–3 | 3 |
| A10 | full reference | reference + attention | intact | 5170001–3 | 3 |
| A10ₕ | full reference | reference + attention | hidden-shuffled | 5170001–3 | 3 |
| B10 | kernel removed | reference + attention | intact | 5170001–3 | 3 |

- **Stage 0 (A00, A00′, B00) is the gate, 7 runs.** It runs the plain reference code: the pinned checkout, the `num_workers = 0` patch, and the memo patch if the probe's memo check passed (§2). It contains no read-out or shuffle code (records say `hooks: "absent"`). So stage 0 needs nothing from the read-out work, and it may start once the §8 "before the commit" items are done and committed.
  - A00′ runs on a different instance from A00 seed 5170001, so HA-0(c) measures cross-box determinism, not one box twice.
- **Stage 1 (A10, A10ₕ, B10) is 9 runs.** It launches once the §8 "before stage 1" items pass, together with stage 0 or after it, but **it is read only if HA-0 passes.**
  - The analyser prints HA-0 first and computes nothing after a failed HA-0, even when stage-1 records exist (§7). So the launch time changes when stage 1 runs, not whether it counts.
  - Launching it with stage 0 makes the series one wave instead of two, which roughly doubles AWS's advantage over the Mac by compute time (below). Launching stage 0 first lets it run while the stage-1 code is still being written, and reads HA-0 before 9 runs are spent. Either is allowed. Which one was used is recorded in the RESULT.
  - If HA-0 fails, any stage-1 runs that exist are reported as run and unread, and the series stops as §5 says.
  - **The key check is the hooks-disabled one.** With the read-out and shuffle hooks installed but disabled, the stage-1 harness must reproduce the stage-0 code path to the bit over 2 epochs, at §2's configuration.
  - **That is what licenses pairing stage-1 runs with stage-0 controls.** The failure it guards against: a read-out module built after `set_seed` draws from the RNG for its Glorot init even when disabled. That shifts every later dropout mask and gives each arm a different noise stream without changing a line of the reference.
  - **The fix, if it fires,** is to build the read-out from its own generator. Accepting the shift is not an option.
- Stage-2 A00 runs (seeds 5170004–5) use the stage-0 code path too.
- **Every run gets its own c8g.8xlarge (32 vCPU).** Stage 0 is 7 instances (224 vCPU) and stage 1 is 9 (288 vCPU). Launched together they are 512 vCPU, the account's entire spot quota, so the launcher refuses to start while any other instance runs. A triggered stage 2 is a further wave of 6.
- **Expected time, from the probe's epoch walls at the pick (memo on):**
  - per 25-tap run: 33.9 s + 149 × 24.1 s ≈ 1.01 h, plus the test pass and about 3 minutes of bootstrap: **about 1.07 h**;
  - per kernel-removed run: **about 0.67 h**;
  - each wave takes about one 25-tap run.
  - The read-out's added cost is not yet measured.
- **Dropped from the earlier draft:** the exploratory input-shuffle arm, and seeds 4 and 5 as an unconditional cost.

**Stage 2 is a registered, conditional extension.** It is not a re-run to improve a verdict. It runs only if stage 1 is ambiguous, meaning HA-1 is neither met nor refuted at n = 3 (§5).
- That is G in [+0.003, +0.020) with at least 2 of 3 seed pairs positive, or G ≥ +0.020 with exactly 2 of 3.
- An earlier wording ("G in the band, *or* exactly 2 of 3") overlapped HA-1r, for example at G < 0.003 with 2/3. HA-1r takes precedence.

It also runs if the stage-0 same-seed rerun differs from A00 seed 5170001 by more than 0.005, because three seeds cannot then resolve the 0.020 bar.
- It adds seeds 5170004 and 5170005 to A00, A10 and A10ₕ: 6 runs on 6 instances at once, about one run's wall time more.
- The verdict is then read at n = 5 with the n = 5 bars.
- Whether stage 2 runs is decided by the analyser from stage-0 and stage-1 data alone, by the rules above. No other trigger exists.

**Platform: AWS Graviton, fresh controls, one run per instance (chosen by the owner 2026-11-08).** The owner's rule: the Mac GPU if it is faster, and AWS only if AWS is 3–4× faster than the Mac. Measured and inferred:
- **Per run, the Mac GPU is faster than Graviton at the archived settings.** 0.9–1.4 h against 4.59 h at 25 taps (5 threads, alone; `graviton_profile/RESULT.md`), and 0.6–0.8 h against the archived 1.57 h without the kernel.
- **For the whole series AWS is faster, because every run is independent.** The Mac runs 16 one at a time: 10 × 0.89–1.38 h + 6 × 0.59–0.81 h = about **12.4–18.7 h** of GPU time.
  - At the pick, a 25-tap run is about 1.07 h (§4).
  - With stage 1 waiting for HA-0 (two waves), the series takes about 2.2 h: **about 5.6–8.5×**.
  - In one wave it takes about 1.1 h, **about 11–17×**, but it needs the whole spot quota.
  - Both clear the owner's bar. Before the probe, at 10 threads and the archived oneDNN setting, two waves came to only about 2.1–3.2×.
- **Reusing the archived controls bit for bit was rejected.** It would save 6 runs, but it fixes the slow archived settings (5 threads, oneDNN on): about 4.5–4.9 h per run, 2.5–4.2× the Mac. Fresh controls can use a faster configuration, and HA-0 then runs at the same configuration as everything it gates.
- **The cost is 6 extra control runs and a cross-platform step in HA-0.** HA-4 compares two gains measured at one configuration, so it carries no cross-configuration caveat.
- **Calendar time.** AWS is not under the Mac's one-heavy-job lock, which waited about 2 h at a load of 56–105 on 2026-11-08.
- **Spot interruption is an infrastructure fault (§6.1).** Each worker polls the instance-metadata interruption notice and writes it into its run log, so a re-run is licensed by the record.

*Alternatives considered and not taken:*
- **The Mac GPU (MPS).** It is faster per run but serial. MPS arithmetic also differs from CPU (epoch-1 loss for seed 5170001: 4.493366047739983 on MPS against 4.5205655843019485 on CPU at 25 taps, 2026-11-08 probe), so it too needs fresh controls. It is the fallback if the AWS series cannot run. Its memo check is `gate_checks/run_memo_check.sh`.
- **An NVIDIA GPU instance.** About 80% of the compute is convolution, which suits a GPU. But the account's G-instance quota is 8 vCPU, so only two small GPU instances (g6.xlarge) can run at once. To tie a 16-wide Graviton fan-out, a GPU run would need to finish in about 20 minutes, which is not measured. A GPU would also need TF32 switched off in cuDNN, deterministic algorithms, and an A/A check. Not pursued unless the quota is raised.
- **Mac CPU.** It reproduces the archived Mac epoch 1 exactly, but it is 1.5–3× slower per run than MPS.

**Out of scope here:** BINN's 2026-11-05 kernel work (cbbc273) does not touch this series, because it runs the PyTorch reference.

## 5. Hypotheses, with the bars fixed now

Definitions:
- G = mean over seeds of (A10 − A00): the read-out's gain on the kernel substrate.
- Gₕ = mean(A10ₕ − A00): the gain left when the read-out sees order destroyed.
- DiD = G − Gₕ = mean(A10 − A10ₕ).
- G_B = mean(B10 − B00).

Decisions rest on bars and on sign counts across seed pairs. A percentile bootstrap over 5 or 3 paired differences under-covers, so intervals are reported as **descriptive only**. That matches the kernel ablation, which decided on means at n = 3.

**HA-0, the gate (stage 0).** Read before any stage-1 record is read. All three parts must hold:
- **(a) Configuration move, kernel substrate.** |mean(A00) − 0.9376| ≤ 0.010, where 0.9376 is the pinned macOS CPU mean. This is HK-4's bar for the macOS → Graviton move, which landed at a drift of 0.0011.
- **(b) Configuration move, kernel-removed substrate.** |mean(B00) − 0.6276| ≤ 0.010, where 0.6276 is the archived Graviton Arm B mean. That mean was measured at 5 threads with oneDNN on. At §2's configuration it is a reference for a move, not a value expected to the bit.
- **(c) Same-seed rerun, on another instance.** A00′ is reported against A00 seed 5170001. Identical to every digit means training is deterministic across boxes at this configuration. Any difference is the measured same-seed noise floor and is printed beside every seed-pair difference. A difference over 0.005 does not stop the series; it triggers stage 2 (§4).

If (a) or (b) fails, §2's configuration does not reproduce the reference. The series stops, nothing below is read (including stage-1 records that already exist), and any continuation is a dated amendment. The likely one: re-run at the archived settings, 5 threads with oneDNN on.

The HA-1 bar was set against the CPU three-seed spread of 0.0022. This series' A00 spread is reported next to it.

**HA-1, the architecture criterion.** At stage 1 (n = 3): G ≥ +0.020, and A10 > A00 in 3 of 3 seed pairs. At stage 2 (n = 5, only if triggered): G ≥ +0.010, and 5 of 5 seed pairs positive (p = 1/32 under a sign null). The stage-1 bar is higher because 3/3 alone carries p = 1/8. The bar is about 4.5× the reference's three-seed spread of 0.0022.
- If met, together with HA-2, the paper may say: "an attention read-out adds +G on top of learned delays, and the addition depends on the time order it reads."
- That is the only form of architecture claim this series can license (§6).

**HA-1r, the refutation.** At stage 1: G < +0.003, or at most 1 of 3 seed pairs positive. At stage 2: G < +0.003, or at most 3 of 5 positive. Then the read-out adds nothing measurable to a learned-delay substrate, §3.8's reading about kind gains direct support, and the manuscript says plainly that no architecture claim follows.

**HA-2, the gain depends on order (read only if HA-1 is met).** DiD ≥ 0.5 · G, and A10 > A10ₕ in every seed pair at the stage where HA-1 was read. If HA-1 is met and HA-2 is not, the gain does not depend on order. It is reported as capacity, not time-reading.

**HA-3, the paper's own prediction, on headroom.** A raw-gain bar would be met by the ceiling alone: the reference leaves 0.0624 of headroom. So the prediction is stated as the share of remaining error the read-out removes:
- the instrument's share is 0.1258 / (1 − 0.7057) = 0.427;
- the registered prediction is G / (1 − mean(A00)) < 0.214, half of that.

In words: where the substrate already integrates over time, the read-out removes less than half the share of remaining error it removed where no such integration existed. If the share is ≥ 0.214, the read-out and the kernel are not competing for the same information, and §3.8's sentence about kind is withdrawn.

**HA-4, the converse.** G_B ≥ G + 0.030, and B10 > B00 in 3 of 3 seed pairs: the read-out buys more where the kernel is absent. G here is the stage-1 G (seeds 5170001–3), the same seeds as B, even when HA-1 is read at stage 2. G_B / 0.3111, the share of the kernel's value recovered, is reported as descriptive with no bar.

**Ties and bars.** A seed pair counts as positive only if its difference is strictly above 0, so an exact tie is not positive. Every bar comparison allows a tolerance of 1e-12. Accuracies are multiples of 1/2264, so that tolerance only absorbs float rounding of a value that sits exactly on a bar. It cannot move a real difference across one.


## 6. What this series may NOT do

1. **No re-runs to improve a verdict, and no seeds added beyond the conditional stage 2 in §4.** Sixteen runs (7 in stage 0, 9 in stage 1), plus six only under §4's rules, each once. A failed run is re-run only for an infrastructure fault named in the run log, with the same seed, and every such re-run is listed. A spot interruption is such a fault when the worker's log records the instance-metadata interruption notice. A run that ends without either a record or a logged fault is investigated, not silently re-run.
2. **No tuning after registration.** The read-out uses the reference's schedule. That disadvantages R1 against a reference its authors tuned, and the result is written as "at the reference's schedule". A tuned-read-out follow-up needs its own registration, with selection on a validation split carved from the training split, never on test.
3. **No state-of-the-art claim, whatever the outcome.** The clean protocol, `augment = False` and the 10 ms bins differ from the published DCLS 95.07%. A10 is never set against published numbers as a ranking.
4. **No generalisation beyond SHD.** A second dataset (SSC) needs its own registration.
5. **Every arm count, bar and seed above is final once committed.** Changes before any run are dated amendments with reasons, in the style of the kernel ablation's §6–7.

## 7. Analyser

`scripts/analyse_attention_on_delays.py` is committed with this registration and is the authority on every verdict. It is drafted, with 29 tests on synthetic corpora, at `binn-wsp-study/analyser/`.
- **It reads only the run records.** It never reads a test split, a checkpoint or a log.
- **Each stage is evaluable on its own.**
  - HA-0 is read from the 7 stage-0 cells with every later cell absent.
  - Stage 1 is read only when all 9 of its cells are present, and stage 2 only when all 6 of its cells are.
  - A missing cell in a stage reports `NOT EVALUABLE` for that stage. It never computes a verdict from fewer runs than registered.
- **It prints HA-0 first.** If HA-0 fails, it stops there, and nothing after it is computed, even if later records exist.
- **Stage-2 records that exist when stage 2 was not triggered make the corpus INVALID.**
- **Record contract (`attn-on-delays-run-v1`).** Every field of the reference's `shd-reference-v1` payload, plus:
  - `arm` (A00, A00r, B00, A10, A10h, B10);
  - `max_delay`, `readout`, `order` and `hooks` (`absent` in stage 0 and stage-2 A00, `enabled` otherwise);
  - `memo`, `torch_threads`, `mkldnn` (`on`, `off` or `bwd-off`), `instance_type`, `machine` (`aarch64`), `torch_version` (2.13.0) and `reference_commit` (d169b4e);
  - `instance_id`, so HA-0(c)'s different-box requirement is checkable.
- **Validation.** The arm is read from the record, never from a file name. Every field must match the arm's registered values, and `device` must be `cpu`. `torch_threads`, `mkldnn` and `instance_type` must equal §2's pick, which is written into the analyser's constants before the commit. A record with a test read during training, a duplicate cell, an unregistered seed, a `memo` value differing across records, or A00′ on the same instance as A00 seed 5170001 makes the corpus INVALID. Nothing is computed from an invalid corpus.
- **Exit status:** 0 read, 1 invalid, 2 not evaluable, 3 HA-0 failed.

## 8. Checklist

The checklist is split in two because the registration commit needs no read-out code.
- **The registration commit** freezes the arms, the bars, the read-out's definition (§3) and the analyser.
- **The stage-1 code** (the read-out port and the hooks) may land after the commit, but only through the "before stage 1" tests below. Their pass criteria are fixed here.
- If stages 0 and 1 launch together (§4), every item in both groups passes before the first run.
- **That code may not change** any arm, bar, seed or definition in this document. If it would need to, the change is a dated amendment, made before any stage-1 run.

### Before the commit, and so before stage 0

- [x] **The configuration probe (§2) has run, and its pick, timings and memo verdict are recorded in §2 and §4.** Run 2026-11-08: pick c8g.8xlarge / 32 threads / oneDNN off, with memo and A/A PASS.
  - Launcher: `binn-wsp-study/aws_attn/launch_probe.py --body optimise`. It launches one c7g.8xlarge and one c8g.8xlarge (one-time spot, about $0.45–0.60 per hour each), each with a 100-minute shutdown ceiling set in user-data before anything else runs.
  - **The A/A control:** two unpatched 2-epoch runs at the pick must be bit-identical on the batch stream, each epoch's loss and accuracy bits, and the final state hash. If they are not, the memo verdict is NOT EVALUABLE, the memo is not used, and the series does not start until a dated amendment says how runs on different boxes are comparable.
  - **The memo verdict** (patched against unpatched, same fields) is then read for `max_delay` 25 and 1. Comparator: `gate_checks/compare_memo_check.py <dir> cpu`, 11 tests.
  - **Why 2 epochs:** epoch 1 only fills the memo.
  - **Status, 2026-11-08:** run. Both instances terminated themselves after about 20 minutes. Results are in `aws_attn/fresh_config_probe/`.
- [ ] **The analyser is committed and its tests pass,** with §2's pick in its constants. It returns `NOT EVALUABLE` on an empty corpus, and it reads HA-0 from stage 0 alone. Drafted at `binn-wsp-study/analyser/`.
- [ ] **The campaign launcher writes `attn-on-delays-run-v1` records** at §2's configuration, one run per instance, with `hooks: "absent"` in stage 0, `memo` set from the check above, `instance_id`, and the spot-interruption poll in the worker log. It extends `scripts/aws/kernel_ablation/launch.py` by import, as `launch_probe.py` does, rather than adding a second launcher.
  - **Written 2026-11-08. Not yet run on a box, but executed locally.** The real `body_cell.sh` ran over copies built by the bootstrap's own `make_copy` from the pinned checkout, with stub `aws` and `curl` and a stub `clean_main.py` standing in for the 150 epochs. That covered the B00, A00 and A00r cells: the arm patch, seed, hash seed, record name, the upload to `records/` and the spot-poll shutdown were all checked. Files: `binn-wsp-study/aws_attn/launch_campaign.py`, `bootstrap.sh` (shared with the probes), `body_cell.sh` and `attn_run.py`.
  - `attn_run.py` calls the reference's own `clean_main.main()` unmodified. It builds the record with the analyser's own `ARMS` table and reads the configuration back from torch after the run.
  - `test_attn_run.py` has 11 tests: stage-0 records it builds are read by the analyser, and the launcher's cell selection refuses stage-1 arms, unregistered seeds and duplicates.
  - The launcher refuses to start unless:
    - the prereg and analyser are committed in BINN, unchanged from HEAD, and the uploaded analyser is that committed file;
    - the probe's memo verdict shows six PASS lines;
    - the cell has no record and no unfinished earlier launch (a re-run needs `--rerun-reason` and is written to a launch ledger);
    - a stage-2 cell is only launched when the analyser, run over the S3 records, reports stage 2 triggered;
    - running instances plus this launch fit the live spot quota.
  - Its 2026-11-08 dry run passed every check except attestation.
  - **Only `records/` in S3 is the corpus.** Each box's `results/<instance>/` holds logs and setup output, which the analyser rejects. The two are never merged.
- [ ] **The novelty check below has been repeated on the commit date.**

### Before stage 1 is launched (§4)

- [ ] **The PyTorch read-out port and its Rust parity test (§3)** are written and passing.
- [ ] **The `hidden-shuffled` operator is wired on s(t).** Tests assert that each sample's per-unit spike counts are preserved and its time order is changed.
- [ ] **The hooks-disabled check passes (§4).** At §2's configuration, on §2's instance type, 2 epochs of seed 5170001 with the read-out and shuffle hooks installed but disabled give the same batch stream, epoch loss and accuracy bits, and final state hash as the stage-0 code path.
  - Each `max_delay` setting is checked.
  - It has its own A/A control on the campaign instance type: two 2-epoch runs of the unchanged stage-0 code must be bit-identical on the same fields. The probe's A/A ran on a .8xlarge, and determinism is not assumed across instance sizes. If this control fails, the hooks-disabled check is not evaluable, and stage 1 does not launch until a dated amendment says how pairing is justified.
- [ ] **One forward and one backward pass of each stage-1 arm** (A10, A10ₕ, B10) run at §2's configuration, as in the kernel ablation's §7, with T_h asserted at 124 and 100.
- [x] **Novelty check for "attention read-out on a learned-delay SNN", 2026-11-08: not found.**
  - **How it was searched:** six queries through ScholarLM and firecrawl, with arXiv full text read where a candidate looked close. ScholarLM provider coverage was partial (dblp returned 202, core had no credentials). Absence of a result is not proof.
  - **Closest prior work, none of which is this design:**
    - SpikCommander (arXiv 2511.07883, AAAI 2026): spiking self-attention inside the blocks, no learned delays.
    - SpikCTU (arXiv 2610.14594): a recurrent unit reading a fixed memory by attention, no delays.
    - STSC-SNN (arXiv 2210.05241): temporal convolution plus gating.
    - Delay papers (DCLS 2306.17670, DelRec 2509.24852, SiLIF 2506.06374): rate or leaky-integrator read-outs.
  - **Repeat before commit:** re-run this check immediately before the commit, because the field moves monthly.
