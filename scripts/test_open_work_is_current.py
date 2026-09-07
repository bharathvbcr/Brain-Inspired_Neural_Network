#!/usr/bin/env python3
"""The open-work register may not describe a state the artefacts have left.

Three entries in `results/TODO_2026-08-07_OPEN_WORK.md` were found stale on one
day, 2026-09-07, and they are one class rather than three accidents:

  * `matrix_authorized` -- flagged `[!]`, "human decision required", for five
    weeks after the six reference cells were re-run and the gate went `true`.
  * The provenance flag -- flagged `[!]`, "default-off, awaiting a human
    decision", for 33 days after `PROVENANCE_DISCHARGE_ENABLED` was set `True`
    by explicit human authorization, with the code comment recording that the
    judgement "has now been made".
  * Gate F -- an entry asking for 13/13 while the committed report already
    carried 14 and `--all` re-runs 218.

The class is: **a register that nothing verifies against the artefacts it
describes**, which is the same sentence the register itself uses about the
record checks pointing at a corpus that had moved. A `[!]` is the worst case,
because it parks an item on a person who has already answered.

This file checks the claims that are mechanically checkable. It does not, and
cannot, check prose -- so it is a floor, not a guarantee, and the two entries it
covers are named rather than discovered.

Run: python3 scripts/test_open_work_is_current.py
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TODO = ROOT / "results/TODO_2026-08-07_OPEN_WORK.md"

#: `- [ ]` and `- [!]` are open; `- [x]` is closed and `- [~]` is in flight.
#: An entry runs to the next bullet at the same indent, so a closed entry may
#: discuss the state it used to be in without tripping anything here.
OPEN_ITEM = re.compile(r"^- \[([ !])\] (.*?)(?=^- \[|\Z)", re.M | re.S)


def open_entries() -> list[tuple[str, str]]:
    return [(m.group(1), m.group(2)) for m in OPEN_ITEM.finditer(TODO.read_text())]


class TheRegisterAgreesWithTheArtefacts(unittest.TestCase):
    """Each case is a staleness that actually happened, turned into a check."""

    def test_the_parser_sees_both_open_and_closed_items(self):
        """A regex matching nothing would pass every test below forever."""
        text = TODO.read_text()
        self.assertGreater(len(open_entries()), 0, "no open item parsed")
        self.assertIn("- [x]", text, "no closed item in the register")
        closed = len(re.findall(r"^- \[x\]", text, re.M))
        self.assertGreater(closed, 3, "the closed set looks too small to trust")

    def test_no_open_item_says_the_provenance_flag_is_undecided(self):
        """It was decided 2026-08-05 and the entry said otherwise for 33 days."""
        runner = ROOT / "scripts/shd_calibration/runner.py"
        source = runner.read_text()
        match = re.search(r"^PROVENANCE_DISCHARGE_ENABLED = (True|False)$",
                          source, re.M)
        self.assertIsNotNone(match, "PROVENANCE_DISCHARGE_ENABLED not found; "
                                    "this check is anchored to it by name")
        if match.group(1) != "True":
            self.skipTest("the flag is off, so an open entry about it is honest")
        for mark, body in open_entries():
            head = body.splitlines()[0] if body.splitlines() else ""
            if "provenance flag" in head.lower():
                self.fail(
                    f"open item `- [{mark}]` says: {head!r}\n"
                    f"but {runner.relative_to(ROOT)} has "
                    f"PROVENANCE_DISCHARGE_ENABLED = True, set 2026-08-05 by "
                    f"explicit human authorization. Close the entry or correct "
                    f"the flag; a `[!]` parks a decision on someone who has "
                    f"already made it.")

    def test_no_open_item_says_matrix_authorized_is_unresolved(self):
        """False since 2026-08-03, true since 2026-08-23, open until 2026-09-07."""
        gates = ROOT / "results/shd_instrument_v4/gates.json"
        if not gates.exists():
            self.skipTest("no v4 gates file to check against")
        state = json.loads(gates.read_text())
        if state.get("matrix_authorized") is not True:
            self.skipTest("the gate is not authorized, so an open entry is honest")
        for mark, body in open_entries():
            head = body.splitlines()[0] if body.splitlines() else ""
            if "matrix_authorized" in head:
                self.fail(
                    f"open item `- [{mark}]` says: {head!r}\n"
                    f"but {gates.relative_to(ROOT)} carries "
                    f"matrix_authorized: true.")

    def test_a_blocked_item_is_rarer_than_an_open_one(self):
        """`[!]` means a person must act. Three of three were found stale on one
        day, so the count itself is worth watching: a register full of them is
        a register nobody is reading."""
        marks = [m for m, _ in open_entries()]
        blocked = marks.count("!")
        self.assertLessEqual(
            blocked, 3,
            f"{blocked} items are flagged `[!]` (human decision required). "
            f"Every one found on 2026-09-07 had already been decided. Check "
            f"each against its artefact before adding more.")


if __name__ == "__main__":
    unittest.main(verbosity=2)
