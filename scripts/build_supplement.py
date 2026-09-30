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
import io
import os
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

#: The project's own name. The public repository is called BINN and so are its
#: crates, so "BINN" or `binn-lab` in a double-blind submission is one search
#: away from the author. It is renamed CONSISTENTLY -- file contents and paths
#: alike, and the anonymous PDF uses this same table (`build_paper.py`) -- so
#: the commands the paper prints still run against the code the archive holds.
#: The look-arounds keep ordinary words intact: "binning", "binned".
def _case_variants(name: str, replacement: str) -> list[tuple[re.Pattern[str], str]]:
    """UPPER, Capitalised and lower forms of one name, each a whole token.
    `binn_core`, `BINN_X` and `Binn` all match; `binning`, `Binned` do not."""
    return [
        (re.compile(rf"(?<![A-Za-z]){name.upper()}(?![A-Za-z])"), replacement.upper()),
        (re.compile(rf"(?<![A-Za-z]){name.capitalize()}(?![a-z])"), replacement.capitalize()),
        (re.compile(rf"(?<![A-Za-z]){name}(?![a-z])"), replacement),
    ]


PROJECT_RENAMES: list[tuple[re.Pattern[str], str]] = [
    *_case_variants("binn", "anon"),
    # The author's own published crates, vendored because the workspace
    # depends on them by path. Their names lead to the same GitHub account.
    *_case_variants("sparsl", "spkernel"),
    *_case_variants("tessl", "tkernel"),
]

#: What must not survive, checked after redaction over every byte of every
#: staged file, binary included. Deliberately broader than the rewrites.
IDENTIFYING = re.compile(
    rb"bharath|vbcr|vaddaram|brain-inspired_neural_network"
    rb"|(?<![a-z])(binn|sparsl|tessl)(?![a-z])", re.I)


def rename_project(text: str) -> tuple[str, int]:
    total = 0
    for pattern, replacement in PROJECT_RENAMES:
        text, n = pattern.subn(replacement, text)
        total += n
    return text, total


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
    text, n = rename_project(text)
    return text, total + n


def _path_deps(manifest: dict) -> list[tuple[str, str]]:
    """(dependency name, path) for every `path =` dependency in a Cargo
    manifest, in every dependency table including target-specific ones."""
    tables = [manifest]
    tables += list(manifest.get("target", {}).values())
    out = []
    for table in tables:
        for key in ("dependencies", "dev-dependencies", "build-dependencies"):
            for name, spec in table.get(key, {}).items():
                if isinstance(spec, dict) and "path" in spec:
                    out.append((name, spec["path"]))
    return out


#: The author's own published crates the workspace depends on. Their names
#: lead to the author's accounts on crates.io and GitHub, so the archive
#: renames them (PROJECT_RENAMES), and a renamed registry dependency cannot
#: resolve. So each is vendored under `vendor/` from the exact `.crate` file
#: Cargo.lock pins, and the manifests that name it are repointed there.
AUTHOR_CRATES = ("sparsl", "tessl")


def _lock_entry(name: str) -> dict:
    import tomllib
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    entries = [p for p in lock.get("package", []) if p["name"] == name]
    if len(entries) != 1:
        fail(f"Cargo.lock has {len(entries)} entries for {name}; the vendored "
             "copy needs exactly one.")
    return entries[0]


def locked_version(name: str) -> str:
    return _lock_entry(name)["version"]


def vendored_files(name: str) -> list[tuple[str, bytes]]:
    """[(path inside crate, bytes)] from the `.crate` Cargo.lock pins.

    Read from cargo's own download cache and checked against the lock's
    sha256, so the archive carries byte-for-byte what `cargo build --locked`
    compiles, not a checkout that may have moved. Cargo's bookkeeping files
    are dropped; the normalised Cargo.toml the registry serves is kept."""
    import hashlib
    import tarfile
    entry = _lock_entry(name)
    version, checksum = entry["version"], entry.get("checksum")
    if not checksum:
        fail(f"Cargo.lock has no checksum for {name}; it is not a registry "
             "dependency, so there is no published package to vendor.")
    home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
    found = sorted(home.glob(f"registry/cache/*/{name}-{version}.crate"))
    if not found:
        fail(f"{name}-{version}.crate is not in {home}/registry/cache. Run "
             "`cargo fetch --locked` first.")
    data = found[0].read_bytes()
    got = hashlib.sha256(data).hexdigest()
    if got != checksum:
        fail(f"{found[0]} has sha256 {got}; Cargo.lock pins {checksum}.")
    skip = {".cargo_vcs_info.json", "Cargo.toml.orig", "Cargo.lock"}
    prefix = f"{name}-{version}/"
    files = []
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tar:
        for member in tar.getmembers():
            if not member.isfile() or not member.name.startswith(prefix):
                continue
            inner = member.name[len(prefix):]
            if inner in skip:
                continue
            files.append((inner, tar.extractfile(member).read()))
    if not files:
        fail(f"{found[0]} contains no files.")
    return files


