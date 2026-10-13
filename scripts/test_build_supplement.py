#!/usr/bin/env python3
"""The supplement's redaction removes every identity it claims to, and the
sweep after it catches what the redaction misses.

The committed record names its author in run manifests (`/Users/<name>/`),
licences, the README's website link and the CI config. None of that shows in
the PDF, which is the only thing `build_paper.py --check` reads -- so a
supplement built straight from the repository would de-anonymise a
double-blind submission while every paper check stayed green.
"""
from __future__ import annotations

import importlib.util
import io
import json
import pathlib
import unittest
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "build_supplement", ROOT / "scripts/build_supplement.py")
bs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bs)


class FreshCloneTest(unittest.TestCase):
    """No manifest may depend on a path outside the repository.

    Until 2026-09-29 `binn-core` depended on sparsl and tessl by paths into
    sibling checkouts, so no fresh clone of this repository built, and nor did
    a supplement copied from it. Both are now exact crates.io pins; this is the
    guard that keeps them so."""

    def test_the_repository_has_no_path_dependency_outside_itself(self):
        plan = bs.vendoring_plan(bs.source_files())
        self.assertEqual(sorted(plan.get("binn-core/Cargo.toml", [])),
                         ["sparsl", "tessl"])

    def test_every_crates_io_patch_is_used(self):
        """Cargo records a `[patch]` entry that matches nothing in the graph as
        `[[patch.unused]]` and builds anyway, so a dead patch ships in every
        clone and supplement unnoticed. `patches/block` was one: it served the
        `metal` crate, which is no longer in the lock file, and it stayed until
        2026-09-29."""
        lock = (ROOT / "Cargo.lock").read_text()
        self.assertNotIn("[[patch.unused]]", lock)

    def test_a_path_outside_the_repository_is_refused(self):
        import tempfile
        with tempfile.TemporaryDirectory() as tmp:
            repo = pathlib.Path(tmp) / "repo"
            (repo / "core").mkdir(parents=True)
            (repo / "core/Cargo.toml").write_text(
                '[package]\nname = "core"\n\n'
                '[dependencies]\nsibling = { path = "../../sibling" }\n')
            saved = bs.ROOT
            bs.ROOT = repo
            try:
                with self.assertRaises(SystemExit):
                    bs.vendoring_plan(["core/Cargo.toml"])
            finally:
                bs.ROOT = saved

    def test_both_pin_forms_are_repointed_and_other_keys_kept(self):
        sparsl, tessl = bs.locked_version("sparsl"), bs.locked_version("tessl")
        text = (f'sparsl = "={sparsl}"\n'
                f'tessl = {{ version = "={tessl}", optional = true }}\n')
        text = bs.repoint(text, "binn-core/Cargo.toml", "sparsl")
        text = bs.repoint(text, "binn-core/Cargo.toml", "tessl")
        self.assertEqual(text,
                         'sparsl = { path = "../vendor/sparsl" }\n'
                         'tessl = { path = "../vendor/tessl", optional = true }\n')

    def test_a_pin_off_the_locked_version_is_refused(self):
        with self.assertRaises(SystemExit):
            bs.repoint('sparsl = "=0.0.1"\n', "binn-core/Cargo.toml", "sparsl")


