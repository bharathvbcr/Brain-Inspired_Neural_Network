#!/usr/bin/env python3
"""Enumerate every cell in the AWS attention campaign, as a deterministic list.

One JSON object per cell. The list is the campaign: `launch.py` shards it,
`bootstrap.sh` runs a shard, and `collect.py` reconciles what came back against
it. Nothing downstream invents a cell that is not in here.

The `id` is the file name in S3 and on disk. It is built from the fields that
change the result and nothing else, so a cell that is already in the results
bucket is skipped rather than re-run — which is what makes a spot interruption
cost one cell instead of a wave.
"""

from __future__ import annotations

import argparse
import json
import sys

# 12 seeds everywhere a hypothesis is tested. The instrument's own H1 flipped
# from SUPPORTED to NOT SUPPORTED between n=3 and n=6 with the effect size
# unchanged - only the resolution moved. Three seeds is not enough to publish on.
SEEDS = [5170001 + i for i in range(12)]
ANCHOR = ("published-2ms", "adjacent-sum-5")


def cell(wave, arm, hidden, epochs, seed, *, contract=ANCHOR[0], geometry=ANCHOR[1],
         attn_dim=None, attn_layers=None, temporal="intact", surrogate_scale=None,
         clip_grad_norm=None, n_train=8156):
    parts = [wave, arm.replace("+", "-"), f"h{hidden}", f"e{epochs}", contract, geometry]
    if attn_dim is not None:
        parts.append(f"d{attn_dim}l{attn_layers}")
    if temporal != "intact":
        parts.append(temporal)
    if surrogate_scale is not None:
        parts.append(f"ss{surrogate_scale}")
    if clip_grad_norm is not None:
        parts.append(f"clip{clip_grad_norm}")
    parts.append(f"s{seed}")
    return {
        "id": "__".join(parts),
        "wave": wave,
        "arm": arm,
        "hidden": hidden,
        "epochs": epochs,
        "seed": seed,
        "contract": contract,
        "geometry": geometry,
        "attn_dim": attn_dim,
        "attn_layers": attn_layers,
        "temporal": temporal,
        "temporal_seed": 5170001 if temporal != "intact" else None,
        "surrogate_scale": surrogate_scale,
        "clip_grad_norm": clip_grad_norm,
        "n_train": n_train,
        "n_inputs": 700 if geometry == "channels-700" else 140,
    }


def wave1_converged():
    """T1 - the registered next step: does the pilot's +0.1702 survive convergence?

    e400 is the budget the recorded width axis is converged at, so arm A is
    directly comparable to the recorded 0.7032 *on the same machine*; against a
    different machine it is not, which is why A is re-run here rather than cited.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w1", "ff+fixed", 128, 400, seed))
        cells.append(cell("w1", "ff+fixed+attn", 128, 400, seed, attn_dim=32, attn_layers=1))
        cells.append(cell("w1", "ff+fixed", 192, 400, seed))  # capacity control
        # Mechanism, at convergence this time.
        cells.append(cell("w1", "ff+fixed", 128, 400, seed, temporal="bin-shuffled"))
        cells.append(cell("w1", "ff+fixed+attn", 128, 400, seed, attn_dim=32, attn_layers=1,
                          temporal="bin-shuffled"))
    return cells


def wave2_design_space():
    """T2 - the four axes the pilot named as untested. e100, held fixed across the sweep."""
    cells = []
    for seed in SEEDS:
        for dim in (16, 32, 64, 128):
            cells.append(cell("w2dim", "ff+fixed+attn", 128, 100, seed, attn_dim=dim, attn_layers=1))
        for layers in (1, 2, 4):
            cells.append(cell("w2lyr", "ff+fixed+attn", 128, 100, seed, attn_dim=32, attn_layers=layers))
        cells.append(cell("w2dim", "ff+fixed", 128, 100, seed))  # shared control
    return cells


def wave3_scope():
    """T3 - the axes the 0.7378 ceiling is actually scoped on."""
    cells = []
    for seed in SEEDS:
        for hidden in (128, 256, 512, 1024):
            cells.append(cell("w3wid", "ff+fixed+attn", 128 if False else hidden, 400, seed,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w3wid", "ff+fixed", hidden, 400, seed))
        # channels-700 is the binding scope limit on the ceiling and is unrun at
        # convergence for any arm.
        cells.append(cell("w3geo", "ff+fixed+attn", 128, 400, seed, geometry="channels-700",
                          attn_dim=32, attn_layers=1))
        cells.append(cell("w3geo", "ff+fixed", 128, 400, seed, geometry="channels-700"))
        # Resolution invariance was the observation that started the whole
        # "rate coder" reading. Does attention break it?
        for contract in ("published-10ms", "fixed-t100", "fixed-t250", "fixed-t500"):
            cells.append(cell("w3con", "ff+fixed+attn", 128, 100, seed, contract=contract,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w3con", "ff+fixed", 128, 100, seed, contract=contract))
    return cells


def wave4_recurrent():
    """T4 - rec+alif is unmeasured, not refuted. Does the attention read-out change that?

    **The 2026-08-20 run of this wave diverged 24 of 24, and the clipping below is
    why.** The rationale that used to stand here said the record named clipping as
    a lever needed to get a recurrent cell to complete. The record says the
    opposite, in a document titled `MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`
    - "gradient clipping does not rescue rec+alif at h512, and cannot".

    The recurrent arm's own epoch-mean gradient norm exceeds 1.0 in 100 of 100
    epochs, so a 1.0 threshold taken from the healthy `ff+fixed` scale binds on
    essentially every step. That is not outlier suppression, it is unconditional
    renormalisation to a constant norm, and under Adam it removes the
    second-moment damping the arm needs to absorb its excursions and recover.

    Kept as written so the diverged wave stays reproducible. The re-run is
    `wave11_recurrent_unclipped`, registered in
    `results/AMENDMENT_2026-08-22_WAVE4_WITHOUT_CLIPPING.md`.
    See `results/FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`.
    """
    cells = []
    for seed in SEEDS[:6]:
        for scale in (1.0, 0.4):
            cells.append(cell("w4rec", "rec+alif", 256, 100, seed, surrogate_scale=scale,
                              clip_grad_norm=1.0))
            cells.append(cell("w4rec", "rec+alif+attn", 256, 100, seed, attn_dim=32, attn_layers=1,
                              surrogate_scale=scale, clip_grad_norm=1.0))
    return cells


def wave5_budget_ladder():
    """W5 - the budget-invariant convergence test W1-4 could not provide.

    `PREREG_2026-08-19_SHD_ATTENTION_BUDGET_LADDER.md`, registered before any
    e800 cell existed. The e400 rung is wave 1; this adds e800 for both arms so
    the instrument's own final-doubling rule can be applied.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w5bud", "ff+fixed+attn", 128, 800, seed, attn_dim=32, attn_layers=1))
        cells.append(cell("w5bud", "ff+fixed", 128, 800, seed))
    return cells


