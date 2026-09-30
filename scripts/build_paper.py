#!/usr/bin/env python3
"""Build the anonymous TMLR submission, and its named preprint, from `results/PAPER_DRAFT.md`.

The manuscript's single source of truth is the markdown. Hand-maintaining a
parallel `.tex` would put two documents in the repository that are supposed to
say the same thing, and this repository has already had that failure twice --
once with a venue note describing a paper that no longer existed, and once with
a figure drawing a superseded value block at camera-ready quality. So the
`.tex` is generated, every time, and is not edited.

What this refuses to do, and why each refusal exists:

  * Build with a MODIFIED style file. Every file of the official TMLR style
    is pinned by hash (see `paper/STYLE_PROVENANCE.md`): the NeurIPS episode
    showed that the most findable copy of a template can be an edited one.
  * Build with a section in NEITHER half of the split manifest. A section that
    silently vanishes from a nine-page paper is invisible in the output; the
    build stops instead.
  * Report success on a paper whose main text runs past TMLR's twelve pages.
    TMLR has no hard limit, but above twelve it stops promising its review
    timeline, so a thirteenth page is a cost nobody chose.
  * Report success on a submission missing either of TMLR's first-page
    requirements: a ONE-paragraph abstract, and a footnote on the first page
    disclosing LLM use.
  * Report success on a PDF that DE-ANONYMISES the submission. Checked against
    the rendered text and the PDF metadata both, because the two fail
    separately -- a clean manuscript can still ship an author name in
    /Author, and pdflatex writes that from the environment without being asked.

Usage:
    python3 scripts/build_paper.py                      # build and check the submission
    python3 scripts/build_paper.py --check              # check a build already on disk
    python3 scripts/build_paper.py --variant preprint   # the named arXiv version

The preprint is the same source under TMLR's own `[preprint]` option, which
names the author and removes every mention of TMLR. It builds into its own
directory, so the anonymous build on disk can never be the named one.
"""
from __future__ import annotations

import argparse
import hashlib
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
PAPER = ROOT / "paper"
DRAFT = ROOT / "results/PAPER_DRAFT.md"
BIB = ROOT / "results/references.bib"
MANIFEST = PAPER / "split_manifest.toml"
BUILD = PAPER / "build"
PREPRINT_BUILD = PAPER / "build-preprint"
FIGDIR = ROOT / "results/runs/2026-07-23-paper-hard-both/figures"

#: The official TMLR style, byte for byte, from the archive TMLR's author guide
#: links (github.com/JmlrOrg/tmlr-style-file, commit 7bf90efe). All three files
#: ship, `fancyhdr.sty` included, so the build does not depend on whichever
#: copy the local TeX installation carries. `paper/STYLE_PROVENANCE.md` records
#: how each hash was obtained.
STYLE_FILES = {
    "tmlr.sty": "816214ff5919aa457b6b443bee52b15d9561421417b7f8a50cc84651519f0002",
    "tmlr.bst": "306fd454cf40771bee01293eeb98d2c1cd5f4e11ed0cd7296b335f354fc45206",
    "fancyhdr.sty": "3d2922548e0e5f1a6c5676eda6ebb6dc20d7d305b4d8c2be5f1c833fb1084e6d",
}

#: TMLR has no page limit. Its author guide says a submission with more than
#: twelve pages of main content loses the normal review timeline, so twelve is
#: what this build holds the main text to. References and appendices do not
#: count, which is what the `\appendix` marker in the generated source
#: separates.
PAGE_LIMIT = 12

#: `submission` is anonymous and goes to OpenReview; `preprint` names the author
#: and goes to arXiv. The style option and the author block are the only
#: differences, so no manuscript text changes between them.
VARIANTS = {
    "submission": {"build": BUILD, "option": "", "author": ""},
    "preprint": {
        "build": PREPRINT_BUILD,
        "option": "[preprint]",
        "author": (r"\name Bharath Chandra Vaddaram \email bharath.vbcr@gmail.com \\"
                   "\n" r"\addr Independent Researcher"),
    },
}

#: What TMLR's style prints on page 1 in anonymous mode, and only then.
ANONYMOUS_BANNER = "Paper under double-blind review"

#: Anything that would identify the authors in a double-blind submission. Each
#: is checked against the EXTRACTED PDF TEXT, not the source, because that is
#: what a reviewer sees; and against the PDF's metadata separately.
IDENTIFYING = [
    ("bharath", "the author's name"),
    ("vbcr", "the author's email local part"),
    ("gmail", "a personal email domain"),
    ("/Users/", "an absolute home path leaking a username"),
]

#: Strings that exist verbatim in the author's public repository, so a code
#: search on any one resolves to the author: dated record filenames, paths
#: under `results/`, and configuration hashes. Matched case-insensitively
#: against the extracted text of the SUBMISSION only; the preprint names the
#: author and may carry them. Added 2026-09-29, after an audit found thirty.
REPO_IDENTIFIERS = re.compile(
    r"\b(?:result|finding|amendment|defect|audit|citations|prereg\w*)_\d{4}-\d{2}-\d{2}"
    r"|\bresults/|\bc1-[0-9a-f]{8,}", re.I)

#: Wording that says how many authors there are. The paper writes "we", which
#: does not; "the author" on page 1 of a double-blind submission says one, and
#: an unaffiliated solo author is a small search. SUBMISSION only, like the
#: identifiers above. Added 2026-09-29, when the LLM footnote said it.
AUTHOR_COUNT = re.compile(r"\b(?:the|sole|single|lone)\s+author\b", re.I)