class RedactionTest(unittest.TestCase):

    CASES = [
        '"path": "/Users/bharath/Code/parameter_golf/binn/cells/a.json"',
        "Copyright (c) 2026 Bharath Chandra Vaddaram",
        '<a href="https://binn.vbcr.dev/">Explore</a>',
        "contact bharath.vbcr@gmail.com for data",
        "git clone https://github.com/bharathvbcr/Brain-Inspired_Neural_Network.git",
        "/private/tmp/claude-501/-Users-bharath-Code-parameter-golf/3ade",
        "Bharath's stated lean: ignore it",
        "**Owner:** Bharath · **Status:** closed",
    ]

    def test_every_known_form_is_redacted_and_leaves_nothing_the_sweep_finds(self):
        for case in self.CASES:
            out, n = bs.redact(case)
            self.assertGreater(n, 0, case)
            self.assertIsNone(bs.IDENTIFYING.search(out.encode()),
                              f"{case!r} -> {out!r} still identifies")

    def test_paths_stay_paths(self):
        """A manifest path must remain a well-formed path after redaction, or
        a script that reads it would fail in the reviewer's copy for a reason
        that has nothing to do with the science."""
        out, _ = bs.redact('"/Users/bharath/Code/x/cell.json"')
        self.assertEqual(out, '"/home/anon/Code/x/cell.json"')

    def test_text_without_an_identity_is_untouched(self):
        text = "gain +0.1275, 32/32 seeds; see RESULT_2026-09-09_W29.md\n"
        self.assertEqual(bs.redact(text), (text, 0))

    def test_the_sweep_is_case_insensitive_and_reads_bytes(self):
        """Binary files are copied without redaction, so the byte-level sweep
        is the only thing standing between them and the archive."""
        for needle in (b"BHARATH", b"Vbcr", b"vaddaram",
                       b"Brain-Inspired_Neural_Network"):
            self.assertIsNotNone(bs.IDENTIFYING.search(b"\x00\x01" + needle))

    def test_the_cap_is_tmlr_s(self):
        self.assertEqual(bs.SIZE_CAP, 100 * 1024 * 1024)

    def test_generated_output_and_ci_wiring_are_excluded(self):
        for path in ("paper/build/main.pdf", "paper/build-preprint/main.pdf",
                     ".github/workflows/ci.yml", ".devcouncil/repo_map.json"):
            self.assertIsNotNone(bs.EXCLUDE.match(path), path)
        self.assertIsNone(bs.EXCLUDE.match("results/PAPER_DRAFT.md"))


class TempRepoCase(unittest.TestCase):
    """A real throwaway git repository, with the module's paths pointed at it."""

    def setUp(self):
        import tempfile
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        base = pathlib.Path(self._tmp.name).resolve()
        self.repo = base / "repo"
        self.repo.mkdir()
        self.out = base / "out"
        self._saved = (bs.ROOT, bs.OUT_DIR, bs.STAGE, bs.ARCHIVE,
                       *(getattr(bs, n, None) for n in
                         ("PARTIAL", "UNVERIFIED", "CARGO_TARGET")))
        bs.ROOT = self.repo
        self.addCleanup(self._restore)
        self.git("init", "-q")

    def _restore(self):
        bs.ROOT, bs.OUT_DIR, bs.STAGE, bs.ARCHIVE = self._saved[:4]
        for name, value in zip(("PARTIAL", "UNVERIFIED", "CARGO_TARGET"),
                               self._saved[4:]):
            if value is not None:
                setattr(bs, name, value)

    def git(self, *args):
        import subprocess
        return subprocess.run(
            ["git", "-c", "user.email=t@example.org", "-c", "user.name=t",
             "-c", "commit.gpgsign=false", *args],
            cwd=self.repo, check=True, capture_output=True, text=True).stdout

    def write(self, rel, text="x\n", mode=None):
        path = self.repo / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        if mode:
            path.chmod(mode)
        return path

    def commit_all(self):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "c")


