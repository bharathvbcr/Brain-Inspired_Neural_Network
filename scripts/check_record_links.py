#!/usr/bin/env python3
"""Every internal link in `results/*.md` must resolve.

# Why

The record is a citation graph. A result cites the preregistration that governs
it, a finding cites the document it withdraws, the paper cites both — and the
whole discipline of "read this only through the document that retired it"
depends on the link actually going there.

A broken link is not cosmetic here. It is a citation to a claim the reader
cannot check, in a repository whose central argument is that every claim should
be checkable.

An audit on 2026-08-23 found **one** broken link in 707: `c1-micro-1e4.md`
pointed at `../c1_mac_probe.md` when the file sits beside it. One in seven
hundred is a good ratio and this exists to keep it.

    python3 scripts/check_record_links.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "results"

#: `[text](target)`. Anchors, mail and web links are somebody else's problem.
LINK = re.compile(r"\[([^\]]+)\]\(([^)\s]+)\)")
EXTERNAL = ("http://", "https://", "mailto:", "#")


def main() -> int:
    broken: list[tuple[str, str, str]] = []
    checked = 0
    documents = sorted(RESULTS.glob("*.md"))
    if len(documents) < 100:
        print(f"only {len(documents)} documents found in results/; the scan is "
              "not looking where it thinks it is", file=sys.stderr)
        return 1

    for doc in documents:
        for match in LINK.finditer(doc.read_text(errors="replace")):
            label, target = match.group(1), match.group(2)
            if target.startswith(EXTERNAL):
                continue
            checked += 1
            # Strip an anchor: `FILE.md#section` resolves to `FILE.md`.
            path = (doc.parent / target.split("#", 1)[0]).resolve()
            if not path.exists():
                broken.append((doc.name, label, target))

    if checked < 500:
        print(f"only {checked} internal links found; the pattern has stopped "
              "matching and this check is not evidence", file=sys.stderr)
        return 1

    print(f"{checked} internal links checked across {len(documents)} documents")
    if broken:
        print(f"\n{len(broken)} broken:\n")
        for name, label, target in broken:
            print(f"  {name}")
            print(f"      [{label}] -> {target}")
        return 1
    print("all resolve")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