#: `PAPER_FIGURE_SPEC.md` is the owner of what every figure may and may not
#: claim, and it already carries a `**Caption (required wording):**` block for
#: each one. So the captions are READ FROM IT rather than written here: a
#: caption maintained in two places is a caption that will disagree with
#: itself, and this repository has already shipped one figure drawing a
#: superseded value block at camera-ready quality. A figure the spec gives no
#: required wording for is not typeset, and the build says which.
SPEC = ROOT / "results/PAPER_FIGURE_SPEC.md"
CAPTION_BLOCK = re.compile(
    r"\*\*Caption \(required wording\):\*\*\s*\n?\u201c(.*?)\u201d", re.S)
ARTWORK = re.compile(r"\*\*Artwork target:\*\* `[^`]*figures/([A-Za-z0-9_]+)\.")

#: Which half each spec figure belongs to. The lead programme's five figures
#: carry the main text; the matched programme's two follow its sections into
#: the appendix. A figure the spec captions but this does not place is an
#: error, the same way an unplaced section is.
#: Sections whose tables are set a size smaller than the rest: Appendix K's
#: ten columns carry three bracketed intervals each, which wrap mid-bracket
#: at the \small every other table uses.
FOOTNOTESIZE_TABLES = {"Appendix K — Every operating point, with intervals"}

FIGURE_PLACEMENT = {
    "leadfig1_the_conditional": "main",
    "leadfig2_headline_accuracy": "main",
    # Both followed their sections into the appendix on 2026-09-07: the width
    # ladder to Appendix E and the resolution ladder to Appendix I. A figure
    # placed away from the text that reads it is worse than no figure.
    "leadfig3_width_ladder": "appendix",
    "leadfig4_resolution_ladder": "appendix",
    # Followed §3.7 into the appendix on 2026-09-29, when the main text was
    # rewritten around the SHD claim and the substrate question became support.
    "figS_substrate": "appendix",
    # The matched programme's two figures left with its sections on 2026-09-29
    # ([omitted] in the manifest). "omitted" is a placement, not an absence:
    # they are still captioned by the spec, and still drawn, just not typeset.
    "figM_mechanism_richness_addressability": "omitted",
    "fig2_matched_means": "omitted",
}


def spec_captions() -> list[tuple[str, str]]:
    """(artwork stem, required caption) in spec order.

    Each caption block is attributed to the artwork target that follows it.
    If a block has no target the build stops: an unattributed caption would
    otherwise be dropped, and a figure would ship with no caption at all.
    """
    text = SPEC.read_text()
    out: list[tuple[str, str]] = []
    for match in CAPTION_BLOCK.finditer(text):
        target = ARTWORK.search(text, match.end())
        if target is None:
            fail(f"a required caption in {SPEC.name} (at offset {match.start()}) "
                 "is followed by no `**Artwork target:**` line, so the build "
                 "cannot tell which figure it captions.")
        out.append((target.group(1), " ".join(match.group(1).split())))
    if not out:
        fail(f"{SPEC.name} yielded no required captions. The spec owns the "
             "caption text; an empty parse would silently produce a paper of "
             "uncaptioned figures.")
    unplaced = [stem for stem, _ in out if stem not in FIGURE_PLACEMENT]
    if unplaced:
        fail("the figure spec requires captions for figures this build does "
             "not place:\n" + "\n".join(f"  - {s}" for s in unplaced)
             + "\nAdd each to FIGURE_PLACEMENT as 'main', 'appendix' or 'omitted'.")
    odd = sorted(s for s, half in FIGURE_PLACEMENT.items()
                 if half not in ("main", "appendix", "omitted"))
    if odd:
        fail("FIGURE_PLACEMENT has a placement that is not main, appendix or "
             "omitted:\n" + "\n".join(f"  - {s}" for s in odd))
    missing = sorted(set(FIGURE_PLACEMENT) - {stem for stem, _ in out})
    if missing:
        fail("FIGURE_PLACEMENT names figures the spec no longer captions:\n"
             + "\n".join(f"  - {s}" for s in missing))
    return out


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def verify_style(where: pathlib.Path = PAPER) -> None:
    """Every style file in `where` is the pinned one. Run on `paper/` before a
    build and on the build directory after, because the copy pdflatex reads
    is the one that matters."""
    for name, want in STYLE_FILES.items():
        path = where / name
        if not path.exists():
            fail(f"{path} is missing. See paper/STYLE_PROVENANCE.md for how to "
                 "obtain it; do not substitute a template found by search.")
        got = hashlib.sha256(path.read_bytes()).hexdigest()
        if got != want:
            fail(f"{path} is not the verified file.\n"
                 f"  expected {want}\n  got      {got}\n"
                 "If the style was deliberately updated, fetch it from the "
                 "archive TMLR's author guide links, record where it came from "
                 "in paper/STYLE_PROVENANCE.md, and update STYLE_FILES here.")


def split_sections(text: str) -> list[tuple[str, str]]:
    """(heading, body) for every `##`/`###` heading, in document order."""
    out: list[tuple[str, str]] = []
    parts = re.split(r"^(#{2,3}) (.+)$", text, flags=re.M)
    # parts[0] is the front matter before the first heading.
    for i in range(1, len(parts), 3):
        out.append((parts[i + 1].strip(), parts[i + 2]))
    return out


RECORD_LINK = re.compile(r"\[`?([^\]`]+?)`?\]\(([A-Za-z0-9_./-]+\.md)\)")


def rewrite_record_links(body: str, register: dict[str, int]) -> str:
    """Turn `[NAME](NAME.md)` into a numbered marker.

    The markdown cites its own record by filename, inline, which is right for
    a document read in a repository and ruinous in a nine-page paper: the
    names run to sixty characters and there are hundreds of occurrences. Each
    distinct record gets one number, printed once in the appendix. Nothing is
    dropped -- a provenance pointer that vanished in typesetting would be the
    same defect as a citation that was never checked.
    """
    def sub(match: re.Match) -> str:
        target = match.group(2).rsplit("/", 1)[-1]
        if target not in register:
            register[target] = len(register) + 1
        return rf"\rec{{{register[target]}}}"
    return RECORD_LINK.sub(sub, body)


