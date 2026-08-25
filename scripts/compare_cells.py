"""Bit-exact comparison of two instrument cell JSONs.

Gate F regresses recorded cells, but every recorded cell is `ff+fixed`. The
other three arms have no recorded output, so a shared-kernel change can move
them with every gate still green. This compares an arm cell captured before a
kernel change against the same cell after it, at real training density —
which is the level fixture parity does not reach (see
`AMENDMENT_2026-08-02_INSTRUMENT_KERNEL_AND_FRAMING.md`).

    .venv-shd/bin/python scripts/compare_cells.py before.json after.json

Exit status is 1 on any mismatch, so it can gate a sweep.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gate_f_rust import COMPARED_FIELDS  # noqa: E402

# `wall_secs` is a timing, not a result — a kernel optimisation is *supposed* to
# move it. Everything else the cell reports is a measurement and must not move.
IGNORED = {"wall_secs"}


def compare(before: Path, after: Path) -> list[str]:
    left = json.loads(before.read_text())
    right = json.loads(after.read_text())
    problems = []
    # A floor on what was actually examined. Without it, two empty cells
    # compared zero fields and reported "bit-identical" — and this is the only
    # guard covering the three arms Gate F cannot reach, so "compared nothing"
    # read as "the kernel did not move them". The required set is Gate F's own
    # `COMPARED_FIELDS` rather than a second list, so the two cannot drift.
    absent = [f for f in COMPARED_FIELDS
              if f not in left or f not in right]
    if absent:
        problems.append(
            f"CANNOT COMPARE: {len(absent)} measurement(s) absent from one or "
            f"both cells ({', '.join(absent[:4])}"
            f"{', ...' if len(absent) > 4 else ''}). Comparing what remains "
            "would report agreement over fields that are not there."
        )
        return problems
    for key in sorted(set(left) | set(right)):
        if key in IGNORED:
            continue
        if key not in left:
            problems.append(f"{key}: absent before, {right[key]!r} after")
        elif key not in right:
            problems.append(f"{key}: {left[key]!r} before, absent after")
        # `repr` rather than `==` so 1.0 and 1 cannot compare equal, and so a
        # float that round-trips to a different literal is caught.
        elif repr(left[key]) != repr(right[key]):
            problems.append(f"{key}: {left[key]!r} -> {right[key]!r}")
    return problems


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__)
        return 2
    before, after = Path(argv[0]), Path(argv[1])
    problems = compare(before, after)
    label = before.stem
    if problems:
        print(f"[ FAIL] {label}")
        for problem in problems:
            print(f"        {problem}")
        return 1
    speed = ""
    try:
        was = json.loads(before.read_text())["wall_secs"]
        now = json.loads(after.read_text())["wall_secs"]
        speed = f"  ({was:.1f}s -> {now:.1f}s, {was / max(now, 1e-9):.2f}x)"
    except KeyError:
        pass
    print(f"[  ok ] {label} bit-identical{speed}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
