#!/usr/bin/env python3
"""Wave 29's platform gate — how far did `aarch64-apple-darwin` move the anchor?

Frozen with `AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md` and
committed before the first wave-29 cell exists.

Wave 29 is produced on Apple libm; every wave before it was produced on glibc,
and `FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md` establishes by direct
measurement that the two disagree on `exp`, `ln`, `tanh`, `powf` and the
surrogate sigmoid across the whole domain. So the honest question is not "is the
wave valid" -- every clause is a within-block paired contrast and stays valid by
construction -- but "how far did the platform move the thing the bars are set
against, and is that small next to the bars".

**This is a diagnostic, not a clause.** It reports whatever it finds. It may not
void a cell, adjust a number, or change a verdict: `analyse_wave29.py` is the
authority on every verdict and this file never touches it. A gate that could
overrule the frozen analyser would be re-reading the wave against the fleet,
which is exactly what the amendment's section 4 exists to prevent.

Exit codes: 0 inside the bar, 1 outside it (NOT EVALUABLE on platform grounds),
2 nothing to check.

Run: python3 scripts/aws/check_wave29_platform.py
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from cell_validity import validity_problems  # noqa: E402

#: The fleet's `ff+fixed` h128/e400 `published-2ms`/`adjacent-sum-5` intact mean,
#: recomputed from the 12 `w26pos` cells for the amendment. Hard-coded rather
#: than recomputed at run time for the same reason `TRAIN_BATCH_SIZE` is: the
#: comparison must be against the number the amendment registered, not against
#: whatever the corpus says on the day this is run. `test_campaign_tooling.py`
#: asserts the two agree, so a corpus that moves is caught rather than silently
#: shifting the gate underneath the registration.
FLEET_INTACT_MEAN = 0.7062
#: The campaign's own bar, unchanged since wave 21 and the same 0.03 every
#: wave-29 clause is tested against. Deliberately not a tighter number invented
#: here: a platform gate stricter than the bars it protects would fail the wave
#: over a difference no clause could have seen.
BAR = 0.03
#: Below this there is no mean worth taking, and reporting one would be worse
#: than reporting none. Matches the analyser's own pairing floor.
MIN_CELLS = 9

ANCHOR_HIDDEN = 128
WAVE = "w29asy"
SEED = re.compile(r"__s(\d+)\.json$")
REGISTERED_SEEDS = frozenset(range(5290001, 5290013))


def intact_rate_cells(results: Path) -> tuple[list[float], int, int]:
    """(accuracies, voided, off-block) for this wave's intact `ff+fixed` arm.

    The same filters the frozen analyser applies, for the same reasons: off-anchor
    cells are dropped and counted aloud, seeds outside the registered block are
    refused by value, and `validity_problems` decides validity rather than this
    file inventing a second opinion about it.
    """
    accuracies: list[float] = []
    voided = 0
    offblock = 0
    for path in sorted(results.glob("*.json")):
        if not path.name.startswith(WAVE + "__"):
            continue
        found = SEED.search(path.name)
        if not found:
            continue
        if int(found.group(1)) not in REGISTERED_SEEDS:
            offblock += 1
            continue
        try:
            cell = json.loads(path.read_text())
        except ValueError:
            continue
        if not (isinstance(cell, dict) and "accuracy" in cell):
            continue
        if cell.get("arm") != "ff+fixed" or cell.get("hidden") != ANCHOR_HIDDEN:
            continue
        if (cell.get("temporal_condition") or "intact") != "intact":
            continue
        if "d32l4" in path.name:
            continue
        if validity_problems(cell):
            voided += 1
            continue
        accuracies.append(cell["accuracy"])
    return accuracies, voided, offblock


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results",
                        default=str(ROOT / "results/shd_attention_wave29_local"))
    args = parser.parse_args()

    results = Path(args.results)
    print("wave 29 — platform gate (aarch64-apple-darwin vs the glibc fleet)")
    print(f"  results       : {results}")
    if not results.is_dir():
        print(f"\nNOTHING TO CHECK: {results} does not exist. This is not a "
              f"verdict, and it is not a pass.")
        return 2

    accuracies, voided, offblock = intact_rate_cells(results)
    print(f"  intact cells  : {len(accuracies)}")
    print(f"  voided        : {voided}")
    if offblock:
        print(f"  dropped, seed outside the registered block: {offblock}")

    if len(accuracies) < MIN_CELLS:
        print(f"\nNOTHING TO CHECK: {len(accuracies)} valid intact rate cells, "
              f"fewer than the {MIN_CELLS} this gate needs to take a mean. "
              f"This is not a verdict, and it is not a pass.")
        return 2

    local = statistics.fmean(accuracies)
    spread = statistics.stdev(accuracies) if len(accuracies) > 1 else 0.0
    delta = local - FLEET_INTACT_MEAN

    print(f"\n  fleet  intact mean : {FLEET_INTACT_MEAN:.4f}   (12 w26pos cells, glibc)")
    print(f"  local  intact mean : {local:.4f}   (sd {spread:.4f}, "
          f"{len(accuracies)} cells, Apple libm)")
    print(f"  platform delta     : {delta:+.4f}   against a bar of {BAR}")

    if abs(delta) <= BAR:
        print(f"\nINSIDE THE BAR. The platform moved this anchor by "
              f"{abs(delta):.4f}, less than the {BAR} every wave-29 clause is "
              f"tested against. Report the number; the verdicts stand as "
              f"analyse_wave29.py prints them.")
        return 0

    print(f"\nOUTSIDE THE BAR — NOT EVALUABLE ON PLATFORM GROUNDS. The platform "
          f"moved this anchor by {abs(delta):.4f}, more than the {BAR} the "
          f"clauses are tested against, so a bar calibrated on the fleet is not "
          f"a bar this wave can be judged by.")
    print("  The cells are kept and the DiD is still a valid within-platform "
          "comparison, so print it. But enter no verdict against H29-1 or "
          "H29-2, cite the wave nowhere in the paper, and re-register for the "
          "fleet.")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