def record_register(manifest: dict, sections: list[tuple[str, str]]) -> dict[str, int]:
    """{record filename: marker number}, in the order the build meets them.

    The build numbers records as it renders (main text, appendix, then the
    abstract and the disclosure); this walks the same order without
    rendering, so the supplement's marker map and the paper's markers cannot
    disagree. `build` fails if rendering registers anything this did not."""
    bodies = dict(sections)
    register: dict[str, int] = {}
    for heading in list(manifest["main"]["sections"]) + list(manifest["appendix"]["sections"]):
        rewrite_record_links(bodies[heading], register)
    for body in front_matter(sections, manifest):
        rewrite_record_links(body, register)
    return register


def record_map() -> str:
    """`RECORDS.md` for the anonymous supplement: marker to record path.

    The submission prints markers only, because every record filename exists
    verbatim in the author's public repository and a code search on one
    resolves to the author. The map travels in the supplement instead."""
    manifest = tomllib.loads(MANIFEST.read_text())
    register = record_register(manifest, split_sections(DRAFT.read_text()))
    rows = []
    for name, n in sorted(register.items(), key=lambda kv: kv[1]):
        if not (ROOT / "results" / name).exists():
            fail(f"record marker r{n} points at results/{name}, which does not exist.")
        rows.append(f"| r{n} | `results/{name}` |")
    corpora = []
    for label, path in CORPORA:
        if not (ROOT / path).exists():
            fail(f"the corpus {label!r} is listed at {path}, which does not exist.")
        corpora.append(f"| {label} | `{path}` |")
    return ("# Record markers\n\n"
            "The paper cites its machine-written records by marker (r1, r2, ...).\n"
            "This table maps each marker to its file in this archive.\n\n"
            "| marker | record |\n|---|---|\n" + "\n".join(rows) + "\n\n"
            "## Run corpora\n\n"
            "The paper's reproducibility section names five corpora by content; their paths:\n\n"
            "| corpus | path |\n|---|---|\n" + "\n".join(corpora) + "\n")


#: The five run corpora the paper's reproducibility section names by content.
CORPORA = [
    ("waves 1-7", "results/shd_attention_campaign_v1/cells"),
    ("waves 8-25", "results/shd_attention_campaign_v2"),
    ("x86-64 scope runs", "results/azure-d32l4-scope-v1/results"),
    ("waves 26-28", "results/shd_attention_campaign_v3"),
    ("wave 29 (Apple silicon)", "results/shd_attention_wave29_local"),
]


def repo_identifier_problems(text: str) -> list[str]:
    """One problem per distinct repository identifier in `text` (capped at
    ten named, with the total), so a leak is named rather than counted."""
    found = sorted({m.group(0).lower() for m in REPO_IDENTIFIERS.finditer(text)})
    if not found:
        return []
    shown = ", ".join(repr(f) for f in found[:10])
    return [f"the rendered PDF carries {len(found)} string(s) that exist verbatim "
            f"in the public repository ({shown}{', ...' if len(found) > 10 else ''}); "
            "a code search on any resolves to the author."]


def author_count_problems(text: str) -> list[str]:
    """One problem per distinct phrase in `text` that counts the authors."""
    found = sorted({" ".join(m.group(0).lower().split())
                    for m in AUTHOR_COUNT.finditer(text)})
    if not found:
        return []
    return [f"the rendered PDF says {', '.join(repr(f) for f in found)}, which "
            "tells a reviewer how many authors there are; write \"we\"."]


def record_section(variant: str, register: dict[str, int], record_rows: str) -> str:
    """The record-reference appendix. The preprint lists every filename; the
    submission lists none, because each exists verbatim in the author's public
    repository, and points at the supplement's RECORDS.md instead."""
    if variant == "submission":
        return "\n".join([
            r"\section*{Record references}",
            rf"Provenance markers r1--r{len(register)} in the text point to "
            r"machine-written records in the anonymised supplement, whose "
            r"\texttt{RECORDS.md} maps each marker to its file.",
        ])
    return "\n".join([
        r"\section*{Record references}",
        r"Every provenance pointer in the text resolves to a machine-written "
        r"record in the accompanying repository, listed here rather than inline.",
        r"\begin{longtable}{@{}ll@{}}\toprule",
        record_rows,
        r"\bottomrule\end{longtable}",
    ])


def to_latex(markdown: str) -> str:
    proc = subprocess.run(
        # --no-highlight: a fenced block tagged with a language makes pandoc
        # emit `Shaded`/`Highlighting`, which need pandoc's own colour macros in
        # the preamble. Without them the build dies at `\begin{Shaded}` with no
        # PDF. Untagged fences are unaffected, so this turns a whole class of
        # "someone wrote ```bash" build failures into plain verbatim.
        # --natbib: emit `\citep{}`/`\citet{}` for `[@key]`/`@key` rather than
        # escaping them into the text. The style file loads natbib already, and
        # `\bibliography{references}` was being emitted with nothing citing it,
        # so bibtex used 0 of 18 entries and the reference list did not exist.
        # The draft contains no other `@`, so nothing pre-existing can be
        # reinterpreted as a citation key by turning this on.
        ["pandoc", "-f", "markdown+pipe_tables", "-t", "latex",
         "--top-level-division=section", "--wrap=preserve", "--no-highlight",
         "--natbib"],
        input=markdown, capture_output=True, text=True)
    if proc.returncode != 0:
        fail(f"pandoc failed: {proc.stderr.strip()}")
    return proc.stdout


