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
  3. the evidence checks still pass INSIDE the redacted copy (the default;
     `--no-verify` skips them).

The third is the one that matters. Redaction rewrites bytes in files the
record checks read, and an archive whose evidence no longer verifies would
look exactly like one that does. A check that could not run must not look
like a check that passed, so a build that skipped them exits 3, not 0, and
is written as `supplement.unverified.zip`, never as `supplement.zip`.

Provenance. Only TRACKED files are staged, and only from a clean tree: an
untracked scratch file would otherwise ship with whatever it contains. The
archive carries `BUILD_INFO.json` (HEAD commit, tree hash, dirty flag, and a
digest of the staged sources). `--allow-dirty` builds from a dirty tree and
stamps the archive `"dirty": true`.

A previous archive is deleted before anything else, and the new one is
written as `supplement.zip.partial` and renamed only after the size cap and
the checks pass, so a failed build never leaves a stale `supplement.zip`.

Exit codes: 0 built and verified; 1 failed; 3 built but NOT verified.

Usage:
    python3 scripts/build_supplement.py              # redact, sweep, zip, verify
    python3 scripts/build_supplement.py --no-verify  # ... skip the checks (exit 3)
    python3 scripts/build_supplement.py --allow-dirty
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import zipfile
from collections.abc import Callable

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "paper" / "build"
STAGE = OUT_DIR / "supplement"
ARCHIVE = OUT_DIR / "supplement.zip"
#: Where the archive is written until it has passed every check, and where a
#: build that skipped the checks is left: neither name can be mistaken for the
#: submission.
PARTIAL = OUT_DIR / "supplement.zip.partial"
UNVERIFIED = OUT_DIR / "supplement.unverified.zip"
CARGO_TARGET = OUT_DIR / "supplement-target"
EXIT_FAILED, EXIT_UNVERIFIED = 1, 3

#: TMLR author guide: supplementary material up to 100 MB, PDF or ZIP.
SIZE_CAP = 100 * 1024 * 1024

#: Tracked paths that are not part of the evidence and are left out whole.
#: `.devcouncil/` is a local code-navigation index; `.github/` is CI wiring
#: that names the hosting account; `paper/build*` is generated output.
EXCLUDE = re.compile(r"^(\.devcouncil/|\.github/|paper/build)")

#: Rewrites applied to every text file, in order: the specific forms first so
#: that the generic fallbacks at the end cannot half-rewrite them.
#: AWS identifiers in the campaign record. The account id and the bucket names
#: built from it identify the author's cloud account; an instance id is
#: replaced by a stable pseudonym (a digest of the id, so two instances stay
#: two instances and a file named by one still lines up with the text that
#: cites it). The pseudonym is not itself `i-<17 hex>`, so IDENTIFYING cannot
#: match what the rewrite leaves behind.
AWS_ACCOUNT_ID = "511192439661"
AWS_ACCOUNT_PLACEHOLDER = "123456789012"
ACCOUNT_ID_RE = re.compile(rf"(?<![0-9]){AWS_ACCOUNT_ID}(?![0-9])")
INSTANCE_ID_RE = re.compile(r"(?<![A-Za-z0-9])i-[0-9a-f]{17}(?![0-9a-f])", re.I)


def _pseudonymise_instance(match: re.Match[str]) -> str:
    return "i-anon" + hashlib.sha256(match.group(0).lower().encode()).hexdigest()[:11]


#: Identifier rewrites that apply to paths as well as contents.
IDENTIFIER_REWRITES: list[tuple[re.Pattern[str], str | Callable[[re.Match[str]], str]]] = [
    (ACCOUNT_ID_RE, AWS_ACCOUNT_PLACEHOLDER),
    (INSTANCE_ID_RE, _pseudonymise_instance),
]

REDACTIONS: list[tuple[re.Pattern[str], str | Callable[[re.Match[str]], str]]] = [
    *IDENTIFIER_REWRITES,
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
    rb"|(?<![a-z])(binn|sparsl|tessl)(?![a-z])"
    rb"|(?<![0-9])" + AWS_ACCOUNT_ID.encode() + rb"(?![0-9])"
    rb"|(?<![a-z0-9])i-[0-9a-f]{17}(?![0-9a-f])", re.I)


def rename_project(text: str) -> tuple[str, int]:
    total = 0
    for pattern, replacement in PROJECT_RENAMES:
        text, n = pattern.subn(replacement, text)
        total += n
    return text, total


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def _git(*args: str) -> bytes:
    proc = subprocess.run(["git", *args], cwd=ROOT, capture_output=True)
    if proc.returncode != 0:
        fail(f"`git {' '.join(args)}` failed: {proc.stderr.decode().strip()}")
    return proc.stdout


