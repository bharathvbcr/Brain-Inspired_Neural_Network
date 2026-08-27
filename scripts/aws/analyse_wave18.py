#!/usr/bin/env python3
"""Waves 18-19 verdicts, exactly as registered in
`results/PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`. **Frozen in the
same commit as that preregistration and before the first cell of either wave
existed.**

Four places here are where a silent bug would change a verdict rather than
crash, so each is stated:

* **H18-1 is decided on the argmax over four depths, not on L2 alone.** The
  motivating observation was L2's +0.0392, and an analyser that only asked "is
  L2 best?" would confirm the observation it came from. It asks which depth is
  best, and names the answer whatever it is.

* **H18-2 has two halves and both must hold.** "Sick arms collapse" alone is
  satisfiable by an arm that is sick and an arm that is healthy and equally bad;
  the second half -- no healthy arm below -0.05 -- is what makes it falsifiable.
  Registering only the first half would have made a rule that h1024/L1 already
  breaks look like it passes.

* **H18-4 is byte-identity against `w15col`, and it is destructive.** If the
  twelve duplicated L2 cells differ from the ones wave 15 produced, the fleet is
  not reproducing itself and **every verdict in this file is void**, including
  the ones that would otherwise be favourable. It is checked first and it
  suppresses the rest.

* **Every comparison is paired on seed** over cells where both arms are valid,
  and a comparison below its registered pair floor carries **no numbers at all**
  -- not a mean, not a direction. An arm that lost seeds must not look like an
  arm that had them.
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
SEEDS_W18 = [5170001 + i for i in range(20)]

# --- Registered thresholds, named so a reader can check them against the prereg
#     without reading the code that applies them. -----------------------------
H18_1_MARGIN = 0.02          # interior max must clear both endpoints by this
H18_1_INTERIOR = (2, 3)      # the depths that count as interior on this ladder
H18_2_SICK_NORM = 1.0        # median epoch-mean norm at or above which an arm is sick
H18_2_SICK_GAIN = -0.10      # a sick arm's gain must be at or below this
H18_2_HEALTHY_FLOOR = -0.05  # a healthy arm's gain must not be below this
H18_3_GAIN = 0.02
H18_3_POSITIVE = 15          # of 20
MIN_VALID_PER_ARM = 9        # below this an arm is NOT EVALUABLE

DEPTHS = (1, 2, 3, 4)

#: Archived corpora holding the reused controls and the wave-15 L2 cells that
#: H18-4 regresses against. Same pinned binary, deterministic instrument.
ARCHIVE_V1 = ROOT / "results/shd_attention_campaign_v1/cells"
ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"

SCIENTIFIC_FIELDS = (
    "accuracy", "mean_loss", "mean_gradient_norm", "mean_update_rms",
    "mean_firing_rate", "silent_fraction", "saturated_fraction",
    "majority_prediction", "classes_predicted", "non_finite_events",
    "tail_loss_improvement", "epoch_mean_loss", "epoch_mean_gradient_norm",
    "epoch_max_gradient_norm", "epoch_max_gradient_step",
)


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
    """Median over cells of the median epoch-mean gradient norm. Medians rather
    than means because the quantity spans eight orders of magnitude and one
    exploded cell would carry a mean."""
    per_cell = [statistics.median(c["epoch_mean_gradient_norm"])
                for c in cells.values() if c.get("epoch_mean_gradient_norm")]
    return statistics.median(per_cell) if per_cell else None


def byte_identical(new: dict, archived: dict) -> list[str]:
    """Every scientific field, by repr so 1.0 and 1 cannot compare equal."""
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
    parser.add_argument("--archive", default=str(ARCHIVE_V2),
                        help="corpus holding the reused controls and the "
                             "wave-15 L2 cells H18-4 regresses against. "
                             "Defaults to the committed campaign-v2 corpus; "
                             "overridable so the destructive check can be "
                             "exercised against a constructed archive rather "
                             "than only skipped when the corpus is not yet "
                             "re-frozen.")
    parser.add_argument("--out")
    args = parser.parse_args()

    plan = {e["id"]: e for e in json.loads(Path(args.plan).read_text())}
    res = Path(args.results)
    archive = Path(args.archive)
    anchor = "published-2ms__adjacent-sum-5"
    lines: list[str] = ["# Waves 18-19 verdicts", ""]

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

    # --- H18-4 first: it is destructive and it gates everything below -------
    lines.extend(["", "## H18-4 — does the fleet reproduce itself?", ""])
    l2_stem = f"w18dep__ff-fixed-attn__h1024__e400__{anchor}__d32l2"
    w15_stem = f"w15col__ff-fixed-attn__h1024__e400__{anchor}__d32l2"
    checked, differing_cells = 0, []
    for seed in SEEDS:
        new = res / f"{l2_stem}__s{seed}.json"
        old = archive / f"{w15_stem}__s{seed}.json"
        if not (new.is_file() and old.is_file()):
            continue
        checked += 1
        fields = byte_identical(json.loads(new.read_text()), json.loads(old.read_text()))
        if fields:
            differing_cells.append((seed, fields))
    if checked < MIN_VALID_PER_ARM:
        lines.append(f"**H18-4: NOT EVALUABLE** — {checked} of 12 duplicated cells "
                     f"present on both sides.")
        harness_ok = None
    elif differing_cells:
        harness_ok = False
        lines.append(f"**H18-4: NOT MET** — {len(differing_cells)} of {checked} "
                     f"duplicated cells differ from `w15col` on a scientific field.")
        lines.append("")
        for seed, fields in differing_cells:
            lines.append(f"- seed {seed}: {', '.join(fields)}")
        lines.extend(["", "**Every verdict below is VOID.** The fleet is not "
                      "reproducing cells it has already produced under the same "
                      "pinned binary, so no comparison in these waves — favourable "
                      "or not — is licensed, and the comparability of waves 15-17 "
                      "to the archive is in question too."])
    else:
        harness_ok = True
        lines.append(f"**H18-4: MET** — {checked}/{checked} duplicated cells "
                     f"byte-identical to `w15col` across every scientific field.")

    void = harness_ok is False

    # --- H18-1 / H18-2: the depth ladder -----------------------------------
    control = load(res, f"w18dep__ff-fixed__h1024__e400__{anchor}", SEEDS_W18)
    lines.extend(["", "## H18-1 — is the optimum in read-out depth interior?", "",
                  "| depth | pairs | rate | attention | gain | positive | median epoch-mean norm |",
                  "|---|---:|---:|---:|---:|---:|---:|"])
    ladder: dict[int, dict] = {}
    for depth in DEPTHS:
        cells = load(res, f"w18dep__ff-fixed-attn__h1024__e400__{anchor}__d32l{depth}",
                     SEEDS_W18)
        if len(cells) < MIN_VALID_PER_ARM or len(control) < MIN_VALID_PER_ARM:
            lines.append(f"| L{depth} | {len(cells)} valid | — | — | — | — | — |")
            continue
        gain, positive, pairs = paired(cells, control)
        if pairs < MIN_VALID_PER_ARM:
            lines.append(f"| L{depth} | {pairs} | — | — | — | — | — |")
            continue
        shared = sorted(set(cells) & set(control))
        norm = median_epoch_mean_norm(cells)
        ladder[depth] = {"gain": gain, "positive": positive, "pairs": pairs, "norm": norm}
        lines.append(
            f"| **L{depth}** | {pairs} | "
            f"{statistics.median(control[s]['accuracy'] for s in shared):.4f} | "
            f"{statistics.median(cells[s]['accuracy'] for s in shared):.4f} | "
            f"{gain:+.4f} | {positive}/{pairs} | {norm:.3f} |")
    lines.append("")

    if void:
        lines.append("**H18-1: VOID** — H18-4 failed.")
    elif len(ladder) < len(DEPTHS):
        missing = [f"L{d}" for d in DEPTHS if d not in ladder]
        lines.append(f"**H18-1: NOT EVALUABLE** — the ladder is missing "
                     f"{', '.join(missing)}, and an argmax cannot be read off an "
                     f"incomplete ladder.")
    else:
        best = max(ladder, key=lambda d: ladder[d]["gain"])
        margin = min(ladder[best]["gain"] - ladder[1]["gain"],
                     ladder[best]["gain"] - ladder[4]["gain"])
        interior = best in H18_1_INTERIOR
        met = interior and margin >= H18_1_MARGIN
        lines.append(f"**H18-1: {'MET' if met else 'NOT MET'}** — the largest gain is "
                     f"at **L{best}** ({ladder[best]['gain']:+.4f}), clearing the "
                     f"nearer endpoint by {margin:+.4f} "
                     f"(bar: interior depth, margin >= {H18_1_MARGIN:.2f}).")
        if not interior:
            lines.append(f"The maximum is at an **endpoint**. "
                         + ("Depth monotonically hurts at h1024; the wave-15 L2 "
                            "result was a twelve-seed accident."
                            if best == 1 else
                            "**The archived collapse does not reproduce on this "
                            "fleet.** That discrepancy, not the depth ladder, is "
                            "the finding, and the wave-15 verdicts rest on the "
                            "same archive."))
        elif not met:
            lines.append("The maximum is interior but does not clear both endpoints "
                         "by the registered margin. The shape is **unresolved** and "
                         "no rewrite of the paper's scope limit is licensed.")
        elif best == 3:
            lines.append("The maximum is at **L3**, the deeper of the two interior "
                         "rungs, so the ladder does not bound it from above. A rung "
                         "at L5 is required before the optimum can be located, as "
                         "registered in the preregistration's outcome table.")

    # --- H18-2: the collapse, and only the collapse -------------------------
    lines.extend(["", "## H18-2 — is the collapse numerical?", ""])
    if void:
        lines.append("**H18-2: VOID** — H18-4 failed.")
    elif not ladder:
        lines.append("**H18-2: NOT EVALUABLE** — no depth arm reached its pair floor.")
    else:
        breaches = []
        for depth, row in sorted(ladder.items()):
            sick = row["norm"] >= H18_2_SICK_NORM
            if sick and row["gain"] > H18_2_SICK_GAIN:
                breaches.append(f"L{depth}: norm {row['norm']:.3f} >= "
                                f"{H18_2_SICK_NORM} but gain {row['gain']:+.4f} > "
                                f"{H18_2_SICK_GAIN}")
            if not sick and row["gain"] < H18_2_HEALTHY_FLOOR:
                breaches.append(f"L{depth}: norm {row['norm']:.3f} < "
                                f"{H18_2_SICK_NORM} but gain {row['gain']:+.4f} < "
                                f"{H18_2_HEALTHY_FLOOR}")
        if len(ladder) < len(DEPTHS):
            lines.append(f"Read over the {len(ladder)} arm(s) that reached their pair "
                         f"floor; the rule is registered over all four.")
        lines.append(f"**H18-2: {'NOT MET' if breaches else 'MET'}** — "
                     f"sick arms (norm >= {H18_2_SICK_NORM}) must have gain "
                     f"<= {H18_2_SICK_GAIN}, healthy arms must not fall below "
                     f"{H18_2_HEALTHY_FLOOR}.")
        if breaches:
            lines.append("")
            lines += [f"- {b}" for b in breaches]

    # --- H18-3: L2 on its own cells, at n=20 --------------------------------
    lines.extend(["", "## H18-3 — is L2's advantage a seed artefact?", ""])
    if void:
        lines.append("**H18-3: VOID** — H18-4 failed.")
    elif 2 not in ladder:
        lines.append("**H18-3: NOT EVALUABLE** — the L2 arm did not reach its pair floor.")
    else:
        row = ladder[2]
        met = (row["gain"] >= H18_3_GAIN and row["positive"] >= H18_3_POSITIVE
               and row["pairs"] >= H18_3_POSITIVE)
        if row["pairs"] < H18_3_POSITIVE:
            lines.append(f"**H18-3: NOT EVALUABLE** — {row['pairs']} pairs, and the "
                         f"registered bar of {H18_3_POSITIVE}/20 cannot be reached.")
        else:
            lines.append(f"**H18-3: {'MET' if met else 'NOT MET'}** — gain "
                         f"{row['gain']:+.4f}, positive in {row['positive']}/"
                         f"{row['pairs']} (bar: >= {H18_3_GAIN:+.2f}, "
                         f">= {H18_3_POSITIVE}/20).")

    # --- H19-1: does the optimum move with width? ---------------------------
    lines.extend(["", "## H19-1 — does the optimal depth fall as width rises?", ""])
    h768_control = load(res, f"w16lad__ff-fixed__h768__e400__{anchor}", SEEDS)
    if not h768_control:
        h768_control = load(archive, f"w16lad__ff-fixed__h768__e400__{anchor}", SEEDS)
    h768 = {}
    for depth, root, wave in ((2, res, "w19int"), (4, res, "w16lad")):
        cells = load(root, f"{wave}__ff-fixed-attn__h768__e400__{anchor}__d32l{depth}",
                     SEEDS)
        if not cells:
            cells = load(archive,
                         f"{wave}__ff-fixed-attn__h768__e400__{anchor}__d32l{depth}",
                         SEEDS)
        gain, positive, pairs = paired(cells, h768_control)
        if pairs >= MIN_VALID_PER_ARM:
            h768[depth] = {"gain": gain, "positive": positive, "pairs": pairs}
    lines.extend(["| width | L2 gain | L4 gain | deeper wins? |", "|---|---:|---:|---|"])
    for label, table in (("h768", h768), ("h1024", ladder)):
        if 2 in table and 4 in table:
            lines.append(f"| {label} | {table[2]['gain']:+.4f} | "
                         f"{table[4]['gain']:+.4f} | "
                         f"{'yes' if table[4]['gain'] > table[2]['gain'] else 'no'} |")
        else:
            lines.append(f"| {label} | — | — | **NOT EVALUABLE** |")
    lines.append("")
    if void:
        lines.append("**H19-1: VOID** — H18-4 failed.")
    elif not ({2, 4} <= set(h768) and {2, 4} <= set(ladder)):
        lines.append("**H19-1: NOT EVALUABLE** — the reversal needs L2 and L4 at both "
                     "widths, and at least one is below its pair floor.")
    else:
        reversed_ = (h768[4]["gain"] > h768[2]["gain"]
                     and ladder[2]["gain"] > ladder[4]["gain"])
        lines.append(f"**H19-1: {'MET' if reversed_ else 'NOT MET'}** — the ordering "
                     f"of L2 and L4 "
                     f"{'reverses' if reversed_ else 'does not reverse'} between h768 "
                     f"and h1024.")
        if not reversed_:
            lines.append("Optimal depth does not move with width across this range. "
                         "The h1024 collapse is then a threshold in width at fixed "
                         "depth, which is what wave 16 was built to test.")

    # --- stability, reported and never voiding ------------------------------
    lines.extend(["", "## Stability warnings", "",
                  "Reported, and **never voiding** — gradient magnitude is the "
                  "quantity under study in H18-2 and voiding on it would decide the "
                  "question by definition.", ""])
    warned = 0
    for cid in sorted(plan):
        path = res / f"{cid}.json"
        if not path.is_file():
            continue
        for warning in stability_warnings(json.loads(path.read_text())):
            lines.append(f"- `{cid}`: {warning}")
            warned += 1
    if not warned:
        lines.append("None.")

    text = "\n".join(lines) + "\n"
    if args.out:
        Path(args.out).write_text(text)
    else:
        print(text)
    # 2 when the plan is incomplete or any cell is invalid, matching
    # `analyse_wave15.py`. An analyser that exits 0 on a partial run lets a
    # caller checking the exit code read an unfinished wave as a finished one.
    return 0 if ran == len(plan) and not invalid else 2


if __name__ == "__main__":
    raise SystemExit(main())