def wave6_learning_curve():
    """W6 - the same-machine budget ladder.

    `PREREG_2026-08-19_SHD_ATTENTION_LEARNING_CURVE.md`, registered before any
    e20 or e50 cell existed. e100/e400/e800 already exist as waves 2/1/5; these
    are the two rungs that turn a cross-machine inference into a measurement.
    """
    cells = []
    for seed in SEEDS:
        for epochs in (20, 50):
            cells.append(cell("w6crv", "ff+fixed+attn", 128, epochs, seed,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w6crv", "ff+fixed", 128, epochs, seed))
    return cells


def wave7_ladder_floor():
    """W7 - lower the ladder's floor below e20.

    `PREREG_2026-08-20_SHD_ATTENTION_LADDER_FLOOR.md`, registered before any e5
    or e10 cell existed. Wave 6's "20 epochs" is its own floor, not a measured
    convergence point; these rungs are what turn an upper bound into a number -
    or, if the arm is converged at e5 too, into a lower upper bound honestly
    labelled as one.
    """
    cells = []
    for seed in SEEDS:
        for epochs in (5, 10):
            cells.append(cell("w7flr", "ff+fixed+attn", 128, epochs, seed,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w7flr", "ff+fixed", 128, epochs, seed))
    return cells


def wave8_headline_scope():
    """W8 - does the d32/L4 headline survive off the anchor?

    Everything the campaign knows about scope was measured at d32/**L1**: wave 3
    found the gain inverting by h1024 and failing seed-consistency on
    `channels-700`, both with a single attention block. The result the paper
    leads with is d32/**L4**. Carrying wave 3's scope limits over to it is an
    extrapolation across exactly the axis wave 2 showed matters most, so this
    wave measures them instead of assuming them.

    Controls are reused, not re-run: the matched `ff+fixed` cells at h512/h1024
    (`w3wid`) and at `channels-700` (`w3geo`) already exist at e400 from the same
    binary on the same fleet architecture, so a fresh copy would add cost and no
    information. `published-10ms` at e400 has no control on disk, so one is
    generated here.
    """
    cells = []
    for seed in SEEDS:
        # Geometry: the standard 700-channel SHD input, at the headline config.
        cells.append(cell("w8geo", "ff+fixed+attn", 128, 400, seed,
                          geometry="channels-700", attn_dim=32, attn_layers=4))
        # Width: does depth rescue the inversion wave 3 found at L1?
        for hidden in (512, 1024):
            cells.append(cell("w8wid", "ff+fixed+attn", hidden, 400, seed,
                              attn_dim=32, attn_layers=4))
        # Contract: the other literature-comparable contract, at convergence.
        cells.append(cell("w8con", "ff+fixed+attn", 128, 400, seed,
                          contract="published-10ms", attn_dim=32, attn_layers=4))
        cells.append(cell("w8con", "ff+fixed", 128, 400, seed, contract="published-10ms"))
        # Depth ladder at convergence: L1 (wave 1) and L4 (the registered run)
        # exist at e400, L2 does not - a hole exactly where the e100 ladder
        # showed its largest step.
        cells.append(cell("w8lyr", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=2))
    return cells


def wave9_headline_mechanism():
    """W9 - the mechanism control for the configuration the paper leads with.

    The temporal-order claim rests on W1-3: the bin-shuffled arm was worse in
    12/12 seeds at e400. That control was measured at d32/**L1**. The headline is
    d32/**L4**. Wave 8 exists because carrying an L1 scope limit onto an L4 result
    is an extrapolation; carrying an L1 *mechanism control* onto it is the same
    error, applied to the claim rather than to its scope.

    `w9dim` asks the obvious follow-up wave 2 left open: dimension was monotone in
    the gain at e100 and only d32 was ever run at convergence, so d32 is the
    tested configuration, not the chosen one.

    Controls reused: `ff+fixed` bin-shuffled at h128/e400 on the anchor already
    exists from wave 1, same binary, same seeds.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w9shf", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, temporal="bin-shuffled"))
        cells.append(cell("w9dim", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=64, attn_layers=4))
    return cells


def wave10_resolution_ladder():
    """W10 - the temporal-resolution ladder, on the family that isolates it.

    Wave 8's S-5 compared `published-10ms` to `published-2ms`, which varies the
    timestep count, the bin width, AND the per-sample variability of `t` all at
    once. `fixed-tN` divides one fixed 1400 ms window into exactly N frames, so
    every sample has the same `t` and only the resolution moves: 14.0 / 5.6 / 2.8
    ms bins across a 5x span.

    Both arms are generated. Nothing is reused, because no `fixed-t*` cell exists
    at e400 for either arm anywhere in the record - the Azure campaign that
    registered them ran out of credit with all four of its control arms unrun.
    """
    cells = []
    for seed in SEEDS:
        for contract in ("fixed-t100", "fixed-t250", "fixed-t500"):
            cells.append(cell("w10con", "ff+fixed+attn", 128, 400, seed,
                              contract=contract, attn_dim=32, attn_layers=4))
            cells.append(cell("w10con", "ff+fixed", 128, 400, seed, contract=contract))
    return cells


def wave11_recurrent_unclipped():
    """W11 - wave 4, re-run with the flag that diverged it removed.

    Registered in `results/AMENDMENT_2026-08-22_WAVE4_WITHOUT_CLIPPING.md`.
    Identical to `wave4_recurrent` in every respect except `clip_grad_norm`,
    which goes from 1.0 to None: same arms, width, budget, surrogate ladder,
    seeds, contract, geometry and attention configuration.

    Wave 4 diverged 24 of 24 and was written up as the recurrent arm being
    unusable. A paired control - same binary, seed, initial weights and data
    order, differing only in the clip flag - overflows at optimizer step 244
    clipped and completes 100 epochs unclipped. Thirteen `rec+alif` cells at
    exactly this width and budget were already on disk, all unclipped, all with
    zero non-finite events. See
    `results/FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`.

    The registered completion expectation is >= 18 of 24, not 24 of 24: the
    numerical marginality is real and independent of clipping, and the unclipped
    record at this operating point is 13 of 15.
    """
    cells = []
    for seed in SEEDS[:6]:
        for scale in (1.0, 0.4):
            cells.append(cell("w11rec", "rec+alif", 256, 100, seed, surrogate_scale=scale))
            cells.append(cell("w11rec", "rec+alif+attn", 256, 100, seed, attn_dim=32,
                              attn_layers=1, surrogate_scale=scale))
    return cells


def wave12_adaptation_by_attention():
    """W12 - is the read-out adding order sensitivity, or substituting for it?

    Registered in `results/PREREG_2026-08-22_ADAPTATION_BY_ATTENTION.md`.

    Every cell in waves 1-10 that carries the anchor configuration is on
    `ff+fixed`. The campaign therefore cannot distinguish two readings of the
    headline +0.1258:

      * attention adds temporal structure the substrate cannot represent, or
      * attention substitutes for the threshold adaptation this substrate does
        not have.

    ETLP's conclusion - quoted in `binn-lab/experiments/shd_arch_ablation.rs` -
    is that threshold adaptation and a recurrent topology are what a spiking net
    needs for rich temporal structure. Neither is in `ff+fixed`, and attention
    was added instead of either. The factorial has never been run.

    This wave is the adaptation half: {fixed, adaptive} x {rate read-out,
    +attention d32/L4} at the anchor, 12 seeds, e400.

    The `ff+fixed` corners are NOT generated. Twelve seeds of `ff+fixed` (w1) and
    twelve of `ff+fixed+attn` d32/L4 (r1cal) already exist at exactly this
    configuration from the same pinned binary, and are reused under the manifest
    hash check waves 8 and 9 use. 24 new cells, 24 reused.

    **The recurrent half is deferred, and the reason is measured rather than
    assumed.** Wave 11 ran the recurrent arms unclipped at h256/e100 and
    completed 15 of 24 - `rec+alif` 7 of 12, `rec+alif+attn` 8 of 12. Under the
    campaign's own validity rule an arm with any diverged cell reports zero
    usable cells, so a 12-seed recurrent arm at ~60% per-cell completion cannot
    carry a verdict. Making it complete is a numerical-stability question with
    its own registration, not something to spend a wave discovering again.

    No clipping and no surrogate scaling on any cell. Wave 4 diverged 24 of 24
    because `--clip-grad-norm 1.0` bound on essentially every step
    (`FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`), and any scale deviation
    would make these cells incomparable to the reused controls, which were run at
    the registered default.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w12ada", "ff+alif", 128, 400, seed))
        cells.append(cell("w12ada", "ff+alif+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4))
    return cells


def wave13_recurrent_stability():
    """W13 - can the recurrent arms complete at the anchor budget at all?

    Registered in `results/PREREG_2026-08-23_RECURRENT_STABILITY.md`.

    Wave 12 deferred the recurrent half of the substrate factorial because wave
    11 completed 15 of 24 unclipped at h256/e100, and under the campaign's own
    rule an arm with any diverged cell reports zero usable cells. This wave does
    not attempt that measurement. It asks the prior question - **is there an
    operating point at the anchor budget where these arms complete** - and its
    registered outcome is a completion rate, not an accuracy.

    Running the measurement instead would be spending a wave to re-learn wave
    11. The evidence says so rather than intuition:

      * Wave 11 removed the clip that diverged wave 4 and nine cells still died,
        at optimizer steps 438-1035 against wave 4's median of ~176. Removing
        the flag delayed the divergence rather than ending it.
      * **No recurrent cell at h128 has ever run past e20.** The two that exist
        already show the trouble: `rec+alif` completes with a peak gradient norm
        of 1.166e11, and `rec+fixed` - never stress-tested before - records
        **2 non-finite events**, which the validity gate voids. e400 is 12,800
        optimizer steps against e20's ~640, twenty times deeper into the range
        where wave 11's cells died.

    So `rec+fixed` is not the stable alternative to `rec+alif`, and neither has
    been shown to survive the budget the anchor requires.

    Two arms x two surrogate scales x 12 seeds = 48 cells, at the anchor width,
    contract, geometry and budget. No attention: it roughly quadruples the cost
    and wave 11 showed `rec+alif+attn` diverges too, so it buys nothing until a
    substrate completes on its own.

    No clipping, on either lever. `--clip-grad-norm` is what diverged wave 4
    (`FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`), and `--clip-sample-grad-norm`
    is untried at any threshold - introducing an untried parameter into the wave
    that is meant to characterise the baseline would confound exactly the thing
    being measured. It is named in the prereg as the next lever, not this one.
    """
    cells = []
    for seed in SEEDS:
        for arm in ("rec+fixed", "rec+alif"):
            for scale in (1.0, 0.4):
                cells.append(cell("w13rec", arm, 128, 400, seed, surrogate_scale=scale))
    return cells


def wave14_recurrent_measurement():
    """W14 - the recurrent half of the substitution test, at wave 13's operating point.

    Registered in `results/PREREG_2026-08-23_RECURRENT_MEASUREMENT.md`.

    Wave 12 refuted substitution on the adaptation axis: attention's gain is
    +0.1258 on `ff+fixed` and +0.1285 on `ff+alif`, a difference of +0.0027 that
    is positive in 6 of 12 seeds. The recurrence axis was deferred because no
    recurrent arm could complete a 12-seed arm at the anchor budget.

    Wave 13 found the operating point: `rec+alif` at **surrogate scale 0.4**
    completes 11 of 12 at h128 / `published-2ms` / `adjacent-sum-5` / e400.

    **Every arm here runs at scale 0.4**, including the feed-forward pair. That
    is the whole reason `ff+fixed` and `ff+fixed+attn` are regenerated rather
    than reused: the 24 archived anchor controls ran at the registered default
    of 1.0, and comparing a gain measured at 0.4 against one measured at 1.0
    would confound the substrate with the scale - which is the confound this
    wave exists to avoid, not to introduce.

    `rec+alif` itself is **not** generated. Wave 13 ran exactly this
    configuration - same arm, width, budget, contract, geometry, scale, seeds
    and binary - and the instrument is deterministic, so re-running it would
    produce byte-identical cells. The eleven completing cells are reused and the
    twelfth is recorded as diverged.

    36 new cells, 12 reused.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w14sub", "rec+alif+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, surrogate_scale=0.4))
        cells.append(cell("w14sub", "ff+fixed", 128, 400, seed, surrogate_scale=0.4))
        cells.append(cell("w14sub", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, surrogate_scale=0.4))
    return cells


# --- Seeds beyond the original twelve -------------------------------------
#
# `SEEDS` is the terminal seed count for the campaigns registered against it:
# `PREREG_2026-08-20_AZURE_D32L4_SCOPE.md` says "twelve is the terminal seed
# count; no thirteenth seed is available to rescue a marginal verdict", and that
# binds every hypothesis registered under it. It does not bind a *new*
# registration, which may declare a larger n in advance — the rule exists to
# stop seeds being added until a marginal result tips, not to cap precision
# forever.
#
# These continue the same arithmetic sequence, so a cell at seed 5170013 is
# generated by the same rule as one at 5170001 and neither was chosen.
SEEDS_EXTENDED = [5170001 + i for i in range(32)]
#: Wave 18's ladder runs twenty seeds: the twelve that make the H18-4
#: byte-identity check against `w15col` possible, plus eight more.
SEEDS_W18 = [5170001 + i for i in range(20)]


def wave15_the_h1024_collapse():
    """W15 - is the h1024/L4 collapse an optimisation failure or a capacity limit?

    Registered in `results/PREREG_2026-08-25_THE_H1024_COLLAPSE.md`.

    Wave 8 measured the read-out's gain inverting at h1024 (-0.1618) and the
    paper carries that as one of two load-bearing scope limits. Beside every one
    of those cells the archive also carries a gradient norm, and nobody read it.

        arm                     epoch-mean norm (median)   accuracy sd
        h128  d32/L1                     0.023               0.0163
        h512  d32/L1                     0.023               0.0276
        h1024 d32/L1                     0.025               0.0094
        h512  d32/L4                     0.460               0.0192
        h1024 d32/L4                    55.494               0.0866
        h1024 rate-only                  ~1.0                 --

    Across the 24 h1024 attention cells (d32/L4 and d64/L4), Spearman rho
    between accuracy and log10 max gradient norm is **-0.970**. And the arm is
    not sick at that width in general: **h1024 at L1 is completely healthy** -
    0.7227 accuracy, sd 0.0094, epoch-mean norm 0.025, indistinguishable from
    h128 and h512 at the same depth. So this is not width. It is depth AND
    width together, and only in the arm with the read-out attached: the
    rate-only control at h1024 has norms of order 1 and does not collapse.

    Every collapsing cell passes the preregistered validity gate - 20 classes,
    majority near 0.10, no non-finite events. They are not degenerate readouts.
    They are numerically sick, which `cell_validity.py` reports and deliberately
    never voids; its own header records 1.13e8 at h1024 as the largest norm in
    the corpus without connecting it to the arm's collapse.

    ## The levers, and why these thresholds

    `--surrogate-scale` reduces gradient magnitude at source and is the lever
    wave 13 found stabilises `rec+alif`. It is the primary.

    `--clip-grad-norm` is the secondary and needs its threshold justified,
    because clipping is what destroyed wave 4
    (`FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`). The mechanism there
    was that a threshold of 1.0 sat *below* the arm's typical norm, so it bound
    on essentially every step: not outlier suppression but unconditional
    renormalisation, which removes the Adam second-moment damping that lets the
    arm absorb excursions.

    So the threshold is taken from this arm's own pooled epoch-mean
    distribution rather than from a healthy arm's:

        p50 = 42.3    p75 = 186.2    p90 = 1298.9    p95 = 3068.1

    **1000.0** binds on about a tenth of epochs - outlier suppression, with the
    typical step untouched. A threshold of 20.0 would have bound on ~65% of them
    and is exactly the wave-4 regime; it is named here so that the choice is
    visibly a choice, and it is not run.

    ## Controls

    `h512 d32/L4` under the same clip is a **no-op control**: that arm's epoch
    means run 0.005-0.753 and its epoch maxima reach 15.8, so a 1000.0 threshold
    can never bind and the cells must come back **byte-identical** to the
    archived w8wid cells. Wave 4 showed the clipping implementation can be
    destructive when it binds; nothing has shown it is inert when it does not.
    This is that check, and it fails loudly if the flag perturbs a run it should
    not touch.

    `h1024 rate-only` under the same clip separates a change in the gain from a
    change in the control beneath it.

    `h1024 d32/L2` fills the depth axis at the collapsing width. L1 is healthy
    and L4 is not; if the explosion is monotone in depth, L2 sits between, and
    if it is a threshold, L2 is on one side or the other.

    72 new cells. The scale-1.0 h1024 arm and the unclipped h1024 rate control
    are reused from w8wid and w3wid - same binary, same seeds, deterministic
    instrument, so re-running them would produce byte-identical output.
    """
    cells = []
    for seed in SEEDS:
        # The depth rung the axis is missing at the collapsing width.
        cells.append(cell("w15col", "ff+fixed+attn", 1024, 400, seed,
                          attn_dim=32, attn_layers=2))
        # Primary lever: reduce the gradient at source.
        for scale in (0.5, 0.25):
            cells.append(cell("w15col", "ff+fixed+attn", 1024, 400, seed,
                              attn_dim=32, attn_layers=4, surrogate_scale=scale))
        # Secondary lever, threshold from this arm's own p90.
        cells.append(cell("w15col", "ff+fixed+attn", 1024, 400, seed,
                          attn_dim=32, attn_layers=4, clip_grad_norm=1000.0))
        # Control: the same lever on the arm beneath the gain.
        cells.append(cell("w15col", "ff+fixed", 1024, 400, seed,
                          clip_grad_norm=1000.0))
        # No-op control: the lever cannot bind here, so these must reproduce
        # the archived w8wid cells byte for byte.
        cells.append(cell("w15col", "ff+fixed+attn", 512, 400, seed,
                          attn_dim=32, attn_layers=4, clip_grad_norm=1000.0))
    return cells


def wave16_width_ladder_filled():
    """W16 - locate the collapse, on a ladder with no rung resting on four cells.

    Registered in `results/PREREG_2026-08-25_THE_H1024_COLLAPSE.md` section 5.

    The d32/L4 width ladder currently reads +0.1258 at h128, +0.0962 at h256,
    +0.0876 at h512, -0.1618 at h1024. The h256 rung is **four cells from the
    truncated Azure campaign** - the only ones of their kind anywhere - and
    between h512 and h1024 there is nothing at all. A gain that decays gently
    and then falls 0.25 in one doubling is either a threshold or an unmeasured
    slope, and four points cannot tell them apart.

    h256 is regenerated at the full seed count rather than reusing the four
    Azure cells, because a rung at n=4 beside rungs at n=12 makes the ladder's
    shape depend on which rung you trust. The four Azure cells then become a
    cross-ISA check on the new ones rather than the evidence itself.

    96 new cells; rate controls at h256 and h512 are reused from w3wid.
    """
    cells = []
    for seed in SEEDS:
        for hidden in (256, 384, 768):
            cells.append(cell("w16lad", "ff+fixed+attn", hidden, 400, seed,
                              attn_dim=32, attn_layers=4))
            # h384 and h768 have no archived rate control; h256 does, but is
            # regenerated here so every rung of the ladder is measured the same
            # way rather than three from one campaign and one from another.
            cells.append(cell("w16lad", "ff+fixed", hidden, 400, seed))
    return cells


def wave17_headline_at_thirty_two_seeds():
    """W17 - the headline and its mechanism control, at n=32 rather than n=12.

    Registered in `results/PREREG_2026-08-25_THE_H1024_COLLAPSE.md` section 6.

    Every headline number in the paper rests on twelve seeds: the 0.8320, the
    +0.1258, the 12/12 above 0.80, and the bin-shuffle contrast that carries the
    whole mechanism claim. Twelve was the terminal count for those
    registrations and it is not being extended to rescue anything - all four
    numbers already clear their bars comfortably. It is being extended because
    the mechanism claim is the paper's strongest result and n=12 is a thin base
    for it.

    Only the twenty new seeds are generated. The archived twelve are
    bit-identical under the same pinned binary and are reused, giving n=32.

    80 new cells, 48 reused.
    """
    cells = []
    for seed in SEEDS_EXTENDED[12:]:
        cells.append(cell("w17hdl", "ff+fixed", 128, 400, seed))
        cells.append(cell("w17hdl", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4))
        cells.append(cell("w17hdl", "ff+fixed", 128, 400, seed,
                          temporal="bin-shuffled"))
        cells.append(cell("w17hdl", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, temporal="bin-shuffled"))
    return cells


def wave18_depth_ladder_at_h1024():
    """W18 - is the optimum in read-out depth at h1024 interior?

    Registered in `results/PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`.

    Wave 15 registered L2 as lying *between* L1's -0.0159 and L4's -0.1618 and
    got +0.0392, positive in 12/12 - outside the interval, above both endpoints,
    in the one direction the registration had no branch for. H15-3 is NOT MET as
    registered, and re-reading its own cells is not a licence to reinterpret it.

    All four depths are regenerated rather than assembled from three campaigns.
    The h256 rung of wave 16's width ladder was rejected for resting on four
    Azure cells while its neighbours rested on twelve; a depth ladder with two
    archived rungs, one new rung and one missing rung has the same defect in a
    different axis. Seeds 1-12 of the L2 arm duplicate cells `w15col` has
    already produced, which makes them the wave's harness check (H18-4) rather
    than waste - the first registered check in this campaign that would catch a
    silent change in the execution environment rather than in the code.

    100 cells: four depths x 20 seeds, plus a 20-seed h1024 rate-only control.
    """
    cells = []
    for seed in SEEDS_W18:
        for layers in (1, 2, 3, 4):
            cells.append(cell("w18dep", "ff+fixed+attn", 1024, 400, seed,
                              attn_dim=32, attn_layers=layers))
        # The ladder's own control, at the ladder's own seed count. w3wid's
        # h1024 rate arm stops at twelve, and a control that runs out before its
        # treatments turns the last eight seeds into unpaired cells.
        cells.append(cell("w18dep", "ff+fixed", 1024, 400, seed))
    return cells


def wave19_does_the_optimum_move_with_width():
    """W19 - does the best read-out depth fall as width rises?

    Registered in `results/PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`
    section 3, and evaluable independently of wave 18's own hypotheses.

    At h512 the deeper read-out wins (+0.0876 at L4 against +0.0043 at L1); at
    h1024 it collapses. If optimal depth falls as width rises, h768 sits between
    - L4 still ahead of L2, but by less than at h512. Only the h768/L2 arm is
    missing: h768/L4 and the h768 rate control are `w16lad` cells from this same
    campaign, same fleet, same binary, same twelve seeds.

    12 cells.
    """
    return [cell("w19int", "ff+fixed+attn", 768, 400, seed,
                 attn_dim=32, attn_layers=2)
            for seed in SEEDS]


def wave20_the_recurrent_claim_at_thirty_two_seeds():
    """W20 - the paper's most fragile claim, at n=32.

    Registered in
    `results/PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md`.

    PAPER_DRAFT section 3.7 states that its recurrent comparison "rests on ten
    pairs, the registered minimum" and that "one further loss on either would
    have made the comparison unreportable". It also states that the surviving
    pairs are those that did not diverge and that divergence is not random.
    Over those ten pairs the Spearman correlation between the paired gain and
    log10 of the peak gradient norm is -0.648: among the cells that completed,
    the ones nearest divergence show the smaller gains.

    Twenty more seeds on each of four arms retires the first limit and makes the
    second measurable. The feed-forward reference is extended too, because the
    claim is a DIFFERENCE of gains and pairing thirty-two against twelve would
    silently drop back to twelve.

    Every arm at surrogate scale 0.4, the constraint wave 14 established so that
    substrate and scale cannot be confounded. 80 new cells; the archived twelve
    of each arm are reused under the same pinned binary.
    """
    cells = []
    for seed in SEEDS_EXTENDED[12:]:
        cells.append(cell("w20rec", "rec+alif", 128, 400, seed, surrogate_scale=0.4))
        cells.append(cell("w20rec", "rec+alif+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, surrogate_scale=0.4))
        cells.append(cell("w20rec", "ff+fixed", 128, 400, seed, surrogate_scale=0.4))
        cells.append(cell("w20rec", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, surrogate_scale=0.4))
    return cells


def wave21_the_mechanism_across_the_design_space():
    """W21 - the paper's lead control, everywhere except where it already is.

    `PAPER_DRAFT.md` leads with a difference-in-differences: attention's cost
    under bin-shuffling against the rate read-out's own cost, +0.1347 against
    +0.0142 at n=32. `scripts/mechanism_coverage.py` derives where that contrast
    can actually be computed and the answer is **two of nineteen operating
    points, both h128 / `published-2ms` / `adjacent-sum-5`**. Seventeen rungs
    carry the intact arms and no destruction control at all.

    So the campaign's best-evidenced claim is also its narrowest, and a reviewer
    asking whether the mechanism generalises has nothing to read. This wave is
    that question.

    It also makes the width collapse testable rather than merely reported. §3.5
    calls the h1024 inversion an anomaly "with no citation to lean on". If the
    read-out's benefit is what temporal order buys, then where the benefit
    inverts there should be nothing order-dependent left to destroy, and the
    shuffle cost should collapse with it. That is a prediction the corpus cannot
    currently answer at any width but one.

    Two arms per operating point, twelve seeds, everything else reused:

      * `ff+fixed` bin-shuffled -- the rate read-out's own shuffle cost, which
        is what makes this a difference of differences rather than a drop. It
        exists only at h128 on the anchor; every other width and geometry needs
        its own, and at 4-24 minutes a cell it is the cheap half.
      * `ff+fixed+attn` d32/L4 bin-shuffled.

    The intact halves of all fourteen arms are already in the corpus at n=12,
    from the same pinned binary and the same seeds.

    Five widths complete the ladder wave 16 filled for the gain, so the gain and
    its shuffle cost can be read against each other rung by rung rather than at
    a single point. Two further points move the geometry and the contract
    instead of the width, because "generalises across width" and "generalises
    across binning" are different claims and the paper currently supports
    neither.
    """
    cells = []
    points = [
        # (hidden, contract, geometry)
        (256, ANCHOR[0], ANCHOR[1]),
        (384, ANCHOR[0], ANCHOR[1]),
        (512, ANCHOR[0], ANCHOR[1]),
        (768, ANCHOR[0], ANCHOR[1]),
        (1024, ANCHOR[0], ANCHOR[1]),
        (128, ANCHOR[0], "channels-700"),
        (128, "published-10ms", ANCHOR[1]),
    ]
    for hidden, contract, geometry in points:
        for seed in SEEDS:
            cells.append(cell("w21mec", "ff+fixed", hidden, 400, seed,
                              contract=contract, geometry=geometry,
                              temporal="bin-shuffled"))
            cells.append(cell("w21mec", "ff+fixed+attn", hidden, 400, seed,
                              contract=contract, geometry=geometry,
                              attn_dim=32, attn_layers=4,
                              temporal="bin-shuffled"))
    return cells


WAVES = {
    "w1": wave1_converged,
    "w2": wave2_design_space,
    "w3": wave3_scope,
    "w4": wave4_recurrent,
    "w5": wave5_budget_ladder,
    "w6": wave6_learning_curve,
    "w7": wave7_ladder_floor,
    "w8": wave8_headline_scope,
    "w9": wave9_headline_mechanism,
    "w10": wave10_resolution_ladder,
    "w11": wave11_recurrent_unclipped,
    "w12": wave12_adaptation_by_attention,
    "w13": wave13_recurrent_stability,
    "w14": wave14_recurrent_measurement,
    "w15": wave15_the_h1024_collapse,
    "w16": wave16_width_ladder_filled,
    "w17": wave17_headline_at_thirty_two_seeds,
    "w18": wave18_depth_ladder_at_h1024,
    "w19": wave19_does_the_optimum_move_with_width,
    "w20": wave20_the_recurrent_claim_at_thirty_two_seeds,
    "w21": wave21_the_mechanism_across_the_design_space,
}


def estimated_seconds(cell: dict) -> float:
    """Rough single-core cost, for scheduling only. Never used in analysis.

    Mirrors `estimate_cost.py`'s calibration: a 9.6 s/epoch base at h128 scaled
    by width, plus an attention term whose core scales with timesteps squared.
    Precision does not matter - only the ordering does.
    """
    timesteps = {"published-2ms": 358, "published-10ms": 72,
                 "fixed-t100": 100, "fixed-t250": 250, "fixed-t500": 500}
    t = timesteps[cell["contract"]]
    base = 9.6 * (cell["hidden"] / 128)
    if cell["geometry"] == "channels-700":
        base *= 1.15
    if cell["arm"].startswith("rec"):
        base *= 3.0 * (cell["hidden"] / 256)
    attention = 0.0
    if cell["attn_dim"]:
        core = 58.1 * 0.74 * (t / 358) ** 2 * (cell["attn_dim"] / 32) * cell["attn_layers"]
        io = 58.1 * 0.26 * (t / 358) * (cell["hidden"] / 128) * (cell["attn_dim"] / 32)
        attention = core + io
    return (base + attention) * cell["epochs"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--waves", default="w1,w2,w3,w4,w5,w6,w7",
                        help="comma-separated subset of " + ",".join(WAVES))
    parser.add_argument("--out", default="-")
    parser.add_argument("--priority", default="w1,w6,w7",
                        help="comma-separated wave-label prefixes to schedule first. "
                             "Must match at least one cell or the plan is refused - "
                             "a priority set that silently matches nothing is how the "
                             "cheapest wave ended up queued last.")
    args = parser.parse_args()

    cells = []
    for name in args.waves.split(","):
        name = name.strip()
        if name not in WAVES:
            parser.error(f"unknown wave {name!r}; expected some of {sorted(WAVES)}")
        cells.extend(WAVES[name]())

    seen = {}
    for entry in cells:
        if entry["id"] in seen:
            raise SystemExit(f"duplicate cell id {entry['id']} - the plan is not injective")
        seen[entry["id"]] = entry

    # Longest-processing-time-first. `claim_next.py` takes the first unclaimed
    # cell in plan order, so plan order IS the schedule.
    #
    # Registration order puts the 8-14 hour h1024 cells near the end, where they
    # start after everything else has drained and each one then extends the
    # campaign by its full length. Sorting longest-first starts them immediately,
    # so they run underneath the short cells instead of after them, and the tail
    # is made of minutes rather than hours. This is the standard makespan result
    # and it is worth more here than any thread-count tuning.
    #
    # Purely a scheduling change: ids, seeds, arms and thresholds are untouched,
    # and a cell computes the same thing whenever it runs.
    # Wave 1 first, then longest-first across everything else.
    #
    # Pure longest-processing-time-first minimises makespan but buries wave 1 -
    # the primary contrast, and the only wave a paper can be written from -
    # behind the 26-hour h1024 cells. Finishing all 420 cells four hours sooner
    # is worth less than having the headline result a day earlier, and wave 1 is
    # only 60 cells, so front-loading it costs little of the makespan gain.
    #
    # Within each group, longest-first still applies, so each group's own tail is
    # short.
    # Prefix match, not exact: wave labels carry a suffix (`w6crv`, `w2dim`), so
    # an exact `in ("w1", "w6")` test silently never matched w6 and left the
    # cheapest, most decision-relevant wave queued at index 336 of 468.
    priority = tuple(p.strip() for p in args.priority.split(",") if p.strip())
    # Controls before treatments, then longest-first within each.
    #
    # Longest-first alone minimises makespan and is why waves 15-17 reached the
    # halfway mark with every attention arm at 12/12 and every rate control at
    # zero: 112 cells done and almost nothing pairable. A gain needs both arms,
    # so a schedule that finishes all the treatments first has produced no
    # evidence at all until it is nearly done -- and a spot reclaim at 50% would
    # have thrown away half a campaign for one usable contrast.
    #
    # Controls are the cheap half by a wide margin (0.09-0.65 h against 3-7 h),
    # so running them first costs almost nothing on the makespan -- under 6% of
    # wave 20's slot-hours -- and makes every treatment cell pairable the moment
    # it lands.
    def is_treatment(c):
        return c["attn_dim"] is not None
    cells.sort(key=lambda c: (not c["wave"].startswith(priority),
                              is_treatment(c),
                              -estimated_seconds(c)))
    if not priority:
        # Shortest-processing-time-first: maximises the number of COMPLETED
        # cells at any point, so a campaign that stops early leaves whole
        # evaluable arms rather than a set of half-finished expensive ones.
        # This is the deliberate opposite of the longest-first default below.
        cells.sort(key=estimated_seconds)
        print(f"scheduled shortest-first: {len(cells)} cells", file=sys.stderr)
        return emit(cells, args)
    promoted = sum(1 for c in cells if c["wave"].startswith(priority))
    if promoted == 0:
        raise SystemExit(f"priority waves {priority} matched no cell - check the labels")
    # The other half of the same defect. A prefix set that matches NOTHING was
    # already refused above; a prefix set that matches EVERYTHING was not, and is
    # just as silent. On 2026-08-25 the default `w1,w6,w7` matched all 224 cells
    # of waves w15col, w16lad and w17hdl -- `w1` is a prefix of `w15` and `w16`
    # and `w17` -- so "priority" promoted the entire plan, the tie-break fell
    # through to longest-first, and the campaign was scheduled most-expensive
    # cell first with the cheap decisive waves at the end.
    #
    # That is precisely the ordering that cost the Azure campaign every arm it
    # cared about when it stopped early
    # (`results/RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md`), and the
    # lesson was already written down here when it happened again.
    if promoted == len(cells) and len(set(c["wave"] for c in cells)) > 1:
        raise SystemExit(
            f"priority waves {priority} matched ALL {promoted} cells across "
            f"{sorted(set(c['wave'] for c in cells))} - the prefixes are too "
            "short to prioritise anything, so the plan would be ordered purely "
            "longest-first. Name the waves precisely, or pass --priority '' to "
            "schedule shortest-first on purpose."
        )
    print(f"scheduled first: {promoted} cells from waves {priority}", file=sys.stderr)

    return emit(cells, args)


def emit(cells, args) -> int:
    payload = json.dumps(cells, indent=2) + "\n"
    if args.out == "-":
        sys.stdout.write(payload)
    else:
        with open(args.out, "w") as handle:
            handle.write(payload)
        print(f"{len(cells)} cells -> {args.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
