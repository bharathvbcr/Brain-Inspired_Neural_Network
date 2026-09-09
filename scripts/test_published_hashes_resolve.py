#!/usr/bin/env python3
"""A published reproduction command must name a hash the binary can resolve.

# Why this exists

On 2026-08-25 the matched-architecture configs mixed `MATCHED_INPUT_SCALE` and
the forward graph into their hashes, because the old ones did not: the silent-
initialisation repair moved the input scale 0.5 -> 2.0 while the constant was
outside the hash, so one label named two different experiments either side of
the repair. Retiring them was the correct outcome and
`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` says so — `from_hash` returns `None`
for all four, and being told "unknown hash" is better than being handed
different numbers under the name you asked for.

What nothing checked is where those four labels still appear. On 2026-08-29
they were still the three `--config-hash` arguments in section A of
`REPRO_ARTIFACT_CHECKLIST.md`, which is the document a reviewer runs, and the
fourth was in section E. **Three of the package's headline reproduction
commands could not run**, and the checklist ticked them.

They were also still the primary values in `PAPER_METRICS_FULL.md` Table A and
in every row of `PAPER_VERIFY.md`, with no banner on either — the superseded
pre-repair block, published as current, in two documents the number sweep
deliberately does not read because they are the paper's own downstream
artefacts (`check_every_number.py`, `PAPER_SIDE`).

# What this asserts

1. The retired/current pairs written in the Rust freeze comments agree with the
   hash each freeze test actually asserts. A comment that drifts from the code
   it documents is how the next one of these starts.
2. No `--config-hash` argument anywhere in the record names a retired hash.
   A command that cannot run must not be published as a reproduction step.
3. Every appearance of a retired hash in a paper-side document is in a
   paragraph that marks it retired. Citing one as a live result is the defect;
   recording that it existed and was withdrawn is the point of keeping it.

4. Every published `--config-hash` argument names a hash its suite actually
   accepts. **(2) is a blocklist and this is the allowlist it needed.**

   Assertion (2) asks whether a published hash was *retired*. A hash that was
   never a preset was never retired either, so it passes (2) unseen — and on
   2026-09-07 eight of them did. `PAPER_SKELETON.md` §10 published
   `--matched-forward <graph> --config-hash <the hash that combination mints>`
   for all four matched suites on both graphs. `--config-hash` resolves a
   *preset*; `--matched-forward` then overrides it and mints a **new** hash,
   which is not a preset, so passing the minted hash back in fails with
   `unknown ... hash`. Every one of the eight was unrunnable.

   The block those eight lines sit in is *itself* the repair of the 2026-08-25
   round of this defect, and opens with "Those commands could not run as
   written." The repair reintroduced the same class through a different door,
   which is the argument for an allowlist: (2) can only catch a hash somebody
   already knew was dead.

   The unit is the paragraph and not the line, because markdown prose wraps: a
   line-scoped version of this test reported `PAPER_SKELETON.md` §9.2, which
   names the three hashes on one line and calls them "deliberately retired" on
   the next. A check that pushes authors to cram a marker word onto whichever
   line a hash happened to land on is measuring the wrapping.

Run: python3 scripts/test_published_hashes_resolve.py
"""

from __future__ import annotations

import re
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.check_every_number import PAPER_SIDE  # noqa: E402

#: The four suites whose hashes moved on 2026-08-25. Named rather than globbed:
#: there are four, `RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` §8 lists exactly
#: four, and a glob that silently found three would pass every test below.
CONFIGS = (
    "match_config.rs",
    "dfa_match_config.rs",
    "rl_match_config.rs",
    "eventprop_match_config.rs",
)

#: Words that mark a hash as no longer live. Any one of them on the same line
#: as the hash discharges it. `pre-repair` is here because that is the phrase
#: `PUBLISHABLE_CLAIMS.md` already uses for exactly this.
DISCHARGED = (
    "retired", "archived", "superseded", "withdrawn", "pre-repair",
    "not citable", "no longer resolve", "does not resolve", "stale",
)