#: The spec's figure identities ("Figure 1", "Figure S", "Figure M") are the
#: names the prose uses; LaTeX numbers the typeset figures in float order. So a
#: prose reference is rewritten to `\ref` its stem, and a reference to a spec
#: figure that is NOT typeset stops the build -- it would point a reader at a
#: figure the paper does not contain.
SPEC_FIGURE_ID = {
    "1": "leadfig1_the_conditional",
    "2": "leadfig2_headline_accuracy",
    "3": "leadfig3_width_ladder",
    "4": "leadfig4_resolution_ladder",
    "S": "figS_substrate",
}

TOP_LEVEL = re.compile(r"^(\d+)\.\s")
SUB_LEVEL = re.compile(r"^(\d+\.\d+)\s")
APPENDIX_HEAD = re.compile(r"^Appendix ([A-Z])\b")


def plan_numbering(main: list[str], appendix: list[str], promote: set[str],
                   titles: dict[str, str]) -> tuple[list[tuple[str, str, str]], dict[str, str]]:
    """Paper numbers for every placed section, and the map the prose needs.

    The draft numbers its headings for the repository record ("3.5", "Appendix
    E"), and other documents cite those numbers, so the draft keeps them. The
    paper cannot: its main text would read 2.3, 2.4, 3.5, 3.7, 4.6 with the
    gaps living in the appendix. So the build numbers sections in manifest
    order -- main text 1, 2, ... with subsections N.k; appendices A, B, ... --
    and returns (heading, paper number, display title) plus {draft id: paper
    id} for rewriting cross-references.

    In the appendix, an entry starting "Appendix X" or "Abstract", or listed in
    `promote`, opens a new lettered appendix; any other entry is a subsection
    of the one before it.
    """
    placed: list[tuple[str, str, str]] = []
    ids: dict[str, str] = {}

    def title_of(heading: str) -> str:
        if heading in titles:
            return titles[heading]
        for pattern in (TOP_LEVEL, SUB_LEVEL):
            if pattern.match(heading):
                return pattern.sub("", heading, count=1).strip()
        return re.sub(r"^Appendix [A-Z]\s*—\s*", "", heading).strip()

    top = sub = 0
    for heading in main:
        if TOP_LEVEL.match(heading):
            top, sub = top + 1, 0
            number = str(top)
            ids[TOP_LEVEL.match(heading).group(1)] = number
        elif SUB_LEVEL.match(heading):
            if top == 0:
                fail(f"main-text subsection {heading!r} comes before any section.")
            sub += 1
            number = f"{top}.{sub}"
            ids[SUB_LEVEL.match(heading).group(1)] = number
        else:
            fail(f"main-text heading {heading!r} carries no draft number, so the "
                 "build cannot place it or resolve references to it.")
        placed.append((heading, number, title_of(heading)))

    letter, sub = "", 0
    for heading in appendix:
        opens = (APPENDIX_HEAD.match(heading) or heading.startswith("Abstract")
                 or heading in promote)
        if opens:
            letter, sub = chr(ord(letter) + 1) if letter else "A", 0
            number = letter
        else:
            if not letter:
                fail(f"appendix entry {heading!r} comes before any appendix opens.")
            sub += 1
            number = f"{letter}.{sub}"
        if APPENDIX_HEAD.match(heading):
            ids[f"Appendix {APPENDIX_HEAD.match(heading).group(1)}"] = number
        for pattern in (SUB_LEVEL, TOP_LEVEL):
            if pattern.match(heading):
                ids[pattern.match(heading).group(1)] = number
                break
        placed.append((heading, number, title_of(heading)))
    return placed, ids


XREF = re.compile(r"(§\s?|Section )(\d+(?:\.\d+)?)\b|\bAppendix ([A-Z])\b(?!\s*—)")
FIGREF = re.compile(r"\bFigure ([0-9]+|S|M)\b")


def rewrite_xrefs(text: str, ids: dict[str, str], where: str) -> str:
    """Rewrite the draft's section and figure references to the paper's.
    An unresolvable one stops the build and names itself."""
    def section(match: re.Match) -> str:
        if match.group(3):
            key = f"Appendix {match.group(3)}"
            if key not in ids:
                fail(f"{where}: {match.group(0)!r} names an appendix the paper "
                     "does not contain.")
            return f"Appendix {ids[key]}"
        key = match.group(2)
        if key not in ids:
            fail(f"{where}: {match.group(0)!r} names a section the paper does "
                 "not contain.")
        target = ids[key]
        if not target[0].isdigit():
            # A section that moved to the appendix reads as one: "§E" is not
            # a form a reader recognises.
            return f"Appendix {target}"
        return f"{match.group(1)}{target}"

    def figure(match: re.Match) -> str:
        stem = SPEC_FIGURE_ID.get(match.group(1))
        if stem is None:
            fail(f"{where}: {match.group(0)!r} names a spec figure the paper does "
                 "not typeset. Remove the reference or typeset the figure.")
        # A plain space: pandoc escapes `~` in text, which printed "Figure~2".
        return rf"Figure \ref{{fig:{stem}}}"

    return FIGREF.sub(figure, XREF.sub(section, text))


def supplement():
    """`build_supplement.py`, which owns the project-name rename. Loaded by
    path so the anonymous PDF and the anonymous archive share one table."""
    import importlib.util
    spec = importlib.util.spec_from_file_location(
        "build_supplement", ROOT / "scripts/build_supplement.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def figure_block(captions: list[tuple[str, str]], half: str,
                 prepare=lambda text, where: text) -> str:
    out = []
    for stem, caption in captions:
        caption = prepare(caption, f"caption of {stem}")
        if FIGURE_PLACEMENT[stem] != half:
            continue
        if not (FIGDIR / f"{stem}.pdf").exists():
            fail(f"figure {stem}.pdf is missing from {FIGDIR}. Regenerate with "
                 "`cargo run -p binn-lab --features plots --bin paper-figures`.")
        out.append("\n".join([
            r"\begin{figure}[t]", r"  \centering",
            rf"  \includegraphics[width=\linewidth]{{figures/{stem}.pdf}}",
            rf"  \caption{{{to_latex_inline(caption)}}}",
            rf"  \label{{fig:{stem}}}",
            r"\end{figure}", ""]))
    return "\n".join(out)


