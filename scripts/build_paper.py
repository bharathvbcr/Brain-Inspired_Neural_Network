#!/usr/bin/env python3
"""Build the anonymous NeurIPS 2026 submission from `results/PAPER_DRAFT.md`.

The manuscript's single source of truth is the markdown. Hand-maintaining a
parallel `.tex` would put two documents in the repository that are supposed to
say the same thing, and this repository has already had that failure twice --
once with a venue note describing a paper that no longer existed, and once with
a figure drawing a superseded value block at camera-ready quality. So the
`.tex` is generated, every time, and is not edited.

What this refuses to do, and why each refusal exists:

  * Build with a MODIFIED style file. NeurIPS says tweaking the style may be
    grounds for desk rejection, and the first template that comes up in search
    IS tweaked (see `paper/STYLE_PROVENANCE.md`). The hash is pinned.
  * Build with a section in NEITHER half of the split manifest. A section that
    silently vanishes from a nine-page paper is invisible in the output; the
    build stops instead.
  * Report success on a paper that is OVER the page limit. NeurIPS does not
    review those, so "it compiled" is not the property worth reporting.
  * Report success on a PDF that DE-ANONYMISES the submission. Checked against
    the rendered text and the PDF metadata both, because the two fail
    separately -- a clean manuscript can still ship an author name in
    /Author, and pdflatex writes that from the environment without being asked.

Usage:
    python3 scripts/build_paper.py            # build and check
    python3 scripts/build_paper.py --check    # check a build already on disk
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
STY = PAPER / "neurips_2026.sty"
BUILD = PAPER / "build"
FIGDIR = ROOT / "results/runs/2026-07-23-paper-hard-both/figures"

#: The unmodified NeurIPS 2026 style file, agreed on byte-for-byte by two
#: unrelated arXiv submissions. `paper/STYLE_PROVENANCE.md` records how.
STY_SHA256 = "0c1ad36961fcd9198dcc2558cf2793e1df39973bde8264fd701f5e7970672757"

#: Nine content pages, per the NeurIPS 2026 call. References, the checklist and
#: technical appendices are excess to it, which is what the `\appendix` marker
#: in the generated source separates.
PAGE_LIMIT = 9

#: Anything that would identify the authors in a double-blind submission. Each
#: is checked against the EXTRACTED PDF TEXT, not the source, because that is
#: what a reviewer sees; and against the PDF's metadata separately.
IDENTIFYING = [
    ("bharath", "the author's name"),
    ("vbcr", "the author's email local part"),
    ("gmail", "a personal email domain"),
    ("/Users/", "an absolute home path leaking a username"),
]

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
FIGURE_PLACEMENT = {
    "leadfig1_the_conditional": "main",
    "leadfig2_headline_accuracy": "main",
    # Both followed their sections into the appendix on 2026-09-07: the width
    # ladder to Appendix E and the resolution ladder to Appendix I. A figure
    # placed away from the text that reads it is worse than no figure.
    "leadfig3_width_ladder": "appendix",
    "leadfig4_resolution_ladder": "appendix",
    "figS_substrate": "main",
    "figM_mechanism_richness_addressability": "appendix",
    "fig2_matched_means": "appendix",
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
             + "\nAdd each to FIGURE_PLACEMENT as 'main' or 'appendix'.")
    missing = sorted(set(FIGURE_PLACEMENT) - {stem for stem, _ in out})
    if missing:
        fail("FIGURE_PLACEMENT names figures the spec no longer captions:\n"
             + "\n".join(f"  - {s}" for s in missing))
    return out


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def verify_style() -> None:
    if not STY.exists():
        fail(f"{STY} is missing. See paper/STYLE_PROVENANCE.md for how to "
             "obtain it; do not substitute a template found by search.")
    got = hashlib.sha256(STY.read_bytes()).hexdigest()
    if got != STY_SHA256:
        fail(f"{STY.name} is not the verified file.\n"
             f"  expected {STY_SHA256}\n  got      {got}\n"
             "NeurIPS says tweaking the style file may be grounds for desk "
             "rejection, and the most findable third-party 'unified' template "
             "IS tweaked. If you deliberately updated the style file, verify "
             "the new one the way paper/STYLE_PROVENANCE.md describes -- two "
             "independent copies agreeing -- and update STY_SHA256 here.")


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


def figure_block(captions: list[tuple[str, str]], half: str) -> str:
    out = []
    for stem, caption in captions:
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


PREAMBLE = r"""\documentclass{article}
\usepackage{neurips_2026}
\usepackage[utf8]{inputenc}
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
\providecommand{\tightlist}{%
  \setlength{\itemsep}{0pt}\setlength{\parskip}{0pt}}