#: Documents that record the retirement itself, and so quote the hashes as
#: their subject rather than citing them. Each is named with its reason; a new
#: one has to be added here deliberately.
RETIREMENT_RECORDS = {
    # §8 is the table that retired them, and its rows are `retired | current`
    # with the words in the header rather than on every line.
    "RESULT_2026-08-25_MATCHED_ARCH_RERUN.md": "the document that retired them",
    "PREREG_2026-08-25_MATCHED_ARCH_RERUN_ON_BOTH_FORWARDS.md":
        "the preregistration that predicted the break",
}


def freeze_blocks() -> dict[str, tuple[tuple[str, ...], str, str]]:
    """`config file -> (every retired hash, current hash from the comment, frozen)`.

    **`findall`, not `search`.** This read one `retired:` line per file, and a
    family has more than one preset: `--matched-arch` alone has four, and the
    2026-08-25 change mixed `MATCHED_INPUT_SCALE` and the forward graph into
    the hash of *every* `MatchConfig`, so all four moved. Holding one meant the
    blocklist below could never have banned more than a quarter of them, and
    `c1-match-b46b23549b37d90a` -- the pre-repair `ep4` label -- stood in the
    reviewer's checklist and in three paper-side documents until 2026-09-07
    while every test in this file passed.

    A parser shaped so it *cannot* hold the whole answer is the same defect as
    a check that cannot fail. `RetiredSetIsCompleteTest` is what keeps the
    comments honest now: it derives the set from the record and the binary
    rather than trusting that somebody remembered to add a line.
    """
    out = {}
    for name in CONFIGS:
        text = (ROOT / "binn-lab/src" / name).read_text()
        retired = re.findall(r"//\s+retired:\s+(c1-[\w-]+)", text)
        current = re.search(r"//\s+current:\s+(c1-[\w-]+)", text)
        frozen = re.search(r'assert_eq!\(hash,\s*"(c1-[\w-]+)"\)', text)
        assert retired and current and frozen, name
        out[name] = (tuple(retired), current.group(1), frozen.group(1))
    return out


def retired_hashes() -> set[str]:
    return {h for retired, _, _ in freeze_blocks().values() for h in retired}


#: Directories that hold no published claim: build output, tooling caches, and
#: version control. Everything else is in scope.
NOT_THE_RECORD = {".git", "target", "node_modules", ".venv", "__pycache__",
                  ".devcouncil", ".pytest_cache"}


def record_documents() -> list[Path]:
    """Every markdown file in the repository, at any depth.

    This swept `results/` only, and the repository's front door is `README.md`.
    On 2026-09-07 that file was still publishing three `--config-hash` replay
    commands naming hashes retired on 2026-08-25, beside the pre-repair 0.9387
    and 0.9200 the rerun withdrew -- the exact defect this file was written to
    catch, in the one document a visitor reads first, one directory outside the
    only directory it looked at.

    A scope drawn around where the problem was found last time will keep missing
    it. So: everything tracked, minus build output and caches. The per-test
    exclusions still apply -- `runs/` is history and is skipped below, and
    `RETIREMENT_RECORDS` names the documents whose subject IS the retirement.
    """
    return sorted(
        path for path in ROOT.rglob("*.md")
        if not NOT_THE_RECORD & set(path.relative_to(ROOT).parts)
    )