def to_latex_inline(markdown: str) -> str:
    """A caption is a paragraph, not a document: strip pandoc's wrapping."""
    return to_latex(markdown).strip()


#: The class and style lines, which are all that differ between variants.
#: `10pt` is what TMLR's own template passes.
def document_head(variant: str) -> str:
    return "\n".join([r"\documentclass[10pt]{article}",
                      rf"\usepackage{VARIANTS[variant]['option']}{{tmlr}}"])


PREAMBLE = r"""\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{hyperref}
\usepackage{url}
\usepackage{booktabs}
\usepackage{longtable}
\usepackage{array}
\usepackage{calc}
\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{graphicx}
\usepackage{xcolor}
\usepackage{newunicodechar}
%% Every non-ASCII character the manuscript uses, declared explicitly.
%% pdflatex halts on an undeclared one, which is the right behaviour and a
%% terrible way to find out: the build dies mid-run with a character code.
%% `build_paper.py` scans the draft first and refuses to start if it meets one
%% that is not on this list, so the failure names the character and the line.
\newunicodechar{—}{---}
\newunicodechar{–}{--}
\newunicodechar{−}{\ensuremath{-}}
\newunicodechar{×}{\ensuremath{\times}}
\newunicodechar{±}{\ensuremath{\pm}}
\newunicodechar{§}{\S}
\newunicodechar{→}{\ensuremath{\rightarrow}}
\newunicodechar{′}{\ensuremath{{}^{\prime}}}
\newunicodechar{“}{``}
\newunicodechar{”}{''}
\newunicodechar{θ}{\ensuremath{\theta}}
\newunicodechar{σ}{\ensuremath{\sigma}}
\newunicodechar{ρ}{\ensuremath{\rho}}
\newunicodechar{≥}{\ensuremath{\geq}}
\newunicodechar{≤}{\ensuremath{\leq}}
\newunicodechar{≠}{\ensuremath{\neq}}
\newunicodechar{Σ}{\ensuremath{\Sigma}}
\newunicodechar{ₜ}{\ensuremath{{}_{t}}}
\newunicodechar{†}{\ensuremath{^{\dagger}}}
\newunicodechar{‡}{\ensuremath{^{\ddagger}}}
\providecommand{\tightlist}{%
  \setlength{\itemsep}{0pt}\setlength{\parskip}{0pt}}
%% pandoc emits `\def\LTcaptype{none}` for a longtable with no caption, which
%% wants a counter of that name. Defining it is cheaper and less invasive than
%% pulling in `caption`, which fights the venue style over float spacing.
\newcounter{none}
\setlength{\LTpre}{4pt}
\setlength{\LTpost}{4pt}
%% A pointer into the on-disk record. Rendered as a bracketed number in the
%% text and expanded once, in the appendix. The markdown cites these inline by
%% filename, which is correct for a repository and unreadable in a paper.
\newcommand{\rec}[1]{\textsuperscript{\textsc{r}#1}}
\graphicspath{{figures/}}
\hypersetup{hidelinks,pdfauthor={},pdftitle={},pdfsubject={},pdfcreator={},pdfproducer={},pdfkeywords={}}
% A float may not drift past the section that cites it: appendix figures
% once landed pages after their appendix, behind unrelated ones.
\usepackage[section]{placeins}
% Appendix K's ten columns ran 9 pt past the text block at the default 6 pt
% and normal size; the resolution table ran over too.
\setlength{\tabcolsep}{4pt}
\usepackage{etoolbox}
\newcommand{\tablesize}{\small}
\AtBeginEnvironment{longtable}{\tablesize}
"""


#: Declared in PREAMBLE with `newunicodechar`. Kept here as data so the build
#: can refuse an undeclared character by name instead of letting pdflatex halt
#: on a character code thirty lines into a log.
DECLARED_UNICODE = set("—–−×±§→′“”θσρ≥≤≠Σₜ†‡")


def check_unicode(text: str) -> None:
    seen: dict[str, int] = {}
    for lineno, line in enumerate(text.splitlines(), 1):
        for ch in line:
            if ord(ch) > 127 and ch not in DECLARED_UNICODE:
                seen.setdefault(ch, lineno)
    if seen:
        detail = "\n".join(
            f"  U+{ord(ch):04X} {ch!r} first at {DRAFT.name}:{line}"
            for ch, line in sorted(seen.items()))
        fail("the manuscript uses characters this build has no LaTeX mapping "
             f"for:\n{detail}\nAdd each to DECLARED_UNICODE and to the "
             "\\newunicodechar block in PREAMBLE, choosing a mapping that is "
             "right in text mode AND in math mode, or replace the character in "
             "the draft.")


def abstract_problems(body: str) -> list[str]:
    """TMLR: "The abstract must be limited to one paragraph." A blank line in
    the markdown is a paragraph break in the PDF, so that is what is counted."""
    paragraphs = [p for p in re.split(r"\n\s*\n", body.strip()) if p.strip()]
    if len(paragraphs) != 1:
        return [f"the abstract is {len(paragraphs)} paragraphs; TMLR's template "
                "requires exactly one."]
    return []


