#!/usr/bin/env python3
"""Waves 15-17 verdicts, exactly as registered in
`results/PREREG_2026-08-25_THE_H1024_COLLAPSE.md`.

**Frozen in the same commit as that preregistration and before the first cell of
any of these waves existed.** Waves 8-12 froze their analysers before launch and
wave 14 did not, which cost it a correction it should not have needed; this one
follows the earlier practice.

Three things here are places where a silent bug would change a verdict rather
than crash, so each is stated:

  * **H15-2 is tested on gradient norms, not accuracy.** The whole point of
    registering it separately is that "the gain came back" and "the numerics were
    repaired" are different claims. If the first holds and the second does not,
    this analyser must say so rather than let the campaign's own hypothesis
    supply the explanation.
  * **H15-4 is a byte-identity check against the archive**, not a statistical
    one. A clip threshold that cannot bind must leave a run untouched. If it does
    not, every clipped cell in the wave is void -- INCLUDING the ones supporting
    H15-1, which is the direction that costs the campaign its best result.
  * **Every comparison is paired on seed** over cells where both arms are valid,
    and a comparison below its registered pair floor carries **no numbers at
    all**. A mean printed beside a NOT EVALUABLE banner is the shape this
    repository keeps finding.

    python3 scripts/aws/analyse_wave15.py --plan results/shd_attention_campaign_v2/plan_w15_17.json \\
        --results <dir of this wave's cells> [--failures DIR] [--out FILE]

**Point `--results` at a staging directory while the campaign is in flight, not
at `results/shd_attention_campaign_v2/`.** That corpus is baselined by
`test_campaign_tooling.py::test_the_archived_corpus_is_unaffected`, which fires
on any addition -- correctly, since its whole job is to make landing cells a
deliberate act rather than a side effect of collecting them. Dropping 47
in-flight cells into it turned the record gate red on a campaign that had not
finished. Reused controls are read from the archives by absolute path
regardless, so a staging directory analyses identically; land the cells once
when the wave completes and re-freeze the baseline in the same commit.
"""

from __future__ import annotations

import argparse
import json
import os
import statistics
import sys
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from cell_validity import stability_warnings, validity_problems  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent.parent

SEEDS = [5170001 + i for i in range(12)]
SEEDS_EXTENDED = [5170001 + i for i in range(32)]

# --- Registered thresholds, named so a reader can check them against the prereg
#     without reading the code that applies them. -----------------------------
H15_1_GAIN = 0.05           # paired gain a lever must reach at h1024/d32/L4
H15_1_POSITIVE = 9          # of 12
H15_2_HEALTHY_NORM = 1.0    # median epoch-mean norm, the scale of every healthy arm
H15_3_MARGIN = 0.01         # L2 must sit inside (L4, L1) by this much at each end
H16_1_SEPARATION = 0.005    # adjacent rungs must differ by at least this
H16_2_FACTOR = 3.0          # collapse must exceed this multiple of the largest gap
H17_GAIN = 0.05
H17_POSITIVE = 24           # of 32
H17_GATE = 0.80
H17_GATE_SEEDS = 24         # of 32
H17_2_SHUFFLE_FACTOR = 5.0  # attention's shuffle cost vs the rate arm's
MIN_VALID_PER_ARM = 9       # below this an arm is NOT EVALUABLE

#: Archived paired gains at h1024, from `RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`
#: and `w3wid`. H15-3 is stated relative to these two and to nothing else.
ARCHIVED_H1024_L1_GAIN = -0.0159
ARCHIVED_H1024_L4_GAIN = -0.1618

#: Archived corpora holding the reused controls. Same pinned binary, same seeds,
#: deterministic instrument -- re-running them would produce byte-identical
#: cells, so they are read rather than regenerated.
ARCHIVE_V1 = ROOT / "results/shd_attention_campaign_v1/cells"
ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"