class FreezeCommentsTest(unittest.TestCase):
    """The comment and the assertion beside it must say the same thing."""

    def test_all_four_suites_were_parsed(self):
        """A parse that found three would leave one retired hash unbanned."""
        self.assertEqual(len(freeze_blocks()), 4)
        self.assertEqual(
            {name: len(r) for name, (r, _, _) in freeze_blocks().items()},
            {"match_config.rs": 4, "dfa_match_config.rs": 2,
             "rl_match_config.rs": 4, "eventprop_match_config.rs": 2},
            "the per-file retired counts moved; a family retires all of its "
            "presets at once, so a drop here means a comment lost a line")
        self.assertEqual(len(retired_hashes()), 12)

    def test_the_comment_names_the_hash_the_test_freezes(self):
        for name, (_, current, frozen) in freeze_blocks().items():
            with self.subTest(config=name):
                self.assertEqual(
                    current, frozen,
                    f"{name}: the freeze comment says the current hash is "
                    f"{current} but the test asserts {frozen}")

    def test_no_retired_hash_is_also_a_current_one(self):
        current = {c for _, c, _ in freeze_blocks().values()}
        self.assertEqual(retired_hashes() & current, set())

    def test_each_retired_hash_belongs_to_the_family_it_is_filed_under(self):
        """A `c1-rl-` value filed in `match_config.rs` would ban the right hash
        for the wrong reason, and the next reader would trust the wrong file."""
        for name, (retired, current, _) in freeze_blocks().items():
            prefix = current.rsplit("-", 1)[0] + "-"
            for hash_ in retired:
                with self.subTest(config=name, hash=hash_):
                    self.assertTrue(hash_.startswith(prefix),
                                    f"{name} records {hash_}, which is not a "
                                    f"{prefix}* hash")


class ReproductionCommandsTest(unittest.TestCase):
    """`--config-hash` names a hash `from_hash` must still resolve."""

    @classmethod
    def setUpClass(cls):
        cls.retired = retired_hashes()

    def test_the_scan_finds_config_hash_arguments(self):
        """A regex matching nothing would pass the test below forever."""
        found = [h for doc in record_documents()
                 for h in re.findall(r"--config-hash\s+(c1[\w-]+)", doc.read_text())]
        self.assertGreater(len(found), 5, found)

    def test_the_repository_front_door_is_in_scope(self):
        """`README.md` is one directory outside `results/` and is the document
        most likely to be read and least likely to be re-checked. It published
        three retired hashes for thirteen days while every test here passed."""
        docs = record_documents()
        self.assertIn(ROOT / "README.md", docs)
        self.assertTrue(
            re.search(r"--config-hash\s+(c1[\w-]+)", (ROOT / "README.md").read_text()),
            "README.md publishes no replay command; this test now checks nothing")

    def test_the_scan_reaches_outside_results(self):
        """The whole point of the widening. A scope that silently narrows back
        to `results/` would make the front door unguarded again."""
        outside = [d for d in record_documents()
                   if d.relative_to(ROOT).parts[0] != "results"]
        self.assertGreater(len(outside), 10, len(outside))

    def test_no_published_command_names_a_retired_hash(self):
        offences = []
        for doc in record_documents():
            # A run record is a log of what ran, under the hash it ran under.
            # Rewriting one would be falsifying the record; they are history.
            if "runs/" in doc.as_posix():
                continue
            for hash_ in re.findall(r"--config-hash\s+(c1[\w-]+)", doc.read_text()):
                if hash_ in self.retired:
                    offences.append(f"{doc.relative_to(ROOT)}: {hash_}")
        self.assertEqual(offences, [], "\n".join(
            ["a published reproduction command names a hash `from_hash` no "
             "longer resolves; the command cannot run:"] + offences))


#: hash family -> the flags that select the suite owning it. The base `c1-`
#: family takes no suite flag. Every suite prints its own preset list when
#: handed an unknown hash, in one format, which is what makes this checkable
#: without duplicating `known_presets()` in Python -- a duplicate would drift,
#: and a drifting allowlist is worse than none.
SUITE_OF = (
    ("c1-match-", ["--matched-arch"]),
    ("c1-dfa-", ["--matched-dfa"]),
    ("c1-rl-", ["--matched-rl"]),
    ("c1-eventprop-", ["--eventprop"]),
    ("c1-mech-", ["--mech"]),
    ("c1-shd-cal-", ["--shd-cal"]),
    ("c1-shd-full-", ["--shd-full"]),
)

