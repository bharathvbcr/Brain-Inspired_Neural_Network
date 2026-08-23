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

# `assert_eq!` on a length or a count is an assertion about a length, however it
# is spelled. STRONG matches any `assert_eq!`, so before 2026-08-22 a test whose
# sole assertion was `assert_eq!(sig.len(), 4)` was classified as pinning a value
# and skipped -- even though `.len() == 4` on the next line would have been
# caught. `binn-learn/src/credit.rs`'s
# `multi_channel_neuromodulator_computes_combined_signal` is exactly that: its
# only check is a length the function sets structurally, so it passes if
# `compute_signal` returns all zeros.
LENGTH_EQ = re.compile(r"assert_eq!\s*\(\s*[^,]*\.len\(\)\s*,")

# Two operands that differ only in their numeric literals. `assert_eq!(m.for_post(999),
# m.for_post(0))` is true for any function that ignores that argument -- which
# `Modulators::for_post` does, by taking `_post`. This is reported per assertion
# rather than per test, because the containing test is often fine: that one also
# pins a real value on the line above.
NUMBER = re.compile(r"\b\d+(?:\.\d+)?\b")
ASSERT_EQ_ARGS = re.compile(r"assert_eq!\s*\((.+)\)\s*;?\s*$")

# Each detector is calibrated against a real instance in this repository. If a
# detector stops seeing its own instance, its output is not evidence and the run
# says so rather than printing a clean list.
CALIBRATION_CASE = "trains_at_every_depth_without_panicking"
LENGTH_CALIBRATION = "multi_channel_neuromodulator_computes_combined_signal"
TAUTOLOGY_CALIBRATION = "broadcast_modulators_preserve_legacy_scalar"


def split_two_args(text: str) -> tuple[str, str] | None:
    """Split `a, b` at the top-level comma, respecting nesting and strings."""
    depth = 0
    in_string = False
    for i, ch in enumerate(text):
        if ch == '"' and (i == 0 or text[i - 1] != "\\"):
            in_string = not in_string
        elif not in_string:
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
            elif ch == "," and depth == 0:
                return text[:i].strip(), text[i + 1 :].strip()
    return None


def is_tautological_eq(line: str) -> bool:
    """Do both operands of an `assert_eq!` normalise to the same expression?

    Numeric literals are erased before comparing, so a call made twice with
    different constants collapses to one expression. That is not proof the
    assertion is vacuous -- a function may genuinely depend on the argument --
    but it is a shape worth a human look, and it is invisible to every other
    pattern here.
    """
    match = ASSERT_EQ_ARGS.search(line.strip())
    if not match:
        return False
    args = split_two_args(match.group(1))
    if not args:
        return False
    left, right = (NUMBER.sub("N", a) for a in args)
    return left == right and left != "" and "N" in left


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

    def weak(assertion: str) -> bool:
        return bool(WEAK.search(assertion) or LENGTH_EQ.search(assertion))

    def strong(assertion: str) -> bool:
        # A length equality is weak evidence however it is spelled, so it must
        # not short-circuit the test to STRONG.
        if LENGTH_EQ.search(assertion):
            return False
        return bool(STRONG.search(assertion))

    rows, tautologies = [], []
    seen = {CALIBRATION_CASE: False, LENGTH_CALIBRATION: False,
            TAUTOLOGY_CALIBRATION: False}
    for path in sorted(pathlib.Path(args.root).rglob("*.rs")):
        if "/target/" in str(path) or "/patches/" in str(path):
            continue
        for name, body in test_bodies(path.read_text(errors="replace")):
            asserts = [line.strip() for line in body.splitlines() if "assert" in line]
            for assertion in asserts:
                if is_tautological_eq(assertion):
                    tautologies.append((str(path), name, assertion))
                    if name in seen:
                        seen[name] = True
            if not asserts or any(strong(a) for a in asserts):
                continue
            if all(weak(a) for a in asserts):
                rows.append((str(path), name, len(asserts)))
                if name in seen:
                    seen[name] = True

    missed = [case for case, found in seen.items() if not found]
    if missed:
        print(
            f"CALIBRATION FAILED: {missed} not detected.\n"
            "A detector no longer catches the instance it was built for, so its\n"
            "output is not evidence of anything. Fix the patterns before reading on.",
            file=sys.stderr,
        )
        return 0

    print(f"calibration ok ({len(seen)} detectors, each found its own instance)")
    if tautologies:
        print(f"\n{len(tautologies)} assertion(s) whose two sides differ only in a "
              f"numeric literal:\n")
        for path, name, assertion in tautologies:
            print(f"  {path}::{name}")
            print(f"      {assertion}")
        print("\nEach is true for any implementation that ignores that argument.")
    print(f"\n{len(rows)} test(s) whose assertions a degenerate result would satisfy:\n")
    for path, name, count in rows:
        print(f"  {path}::{name}  ({count} assertion(s))")
    print("\nNot all of these are defects - robustness and smoke tests belong here.")
    print("The question for each is: would this pass if the thing under test did nothing?")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
