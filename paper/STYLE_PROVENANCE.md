# Where the TMLR style files came from

**Venue:** Transactions on Machine Learning Research (retargeted 2026-09-29,
after the NeurIPS 2026 deadline had passed).

**Source:** the archive TMLR's own author guide links —
`https://jmlr.org/tmlr/author-guide.html` → `https://github.com/JmlrOrg/tmlr-style-file/archive/refs/heads/main.zip`.
That link was read off the guide page on 2026-09-29, not found by search.
Repository `JmlrOrg/tmlr-style-file`, branch `main`, commit
`7bf90efe3a0debbba703c05c43f3ff7e4d4a2992` (2023-06-30, the last push).

| File | sha256 |
|---|---|
| `tmlr.sty` | `816214ff5919aa457b6b443bee52b15d9561421417b7f8a50cc84651519f0002` |
| `tmlr.bst` | `306fd454cf40771bee01293eeb98d2c1cd5f4e11ed0cd7296b335f354fc45206` |
| `fancyhdr.sty` | `3d2922548e0e5f1a6c5676eda6ebb6dc20d7d305b4d8c2be5f1c833fb1084e6d` |

All three ship with the paper, `fancyhdr.sty` included, so the build does not
depend on whichever copy the local TeX installation carries.
`scripts/build_paper.py` refuses to build if any hash changes, in `paper/` or
in the build directory it copies them to.

**How it is used.** Anonymous submission: `\usepackage{tmlr}`, which prints
"Anonymous authors / Paper under double-blind review" and the header "Under
review as submission to TMLR". Named arXiv preprint: `\usepackage[preprint]{tmlr}`,
which the template documents as the way "to de-anonymize and remove mentions
to TMLR (for example for posting to preprint servers)". Camera-ready, if
accepted: `[accepted]`, plus `\month`, `\year` and `\openreview`.

## The file this replaced

Until 2026-09-29 the paper targeted NeurIPS 2026 with `neurips_2026.sty`
(sha256 `0c1ad36961fcd9198dcc2558cf2793e1df39973bde8264fd701f5e7970672757`).
The official download was unreachable then, so that copy was verified by
agreement: two unrelated arXiv source bundles (2605.19641, 2605.27734) shipped
it byte-identical, and the first search hit — a "unified" GitHub template — was
**rejected** because it was 27 lines longer, i.e. edited. That is the lesson
kept here: take the template from the link the venue itself publishes, and pin
it. The NeurIPS file is removed; it is in the git history.