%% pandoc emits `\def\LTcaptype{none}` for a longtable with no caption, which
%% wants a counter of that name. Defining it is cheaper and less invasive than
%% pulling in `caption`, which fights the NeurIPS style over float spacing.
\newcounter{none}
\setlength{\LTpre}{4pt}
\setlength{\LTpost}{4pt}
%% A pointer into the on-disk record. Rendered as a bracketed number in the
%% text and expanded once, in the appendix. The markdown cites these inline by
%% filename, which is correct for a repository and impossible in nine pages.
\newcommand{\rec}[1]{\textsuperscript{\textsc{r}#1}}
\graphicspath{{figures/}}
\hypersetup{pdfauthor={},pdftitle={},pdfsubject={},pdfcreator={},pdfproducer={},pdfkeywords={}}
"""


#: Declared in PREAMBLE with `newunicodechar`. Kept here as data so the build
#: can refuse an undeclared character by name instead of letting pdflatex halt
#: on a character code thirty lines into a log.
DECLARED_UNICODE = set("—–−×±§→′“”θσρ≥≤≠Σₜ")


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


def build() -> None:
    verify_style()
    if not shutil.which("pandoc"):
        fail("pandoc is not installed; the build cannot convert the manuscript.")
    manifest = tomllib.loads(MANIFEST.read_text())
    want_main = list(manifest["main"]["sections"])
    want_appendix = list(manifest["appendix"]["sections"])

    text = DRAFT.read_text()
    check_unicode(text)
    title = text.split("\n", 1)[0].lstrip("# ").strip()
    sections = split_sections(text)
    have = {h for h, _ in sections}

    unplaced = sorted(have - set(want_main) - set(want_appendix))
    if unplaced:
        fail("these sections are in neither half of paper/split_manifest.toml:\n"
             + "\n".join(f"  - {h}" for h in unplaced)
             + "\nAdd each to [main] or [appendix]. A section in neither would "
               "be dropped from the paper without appearing anywhere in the "
               "output, which is the one failure this build refuses to make "
               "quietly.")
    missing = sorted((set(want_main) | set(want_appendix)) - have)
    if missing:
        fail("paper/split_manifest.toml names sections the manuscript no "
             "longer has:\n" + "\n".join(f"  - {h}" for h in missing)
             + "\nThe manifest has drifted from the draft; a stale split "
               "silently reorders the paper.")

    register: dict[str, int] = {}
    def render(wanted: list[str]) -> str:
        chunks = []
        for heading, body in sections:
            if heading not in wanted:
                continue
            level = "##" if re.match(r"^(Abstract|\d+\.\s|Appendix )", heading) else "###"
            chunks.append(f"{level} {heading}\n{rewrite_record_links(body, register)}")
        return to_latex("\n".join(chunks))

    main_tex = render(want_main)
    appendix_tex = render(want_appendix)

    record_rows = "\n".join(
        rf"\textsc{{r}}{n} & \texttt{{\small {name.replace('_', r'\_')}}} \\"
        for name, n in sorted(register.items(), key=lambda kv: kv[1]))

    BUILD.mkdir(exist_ok=True)
    (BUILD / "figures").mkdir(exist_ok=True)
    captions = spec_captions()
    for stem, _ in captions:
        shutil.copy(FIGDIR / f"{stem}.pdf", BUILD / "figures" / f"{stem}.pdf")
    shutil.copy(STY, BUILD / "neurips_2026.sty")
    shutil.copy(BIB, BUILD / "references.bib")

    doc = "\n".join([
        PREAMBLE,
        rf"\title{{{title}}}",
        r"\author{}",
        r"\begin{document}",
        r"\maketitle",
        main_tex,
        figure_block(captions, "main"),
        r"\bibliographystyle{plainnat}",
        r"\bibliography{references}",
        r"\appendix",
        r"\section*{Record references}",
        r"Every provenance pointer in the text resolves to a machine-written "
        r"record in the accompanying repository, listed here rather than "
        r"inline. Each is the frozen analyser's own output for one "
        r"preregistered wave.",
        r"\begin{longtable}{@{}ll@{}}\toprule",
        record_rows,
        r"\bottomrule\end{longtable}",
        appendix_tex,
        figure_block(captions, "appendix"),
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
        # reference list: 9 content pages read as 13. NeurIPS excludes
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


def bibliography_problems() -> list[str]:
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
    log = BUILD / "main.blg"
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


def check() -> int:
    pdf = BUILD / "main.pdf"
    if not pdf.exists():
        fail(f"{pdf} does not exist. Run `python3 scripts/build_paper.py` first. "
             "A missing build and a passing check must not look alike.")
    verify_style()
    problems: list[str] = []

    content, total = content_pages(pdf)
    if content > PAGE_LIMIT:
        problems.append(
            f"the main text is {content} pages against a {PAGE_LIMIT}-page "
            "limit. NeurIPS does not review papers over the limit. Move a "
            "section to [appendix] in paper/split_manifest.toml.")

    body = subprocess.run(["pdftotext", str(pdf), "-"],
                          capture_output=True, text=True).stdout.lower()
    for needle, what in IDENTIFYING:
        if needle.lower() in body:
            problems.append(f"the rendered PDF contains {needle!r} — {what}. "
                            "This is a double-blind submission.")
    meta = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True).stdout
    for line in meta.splitlines():
        field, _, value = line.partition(":")
        if field.strip() in {"Author", "Creator", "Producer", "Keywords", "Subject"}:
            for needle, what in IDENTIFYING:
                if needle.lower() in value.lower():
                    problems.append(
                        f"PDF metadata {field.strip()} carries {needle!r} — "
                        f"{what}. pdflatex writes these from the environment.")
    problems.extend(bibliography_problems())

    if "Anonymous Author" not in subprocess.run(
            ["pdftotext", "-f", "1", "-l", "1", str(pdf), "-"],
            capture_output=True, text=True).stdout:
        problems.append(
            "page 1 does not say 'Anonymous Author(s)'. The style file prints "
            "that only in anonymous mode, so either [final] or [preprint] is "
            "set, or \\maketitle is missing.")

    for line in problems:
        print(f"  FAIL: {line}")
    if problems:
        print(f"\n{len(problems)} problem(s).")
        return 1
    print(f"paper: {content} content page(s) of {PAGE_LIMIT} allowed, "
          f"{total} total; anonymous in body and metadata; style file "
          "unmodified.")
    return 0


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true",
                    help="check an existing build instead of rebuilding")
    args = ap.parse_args()
    if not args.check:
        build()
    sys.exit(check())
