#!/usr/bin/env python3
"""Wave 20 verdicts, exactly as registered in
`results/PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md`. **Frozen
in the same commit as that preregistration and before the first cell existed.**

Four places here would change a verdict rather than crash if they were wrong, so
each is stated:

* **H20-2 gates H20-1.** A difference of gains computed over too few pairs is
  arithmetic, not evidence, and the preregistration says so: if the recurrent
  comparison yields fewer than 24 pairs, H20-1 carries no numbers regardless of
  what they would have been. The gate is applied before the arithmetic, not
  reported beside it.

* **H20-3 is registered AGAINST its own pilot.** Over the ten pairs that exist
  today the correlation is -0.648, and the registered bar is >= -0.30. This
  analyser must be able to return NOT MET, and the outcome table treats that as
  informative rather than as a failure of the wave.

* **Every arm is at surrogate scale 0.4.** Substrate and scale are confounded at
  this operating point unless they are held together, which is why the stems are
  spelled out rather than built from a loop over arms.

* **Peak gradient norm is a covariate here, never a filter.** Voiding on it
  would decide the survivorship question by construction.
"""
from __future__ import annotations

import argparse
import json
import math
import os
import statistics
import sys
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from cell_validity import stability_warnings, validity_problems  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent.parent

SEEDS_W20 = [5170001 + i for i in range(32)]

# --- Registered thresholds, named so a reader can check them against the prereg
#     without reading the code that applies them. -----------------------------
H20_1_DIFFERENCE = 0.03      # difference of gains, the bar wave 14 registered
H20_1_POSITIVE = 24          # of 32
H20_2_MIN_PAIRS = 24         # below this the recurrent comparison carries nothing
H20_3_RHO_FLOOR = -0.30      # pilot over ten pairs is -0.648
PILOT_RHO = -0.648           # stated in the prereg, recorded here so it cannot drift
H20_4_RATIO = 1.0            # headroom-normalised ordering must merely survive

ARCHIVE_V1 = ROOT / "results/shd_attention_campaign_v1/cells"
ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"


def load(roots, stem: str, seeds) -> dict[int, dict]:
    """Valid cells for one arm, keyed by seed. Invalid cells are dropped here
    and counted by the caller -- never silently."""
    out = {}
    for seed in seeds:
        for root in roots:
            path = root / f"{stem}__s{seed}.json"
            if not path.is_file():
                continue
            cell = json.loads(path.read_text())
            if not validity_problems(cell):
                out[seed] = cell
            break
    return out


def paired_deltas(treatment: dict[int, dict], control: dict[int, dict]):
    """Per-seed differences over seeds present in BOTH. Never pooled."""
    shared = sorted(set(treatment) & set(control))
    return {s: treatment[s]["accuracy"] - control[s]["accuracy"] for s in shared}


def spearman(xs: list[float], ys: list[float]) -> float | None:
    """Rank correlation. None below four points, where it is not a number worth
    printing rather than a number worth doubting."""
    n = len(xs)
    if n < 4:
        return None

    def rank(values):
        order = sorted(range(n), key=lambda i: values[i])
        ranks = [0.0] * n
        i = 0
        while i < n:                       # average ties, or a plateau in the
            j = i                          # covariate biases rho toward zero
            while j + 1 < n and values[order[j + 1]] == values[order[i]]:
                j += 1
            shared = (i + j) / 2
            for k in range(i, j + 1):
                ranks[order[k]] = shared
            i = j + 1
        return ranks

    rx, ry = rank(xs), rank(ys)
    mx, my = statistics.fmean(rx), statistics.fmean(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = math.sqrt(sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry))
    return num / den if den else None


