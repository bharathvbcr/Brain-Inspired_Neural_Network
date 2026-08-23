#!/usr/bin/env python3
"""GC1's scanner: banned production-path symbols, in code rather than in prose.

The ban exists so no production path uses a dense matmul or an autograd engine.
A *comment* naming one is not a use, and until 2026-08-22 this check could not
tell the difference: it grepped raw lines, so a comment explaining why the
attention kernel is **not** a matmul failed the gate that exists to ensure there
is no matmul.

Stripping comments makes the check more precise, not weaker -- a symbol inside a
comment is never executed, so no true positive is removed. A trailing comment on
a real line of code is still scanned, because only the comment part is removed,
not the line.

Because that is exactly the kind of change that can quietly stop detecting
anything, this refuses to report clean unless it first catches a known-banned
line and correctly ignores a commented one. A scanner that cannot prove it still
works is not evidence.
"""

from __future__ import annotations

import pathlib
import re
import sys

BANNED = re.compile(r"matmul|dense_layer|autograd|backward\(")

# Injected, never read from disk. If the scanner stops flagging the first or
# starts flagging the second, its output means nothing.
CALIBRATION_POSITIVE = "    let y = matmul(&a, &b);"
CALIBRATION_NEGATIVE = "    // this is deliberately not a matmul, see the note above"


def strip_comments(source: str) -> str:
    """Blank out `//` line comments and `/* */` blocks, preserving line count.

    String literals are respected so a `"//"` inside one is not treated as the
    start of a comment. Raw strings and nested block comments are not handled;
    both are rare in this workspace and erring toward *scanning* text is the
    safe direction for a ban.
    """
    out = []
    i = 0
    n = len(source)
    in_string = False
    in_line_comment = False
    in_block_comment = False
    while i < n:
        ch = source[i]
        nxt = source[i + 1] if i + 1 < n else ""
        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
                out.append(ch)
            else:
                out.append(" ")
            i += 1
        elif in_block_comment:
            if ch == "*" and nxt == "/":
                in_block_comment = False
                out.append("  ")
                i += 2
            else:
                out.append("\n" if ch == "\n" else " ")
                i += 1
        elif in_string:
            if ch == "\\" and nxt:
                out.append(ch + nxt)
                i += 2
                continue
            if ch == '"':
                in_string = False
            out.append(ch)
            i += 1
        elif ch == '"':
            in_string = True
            out.append(ch)
            i += 1
        elif ch == "/" and nxt == "/":
            in_line_comment = True
            out.append("  ")
            i += 2
        elif ch == "/" and nxt == "*":
            in_block_comment = True
            out.append("  ")
            i += 2
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def hits_in(source: str, path: str) -> list[str]:
    stripped = strip_comments(source)
    return [
        f"{path}:{number}:{line.strip()}"
        for number, line in enumerate(stripped.splitlines(), start=1)
        if BANNED.search(line)
    ]


def calibrate() -> str | None:
    if not hits_in(CALIBRATION_POSITIVE, "<calibration>"):
        return "the scanner no longer flags a plain banned call"
    if hits_in(CALIBRATION_NEGATIVE, "<calibration>"):
        return "the scanner flags a banned symbol that is only in a comment"
    mixed = 'let y = matmul(&a, &b); // and a comment saying matmul'
    if len(hits_in(mixed, "<calibration>")) != 1:
        return "the scanner mishandles code with a trailing comment"
    return None


def main() -> int:
    root = pathlib.Path(__file__).resolve().parents[1]
    problem = calibrate()
    if problem:
        print(f"GC1 CALIBRATION FAILED: {problem}", file=sys.stderr)
        return 2

    hits: list[str] = []
    for path in sorted(root.rglob("*.rs")):
        text = str(path)
        if "/target/" in text or path.name.endswith("_baseline.rs"):
            continue
        hits.extend(hits_in(path.read_text(errors="replace"), str(path.relative_to(root))))

    if hits:
        print("GC1 FAIL: banned symbols outside *_baseline.rs:")
        print("\n".join(hits))
        return 1
    print("GC1 PASS: no banned production-path symbols (comments excluded, scanner calibrated)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