def tracked_entries() -> dict[str, str]:
    """{path: git mode} for every tracked path, from the index.

    `--cached` only. This once also listed `--others` ("the working tree as it
    would be committed"), which shipped any untracked scratch file whole. The
    record needs nothing that is not committed, so that is gone."""
    out = _git("ls-files", "--cached", "--stage", "-z").decode("utf-8", "surrogateescape")
    entries = {}
    for record in filter(None, out.split("\0")):
        meta, _, path = record.partition("\t")
        entries[path] = meta.split()[0]
    return entries


def source_files() -> list[str]:
    """Tracked regular files. Gitlinks (submodule checkouts) are directories
    and deleted-but-tracked paths are absent; both are skipped. A tracked
    symlink is refused: the archive cannot represent it, and following it
    would copy whatever it points at, wherever that is."""
    files = []
    for path, mode in tracked_entries().items():
        if EXCLUDE.match(path):
            continue
        if mode == "120000":
            fail(f"{path} is a tracked symlink; the supplement cannot carry it "
                 "faithfully, and following it could copy a file from outside "
                 "the repository.")
        if (ROOT / path).is_file():
            files.append(path)
    return sorted(files)


def dirty_paths() -> list[str]:
    """Everything `git status` reports, untracked files included."""
    out = _git("status", "--porcelain", "-z", "--untracked-files=all")
    return [r[3:].decode("utf-8", "surrogateescape")
            for r in out.split(b"\0") if r]


def provenance(allow_dirty: bool) -> dict:
    """What the archive is built from. A dirty tree is refused unless asked
    for, and then recorded as dirty: HEAD names a commit a reviewer could
    compare against only when the tree matched it."""
    if subprocess.run(["git", "rev-parse", "--verify", "-q", "HEAD"], cwd=ROOT,
                      capture_output=True).returncode != 0:
        fail("the repository has no commit; there is nothing to build from.")
    dirty = dirty_paths()
    if dirty and not allow_dirty:
        fail(f"the working tree is not clean ({len(dirty)} path(s), e.g. "
             f"{dirty[0]!r}). Commit or remove them, or pass --allow-dirty to "
             "build an archive stamped dirty.")
    return {"commit": _git("rev-parse", "HEAD").decode().strip(),
            "tree": _git("rev-parse", "HEAD^{tree}").decode().strip(),
            "dirty": bool(dirty),
            "dirty_path_count": len(dirty)}


def redact_path(rel: str) -> str:
    """The archive path for a repository path: the project rename plus the
    identifier rewrites applied to contents, so a file named by an instance id
    still lines up with the text that cites it."""
    for pattern, replacement in IDENTIFIER_REWRITES:
        rel = pattern.sub(replacement, rel)
    return rename_project(rel)[0]


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


def vendored_files(name: str) -> list[tuple[str, bytes, bool]]:
    """[(path inside crate, bytes, executable)] from the `.crate` Cargo.lock pins.

    Read from cargo's own download cache and checked against the lock's
    sha256, so the archive carries byte-for-byte what `cargo build --locked`
    compiles, not a checkout that may have moved. Cargo's bookkeeping files
    are dropped; the normalised Cargo.toml the registry serves is kept."""
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
            files.append((inner, tar.extractfile(member).read(),
                          bool(member.mode & 0o111)))
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