#: A real hash, not a `c1-<hex>` placeholder in prose. Sixteen hex digits, which
#: every minted hash carries and no placeholder does.
PUBLISHED_HASH = re.compile(r"--config-hash\s+(c1[\w-]*-[0-9a-f]{16})\b")


def c1_binary() -> Path:
    """The built `c1`, or a hard failure.

    Never a skip. A check that could not run must not report what a check that
    ran and passed reports -- that is the whole subject of this file.
    """
    for candidate in (ROOT / "target/release/c1", ROOT / "target/debug/c1"):
        if candidate.is_file():
            return candidate
    built = subprocess.run(
        ["cargo", "build", "--locked", "--release", "-p", "binn-lab", "--bin", "c1"],
        cwd=ROOT, capture_output=True, text=True)
    if built.returncode != 0 or not (ROOT / "target/release/c1").is_file():
        raise AssertionError(
            "the `c1` binary is the authority on which hashes resolve and it "
            f"could not be built, so this check cannot run:\n{built.stderr[-2000:]}")
    return ROOT / "target/release/c1"


def presets_of(binary: Path, flags: list[str]) -> set[str]:
    """Every hash a suite's `from_hash` accepts, read from the binary itself.

    An unknown hash makes each suite print its presets and exit immediately, so
    this costs nothing and cannot start a run.
    """
    proc = subprocess.run([str(binary), *flags, "--config-hash", "__probe__"],
                          cwd=ROOT, capture_output=True, text=True, timeout=120)
    text = proc.stdout + proc.stderr
    found = set(re.findall(r"^\s+(c1[\w-]*-[0-9a-f]{16})\s+\(", text, re.M))
    assert found, (f"probing {flags or ['<base>']} listed no presets; the "
                   f"output format this parses has changed:\n{text[:800]}")
    return found


class PublishedHashIsAPresetTest(unittest.TestCase):
    """Assertion 4. Every published `--config-hash` is one its suite accepts."""

    @classmethod
    def setUpClass(cls):
        cls.binary = c1_binary()
        cls.presets = {prefix: presets_of(cls.binary, flags)
                       for prefix, flags in SUITE_OF}
        cls.presets["c1-"] = presets_of(cls.binary, [])

    def suite_for(self, hash_: str) -> str:
        for prefix, _ in SUITE_OF:
            if hash_.startswith(prefix):
                return prefix
        return "c1-"

    def published(self):
        for doc in record_documents():
            # A run record logs what ran, under the hash it ran under. Rewriting
            # one would be falsifying history; the same exemption as (2).
            if "runs/" in doc.as_posix():
                continue
            for hash_ in PUBLISHED_HASH.findall(doc.read_text()):
                yield doc, hash_

    def test_every_suite_reported_some_presets(self):
        """A probe that silently listed nothing would make every hash pass."""
        for prefix, found in self.presets.items():
            self.assertGreater(len(found), 0, f"{prefix} listed no presets")
        self.assertGreater(sum(len(v) for v in self.presets.values()), 15)

    def test_the_scan_finds_real_hashes(self):
        found = [h for _, h in self.published()]
        self.assertGreater(len(found), 5, found)

    def test_the_pattern_takes_hashes_and_leaves_placeholders(self):
        """Asserted against the pattern, not against the corpus.

        The first version of this checked that no scanned value equalled `c1-`,
        and it could not fail: every `c1-<hex>` placeholder in this repository
        lives in `binn-lab/experiments/c1.rs`, and `record_documents()` sweeps
        markdown. Widening the pattern to swallow placeholders left the corpus
        version green -- verified by doing it. A check that cannot fail is the
        subject of this file, so it is asserted where it can."""
        self.assertEqual(
            PUBLISHED_HASH.findall("--config-hash c1-match-6f6000f148f7d30c"),
            ["c1-match-6f6000f148f7d30c"])
        self.assertEqual(
            PUBLISHED_HASH.findall("--config-hash c1-118207fbc3eaba53"),
            ["c1-118207fbc3eaba53"])
        for placeholder in ("--config-hash c1-<hex>",
                            "--config-hash c1-match-<hex>",
                            "--config-hash c1-shd-cal-<hex>"):
            self.assertEqual(
                PUBLISHED_HASH.findall(placeholder), [],
                f"{placeholder!r} is documentation, not a runnable command; "
                f"scanning it as a hash would report a defect that is not one")

    def test_every_published_config_hash_is_a_preset_of_its_suite(self):
        offences = []
        for doc, hash_ in self.published():
            prefix = self.suite_for(hash_)
            if hash_ not in self.presets[prefix]:
                offences.append(f"{doc.relative_to(ROOT)}: {hash_} "
                                f"(not a preset of {prefix or 'the base suite'})")
        self.assertEqual(offences, [], "\n".join(
            ["a published reproduction command passes --config-hash a value its "
             "suite does not accept, so the command exits with `unknown hash`. "
             "A hash minted by an override (--matched-forward, --max-lag) is "
             "NOT a preset: reproduce with the flags and drop --config-hash."]
            + offences))