def load(root: Path, stem: str, seeds) -> dict[int, dict]:
    """Valid cells for one arm, keyed by seed. Invalid cells are dropped here
    and counted by the caller -- never silently."""
    out = {}
    for seed in seeds:
        path = root / f"{stem}__s{seed}.json"
        if not path.is_file():
            continue
        cell = json.loads(path.read_text())
        if validity_problems(cell):
            continue
        out[seed] = cell
    return out


def paired(treatment: dict[int, dict], control: dict[int, dict]):
    """(mean gain, positive count, pairs). Paired on seed, never pooled."""
    shared = sorted(set(treatment) & set(control))
    if not shared:
        return None, 0, 0
    deltas = [treatment[s]["accuracy"] - control[s]["accuracy"] for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(shared)


def median_epoch_mean_norm(cells: dict[int, dict]) -> float | None:
    """The statistic H15-2 is registered on: median over cells of the median
    epoch-mean gradient norm. Medians rather than means because the quantity
    spans eight orders of magnitude and one exploded cell would carry a mean."""
    per_cell = [statistics.median(c["epoch_mean_gradient_norm"])
                for c in cells.values() if c.get("epoch_mean_gradient_norm")]
    return statistics.median(per_cell) if per_cell else None


SCIENTIFIC_FIELDS = (
    "accuracy", "mean_loss", "mean_gradient_norm", "mean_update_rms",
    "mean_firing_rate", "silent_fraction", "saturated_fraction",
    "majority_prediction", "classes_predicted", "non_finite_events",
    "tail_loss_improvement", "epoch_mean_loss", "epoch_mean_gradient_norm",
    "epoch_max_gradient_norm", "epoch_max_gradient_step",
)


def byte_identical(new: dict, archived: dict) -> list[str]:
    """Every scientific field, by repr so 1.0 and 1 cannot compare equal.

    `clip_grad_norm` itself is expected to differ -- it is the flag under test.
    """
    differing = []
    for field in SCIENTIFIC_FIELDS:
        if field not in new or field not in archived:
            differing.append(f"{field}: absent from one cell")
        elif repr(new[field]) != repr(archived[field]):
            differing.append(field)
    return differing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--failures")
    parser.add_argument("--out")
    args = parser.parse_args()

    plan = {e["id"]: e for e in json.loads(Path(args.plan).read_text())}
    res = Path(args.results)
    anchor = "published-2ms__adjacent-sum-5"
    lines: list[str] = ["# Waves 15-17 verdicts", ""]

    ran = sum(1 for cid in plan if (res / f"{cid}.json").is_file())
    invalid = []
    for cid in plan:
        path = res / f"{cid}.json"
        if path.is_file():
            problems = validity_problems(json.loads(path.read_text()))
            if problems:
                invalid.append((cid, problems))
    failures = len(list(Path(args.failures).glob("*.json"))) if args.failures else 0
    lines.append(f"Coverage: **{ran - len(invalid)} valid / {len(plan)} planned**, "
                 f"{len(invalid)} invalid, {failures} failures, "
                 f"{len(plan) - ran} missing.")
    if invalid:
        lines.extend(["", "Invalid cells, reported rather than dropped:", ""])
        lines += [f"- `{cid}`: {'; '.join(p)}" for cid, p in invalid]

    # --- H15-1 / H15-2: the levers -----------------------------------------
    control = load(ARCHIVE_V1, f"w3wid__ff-fixed__h1024__e400__{anchor}", SEEDS)
    lines.extend(["", "## H15-1 — does any lever restore the gain at h1024/d32/L4?", "",
                  "| lever | pairs | gain | positive | median epoch-mean norm | H15-1 | H15-2 |",
                  "|---|---:|---:|---:|---:|---|---|"])
    levers = [
        ("surrogate scale 0.5", f"w15col__ff-fixed-attn__h1024__e400__{anchor}__d32l4__ss0.5"),
        ("surrogate scale 0.25", f"w15col__ff-fixed-attn__h1024__e400__{anchor}__d32l4__ss0.25"),
        ("clip-grad-norm 1000.0", f"w15col__ff-fixed-attn__h1024__e400__{anchor}__d32l4__clip1000.0"),
    ]
    any_recovered = False
    for label, stem in levers:
        cells = load(res, stem, SEEDS)
        if len(cells) < MIN_VALID_PER_ARM:
            lines.append(f"| {label} | {len(cells)} valid | — | — | — | "
                         f"**NOT EVALUABLE** | **NOT EVALUABLE** |")
            continue
        gain, positive, pairs = paired(cells, control)
        norm = median_epoch_mean_norm(cells)
        met = gain is not None and gain >= H15_1_GAIN and positive >= H15_1_POSITIVE
        healthy = norm is not None and norm < H15_2_HEALTHY_NORM
        any_recovered |= met
        lines.append(
            f"| {label} | {pairs} | {gain:+.4f} | {positive}/{pairs} | {norm:.3f} | "
            f"**{'MET' if met else 'NOT MET'}** | "
            f"**{'MET' if healthy else 'NOT MET'}** |")
    lines.append("")
    lines.append(f"**H15-1: {'MET' if any_recovered else 'NOT MET'}** "
                 f"(bar: gain >= {H15_1_GAIN:+.2f}, positive in >= {H15_1_POSITIVE}/12).")
    if any_recovered:
        lines.append("H15-2 is read only for the arms that met H15-1: an arm whose "
                     "accuracy recovered while its norms stayed high is reported as "
                     "**recovery by an unidentified mechanism** (outcome O-3), and "
                     "the correlation in the prereg does not get to be the "
                     "explanation.")

    # --- H15-3: depth is the axis ------------------------------------------
    l2 = load(res, f"w15col__ff-fixed-attn__h1024__e400__{anchor}__d32l2", SEEDS)
    lines.extend(["", "## H15-3 — is L2 between L1 and L4 at h1024?", ""])
    if len(l2) < MIN_VALID_PER_ARM:
        lines.append(f"**NOT EVALUABLE** — {len(l2)} valid cells, floor {MIN_VALID_PER_ARM}.")
    else:
        gain, positive, pairs = paired(l2, control)
        inside = (ARCHIVED_H1024_L4_GAIN + H15_3_MARGIN < gain
                  < ARCHIVED_H1024_L1_GAIN - H15_3_MARGIN)
        lines.extend([
            f"| depth | gain | source |", "|---|---:|---|",
            f"| L1 | {ARCHIVED_H1024_L1_GAIN:+.4f} | archived `w3wid` |",
            f"| **L2** | **{gain:+.4f}** | this wave, {pairs} pairs, "
            f"{positive}/{pairs} positive |",
            f"| L4 | {ARCHIVED_H1024_L4_GAIN:+.4f} | archived `w8wid` |", "",
            f"**H15-3: {'MET' if inside else 'NOT MET'}** — L2 must lie strictly "
            f"inside ({ARCHIVED_H1024_L4_GAIN:+.4f}, {ARCHIVED_H1024_L1_GAIN:+.4f}) "
            f"by at least {H15_3_MARGIN}.",
        ])

    # --- H15-4: the no-op control ------------------------------------------
    lines.extend(["", "## H15-4 — is the clip inert where it cannot bind?", ""])
    mismatches, compared = [], 0
    for seed in SEEDS:
        new = res / (f"w15col__ff-fixed-attn__h512__e400__{anchor}__d32l4"
                     f"__clip1000.0__s{seed}.json")
        old = ARCHIVE_V2 / f"w8wid__ff-fixed-attn__h512__e400__{anchor}__d32l4__s{seed}.json"
        if not (new.is_file() and old.is_file()):
            continue
        compared += 1
        differing = byte_identical(json.loads(new.read_text()), json.loads(old.read_text()))
        if differing:
            mismatches.append((seed, differing))
    if compared == 0:
        lines.append("**NOT EVALUABLE** — no h512 clipped cell could be compared.")
    elif mismatches:
        lines.append(f"**H15-4: NOT MET** — {len(mismatches)} of {compared} cells differ "
                     "from the archive. The clip flag perturbs runs it cannot bind on, so "
                     "**every clipped cell in this wave is void**, including any that "
                     "supported H15-1 above. Outcome O-4.")
        lines += [f"- seed {s}: {', '.join(f[:6])}" for s, f in mismatches]
    else:
        lines.append(f"**H15-4: MET** — {compared}/{compared} cells byte-identical to "
                     "the archive across every scientific field. The clip is inert "
                     "below its threshold, so the h1024 clipped arm measures clipping "
                     "and not the flag.")

    # --- The clipped rate control: reporting only, no verdict depends on it ---
    #
    # Added 2026-08-26, and it changes NO verdict logic. H15-1's comparator is
    # registered as "`ff+fixed` (unclipped, archived)" and stays exactly that.
    #
    # But the prereg planned twelve `ff+fixed` h1024 cells under the same clip,
    # to "separate a change in the gain from a change in the control beneath
    # it", and this analyser did not print them anywhere. A planned control
    # whose result goes unreported is the defect this repository just spent a
    # day removing from the Azure record, where 35 cells sat in five partially
    # run arms that the write-up passed over in silence.
    #
    # The expectation, stated before these cells land: the h1024 rate arm's
    # epoch-mean norms run ~1.0 with maxima ~1.2, so a 1000.0 threshold cannot
    # bind and these must come back byte-identical to the archive -- the same
    # prediction H15-4 makes for h512, and it must be checked rather than
    # assumed, because assuming it is how the control stops being a control.
    lines.extend(["", "## Clipped rate control (reporting only)", ""])
    clipped_ctl = load(res, f"w15col__ff-fixed__h1024__e400__{anchor}__clip1000.0", SEEDS)
    archived_ctl = load(ARCHIVE_V1, f"w3wid__ff-fixed__h1024__e400__{anchor}", SEEDS)
    shared_ctl = sorted(set(clipped_ctl) & set(archived_ctl))
    if not shared_ctl:
        lines.append("No clipped rate-control cell has landed yet.")
    else:
        moved = [s for s in shared_ctl
                 if byte_identical(clipped_ctl[s], archived_ctl[s])]
        clipped_mean = statistics.fmean(clipped_ctl[s]["accuracy"] for s in shared_ctl)
        archived_mean = statistics.fmean(archived_ctl[s]["accuracy"] for s in shared_ctl)
        lines.extend([
            f"| | clipped | archived unclipped | difference |", "|---|---:|---:|---:|",
            f"| mean accuracy ({len(shared_ctl)} pairs) | {clipped_mean:.6f} "
            f"| {archived_mean:.6f} | {clipped_mean - archived_mean:+.6f} |", "",
        ])
        if moved:
            lines.append(
                f"**The clip moved the control on {len(moved)} of {len(shared_ctl)} "
                "cells.** H15-1's registered comparator is the unclipped archive and "
                "does not change, but the clipped arm's gain can no longer be read as "
                "a property of the treatment alone, and the difference above must be "
                "carried beside it.")
        else:
            lines.append(
                f"**Inert: {len(shared_ctl)}/{len(shared_ctl)} byte-identical to the "
                "archive.** The clip cannot bind on the rate arm, so the clipped "
                "treatment's gain over the unclipped archive is not confounded by the "
                "control moving underneath it.")

    # --- H16: the ladder ----------------------------------------------------
    lines.extend(["", "## H16 — the width ladder at d32/L4", "",
                  "| width | pairs | rate | attention | gain | positive |",
                  "|---|---:|---:|---:|---:|---:|"])
    ladder = []
    rungs = [
        (128, ARCHIVE_V1, f"w1__ff-fixed__h128__e400__{anchor}",
         ARCHIVE_V1, f"r1cal__ff-fixed-attn__h128__e400__{anchor}__d32l4"),
        (256, res, f"w16lad__ff-fixed__h256__e400__{anchor}",
         res, f"w16lad__ff-fixed-attn__h256__e400__{anchor}__d32l4"),
        (384, res, f"w16lad__ff-fixed__h384__e400__{anchor}",
         res, f"w16lad__ff-fixed-attn__h384__e400__{anchor}__d32l4"),
        (512, ARCHIVE_V1, f"w3wid__ff-fixed__h512__e400__{anchor}",
         ARCHIVE_V2, f"w8wid__ff-fixed-attn__h512__e400__{anchor}__d32l4"),
        (768, res, f"w16lad__ff-fixed__h768__e400__{anchor}",
         res, f"w16lad__ff-fixed-attn__h768__e400__{anchor}__d32l4"),
        (1024, ARCHIVE_V1, f"w3wid__ff-fixed__h1024__e400__{anchor}",
         ARCHIVE_V2, f"w8wid__ff-fixed-attn__h1024__e400__{anchor}__d32l4"),
    ]
    for width, rroot, rstem, aroot, astem in rungs:
        rate, attn = load(rroot, rstem, SEEDS), load(aroot, astem, SEEDS)
        gain, positive, pairs = paired(attn, rate)
        if pairs < MIN_VALID_PER_ARM:
            lines.append(f"| h{width} | {pairs} | — | — | — | **NOT EVALUABLE** |")
            ladder.append((width, None))
            continue
        shared = sorted(set(attn) & set(rate))
        lines.append(
            f"| h{width} | {pairs} "
            f"| {statistics.fmean(rate[s]['accuracy'] for s in shared):.4f} "
            f"| {statistics.fmean(attn[s]['accuracy'] for s in shared):.4f} "
            f"| {gain:+.4f} | {positive}/{pairs} |")
        ladder.append((width, gain))

    known = [g for _, g in ladder if g is not None]
    if len(known) < len(ladder):
        lines.extend(["", "**H16-1 and H16-2: NOT EVALUABLE** — the ladder has a "
                      "missing rung, and a shape cannot be read off an incomplete one."])
    else:
        below = known[:-1]
        monotone = all(a - b >= H16_1_SEPARATION for a, b in zip(below, below[1:]))
        gaps = [a - b for a, b in zip(below, below[1:])]
        drop = below[-1] - known[-1]
        threshold = drop > H16_2_FACTOR * max(gaps)
        lines.extend([
            "",
            f"**H16-1: {'MET' if monotone else 'NOT MET'}** — each rung below h1024 "
            f"exceeds the next by >= {H16_1_SEPARATION} "
            f"(gaps: {', '.join(f'{g:+.4f}' for g in gaps)}).",
            f"**H16-2: {'MET' if threshold else 'NOT MET'}** — the drop into h1024 is "
            f"{drop:.4f}, against {H16_2_FACTOR}x the largest gap below it "
            f"({H16_2_FACTOR * max(gaps):.4f}).",
        ])

    # --- H17: the headline at n=32 -----------------------------------------
    lines.extend(["", "## H17 — the headline and its mechanism at n=32", ""])

    def merged(archive_root, archive_stem, new_stem):
        """Archived twelve plus the twenty new seeds, one dict."""
        cells = load(archive_root, archive_stem, SEEDS)
        cells.update(load(res, new_stem, SEEDS_EXTENDED[12:]))
        return cells

    rate = merged(ARCHIVE_V1, f"w1__ff-fixed__h128__e400__{anchor}",
                  f"w17hdl__ff-fixed__h128__e400__{anchor}")
    attn = merged(ARCHIVE_V1, f"r1cal__ff-fixed-attn__h128__e400__{anchor}__d32l4",
                  f"w17hdl__ff-fixed-attn__h128__e400__{anchor}__d32l4")
    gain, positive, pairs = paired(attn, rate)
    if pairs < H17_POSITIVE:
        lines.append(f"**H17-1: NOT EVALUABLE** — {pairs} pairs, and the registered "
                     f"bar of {H17_POSITIVE}/32 cannot be reached.")
    else:
        over = sum(1 for c in attn.values() if c["accuracy"] >= H17_GATE)
        met = gain >= H17_GAIN and positive >= H17_POSITIVE and over >= H17_GATE_SEEDS
        lines.extend([
            f"| n | rate | attention | gain | positive | >= {H17_GATE} |",
            "|---:|---:|---:|---:|---:|---:|",
            f"| {pairs} | {statistics.fmean(c['accuracy'] for c in rate.values()):.4f} "
            f"| {statistics.fmean(c['accuracy'] for c in attn.values()):.4f} "
            f"| {gain:+.4f} | {positive}/{pairs} | {over}/{len(attn)} |", "",
            f"**H17-1: {'MET' if met else 'NOT MET'}** (bars: gain >= {H17_GAIN:+.2f}, "
            f"positive >= {H17_POSITIVE}/32, >= {H17_GATE_SEEDS}/32 at or above "
            f"{H17_GATE}).",
        ])

    rate_shuf = merged(ARCHIVE_V1, f"w1__ff-fixed__h128__e400__{anchor}__bin-shuffled",
                       f"w17hdl__ff-fixed__h128__e400__{anchor}__bin-shuffled")
    attn_shuf = merged(ARCHIVE_V1,
                       f"w1__ff-fixed-attn__h128__e400__{anchor}__d32l1__bin-shuffled",
                       f"w17hdl__ff-fixed-attn__h128__e400__{anchor}__d32l4__bin-shuffled")
    attn_cost, attn_pos, attn_pairs = paired(attn, attn_shuf)
    rate_cost, _, rate_pairs = paired(rate, rate_shuf)
    if attn_pairs < H17_POSITIVE or rate_pairs < MIN_VALID_PER_ARM:
        lines.append(f"\n**H17-2: NOT EVALUABLE** — {attn_pairs} attention pairs and "
                     f"{rate_pairs} rate pairs.")
    else:
        ratio = attn_cost / rate_cost if rate_cost > 0 else float("inf")
        met = (attn_cost >= H17_GAIN and attn_pos >= H17_POSITIVE
               and ratio >= H17_2_SHUFFLE_FACTOR)
        lines.extend([
            "", f"| arm | pairs | intact − shuffled | positive |",
            "|---|---:|---:|---:|",
            f"| attention d32/L4 | {attn_pairs} | {attn_cost:+.4f} | {attn_pos}/{attn_pairs} |",
            f"| rate | {rate_pairs} | {rate_cost:+.4f} | — |", "",
            f"**H17-2: {'MET' if met else 'NOT MET'}** — shuffle cost >= {H17_GAIN:+.2f}, "
            f"positive >= {H17_POSITIVE}/32, and at least {H17_2_SHUFFLE_FACTOR}x the "
            f"rate arm's ({ratio:.1f}x measured).",
        ])

    warnings = []
    for cid in plan:
        path = res / f"{cid}.json"
        if path.is_file():
            for w in stability_warnings(json.loads(path.read_text())):
                warnings.append(f"- `{cid}`: {w}")
    if warnings:
        lines.extend(["", "## Stability warnings", "",
                      "Reported, and **never voiding** — gradient magnitude is the "
                      "quantity under study in H15-2 and voiding on it would decide "
                      "the question by definition.", ""] + warnings)

    report = "\n".join(lines) + "\n"
    if args.out:
        Path(args.out).write_text(report)
    print(report, end="")
    return 0 if ran == len(plan) and not invalid else 2


if __name__ == "__main__":
    raise SystemExit(main())