def stage(info: dict | None = None) -> tuple[int, int, int]:
    if STAGE.exists():
        shutil.rmtree(STAGE)
    info = provenance(allow_dirty=False) if info is None else info
    files = source_files()
    modes = tracked_entries()
    manifest_deps = vendoring_plan(files)
    crates = sorted({name for names in manifest_deps.values() for name in names})
    plan: list[tuple[pathlib.Path | bytes, str, bool]] = [
        (ROOT / rel, rel, modes[rel] == "100755") for rel in files]
    digest = hashlib.sha256()
    for rel in files:
        digest.update(rel.encode("utf-8", "surrogateescape") + b"\0")
        digest.update(hashlib.sha256((ROOT / rel).read_bytes()).digest())
    for name in crates:
        members = vendored_files(name)
        print(f"vendoring {name} {locked_version(name)} from its checksum-verified .crate")
        plan += [(data, f"vendor/{name}/{inner}", exe) for inner, data, exe in members]
    # The anonymous PDF prints record markers only; this is where they resolve.
    plan.append((paper_builder().record_map().encode(), "RECORDS.md", False))
    # Provenance. `source_sha256` digests the staged repository files as they
    # were read, so a dirty build is still pinned to exactly what it contained.
    build_info = dict(info, source_files=len(files), source_sha256=digest.hexdigest())
    plan.append(((json.dumps(build_info, indent=2, sort_keys=True) + "\n").encode(),
                 "BUILD_INFO.json", False))
    changed = rewrites = 0
    renamed_paths: dict[str, str] = {}
    for src, rel, executable in plan:
        out_rel = redact_path(rel)
        if out_rel in renamed_paths:
            fail(f"redacting paths maps both {renamed_paths[out_rel]!r} "
                 f"and {rel!r} to {out_rel!r}; one would overwrite the other.")
        renamed_paths[out_rel] = rel
        dst = STAGE / out_rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        data = src if isinstance(src, bytes) else src.read_bytes()
        # Modes are explicit, not inherited from whatever umask or source
        # checkout this machine has: 0644, or 0755 where git records the
        # executable bit.
        mode = 0o755 if executable else 0o644
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            dst.write_bytes(data)       # binary: copied, then swept below
            dst.chmod(mode)
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
        dst.chmod(mode)
    return len(plan), changed, rewrites


def sweep() -> list[str]:
    """Staged paths whose NAME or CONTENT identifies. A file called
    `notes_<author>_draft.txt` is as identifying as one that says so inside."""
    hits = []
    for path in sorted(STAGE.rglob("*")):
        rel = path.relative_to(STAGE).as_posix()
        if IDENTIFYING.search(rel.encode("utf-8", "surrogateescape")):
            hits.append(f"{rel} (path)")
        elif path.is_file() and IDENTIFYING.search(path.read_bytes()):
            hits.append(rel)
    return hits


def sweep_archive(path: pathlib.Path) -> list[str]:
    """The same sweep over the zip as written: entry names and every
    decompressed byte. What is shipped is checked, not what was staged."""
    hits = []
    with zipfile.ZipFile(path) as zf:
        for entry in zf.infolist():
            if IDENTIFYING.search(entry.filename.encode("utf-8", "surrogateescape")):
                hits.append(f"{entry.filename} (zip entry name)")
            elif IDENTIFYING.search(zf.read(entry)):
                hits.append(f"{entry.filename} (zip entry)")
    return hits