#: The `--matched-forward` reruns, and the only path exempted from the
#: completeness check below. Their `config hash:` headers name values that were
#: never presets and were never retired either: the flag overrides the preset
#: and mints a new hash, so `from_hash` cannot resolve one -- and each still
#: reproduces today from its flags. Naming the directory, and asserting the
#: exemption is load-bearing, keeps this an exemption for a stated reason rather
#: than a list of eight hashes somebody would extend the next time one appeared.
OVERRIDE_MINTED = "results/matched_rerun_2026-08-25/"

#: The line every C1 report writes to say which config produced it. This is the
#: evidence the retired set is derived from: a hash that was a report's header
#: and is not a preset today was a preset once and is not one now.
CONFIG_HASH_HEADER = re.compile(
    r"config hash:\s*`?(c1-(?:match|dfa|rl|eventprop)-[0-9a-f]{16})`?")


class RetiredSetIsCompleteTest(unittest.TestCase):
    """The blocklist is derived from the record, not from somebody's memory.

    §4 of `DEFECT_2026-09-07_A_BLOCKLIST_WHERE_AN_ALLOWLIST_WAS_NEEDED.md` found
    that one retirement of four had been recorded, and §5 left the consequence
    open: the allowlist (assertion 4) reads `--config-hash` *arguments*, so a
    stale preset hash sitting in a **table** -- `c1-match-b46b23549b37d90a` as
    the label of "Break-it v22 undertrain FAIL" -- is not a command and nothing
    checked it. Assertion (3) would have, but only for hashes on the blocklist,
    and those were exactly the ones it never held.

    Enumerating the missing hashes by hand would close it once and leave the
    same hole open for the next retirement. So the set is derived instead:

        every `config hash:` header of the four families, anywhere in the
        record, names either a hash its suite still accepts or a hash the
        suite's freeze comment records as retired.

    A preset that moves and is not recorded fails this the moment its report is
    written, which is the point -- it is the freeze comments that go stale, and
    this is what notices.
    """

    @classmethod
    def setUpClass(cls):
        cls.binary = c1_binary()
        cls.presets = {prefix: presets_of(cls.binary, flags)
                       for prefix, flags in SUITE_OF
                       if prefix in {"c1-match-", "c1-dfa-", "c1-rl-",
                                     "c1-eventprop-"}}
        cls.retired = retired_hashes()

    def headers(self):
        for doc in record_documents():
            for hash_ in CONFIG_HASH_HEADER.findall(doc.read_text()):
                yield doc, hash_

    @staticmethod
    def family(hash_: str) -> str:
        return hash_.rsplit("-", 1)[0] + "-"

    def test_the_header_scan_finds_reports_to_check(self):
        """A regex that matched nothing would pass the assertion below forever,
        which is the failure mode this whole file exists to describe."""
        found = {h for _, h in self.headers()}
        self.assertGreater(len(found), 15, sorted(found))
        self.assertEqual(
            {self.family(h) for h in found},
            {"c1-match-", "c1-dfa-", "c1-rl-", "c1-eventprop-"},
            "a family stopped appearing; the scan has narrowed")

    def test_the_exemption_is_load_bearing_and_narrow(self):
        """It must actually exempt something, and must not be a blanket hole.

        Half that directory is the *recurrent* rerun, whose hash IS the preset,
        so an exemption covering it entirely would be hiding live values behind
        a rule written for the overridden ones."""
        inside = [(d, h) for d, h in self.headers()
                  if OVERRIDE_MINTED in d.as_posix()]
        self.assertTrue(inside, f"{OVERRIDE_MINTED} holds no config-hash "
                                "header; the exemption below exempts nothing")
        minted = [h for _, h in inside if h not in self.presets[self.family(h)]]
        presets = [h for _, h in inside if h in self.presets[self.family(h)]]
        self.assertTrue(minted, "nothing in the exempted directory is override-"
                                "minted, so the exemption is unnecessary")
        self.assertTrue(presets, "every hash in the exempted directory is "
                                 "unresolvable; it is not the rerun archive "
                                 "this exemption was written for")
        self.assertEqual(
            set(minted) & self.retired, set(),
            "an override-minted hash is also recorded as retired; one of the "
            "two is wrong -- a value that was never a preset was never retired")

    def test_every_published_config_hash_is_a_preset_or_a_recorded_retirement(self):
        offences = []
        for doc, hash_ in self.headers():
            if OVERRIDE_MINTED in doc.as_posix():
                continue
            if hash_ in self.presets[self.family(hash_)]:
                continue
            if hash_ in self.retired:
                continue
            offences.append(f"{doc.relative_to(ROOT)}: {hash_}")
        self.assertEqual(sorted(set(offences)), [], "\n".join(
            ["a report was produced under a config hash that its suite no "
             "longer accepts, and no freeze comment records the retirement. "
             "Add a `// retired: <hash>  (<preset> -- <report>)` line to that "
             "suite's config so the blocklist below can see it:"]
            + sorted(set(offences))))