def front_matter(sections: list[tuple[str, str]], manifest: dict) -> tuple[str, str]:
    """(abstract, LLM-use disclosure) bodies, named by the manifest's [front].

    Both are typeset specially -- the abstract in TMLR's abstract environment,
    the disclosure as the first-page footnote TMLR requires -- so neither may
    also appear as an ordinary section. A missing one stops the build rather
    than producing a submission TMLR would return."""
    front = manifest.get("front", {})
    bodies = dict(sections)
    out = []
    for key in ("abstract", "disclosure"):
        heading = front.get(key)
        if heading is None:
            fail(f"paper/split_manifest.toml has no [front] {key} = \"...\". "
                 "TMLR requires both an abstract and a first-page LLM-use "
                 "footnote; the build will not guess which section is which.")
        if heading not in bodies:
            fail(f"[front] {key} names {heading!r}, which the manuscript does "
                 "not have.")
        # A trailing `---` is the draft's divider before the next part, not
        # part of this section; in a footnote it would typeset as a rule.
        out.append(re.sub(r"\n\s*-{3,}\s*$", "", bodies[heading].strip()).strip())
    return out[0], out[1]


def omitted_sections(manifest: dict) -> set[str]:
    """[omitted]: sections of the draft this paper deliberately leaves out.

    Leaving a section out is a decision, so it is written down with its reason
    rather than implied by absence; a section in no list still stops the build.
    References INTO an omitted section need no separate guard: it gets no paper
    number, so `rewrite_xrefs` refuses any typeset text that cites it."""
    omitted = manifest.get("omitted", {})
    sections = set(omitted.get("sections", []))
    if sections and not str(omitted.get("reason", "")).strip():
        fail("paper/split_manifest.toml [omitted] lists sections but gives no "
             "reason; say why they are out of this paper.")
    return sections


