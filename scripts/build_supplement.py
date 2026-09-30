#!/usr/bin/env python3
"""Build the anonymised supplementary archive for the TMLR submission.

The paper cites its own record -- scripts, `RESULT_*` notes, archived cells --
as evidence, numbered R1..Rn in its appendix. Those citations are evidence
only if a reviewer can open them, and TMLR's reviewers see the PDF plus one
supplementary file of at most 100 MB, anonymised like the paper.

The record is not anonymous as committed. Run manifests carry absolute paths
under the author's home directory, the licences carry the author's name, and
the README links the project's public website. So the archive is built from a
REDACTED COPY, and this script refuses to report success unless three things
are true of that copy:

  1. no identifying string survives anywhere in it, text or binary;
  2. the archive is under TMLR's 100 MB cap;
  3. with `--verify`, the evidence checks still pass INSIDE the redacted copy.

The third is the one that matters. Redaction rewrites bytes in files the
record checks read, and an archive whose evidence no longer verifies would
look exactly like one that does. A check that could not run must not look
like a check that passed, so `--verify` is what `record_checks.sh` would need
before this archive is called evidence.

Usage:
    python3 scripts/build_supplement.py            # redact, sweep, zip
    python3 scripts/build_supplement.py --verify   # ... and re-run the checks inside
"""
from __future__ import annotations

import argparse
import pathlib
import re
import shutil
import subprocess
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "paper" / "build"
STAGE = OUT_DIR / "supplement"
ARCHIVE = OUT_DIR / "supplement.zip"

#: TMLR author guide: supplementary material up to 100 MB, PDF or ZIP.
SIZE_CAP = 100 * 1024 * 1024

#: Tracked paths that are not part of the evidence and are left out whole.
#: `.devcouncil/` is a local code-navigation index; `.github/` is CI wiring
#: that names the hosting account; `paper/build*` is generated output.
EXCLUDE = re.compile(r"^(\.devcouncil/|\.github/|paper/build)")

#: Rewrites applied to every text file, in order: the specific forms first so
#: that the generic fallbacks at the end cannot half-rewrite them.
REDACTIONS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"bharath\.vbcr@gmail\.com", re.I), "anonymous@example.org"),
    (re.compile(r"https?://github\.com/bharathvbcr/[A-Za-z0-9_.-]+", re.I),
     "https://anonymized.invalid/repository"),
    (re.compile(r"bharathvbcr", re.I), "anonymous"),
    (re.compile(r"Brain-Inspired_Neural_Network", re.I), "anonymized-repository"),
    (re.compile(r"(https?://)?binn\.vbcr\.dev", re.I), "https://anonymized.invalid"),
    (re.compile(r"Bharath Chandra Vaddaram", re.I), "Anonymous Author"),
    (re.compile(r"/Users/bharath/"), "/home/anon/"),
    (re.compile(r"-Users-bharath-"), "-home-anon-"),
    (re.compile(r"\bBharath's\b"), "the author's"),
    (re.compile(r"\bBharath\b"), "the author"),
    (re.compile(r"bharath|vbcr|vaddaram", re.I), "anon"),
]

#: What must not survive, checked after redaction over every byte of every
#: staged file, binary included. Deliberately broader than the rewrites.
IDENTIFYING = re.compile(rb"bharath|vbcr|vaddaram|brain-inspired_neural_network",
                         re.I)


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def source_files() -> list[str]:
    """Tracked files plus new, not-ignored ones: the working tree as it would
    be committed. Deleted-but-tracked paths are skipped."""
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT, capture_output=True, check=True).stdout.decode()
    return sorted({p for p in out.split("\0")
                   if p and not EXCLUDE.match(p) and (ROOT / p).is_file()})


def redact(text: str) -> tuple[str, int]:
    total = 0
    for pattern, replacement in REDACTIONS:
        text, n = pattern.subn(replacement, text)
        total += n
    return text, total


def stage() -> tuple[int, int, int]:
    if STAGE.exists():
        shutil.rmtree(STAGE)
    files = source_files()
    changed = rewrites = 0
    for rel in files:
        src, dst = ROOT / rel, STAGE / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        data = src.read_bytes()
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            dst.write_bytes(data)       # binary: copied, then swept below
            continue
        new, n = redact(text)
        if n:
            changed += 1
            rewrites += n
        dst.write_text(new, encoding="utf-8")
    return len(files), changed, rewrites


def sweep() -> list[str]:
    hits = []
    for path in sorted(STAGE.rglob("*")):
        if path.is_file() and IDENTIFYING.search(path.read_bytes()):
            hits.append(str(path.relative_to(STAGE)))
    return hits


def archive() -> int:
    if ARCHIVE.exists():
        ARCHIVE.unlink()
    with zipfile.ZipFile(ARCHIVE, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
        for path in sorted(STAGE.rglob("*")):
            if path.is_file():
                # A fixed timestamp: zip entries otherwise carry the local
                # clock, and the archive would differ on every build.
                info = zipfile.ZipInfo(str(path.relative_to(STAGE)),
                                       date_time=(2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                zf.writestr(info, path.read_bytes())
    return ARCHIVE.stat().st_size


#: The evidence checks that read the record and need nothing compiled: every
#: published number recomputed from the cells, every wave number derivable,
#: every internal record link resolving. Run from inside the redacted copy.
VERIFY = [
    ["python3", "scripts/verify_published_numbers.py"],
    ["python3", "scripts/check_every_number.py"],
    ["python3", "scripts/check_record_links.py"],
]


def verify() -> list[str]:
    failures = []
    for cmd in VERIFY:
        proc = subprocess.run(cmd, cwd=STAGE, capture_output=True, text=True,
                              timeout=1800)
        status = "ok" if proc.returncode == 0 else f"FAILED ({proc.returncode})"
        print(f"  inside the redacted copy: {' '.join(cmd[1:])}: {status}")
        if proc.returncode != 0:
            tail = "\n".join((proc.stdout + proc.stderr).splitlines()[-15:])
            failures.append(f"{' '.join(cmd[1:])} fails inside the redacted "
                            f"copy:\n{tail}")
    return failures


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--verify", action="store_true",
                    help="re-run the evidence checks inside the redacted copy")
    args = ap.parse_args()

    n_files, changed, rewrites = stage()
    print(f"staged {n_files} file(s); redacted {rewrites} occurrence(s) in "
          f"{changed} file(s)")
    hits = sweep()
    if hits:
        fail("identifying strings survive redaction in:\n"
             + "\n".join(f"  - {h}" for h in hits[:50])
             + (f"\n  ... and {len(hits) - 50} more" if len(hits) > 50 else "")
             + "\nAdd a rewrite to REDACTIONS or a path to EXCLUDE.")
    size = archive()
    print(f"wrote {ARCHIVE} ({size / 1024 / 1024:.1f} MB of "
          f"{SIZE_CAP / 1024 / 1024:.0f} MB allowed)")
    problems = []
    if size > SIZE_CAP:
        problems.append(f"the archive is {size} bytes, over TMLR's {SIZE_CAP}.")
    if args.verify:
        problems.extend(verify())
    else:
        print("  NOT VERIFIED: the evidence checks were not re-run inside the "
              "redacted copy. Pass --verify before calling this archive "
              "evidence.")
    for p in problems:
        print(f"  FAIL: {p}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