def vendoring_plan(files: list[str]) -> dict[str, list[str]]:
    """{manifest rel path: [author crate names it depends on]}.

    Also the guard that made this necessary: a path dependency outside the
    repository means no fresh clone builds, which is how this repository
    stood until 2026-09-29. That is refused here rather than papered over."""
    import tomllib
    plan: dict[str, list[str]] = {}
    for rel in files:
        if not rel.endswith("Cargo.toml"):
            continue
        manifest = tomllib.loads((ROOT / rel).read_text())
        for name, dep_path in _path_deps(manifest):
            target = ((ROOT / rel).parent / dep_path).resolve()
            if not target.is_relative_to(ROOT):
                fail(f"{rel} depends on {name} by a path outside the "
                     f"repository ({target}); no fresh clone can build that. "
                     "Depend on a published version instead.")
        tables = [manifest] + list(manifest.get("target", {}).values())
        for table in tables:
            for key in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name in table.get(key, {}):
                    if name in AUTHOR_CRATES:
                        plan.setdefault(rel, []).append(name)
    return plan


def repoint(text: str, rel: str, name: str) -> str:
    """Replace a registry pin on `name` with a path to its vendored copy,
    keeping every other key (`optional`, `features`) on the line."""
    depth = len(pathlib.PurePosixPath(rel).parent.parts)
    path = "/".join([".."] * depth + ["vendor", name])
    version = re.escape(locked_version(name))
    bare = re.compile(rf'^{name}\s*=\s*"=?{version}"\s*$', re.M)
    table = re.compile(rf'^({name}\s*=\s*\{{[^}}\n]*?)version\s*=\s*"=?{version}"', re.M)
    if bare.search(text):
        return bare.sub(f'{name} = {{ path = "{path}" }}', text, count=1)
    if table.search(text):
        return table.sub(rf'\g<1>path = "{path}"', text, count=1)
    fail(f"{rel}: found no `{name}` dependency pinned at the locked version to "
         "repoint at its vendored copy.")


def relock() -> None:
    """Re-serialise the renamed Cargo.lock, and prove nothing but the names
    moved.

    Cargo writes the lock sorted by package name, so renaming `binn-*` to
    `anon-*` reorders it, and `--locked` treats a reordered file as a changed
    one. Letting cargo rewrite it offline is only safe if the resolution is
    identical, so the set of (name, version, source, checksum) is compared
    before and after, and any difference -- an upgrade, a new package -- fails.
    """
    import tomllib
    lock = STAGE / "Cargo.lock"

    # The vendored crates legitimately lose their registry source and
    # checksum; for them only name, version and dependencies must survive.
    vendored = {rename_project(name)[0] for name in AUTHOR_CRATES}

    def packages() -> set[tuple]:
        data = tomllib.loads(lock.read_text())
        return {(p["name"], p["version"],
                 None if p["name"] in vendored else p.get("source"),
                 None if p["name"] in vendored else p.get("checksum"),
                 tuple(sorted(p.get("dependencies", []))))
                for p in data.get("package", [])}

    before = packages()
    # `update --workspace` re-resolves only the workspace's own packages and
    # rewrites the lock; unlike `metadata` it downloads nothing, so it runs on
    # a machine whose cargo cache lacks a crate the build never compiles.
    proc = subprocess.run(["cargo", "update", "--workspace", "--offline"],
                          cwd=STAGE, capture_output=True, text=True, timeout=600)
    if proc.returncode != 0:
        fail("cargo could not resolve the redacted workspace offline:\n"
             + proc.stderr[-2000:])
    after = packages()
    if before != after:
        fail("re-serialising the renamed Cargo.lock changed the resolution, not "
             "just the order:\n  added   "
             + "\n  added   ".join(map(str, sorted(after - before)))
             + "\n  removed " + "\n  removed ".join(map(str, sorted(before - after))))


