#!/usr/bin/env python3
"""Wave 12 verdicts, exactly as registered in
`results/PREREG_2026-08-22_ADAPTATION_BY_ATTENTION.md`.

Frozen before the first cell landed. Imports its loader, its pinned-binary
guard, its validity gate and its seed order from `analyse_wave8`, so the reused
`ff+fixed` controls are hash-checked against the wave-1 manifest exactly as
waves 8 and 9 check theirs, and the validity rule has the one owner it has
everywhere else.

    python3 scripts/aws/analyse_wave12.py [--out FILE]
"""

from __future__ import annotations

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from analyse_wave8 import (  # noqa: E402
    V1,
    V2,
    WARNINGS,
    accs,
    assert_one_pinned_binary,
    load,
    mean,
    verdict,
)

ANCHOR = "published-2ms__adjacent-sum-5"

#: Registered thresholds. Named here so a reader can check them against the
#: prereg without reading the code that applies them.
A1_DELTA = 0.03
A2_GAIN = 0.05
A2_SEEDS = 10
A3_GATE = 0.80
A3_SEEDS = 9


def paired_positive(treatment: list[float], control: list[float]) -> int:
    """Seeds where the treatment beat its own seed's control.

    Paired rather than pooled: the seed determines the initial weights and the
    epoch order for both arms, so a per-seed difference removes the variance the
    seed contributes and is the comparison the prereg registers.
    """
    return sum(1 for t, c in zip(treatment, control) if t > c)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="-")
    args = parser.parse_args()

    pinned = assert_one_pinned_binary()
    voided: list[str] = []
    lines: list[str] = []

    def w(text=""):
        lines.append(text)

    # --- arms ----------------------------------------------------------------
    # New this wave.
    alif = accs(load(V2, f"w12ada__ff-alif__h128__e400__{ANCHOR}"), "w12ada ff+alif", voided)
    alif_attn = accs(
        load(V2, f"w12ada__ff-alif-attn__h128__e400__{ANCHOR}__d32l4"),
        "w12ada ff+alif+attn",
        voided,
    )
    # Reused, hash-checked against the wave-1 manifest.
    fixed = accs(load(V1, f"w1__ff-fixed__h128__e400__{ANCHOR}"), "w1 ff+fixed", voided)
    fixed_attn = accs(
        load(V1, f"r1cal__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4"),
        "r1cal ff+fixed+attn",
        voided,
    )

    gain_alif = mean(alif_attn) - mean(alif)
    gain_fixed = mean(fixed_attn) - mean(fixed)
    delta = gain_alif - gain_fixed

    w("# Wave 12 — adaptation × attention at the anchor\n")
    w("Prereg: `PREREG_2026-08-22_ADAPTATION_BY_ATTENTION.md` §3. "
      "24 new cells, plus 24 reused controls from waves 1 and the registered "
      f"run, same pinned binary `{pinned[:12]}`.\n")

    if voided:
        w("## VALIDITY GATES FAILED — nothing below is reportable\n")
        for line in voided:
            w(f"- {line}")
        print("\n".join(lines))
        return 1
    w("**Validity gates: all 48 cells pass.**\n")

    if WARNINGS:
        w(f"**Stability notes ({len(WARNINGS)}), registered as non-voiding:**\n")
        for line in WARNINGS:
            w(f"- {line}")
        w()
    else:
        w("**Stability notes: none — no cell exceeded the recorded peak gradient "
          "norm, and no cell was clipped.**\n")

    # --- the 2x2 -------------------------------------------------------------
    w("## The factorial\n")
    w("| substrate | rate read-out | + attention d32/L4 | gain |")
    w("|---|---:|---:|---:|")
    w(f"| `ff+fixed` *(reused)* | {mean(fixed):.4f} | {mean(fixed_attn):.4f} | "
      f"**{gain_fixed:+.4f}** |")
    w(f"| `ff+alif` | {mean(alif):.4f} | {mean(alif_attn):.4f} | "
      f"**{gain_alif:+.4f}** |")
    w()
    w("| arm | mean | min | max | seeds >= 0.80 |")
    w("|---|---:|---:|---:|---:|")
    for label, values in (("`ff+fixed` *(reused)*", fixed),
                          ("`ff+fixed+attn` *(reused)*", fixed_attn),
                          ("`ff+alif`", alif),
                          ("`ff+alif+attn`", alif_attn)):
        over = sum(1 for v in values if v >= A3_GATE)
        w(f"| {label} | {mean(values):.4f} | {min(values):.4f} | {max(values):.4f} | "
          f"{over}/12 |")
    w()

    # --- verdicts ------------------------------------------------------------
    w("## Registered verdicts\n")

    a1 = abs(delta) >= A1_DELTA
    direction = "shrinks" if delta < 0 else "grows"
    w(f"**A-1** *(primary, two-sided)* the attention gain depends on adaptation: "
      f"gain(`ff+alif`) **{gain_alif:+.4f}** vs gain(`ff+fixed`) "
      f"**{gain_fixed:+.4f}**, difference **{delta:+.4f}**; bar |Δ| ≥ {A1_DELTA} "
      f"-> **{verdict(a1)}**")
    if a1:
        w(f"  - The gain **{direction}** on the adaptive substrate. "
          + ("Attention was partly standing in for adaptation, and the mechanism "
             "claim narrows to the non-adapting substrate."
             if delta < 0 else
             "Attention supplies something adaptation does not, and the mechanism "
             "claim generalises across this axis."))
    else:
        w("  - Flat: adaptation is not what the read-out's advantage rests on. "
          "Substitution is refuted on this axis.")
    w()

    positive = paired_positive(alif_attn, alif)
    a2 = gain_alif >= A2_GAIN and positive >= A2_SEEDS
    w(f"**A-2** attention still helps an adaptive substrate: gain "
      f"**{gain_alif:+.4f}** (bar +{A2_GAIN}), positive in **{positive}/12** "
      f"seeds (bar {A2_SEEDS}) -> **{verdict(a2)}**")
    w()

    over_gate = sum(1 for v in alif if v >= A3_GATE)
    a3 = mean(alif) >= A3_GATE and over_gate >= A3_SEEDS
    w(f"**A-3** adaptation alone clears the gate: `ff+alif` mean "
      f"**{mean(alif):.4f}** (bar {A3_GATE}), **{over_gate}/12** seeds >= "
      f"{A3_GATE} (bar {A3_SEEDS}); `ff+fixed` was {mean(fixed):.4f} "
      f"-> **{verdict(a3)}**")
    w()

    best = max((mean(fixed), "ff+fixed"), (mean(fixed_attn), "ff+fixed+attn"),
               (mean(alif), "ff+alif"), (mean(alif_attn), "ff+alif+attn"))
    w(f"**A-4** *(reported, no verdict)* highest-scoring arm: `{best[1]}` at "
      f"{best[0]:.4f}. **No verdict is issued and none may be inferred** — the "
      "prereg registers this as descriptive for the reason wave 9's M-3 was: a "
      "factorial invites naming a winner after the fact, and that is what "
      "registration exists to prevent.")
    w()

    w("**A-5** stability: every cell passed the validity gate above, which "
      "includes `non_finite_events == 0` and completion.")
    w()

    w("## Scope\n")
    w("- Anchor only: h128, `published-2ms`, `adjacent-sum-5`, e400, d32/L4.")
    w("- **Nothing about recurrence.** The recurrent arms were deferred on "
      "wave 11's measured completion rate; see the prereg §2.")
    w("- Not calibration. No comparison to macOS-recorded numbers.")

    text = "\n".join(lines)
    if args.out == "-":
        print(text)
    else:
        open(args.out, "w").write(text + "\n")
        print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