class WhatIsStagedTest(TempRepoCase):
    """H2: only committed files ship, from a clean tree, with provenance."""

    def test_an_untracked_file_is_not_staged(self):
        self.write("a.txt")
        self.commit_all()
        self.write("scripts/secret_notes.txt", "AKIAABCDEFGHIJKLMNOP\n")
        self.assertEqual(bs.source_files(), ["a.txt"])

    def test_a_dirty_tree_is_refused_untracked_included(self):
        self.write("a.txt")
        self.commit_all()
        self.write("stray.txt")
        with self.assertRaises(SystemExit):
            bs.provenance(allow_dirty=False)

    def test_a_modified_tracked_file_is_refused(self):
        self.write("a.txt")
        self.commit_all()
        self.write("a.txt", "changed\n")
        with self.assertRaises(SystemExit):
            bs.provenance(allow_dirty=False)

    def test_a_staged_but_uncommitted_file_is_refused(self):
        self.write("a.txt")
        self.commit_all()
        self.write("b.txt")
        self.git("add", "b.txt")
        with self.assertRaises(SystemExit):
            bs.provenance(allow_dirty=False)

    def test_allow_dirty_stamps_the_build_dirty(self):
        self.write("a.txt")
        self.commit_all()
        self.write("stray.txt")
        info = bs.provenance(allow_dirty=True)
        self.assertTrue(info["dirty"])
        self.assertEqual(info["commit"], self.git("rev-parse", "HEAD").strip())
        self.assertEqual(info["tree"], self.git("rev-parse", "HEAD^{tree}").strip())

    def test_a_clean_tree_is_stamped_clean(self):
        self.write("a.txt")
        self.commit_all()
        self.assertFalse(bs.provenance(allow_dirty=False)["dirty"])

    def test_a_repository_with_no_commit_is_refused(self):
        with self.assertRaises(SystemExit):
            bs.provenance(allow_dirty=True)

    def test_a_tracked_but_deleted_file_is_skipped(self):
        self.write("a.txt")
        self.write("gone.txt")
        self.commit_all()
        (self.repo / "gone.txt").unlink()
        self.assertEqual(bs.source_files(), ["a.txt"])

    def test_a_tracked_symlink_is_refused(self):
        self.write("a.txt")
        (self.repo / "link").symlink_to("a.txt")
        self.commit_all()
        with self.assertRaises(SystemExit):
            bs.source_files()

    def test_names_with_spaces_and_unicode_survive(self):
        self.git("config", "core.quotepath", "true")
        self.write("dir with space/naïve é.txt")
        self.commit_all()
        self.assertEqual(bs.source_files(), ["dir with space/naïve é.txt"])

    def _stage(self):
        class FakePaper:
            def record_map(self):
                return "records\n"
        saved = bs.paper_builder
        bs.paper_builder = lambda: FakePaper()
        self.addCleanup(setattr, bs, "paper_builder", saved)
        bs.set_out_dir(self.out)
        return bs.stage(bs.provenance(allow_dirty=False))

    def test_stage_writes_build_info_and_modes(self):
        self.write("a.txt")
        self.write("tool.sh", "#!/bin/sh\n", mode=0o755)
        self.commit_all()
        self._stage()
        info = json.loads((bs.STAGE / "BUILD_INFO.json").read_text())
        self.assertEqual(info["commit"], self.git("rev-parse", "HEAD").strip())
        self.assertEqual(info["tree"], self.git("rev-parse", "HEAD^{tree}").strip())
        self.assertIs(info["dirty"], False)
        self.assertRegex(info["source_sha256"], r"^[0-9a-f]{64}$")
        self.assertTrue((bs.STAGE / "tool.sh").stat().st_mode & 0o111)
        self.assertFalse((bs.STAGE / "a.txt").stat().st_mode & 0o111)

    def test_stage_never_stages_an_untracked_file(self):
        self.write("a.txt")
        self.commit_all()
        self.write("scripts/secret_notes.txt", "AKIAABCDEFGHIJKLMNOP\n")
        bs.set_out_dir(self.out)
        info = bs.provenance(allow_dirty=True)
        self._stage_with(info)
        self.assertFalse((bs.STAGE / "scripts/secret_notes.txt").exists())
        self.assertTrue(json.loads((bs.STAGE / "BUILD_INFO.json").read_text())["dirty"])

    def _stage_with(self, info):
        class FakePaper:
            def record_map(self):
                return "records\n"
        saved = bs.paper_builder
        bs.paper_builder = lambda: FakePaper()
        self.addCleanup(setattr, bs, "paper_builder", saved)
        return bs.stage(info)