def paper_builder():
    """`build_paper.py`, loaded by path: it owns the record markers, so the
    supplement's RECORDS.md numbers them exactly as the PDF does."""
    import importlib.util
    spec = importlib.util.spec_from_file_location(
        "build_paper", ROOT / "scripts/build_paper.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def stage() -> tuple[int, int, int]:
    if STAGE.exists():
        shutil.rmtree(STAGE)
    files = source_files()
    manifest_deps = vendoring_plan(files)
    crates = sorted({name for names in manifest_deps.values() for name in names})
    plan: list[tuple[pathlib.Path | bytes, str]] = [(ROOT / rel, rel) for rel in files]
    for name in crates:
        members = vendored_files(name)
        print(f"vendoring {name} {locked_version(name)} from its checksum-verified .crate")
        plan += [(data, f"vendor/{name}/{inner}") for inner, data in members]
    # The anonymous PDF prints record markers only; this is where they resolve.
    plan.append((paper_builder().record_map().encode(), "RECORDS.md"))
    changed = rewrites = 0
    renamed_paths: dict[str, str] = {}
    for src, rel in plan:
        out_rel, _ = rename_project(rel)
        if out_rel in renamed_paths:
            fail(f"renaming the project maps both {renamed_paths[out_rel]!r} "
                 f"and {rel!r} to {out_rel!r}; one would overwrite the other.")
        renamed_paths[out_rel] = rel
        dst = STAGE / out_rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        data = src if isinstance(src, bytes) else src.read_bytes()
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            dst.write_bytes(data)       # binary: copied, then swept below
            continue
        for name in manifest_deps.get(rel, []):
            text = repoint(text, rel, name)
            rewrites += 1
        if rel == "Cargo.toml" and crates:
            # A path dependency inside the workspace directory becomes a member
            # automatically, which would resolve its dev-dependencies against
            # the workspace lock. In the repository these crates come from the
            # registry, outside the workspace; `exclude` keeps them so here.
            header = re.compile(r"^\[workspace\]\n", re.M)
            table = header.split(text, 1)
            if len(table) != 2 or re.search(r"^exclude\s*=",
                                            table[1].split("\n[", 1)[0], re.M):
                fail("the root Cargo.toml has no [workspace] table, or already "
                     "excludes something; vendoring cannot be wired safely.")
            text = header.sub('[workspace]\nexclude = ["vendor"]\n', text, count=1)
        new_text, n = redact(text)
        if n:
            changed += 1
            rewrites += n
        dst.write_text(new_text, encoding="utf-8")
    return len(plan), changed, rewrites


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
#: The last entry compiles the renamed workspace: the project rename touches
#: crate names, paths and `use` lines, and a reviewer's first command is a
#: build. Offline, against the locked dependencies already in the local cargo
#: cache, with its target directory OUTSIDE the staged copy so it is never
#: zipped.
VERIFY = [
    ["python3", "scripts/verify_published_numbers.py"],
    ["python3", "scripts/check_every_number.py"],
    ["python3", "scripts/check_record_links.py"],
    ["python3", "scripts/test_published_hashes_resolve.py"],
    ["cargo", "check", "--workspace", "--all-targets", "--offline", "--locked"],
]
CARGO_TARGET = OUT_DIR / "supplement-target"


def verify() -> list[str]:
    failures = []
    for cmd in VERIFY:
        # Only the direct cargo call is redirected. The python checks that
        # build `c1` themselves look for it under the workspace's own target/,
        # which is safe to populate: the archive is already written.
        env = (dict(os.environ, CARGO_TARGET_DIR=str(CARGO_TARGET))
               if cmd[0] == "cargo" else None)
        proc = subprocess.run(cmd, cwd=STAGE, capture_output=True, text=True,
                              timeout=3600, env=env)
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
    relock()
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