def build(variant: str = "submission") -> None:
    verify_style()
    if not shutil.which("pandoc"):
        fail("pandoc is not installed; the build cannot convert the manuscript.")
    build_dir = VARIANTS[variant]["build"]
    manifest = tomllib.loads(MANIFEST.read_text())
    want_main = list(manifest["main"]["sections"])
    want_appendix = list(manifest["appendix"]["sections"])
    front = manifest.get("front", {})
    want_front = [front[k] for k in ("abstract", "disclosure") if k in front]

    text = DRAFT.read_text()
    check_unicode(text)
    title = text.split("\n", 1)[0].lstrip("# ").strip()
    sections = split_sections(text)
    have = {h for h, _ in sections}
    abstract, disclosure = front_matter(sections, manifest)
    doubled = sorted(set(want_front) & (set(want_main) | set(want_appendix)))
    if doubled:
        fail("these front-matter sections are also listed as body sections, so "
             "they would print twice:\n" + "\n".join(f"  - {h}" for h in doubled))
    problems = abstract_problems(abstract)
    if problems:
        fail(problems[0] + " Condense the `## Abstract` section of the draft.")

    want_omitted = omitted_sections(manifest)
    both = sorted(want_omitted & (set(want_main) | set(want_appendix) | set(want_front)))
    if both:
        fail("these sections are both [omitted] and placed:\n"
             + "\n".join(f"  - {h}" for h in both))
    unplaced = sorted(have - set(want_main) - set(want_appendix) - set(want_front)
                      - want_omitted)
    if unplaced:
        fail("these sections are in neither half of paper/split_manifest.toml:\n"
             + "\n".join(f"  - {h}" for h in unplaced)
             + "\nAdd each to [main], [appendix] or [omitted]. A section in none "
               "would be dropped from the paper without appearing anywhere in "
               "the output, which is the one failure this build refuses to make "
               "quietly.")
    missing = sorted((set(want_main) | set(want_appendix) | set(want_front)
                      | want_omitted) - have)
    if missing:
        fail("paper/split_manifest.toml names sections the manuscript no "
             "longer has:\n" + "\n".join(f"  - {h}" for h in missing)
             + "\nThe manifest has drifted from the draft; a stale split "
               "silently reorders the paper.")

    placed, ids = plan_numbering(want_main, want_appendix,
                                 set(manifest.get("numbering", {}).get("promote", [])),
                                 dict(manifest.get("titles", {})))
    number_of = {heading: (number, title) for heading, number, title in placed}
    bodies = dict(sections)

    def prepare(text: str, where: str) -> str:
        """Record links to markers, cross-references to the paper's numbers,
        and -- in the anonymous variant only -- the project's name to the
        supplement's neutral one, so printed commands match the archive."""
        text = rewrite_xrefs(text, ids, where)
        if variant == "submission":
            text, _ = supplement().rename_project(text)
        return text

    captions = spec_captions()
    caption_of = dict(captions)

    # Seeded from the same walk the supplement's RECORDS.md uses, so the two
    # number every record identically; rendering must add nothing new.
    register = record_register(manifest, sections)
    expected_records = len(register)
    def render(wanted: list[str], half: str) -> str:
        """Each figure is set right after the section that first cites it.
        Emitted at the end of the half, [t] floats could only land after all
        of it -- which is how three main-text figures ended up inside the
        reference list. A figure its half never cites goes at the end."""
        pending = [stem for stem, _ in captions if FIGURE_PLACEMENT[stem] == half]

        def raw(stem: str) -> str:
            # A raw-LaTeX block passes through pandoc untouched, so the whole
            # half converts in ONE call. Per-section calls lost every heading
            # whose body is empty: pandoc with --natbib emits nothing for a
            # document that is only a heading, and "3 Methods", "4 Results"
            # and "5 Discussion" vanished from the PDF.
            block = figure_block([(stem, caption_of[stem])], half, prepare)
            return f"\n```{{=latex}}\n{block}\n```\n"

        chunks = []
        for heading in wanted:
            number, title = number_of[heading]
            # Emitted unnumbered (`{-}`) with the paper's number in the title:
            # letting LaTeX number headings too once printed "0.2 1.
            # Introduction". Top level is a lone number or letter.
            level = "#" if "." not in number else "##"
            body = prepare(rewrite_record_links(bodies[heading], register), heading)
            # A trailing `---` is the draft's divider between parts; in the
            # paper it typeset as a stray rule before the next heading.
            body = re.sub(r"\n\s*-{3,}\s*$", "\n", body.rstrip()) + "\n"
            # Each figure goes right after the PARAGRAPH that first cites it,
            # in citation order. After the whole section, a long section left
            # its figures pages away, and spec order numbered Figure 2 (cited
            # first) as Figure 1; LaTeX numbers floats in emission order.
            paragraphs = body.split("\n\n")
            placed_body = []
            for paragraph in paragraphs:
                placed_body.append(paragraph)
                cited = sorted((paragraph.index(rf"\ref{{fig:{stem}}}"), stem)
                               for stem in pending if rf"\ref{{fig:{stem}}}" in paragraph)
                for _, stem in cited:
                    placed_body.append(raw(stem).strip("\n"))
                    pending.remove(stem)
            body = "\n\n".join(placed_body)
            if heading in FOOTNOTESIZE_TABLES:
                body = ("```{=latex}\n\\renewcommand{\\tablesize}{\\footnotesize}\n```\n\n"
                        + body
                        + "\n\n```{=latex}\n\\renewcommand{\\tablesize}{\\small}\n```\n")
            chunks.append(f"{level} {number} {prepare(title, heading)} {{-}}\n{body}")
        chunks += [raw(stem) for stem in pending]
        return to_latex("\n".join(chunks))

    main_tex = render(want_main, "main")
    appendix_tex = render(want_appendix, "appendix")

    if len(register) != expected_records:
        fail("rendering registered records that record_register() did not; the "
             "supplement's RECORDS.md would number them differently.")
    record_rows = "\n".join(
        rf"\textsc{{r}}{n} & \texttt{{\small {name.replace('_', r'\_')}}} \\"
        for name, n in sorted(register.items(), key=lambda kv: kv[1]))

    abstract_tex = to_latex_inline(prepare(rewrite_record_links(abstract, register), "Abstract"))
    disclosure_tex = to_latex_inline(prepare(rewrite_record_links(disclosure, register),
                                             "LLM-use disclosure"))
    if "\n\n" in disclosure_tex:
        fail("the LLM-use disclosure is more than one paragraph; it is typeset "
             "as a footnote and must be one.")

    BUILD = build_dir
    BUILD.mkdir(exist_ok=True)
    (BUILD / "figures").mkdir(exist_ok=True)
    for stem, _ in captions:
        if FIGURE_PLACEMENT[stem] != "omitted":
            shutil.copy(FIGDIR / f"{stem}.pdf", BUILD / "figures" / f"{stem}.pdf")
    for name in STYLE_FILES:
        shutil.copy(PAPER / name, BUILD / name)
    verify_style(BUILD)
    shutil.copy(BIB, BUILD / "references.bib")

    doc = "\n".join([
        document_head(variant),
        PREAMBLE,
        rf"\title{{{title}}}",
        rf"\author{{{VARIANTS[variant]['author']}}}",
        r"\begin{document}",
        r"\maketitle",
        r"\begin{abstract}",
        # TMLR: LLM use is disclosed "as a footnote on the first page". The
        # abstract is always on the first page, so the footnote hangs off it.
        abstract_tex + rf"\footnote{{{disclosure_tex}}}",
        r"\end{abstract}",
        main_tex,
        # Flush every pending float before the references, so no main-text
        # figure can drift into the reference list.
        r"\clearpage",
        r"\bibliographystyle{tmlr}",
        r"\bibliography{references}",
        r"\appendix",
        record_section(variant, register, record_rows),
        appendix_tex,
        r"\end{document}", ""])
    (BUILD / "main.tex").write_text(doc)
    print(f"wrote {BUILD/'main.tex'} "
          f"({len(want_main)} main section(s), {len(want_appendix)} appendix, "
          f"{len(register)} record reference(s), "
          f"{len(captions)} spec-owned caption(s))")

    env = dict(os.environ, SOURCE_DATE_EPOCH="0")
    for pass_no in range(1, 4):
        proc = subprocess.run(
            ["pdflatex", "-interaction=nonstopmode", "-halt-on-error", "main.tex"],
            cwd=BUILD, capture_output=True, text=True, env=env)
        if proc.returncode != 0:
            tail = "\n".join(proc.stdout.splitlines()[-30:])
            fail(f"pdflatex pass {pass_no} failed:\n{tail}")
        if pass_no == 1:
            # bibtex's exit code was discarded here, and its log with it. It
            # had been failing with "I found no \\citation commands" for as
            # long as the draft has cited in prose: `references.bib` was copied
            # in, `\\bibliography{references}` was emitted, bibtex used 0 of the
            # 18 entries, and the build printed success. A bibliography that
            # cannot be built must not look like one that built empty on
            # purpose, so the outcome is carried to `check()` rather than
            # dropped.
            subprocess.run(["bibtex", "main"], cwd=BUILD,
                           capture_output=True, text=True, env=env)
    print(f"built {BUILD/'main.pdf'}")


def content_pages(pdf: pathlib.Path) -> tuple[int, int]:
    """(content pages, total pages). Content ends where the appendix starts."""
    info = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True)
    total = 0
    for line in info.stdout.splitlines():
        if line.startswith("Pages:"):
            total = int(line.split()[1])
    text = subprocess.run(["pdftotext", "-f", "1", "-l", str(total), str(pdf), "-"],
                          capture_output=True, text=True).stdout
    pages = text.split("\f")
    for index, page in enumerate(pages):
        # Content ends at whichever boundary comes first. Until the
        # bibliography rendered at all there was only one -- the appendix's
        # "Record references" -- and using it was equivalent. With citations
        # emitted, `\bibliography` sits between the main text and `\appendix`,
        # and counting to the appendix charged the manuscript for its own
        # reference list: 9 content pages read as 13. Venues exclude
        # references from the limit, so the count stops at the earlier of the
        # two rather than the later.
        heading = any(line.strip() == "References"
                      for line in page.split("\n"))
        if heading or "Record references" in page:
            return index, total          # 0-based index == pages before it
    return total, total