class SweepNamesTest(TempRepoCase):
    """H3: an identifying token in a PATH is caught, in the stage and the zip."""

    def setUp(self):
        super().setUp()
        bs.set_out_dir(self.out)
        bs.STAGE.mkdir(parents=True)

    def test_a_file_named_for_the_author_is_a_hit_though_its_content_is_clean(self):
        (bs.STAGE / "notes_bharath_draft.txt").write_text("nothing here\n")
        hits = bs.sweep()
        self.assertEqual(len(hits), 1)
        self.assertIn("notes_bharath_draft.txt", hits[0])

    def test_a_directory_named_for_the_author_is_a_hit(self):
        (bs.STAGE / "vbcr").mkdir()
        (bs.STAGE / "vbcr" / "ok.txt").write_text("clean\n")
        self.assertTrue(any("vbcr" in h for h in bs.sweep()))

    def test_clean_names_and_contents_are_no_hit(self):
        (bs.STAGE / "a b" / "é.txt").parent.mkdir()
        (bs.STAGE / "a b" / "é.txt").write_text("clean\n")
        self.assertEqual(bs.sweep(), [])

    def test_zip_entry_names_and_contents_are_swept(self):
        (bs.STAGE / "ok.txt").write_text("clean\n")
        bs.OUT_DIR.mkdir(exist_ok=True)
        zpath = bs.OUT_DIR / "t.zip"
        with zipfile.ZipFile(zpath, "w") as zf:
            zf.writestr("fine.txt", "clean")
            zf.writestr("notes_bharath.txt", "clean")
            zf.writestr("body.txt", "written by vaddaram")
        hits = bs.sweep_archive(zpath)
        self.assertEqual(len(hits), 2)


class AwsIdentifiersTest(unittest.TestCase):
    """M1: the account id, bucket names and instance ids do not ship."""

    CASES = [
        "s3://binn-campaign-v2-511192439661-us-east-1/input/x",
        "default='binn-campaign-511192439661-us-east-1'",
        "instance `i-0ae4798ff32a1c8a7`, `c7g.8xlarge`",
        '"i-01bc8d7fa7e5c84a7",',
        "gates/i-09b29a1152da4405c.json",
    ]

    def test_each_is_redacted_and_leaves_nothing_the_sweep_finds(self):
        for case in self.CASES:
            out, n = bs.redact(case)
            self.assertGreater(n, 0, case)
            self.assertIsNone(bs.IDENTIFYING.search(out.encode()), f"{case!r} -> {out!r}")

    def test_the_sweep_catches_them_if_they_escape(self):
        for needle in (b"511192439661", b"x-511192439661-y", b"i-0ae4798ff32a1c8a7",
                       b"I-0AE4798FF32A1C8A7"):
            self.assertIsNotNone(bs.IDENTIFYING.search(needle), needle)

    def test_ordinary_numbers_and_words_are_not_hits(self):
        for text in (b"5111924396610", b"multi-0ae4798ff32a1c8a7", b"i-0ae4798ff32a1c8a70",
                     b"i-abc", b"digit 1234567890123"):
            self.assertIsNone(bs.IDENTIFYING.search(text), text)

    def test_an_instance_pseudonym_is_stable_distinct_and_in_paths_too(self):
        a1, _ = bs.redact("i-0ae4798ff32a1c8a7")
        a2, _ = bs.redact("see i-0ae4798ff32a1c8a7 again")
        b, _ = bs.redact("i-0bdb69b6270df3a4a")
        self.assertIn(a1, a2)
        self.assertNotEqual(a1, b)
        self.assertEqual(bs.redact_path("gates/i-0ae4798ff32a1c8a7.json"),
                         f"gates/{a1}.json")

    def test_the_account_placeholder_keeps_the_twelve_digit_shape(self):
        out, _ = bs.redact("arn:aws:iam::511192439661:role/x")
        self.assertEqual(out, "arn:aws:iam::123456789012:role/x")


class ArchiveModesTest(TempRepoCase):
    """L2: sensible modes, fixed timestamps, deterministic bytes."""

    def _populate(self):
        bs.set_out_dir(self.out)
        bs.STAGE.mkdir(parents=True)
        for name, mode in (("b.txt", 0o644), ("a.sh", 0o755), ("c/d.txt", 0o600)):
            path = bs.STAGE / name
            path.parent.mkdir(exist_ok=True)
            path.write_text("x\n")
            path.chmod(mode)

    def test_entries_are_0644_or_0755_with_a_fixed_date(self):
        self._populate()
        bs.archive(bs.OUT_DIR / "z.zip")
        with zipfile.ZipFile(bs.OUT_DIR / "z.zip") as zf:
            modes = {i.filename: (i.external_attr >> 16) & 0o7777 for i in zf.infolist()}
            self.assertTrue(all(i.date_time == (2026, 1, 1, 0, 0, 0) for i in zf.infolist()))
            self.assertEqual([i.filename for i in zf.infolist()],
                             sorted(modes))
        self.assertEqual(modes, {"a.sh": 0o755, "b.txt": 0o644, "c/d.txt": 0o644})

    def test_two_builds_are_byte_identical(self):
        import hashlib as h
        self._populate()
        bs.archive(bs.OUT_DIR / "1.zip")
        bs.archive(bs.OUT_DIR / "2.zip")
        self.assertEqual(h.sha256((bs.OUT_DIR / "1.zip").read_bytes()).hexdigest(),
                         h.sha256((bs.OUT_DIR / "2.zip").read_bytes()).hexdigest())