def peak_norm(cell: dict) -> float | None:
    series = cell.get("epoch_max_gradient_norm")
    if not series:
        return None
    finite = [v for v in series if isinstance(v, (int, float)) and math.isfinite(v) and v > 0]
    return max(finite) if finite else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--failures")
    parser.add_argument("--out")
    args = parser.parse_args()

    plan = {e["id"]: e for e in json.loads(Path(args.plan).read_text())}
    res = Path(args.results)
    roots = [res, ARCHIVE_V2, ARCHIVE_V1]
    anchor = "published-2ms__adjacent-sum-5"
    lines: list[str] = ["# Wave 20 verdicts", ""]

    ran = sum(1 for cid in plan if (res / f"{cid}.json").is_file())
    invalid = []
    for cid in plan:
        path = res / f"{cid}.json"
        if path.is_file() and validity_problems(json.loads(path.read_text())):
            invalid.append(cid)
    failures = len(list(Path(args.failures).glob("*.log"))) if args.failures else 0
    lines.append(f"Coverage: **{ran - len(invalid)} valid / {len(plan)} planned**, "
                 f"{len(invalid)} invalid, {failures} failures, "
                 f"{len(plan) - ran} missing.")

    def arm(prefix, spec):
        return load(roots, f"{prefix}__{spec}", SEEDS_W20)

    # The archived twelve of each arm sit under DIFFERENT wave prefixes: the
    # recurrent rate arm is `w13rec`, everything else is `w14sub`. Assuming one
    # prefix for all four silently returned an empty rate arm and a NOT MET on
    # H20-2, which reads exactly like a real finding. Each arm names its own
    # archive, and `merged_arm_is_one_arm` below refuses any pairing whose
    # configuration tails disagree.
    ARCHIVES = {
        "rec_rate": ("w13rec", f"rec-alif__h128__e400__{anchor}__ss0.4"),
        "rec_attn": ("w14sub", f"rec-alif-attn__h128__e400__{anchor}__d32l4__ss0.4"),
        "ff_rate": ("w14sub", f"ff-fixed__h128__e400__{anchor}__ss0.4"),
        "ff_attn": ("w14sub", f"ff-fixed-attn__h128__e400__{anchor}__d32l4__ss0.4"),
    }

    def merged(key):
        archive_prefix, spec = ARCHIVES[key]
        cells = dict(arm(archive_prefix, spec))
        cells.update(arm("w20rec", spec))
        return cells

    rec = {"rate": merged("rec_rate"), "attn": merged("rec_attn")}
    ff = {"rate": merged("ff_rate"), "attn": merged("ff_attn")}
    for label, cells in (("rec+alif", rec["rate"]), ("rec+alif+attn", rec["attn"]),
                         ("ff+fixed", ff["rate"]), ("ff+fixed+attn", ff["attn"])):
        if not cells:
            raise SystemExit(
                f"{label} resolved to zero cells. An empty arm produces a NOT MET "
                f"that reads exactly like a finding, so this refuses instead.")

    rec_deltas = paired_deltas(rec["attn"], rec["rate"])
    ff_deltas = paired_deltas(ff["attn"], ff["rate"])

    # --- H20-2 first: it gates H20-1 ---------------------------------------
    lines.extend(["", "## H20-2 — is the comparison still one loss from unreportable?", "",
                  "| arm | valid of 32 |", "|---|---:|",
                  f"| `rec+alif` | {len(rec['rate'])} |",
                  f"| `rec+alif+attn` | {len(rec['attn'])} |",
                  f"| **usable pairs** | **{len(rec_deltas)}** |", ""])
    reportable = len(rec_deltas) >= H20_2_MIN_PAIRS
    lines.append(f"**H20-2: {'MET' if reportable else 'NOT MET'}** — "
                 f"{len(rec_deltas)} pairs against a floor of {H20_2_MIN_PAIRS}/32.")
    if not reportable:
        lines.append("This operating point does not support the recurrent claim at "
                     "a sample size reachable here. **H20-1 is not licensed** "
                     "regardless of its arithmetic, and is suppressed below.")

    # --- H20-1: the paper's claim ------------------------------------------
    lines.extend(["", "## H20-1 — does the recurrent substrate's larger gain survive?", ""])
    both = sorted(set(rec_deltas) & set(ff_deltas))
    if not reportable:
        lines.append("**H20-1: NOT LICENSED** — H20-2 failed.")
    elif len(both) < H20_1_POSITIVE:
        lines.append(f"**H20-1: NOT EVALUABLE** — {len(both)} seed-paired "
                     f"comparisons, and the bar of {H20_1_POSITIVE}/32 cannot be "
                     f"reached.")
    else:
        diff = [rec_deltas[s] - ff_deltas[s] for s in both]
        mean_diff = statistics.fmean(diff)
        positive = sum(d > 0 for d in diff)
        met = mean_diff >= H20_1_DIFFERENCE and positive >= H20_1_POSITIVE
        lines.extend([
            "| substrate | pairs | rate read-out | + attention | gain |",
            "|---|---:|---:|---:|---:|",
            f"| `rec+alif` | {len(rec_deltas)} | "
            f"{statistics.fmean(rec['rate'][s]['accuracy'] for s in rec_deltas):.4f} | "
            f"{statistics.fmean(rec['attn'][s]['accuracy'] for s in rec_deltas):.4f} | "
            f"**{statistics.fmean(rec_deltas.values()):+.4f}** |",
            f"| `ff+fixed` | {len(ff_deltas)} | "
            f"{statistics.fmean(ff['rate'][s]['accuracy'] for s in ff_deltas):.4f} | "
            f"{statistics.fmean(ff['attn'][s]['accuracy'] for s in ff_deltas):.4f} | "
            f"**{statistics.fmean(ff_deltas.values()):+.4f}** |", "",
            f"**H20-1: {'MET' if met else 'NOT MET'}** — difference of gains "
            f"{mean_diff:+.4f} over {len(both)} seed-paired comparisons, positive "
            f"in {positive}/{len(both)} (bars: >= {H20_1_DIFFERENCE:+.2f}, "
            f">= {H20_1_POSITIVE}/32).",
        ])

    # --- H20-3: survivorship, registered against its own pilot -------------
    lines.extend(["", "## H20-3 — is survivorship shaping the gain?", "",
                  f"Registered bar **ρ >= {H20_3_RHO_FLOOR:+.2f}**, against a pilot "
                  f"of **{PILOT_RHO:+.3f}** over the ten pairs that existed when "
                  f"this was written. The bar predicts the pilot is small-sample "
                  f"noise.", ""])
    covariate = [(rec_deltas[s], peak_norm(rec["attn"][s])) for s in sorted(rec_deltas)]
    usable = [(g, math.log10(p)) for g, p in covariate if p is not None]
    gains_only = [g for g, _ in usable]
    norms_only = [p for _, p in usable]
    rho = spearman(gains_only, norms_only)
    if rho is None:
        # Two different failures, and they must not share a message. "Too few
        # pairs" is a shortage that more cells fix; a constant vector is a
        # degenerate correlation that more cells of the same kind will not.
        # Reporting both as "NOT EVALUABLE - N pairs" reads as a shortage at any
        # N, which is how a check that cannot run comes to look like one that
        # merely lacked data.
        if len(usable) < 4:
            lines.append(f"**H20-3: NOT EVALUABLE** — only {len(usable)} pair(s) "
                         f"carry a usable peak gradient norm; a rank correlation "
                         f"needs at least four.")
        elif len(set(gains_only)) == 1:
            lines.append(f"**H20-3: NOT EVALUABLE** — every one of the "
                         f"{len(usable)} paired gains is identical, so there is "
                         f"no variation for the covariate to explain. This is a "
                         f"degenerate comparison, not a small one.")
        else:
            lines.append(f"**H20-3: NOT EVALUABLE** — every one of the "
                         f"{len(usable)} peak gradient norms is identical, so the "
                         f"covariate is constant. This is a degenerate "
                         f"comparison, not a small one.")
    else:
        met = rho >= H20_3_RHO_FLOOR
        lines.append(f"**H20-3: {'MET' if met else 'NOT MET'}** — ρ = "
                     f"**{rho:+.3f}** over {len(usable)} completing pairs.")
        if not met:
            lines.append(
                "Among the cells that completed, the ones that came closest to "
                "diverging show the smaller gains, and the cells that did not "
                "complete had higher norms still. **The recurrent gain is "
                "measured on a subsample selected for the property that predicts "
                "a large gain.** `PAPER_DRAFT.md` §3.7's limit 4 is promoted from "
                "caveat to finding, and its +0.2612 must be reported as "
                "survivorship-shaped. This does not establish how much of the "
                "gain the selection accounts for — only that it is not nothing.")
        elif rho >= 0.30:
            lines.append("The correlation runs the other way, which nothing "
                         "predicted and which needs its own explanation before "
                         "any of the above is read.")

    # --- H20-4: headroom normalisation, now registered ----------------------
    lines.extend(["", "## H20-4 — does the advantage survive headroom normalisation?", ""])
    if not rec_deltas or not ff_deltas:
        lines.append("**H20-4: NOT EVALUABLE** — an arm carries no pairs.")
    else:
        rec_base = statistics.fmean(rec["rate"][s]["accuracy"] for s in rec_deltas)
        ff_base = statistics.fmean(ff["rate"][s]["accuracy"] for s in ff_deltas)
        rec_gain = statistics.fmean(rec_deltas.values())
        ff_gain = statistics.fmean(ff_deltas.values())
        rec_head, ff_head = 1.0 - rec_base, 1.0 - ff_base
        if rec_head <= 0 or ff_head <= 0 or ff_gain <= 0:
            lines.append("**H20-4: NOT EVALUABLE** — a headroom or a reference "
                         "gain is non-positive, so the ratio is not defined.")
        else:
            ratio = (rec_gain / rec_head) / (ff_gain / ff_head)
            met = ratio > H20_4_RATIO
            lines.extend([
                "| substrate | base | headroom | gain | gain / headroom |",
                "|---|---:|---:|---:|---:|",
                f"| `rec+alif` | {rec_base:.4f} | {rec_head:.4f} | {rec_gain:+.4f} | "
                f"{rec_gain / rec_head:.4f} |",
                f"| `ff+fixed` | {ff_base:.4f} | {ff_head:.4f} | {ff_gain:+.4f} | "
                f"{ff_gain / ff_head:.4f} |", "",
                f"**H20-4: {'MET' if met else 'NOT MET'}** — ratio "
                f"**{ratio:.3f}x** (bar: > {H20_4_RATIO:.1f}x). §3.7's limit 1 "
                f"computed 1.34x post-hoc; this is the registered measurement.",
            ])

    # --- stability, reported and never voiding ------------------------------
    lines.extend(["", "## Stability warnings", "",
                  "Reported, and **never voiding**. Peak gradient norm is the "
                  "covariate H20-3 is measured on; voiding on it would decide the "
                  "survivorship question by construction.", ""])
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
    return 0 if ran == len(plan) and not invalid else 2


if __name__ == "__main__":
    raise SystemExit(main())