#: What bibtex says when the document cites nothing. Its exit status is 0 in
#: that case, so the status alone cannot detect it -- the log is the evidence.
NO_CITATIONS = r"I found no \citation commands"


def bibliography_problems(build_dir: pathlib.Path | None = None) -> list[str]:
    r"""Whether the bibliography the build runs actually produced anything.

    The paper ships `references.bib` and emits `\bibliography{references}`, so a
    reader is promised a reference list. Nothing checked that one was produced.
    It was not: the draft cites in prose -- "Xu, Yuksekgonul and Zou
    (arXiv:2602.00986)" -- rather than with `\cite`, so bibtex read the aux
    file, found no citations, used 0 of 18 entries, and left the section empty
    while the build reported success.

    This is the same shape as every other gate here: a check that could not run
    must not report what a check that ran and passed reports.
    """
    log = (build_dir or BUILD) / "main.blg"
    if not log.exists():
        return ["bibtex left no log, so whether the bibliography built is "
                "unknown. An unknown bibliography and a good one must not "
                "look alike."]
    text = log.read_text(errors="replace")
    if NO_CITATIONS in text:
        entries = len(re.findall(r"^@\w+\{", BIB.read_text(), re.M))
        return [f"the bibliography is EMPTY: bibtex found no \\citation "
                f"commands, so it used 0 of the {entries} entries in "
                f"references.bib. The draft cites in prose rather than with "
                f"\\cite, so the reference list a reader is promised does not "
                f"exist. Either cite with \\cite and rebuild, or stop emitting "
                f"\\bibliography and delete the unused .bib."]
    return []


#: Text the first page must carry for the LLM-use footnote to count as present.
#: The disclosure section is required to say it; the check reads the PDF.
DISCLOSURE_MARKER = "language model"


def check(variant: str = "submission") -> int:
    build_dir = VARIANTS[variant]["build"]
    pdf = build_dir / "main.pdf"
    if not pdf.exists():
        fail(f"{pdf} does not exist. Run `python3 scripts/build_paper.py"
             f"{'' if variant == 'submission' else ' --variant ' + variant}` "
             "first. A missing build and a passing check must not look alike.")
    verify_style()
    verify_style(build_dir)
    problems: list[str] = []

    # The draft is re-read so that a multi-paragraph abstract written after the
    # last build is caught by `--check`, which is what record_checks.sh runs.
    manifest = tomllib.loads(MANIFEST.read_text())
    abstract, _ = front_matter(split_sections(DRAFT.read_text()), manifest)
    problems.extend(abstract_problems(abstract))

    content, total = content_pages(pdf)
    if content > PAGE_LIMIT:
        problems.append(
            f"the main text is {content} pages against a {PAGE_LIMIT}-page "
            "limit. TMLR does not promise its review timeline above twelve. "
            "Move a section to [appendix] in paper/split_manifest.toml.")

    first = subprocess.run(["pdftotext", "-f", "1", "-l", "1", str(pdf), "-"],
                           capture_output=True, text=True).stdout
    if DISCLOSURE_MARKER not in " ".join(first.lower().split()):
        problems.append(
            f"page 1 does not mention {DISCLOSURE_MARKER!r}. TMLR requires LLM "
            "use to be disclosed in a footnote on the first page.")
    problems.extend(bibliography_problems(build_dir))

    if variant == "submission":
        body = subprocess.run(["pdftotext", str(pdf), "-"],
                              capture_output=True, text=True).stdout.lower()
        for needle, what in IDENTIFYING:
            if needle.lower() in body:
                problems.append(f"the rendered PDF contains {needle!r} — {what}. "
                                "This is a double-blind submission.")
        # The project's and the author's crates' names: the public repository
        # is called by the first. Checked with the supplement's own pattern,
        # which spares "binning", so the PDF and the archive share one rule.
        leak = supplement().IDENTIFYING.search(body.encode())
        if leak:
            problems.append(f"the rendered PDF contains {leak.group(0).decode()!r}, "
                            "a name that leads to the public repository.")
        problems.extend(repo_identifier_problems(body))
        problems.extend(author_count_problems(body))
        meta = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True).stdout
        for line in meta.splitlines():
            field, _, value = line.partition(":")
            if field.strip() in {"Author", "Creator", "Producer", "Keywords", "Subject"}:
                for needle, what in IDENTIFYING:
                    if needle.lower() in value.lower():
                        problems.append(
                            f"PDF metadata {field.strip()} carries {needle!r} — "
                            f"{what}. pdflatex writes these from the environment.")
        if ANONYMOUS_BANNER not in " ".join(first.split()):
            problems.append(
                f"page 1 does not say {ANONYMOUS_BANNER!r}. TMLR's style prints "
                "that only in anonymous mode, so [accepted] or [preprint] is "
                "set, or \\maketitle is missing.")
    else:
        if ANONYMOUS_BANNER in " ".join(first.split()) or "TMLR" in first:
            problems.append(
                "the preprint's first page is still the anonymous TMLR "
                "submission. It must name the author and not mention TMLR.")

    for line in problems:
        print(f"  FAIL: {line}")
    if problems:
        print(f"\n{len(problems)} problem(s).")
        return 1
    who = ("anonymous in body and metadata" if variant == "submission"
           else "named, with no mention of TMLR")
    print(f"paper ({variant}): {content} content page(s) of {PAGE_LIMIT}, "
          f"{total} total; one-paragraph abstract; LLM use disclosed on page 1; "
          f"{who}; style files unmodified.")
    return 0


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true",
                    help="check an existing build instead of rebuilding")
    ap.add_argument("--variant", choices=sorted(VARIANTS), default="submission",
                    help="submission (anonymous, OpenReview) or preprint (named, arXiv)")
    args = ap.parse_args()
    if not args.check:
        build(args.variant)
    sys.exit(check(args.variant))