class MainOutcomeTest(TempRepoCase):
    """H4/H5: no stale archive after a failure, and `unverified` is not `passed`."""

    def setUp(self):
        super().setUp()
        self.write("a.txt")
        self.commit_all()
        bs.set_out_dir(self.out)
        self.calls = []
        for name, fn in (("relock", lambda: None),
                         ("verify", lambda: self.calls.append("verify") or self.verify_result)):
            saved = getattr(bs, name)
            setattr(bs, name, fn)
            self.addCleanup(setattr, bs, name, saved)
        self.verify_result = []

        def fake_stage(info=None):
            bs.STAGE.mkdir(parents=True, exist_ok=True)
            (bs.STAGE / "a.txt").write_text("clean\n")
            return 1, 0, 0
        saved = bs.stage
        bs.stage = fake_stage
        self.addCleanup(setattr, bs, "stage", saved)

    def run_main(self, *argv):
        import contextlib
        with contextlib.redirect_stdout(io.StringIO()), \
                contextlib.redirect_stderr(io.StringIO()):
            try:
                return bs.main(list(argv))
            except SystemExit as e:
                return e.code

    def test_verification_runs_by_default_and_a_verified_build_is_final(self):
        self.assertEqual(self.run_main(), 0)
        self.assertEqual(self.calls, ["verify"])
        self.assertTrue(bs.ARCHIVE.exists())
        self.assertFalse(bs.PARTIAL.exists())

    def test_a_build_that_skipped_verification_exits_nonzero_and_is_not_final(self):
        code = self.run_main("--no-verify")
        self.assertEqual(code, bs.EXIT_UNVERIFIED)
        self.assertNotIn(code, (0, 1))
        self.assertEqual(self.calls, [])
        self.assertFalse(bs.ARCHIVE.exists())
        self.assertTrue(bs.UNVERIFIED.exists())

    def test_a_failed_verification_leaves_no_archive_at_all(self):
        self.out.mkdir(parents=True, exist_ok=True)
        bs.ARCHIVE.write_bytes(b"stale")
        self.verify_result = ["boom"]
        self.assertEqual(self.run_main(), 1)
        self.assertFalse(bs.ARCHIVE.exists())
        self.assertFalse(bs.PARTIAL.exists())

    def test_a_refused_dirty_tree_removes_the_previous_archive(self):
        self.out.mkdir(parents=True, exist_ok=True)
        bs.ARCHIVE.write_bytes(b"stale")
        bs.UNVERIFIED.write_bytes(b"stale")
        self.write("stray.txt")
        self.assertEqual(self.run_main(), 1)
        self.assertFalse(bs.ARCHIVE.exists())
        self.assertFalse(bs.UNVERIFIED.exists())

    def test_a_surviving_identifier_fails_and_leaves_no_archive(self):
        def leaky_stage(info=None):
            bs.STAGE.mkdir(parents=True, exist_ok=True)
            (bs.STAGE / "notes_bharath_draft.txt").write_text("clean\n")
            return 1, 0, 0
        bs.stage = leaky_stage
        self.out.mkdir(parents=True, exist_ok=True)
        bs.ARCHIVE.write_bytes(b"stale")
        self.assertEqual(self.run_main(), 1)
        self.assertFalse(bs.ARCHIVE.exists())

    def test_allow_dirty_builds_from_a_dirty_tree(self):
        self.write("stray.txt")
        self.assertEqual(self.run_main("--allow-dirty"), 0)
        self.assertTrue(bs.ARCHIVE.exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