class PaperSideCitationsTest(unittest.TestCase):
    """A retired hash in a paper document must be marked as retired."""

    @classmethod
    def setUpClass(cls):
        cls.retired = retired_hashes()

    def test_the_paper_side_list_is_the_one_the_sweep_uses(self):
        """One owner. If `check_every_number.py` narrows its list, this narrows
        with it rather than keeping a second copy that quietly disagrees."""
        self.assertIn("PAPER_METRICS_FULL.md", PAPER_SIDE)
        self.assertIn("PAPER_VERIFY.md", PAPER_SIDE)
        self.assertGreaterEqual(len(PAPER_SIDE), 10)

    def test_the_scan_finds_retired_hashes_to_check(self):
        """Negative control: if no paper-side document mentioned one, the test
        below would pass while checking nothing."""
        hits = sum(
            doc.read_text().count(h)
            for doc in record_documents() if doc.name in PAPER_SIDE
            for h in self.retired
        )
        self.assertGreater(hits, 0, "no retired hash appears in any paper-side "
                                    "document; the check below is vacuous")

    def test_every_retired_hash_is_marked_where_a_paper_cites_it(self):
        offences = []
        for doc in record_documents():
            if doc.name not in PAPER_SIDE or doc.name in RETIREMENT_RECORDS:
                continue
            line_no = 1
            for para in doc.read_text().split("\n\n"):
                lowered = para.lower()
                for hash_ in sorted(self.retired):
                    if hash_ in para and not any(w in lowered for w in DISCHARGED):
                        offences.append(f"{doc.relative_to(ROOT)}:{line_no}: {hash_}")
                line_no += para.count("\n") + 2
        self.assertEqual(offences, [], "\n".join(
            ["a paper-side document cites a retired config hash without saying "
             "it is retired:"] + offences))


if __name__ == "__main__":
    unittest.main(verbosity=2)