def archive(dest: pathlib.Path | None = None) -> int:
    dest = PARTIAL if dest is None else dest
    if dest.exists():
        dest.unlink()
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
        for path in sorted(STAGE.rglob("*")):
            if path.is_file():
                # A fixed timestamp: zip entries otherwise carry the local
                # clock, and the archive would differ on every build.
                info = zipfile.ZipInfo(path.relative_to(STAGE).as_posix(),
                                       date_time=(2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3          # Unix: external_attr carries a mode
                executable = bool(path.stat().st_mode & 0o111)
                info.external_attr = (0o100755 if executable else 0o100644) << 16
                zf.writestr(info, path.read_bytes())
    return dest.stat().st_size


def set_out_dir(out_dir: pathlib.Path) -> None:
    """Redirect every output path. Used by `--out-dir` and by tests; the
    defaults are paper/build/."""
    global OUT_DIR, STAGE, ARCHIVE, PARTIAL, UNVERIFIED, CARGO_TARGET
    OUT_DIR = out_dir
    STAGE = out_dir / "supplement"
    ARCHIVE = out_dir / "supplement.zip"
    PARTIAL = out_dir / "supplement.zip.partial"
    UNVERIFIED = out_dir / "supplement.unverified.zip"
    CARGO_TARGET = out_dir / "supplement-target"


#: The evidence checks that read the record and need nothing compiled: every
#: published number recomputed from the cells, every wave number derivable,
#: every internal record link resolving. Run from inside the redacted copy.
#: The last entry compiles the renamed workspace: the project rename touches
#: crate names, paths and `use` lines, and a reviewer's first command is a
#: build. Offline, against the locked dependencies already in the local cargo
#: cache, with its target directory OUTSIDE the staged copy so it is never
#: zipped.
#:
#: What this does and does not establish: the python checks RUN against the
#: redacted record; `cargo check` proves the renamed workspace TYPE-CHECKS
#: (every target, tests included) offline against the locked graph. It does not
#: run the Rust tests. `cargo test --workspace` was weighed and left out: the
#: checks above already take about 35 minutes on the author's machine (the
#: python checks compile and run `c1`), a full test run adds an unbounded
#: amount on top, and the rename cannot change a test's outcome without
#: changing whether it compiles. `verify()` prints this scope on every run.
VERIFY_SCOPE = ("verified: published numbers, number derivations, record links, "
                "published hashes (run); Rust workspace type-checks (cargo check, "
                "NOT cargo test: Rust tests were not run)")
VERIFY = [
    [sys.executable, "scripts/verify_published_numbers.py"],
    [sys.executable, "scripts/check_every_number.py"],
    [sys.executable, "scripts/check_record_links.py"],
    [sys.executable, "scripts/test_published_hashes_resolve.py"],
    ["cargo", "check", "--workspace", "--all-targets", "--offline", "--locked"],
]


def verify() -> list[str]:
    failures = []
    for cmd in VERIFY:
        # Only the direct cargo call is redirected. The python checks that
        # build `c1` themselves look for it under the workspace's own target/,
        # which is safe to populate: the archive is already written.
        env = (dict(os.environ, CARGO_TARGET_DIR=str(CARGO_TARGET))
               if cmd[0] == "cargo" else None)
        label = " ".join([pathlib.Path(cmd[0]).name, *cmd[1:]])
        try:
            proc = subprocess.run(cmd, cwd=STAGE, capture_output=True, text=True,
                                  timeout=3600, env=env)
        except subprocess.TimeoutExpired:
            print(f"  inside the redacted copy: {label}: TIMED OUT")
            failures.append(f"{label} did not finish within an hour inside the "
                            "redacted copy; it is unverified, not passed.")
            continue
        status = "ok" if proc.returncode == 0 else f"FAILED ({proc.returncode})"
        print(f"  inside the redacted copy: {label}: {status}")
        if proc.returncode != 0:
            tail = "\n".join((proc.stdout + proc.stderr).splitlines()[-15:])
            failures.append(f"{label} fails inside the redacted copy:\n{tail}")
    return failures


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--no-verify", action="store_true",
                    help="skip re-running the evidence checks inside the "
                         "redacted copy; the build then exits 3 and is written "
                         "as supplement.unverified.zip")
    ap.add_argument("--allow-dirty", action="store_true",
                    help="build from a working tree with uncommitted or "
                         "untracked changes; the archive is stamped dirty")
    ap.add_argument("--out-dir", type=pathlib.Path,
                    help="write under this directory instead of paper/build")
    args = ap.parse_args(argv)
    if args.out_dir:
        set_out_dir(args.out_dir.resolve())

    # A failed build must not leave an earlier archive looking current.
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for stale in (ARCHIVE, PARTIAL, UNVERIFIED):
        stale.unlink(missing_ok=True)
    try:
        return _build(args)
    finally:
        PARTIAL.unlink(missing_ok=True)


def _build(args: argparse.Namespace) -> int:
    info = provenance(args.allow_dirty)
    n_files, changed, rewrites = stage(info)
    print(f"staged {n_files} file(s) from {info['commit'][:12]}"
          f"{' (DIRTY)' if info['dirty'] else ''}; redacted {rewrites} "
          f"occurrence(s) in {changed} file(s)")
    hits = sweep()
    if hits:
        fail("identifying strings survive redaction in:\n"
             + "\n".join(f"  - {h}" for h in hits[:50])
             + (f"\n  ... and {len(hits) - 50} more" if len(hits) > 50 else "")
             + "\nAdd a rewrite to REDACTIONS or a path to EXCLUDE.")
    relock()
    size = archive(PARTIAL)
    print(f"wrote {PARTIAL} ({size / 1024 / 1024:.1f} MB of "
          f"{SIZE_CAP / 1024 / 1024:.0f} MB allowed)")
    problems = []
    if size > SIZE_CAP:
        problems.append(f"the archive is {size} bytes, over TMLR's {SIZE_CAP}.")
    problems += [f"identifying string survives in the written zip: {h}"
                 for h in sweep_archive(PARTIAL)]
    if not problems and not args.no_verify:
        problems.extend(verify())
    for p in problems:
        print(f"  FAIL: {p}")
    if problems:
        return EXIT_FAILED
    if args.no_verify:
        os.replace(PARTIAL, UNVERIFIED)
        print(f"  NOT VERIFIED: the evidence checks were not re-run inside the "
              f"redacted copy. Left as {UNVERIFIED}, not {ARCHIVE.name}; this "
              f"is not evidence. Exit {EXIT_UNVERIFIED}.")
        return EXIT_UNVERIFIED
    os.replace(PARTIAL, ARCHIVE)
    print(f"  {VERIFY_SCOPE}")
    print(f"verified; {ARCHIVE} is final"
          f"{' (stamped DIRTY: not reproducible from a commit)' if info['dirty'] else ''}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
