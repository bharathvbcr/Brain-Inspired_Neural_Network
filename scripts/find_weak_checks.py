#!/usr/bin/env python3
"""Find tests whose assertions are all satisfiable by a degenerate result.

The recurring defect in this workspace is not a wrong number, it is a check that
cannot fail. `deep-snn-scaling`'s gradient ceiling sat at chance for months while
its only training test asserted `is_finite()` and `(0.0..=1.0).contains(..)` —
both true of a constant predictor.

This is a heuristic, not a proof. It is **calibrated against that known
instance** and refuses to report anything if it stops detecting it, so a
regex that quietly stops matching is a failure rather than a clean run.

Usage:  python3 scripts/find_weak_checks.py [--root .]
Exit 0 always: this is a review aid, not a gate. Weak assertions are sometimes
correct (robustness and smoke tests). Read the list, do not obey it.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

# Assertions a degenerate run satisfies.
WEAK = re.compile(
    r"is_finite\(\)"
    r"|\(\s*[\d.]+\s*\.\.=?\s*[\d.]+\s*\)\.contains\("
    r"|\.is_ok\(\)"
    r"|\.is_some\(\)"
    r"|!\s*[\w.]+\.is_empty\(\)"
    r"|\.len\(\)\s*[><=]"
)
# Assertions that pin a value, a real threshold, or exact bits.
STRONG = re.compile(
    r"assert_eq!|assert_ne!|to_bits\(\)"
    r"|chance|floor|ATOL|epsilon|EPS"
    r"|abs\(\)\s*<"
    r"|[><]=?\s*0\.[1-9]"
    r"|is_err\(\)"
)

CALIBRATION_CASE = "trains_at_every_depth_without_panicking"


def test_bodies(text: str):
    # Doc comments and further attributes may sit between `#[test]` and `fn`;
    # an earlier version of this pattern required them to be adjacent and
    # silently stopped matching the calibration case the moment one was
    # documented. That is the failure this scanner exists to find, so it is not
    # allowed to have it.
    header = re.compile(
        r"#\[test\][^\n]*\n"
        r"(?:\s*(?://[^\n]*|#\[[^\]]*\])\n)*"
        r"\s*(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)\s*(?:->[^{]*)?\{"
    )
    for match in header.finditer(text):
        start, depth, i = match.end(), 1, match.end()
        while i < len(text) and depth:
            depth += (text[i] == "{") - (text[i] == "}")
            i += 1
        yield match.group(1), text[start:i]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".")
    args = parser.parse_args()

    rows, calibrated = [], False
    for path in sorted(pathlib.Path(args.root).rglob("*.rs")):
        if "/target/" in str(path) or "/patches/" in str(path):
            continue
        for name, body in test_bodies(path.read_text(errors="replace")):
            asserts = [line.strip() for line in body.splitlines() if "assert" in line]
            if not asserts or any(STRONG.search(a) for a in asserts):
                continue
            if all(WEAK.search(a) for a in asserts):
                rows.append((str(path), name, len(asserts)))
                calibrated |= name == CALIBRATION_CASE

    if not calibrated:
        print(
            f"CALIBRATION FAILED: `{CALIBRATION_CASE}` was not detected.\n"
            "The heuristic no longer catches the instance it was built for, so its\n"
            "output is not evidence of anything. Fix the patterns before reading on.",
            file=sys.stderr,
        )
        return 0

    print(f"calibration ok ({CALIBRATION_CASE} detected)")
    print(f"\n{len(rows)} test(s) whose assertions a degenerate result would satisfy:\n")
    for path, name, count in rows:
        print(f"  {path}::{name}  ({count} assertion(s))")
    print("\nNot all of these are defects - robustness and smoke tests belong here.")
    print("The question for each is: would this pass if the thing under test did nothing?")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
