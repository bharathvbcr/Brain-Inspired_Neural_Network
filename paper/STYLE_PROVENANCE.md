# Where `neurips_2026.sty` came from, and why this copy

**sha256:** `0c1ad36961fcd9198dcc2558cf2793e1df39973bde8264fd701f5e7970672757`
**Self-identification:** `\ProvidesPackage{neurips_2026}[2026-01-29 NeurIPS 2026 submission/camera-ready style file]`
**Lines:** 437.

NeurIPS's author instructions say to use the official style file and that
"tweaking the style files may be grounds for desk rejection." The official
download was not reachable from here — `media.neurips.cc/Conferences/NeurIPS2026/`
returns 404 for both `Styles.zip` and the bare `.sty`, and the style-files page
under `neurips.cc/Conferences/2026/` does too. So the file was obtained instead
from sources that had to have used the real one, and **verified by agreement**:

| Source | sha256 | Verdict |
|---|---|---|
| `arxiv.org/e-print/2605.19641` source bundle | `0c1ad369…0dadb52`… *(see below)* | **used** |
| `arxiv.org/e-print/2605.27734` source bundle | identical to the above | **agrees** |
| `github.com/lizhemin15/NeurIPS-2026-Latex-Unified` | `11660adf…` | **rejected** |

Two unrelated arXiv submissions, by different authors, ship a **byte-identical**
`neurips_2026.sty`. That is the copy here.

**The GitHub mirror was rejected, and this is the reason to record it.** It is
27 lines longer than the arXiv copy. The additions are bilingual explanatory
comments and an extra `\AtBeginDocument` block that warns when `\workshoptitle`
is empty. Harmless in intent and a modification in fact — exactly the kind of
file NeurIPS says may be grounds for desk rejection. It was the first hit for
the obvious search, it is presented as a convenience "unified" template, and
nothing about downloading it would have told anyone it had been edited. Two
independent copies is what told us.

**If you re-obtain this file**, get it from neurips.cc when that is reachable
and check the hash above before replacing this copy.
`scripts/build_paper.py` refuses to build if the hash changes, so a swapped
style file cannot pass silently.
