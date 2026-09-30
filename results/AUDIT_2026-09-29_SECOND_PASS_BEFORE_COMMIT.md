# Audit 2026-09-29 (second pass): the manuscript before the dash-edit commit

**Scope.** The typeset TMLR submission (`paper/build/main.pdf`, 31 pages) and its
sources after the 2026-09-29 checklist and the dash-removal pass. Four
independent read-only passes: numbers against records, the dash edits, a
referee read, and the rendered PDF. The blockers below were re-checked by hand
against the source before being recorded; the rest carry the pass's own
evidence label.

**Verdict.** The anchor numbers, the paired DiD, Appendix K and the literature
values hold. The manuscript still contradicts the 2026-09-29 amendment in three
places, misstates the read-out's initialisation, and prints identifiers that
resolve to the author's public repository. Predicted outcome at TMLR as it
stands: **major revision**.

## Blockers (verified by hand)

1. **The proportion claim survives.** "94.5% of the advantage is contingent on
   temporal order": Figure 1 artwork (`paper_figures.rs` `CONTINGENT_PCT_32/12`,
   96% at n = 12), its caption (`PAPER_FIGURE_SPEC.md`), and §3.8
   (`PAPER_DRAFT.md:618`). Contradicts "presence, not proportion".
2. **Appendix B still reads h1024/L4 as the read-out consuming order** and calls
   it "the paper's **leading open problem**" (`PAPER_DRAFT.md:879`). The
   amendment retired both: it is a point with no gain to lose.
3. **Overfitting is stated three ways**: "not excluded by anything in this
   paper" (`:192`, Figure 3), "That excludes overfitting" (`:873`, the wave-23
   lost-fit evidence), and "leaves the overfitting alternative exactly where it
   was" (`:879`).
4. **Initialisation is misstated.** `:294-295` says the attention arm "*is* its
   rate twin" at initialisation. Only the blocks' `w_o` start at zero
   (`binn-learn/src/shd_attention.rs:368`); the read-out matrix `w_a` is Glorot
   (`:372`), so the logits differ at step 0.
5. **Anonymity.** Record filenames (`RESULT_2026-…`), corpus paths and config
   hashes printed in the paper exist verbatim in the public repository
   `bharathvbcr/Brain-Inspired_Neural_Network`; a code search resolves them to
   the author.

## Should-fix (evidence from the passes)

- **Spearman explanation is wrong** (`:523`): of the five no-gain points only
  h1024/L4 is on the six-width ladder; ρ without it is −0.10. The mismatch is
  carried by h768. Delete the sentence.
- **"No gain to lose" has two definitions**: `--paper` marks † by point estimate
  (5 points, 16/21); `--markdown` flags by lower bound (7 points, 14/21). The
  paper discloses both counts; the script's markdown label should say which.
- **Negative shuffled gains described too narrowly** (`:472`, §4.8): −0.274 is a
  four-block point, and h768/L4 (not one of the five) has −0.132.
- **Reversal and synchrony controls** each include h1024/L1 e400, a † point,
  unflagged.
- **Figure 2** attaches `|e400 − e200| = 0.0002` (an n = 12 quantity) to the
  n = 32 line.
- **Depth scope sentence** ("four of five widths within 0.013", `:527`)
  miscounts against Appendix K.
- **0.087 vs 0.1067**: §4.8 decomposes a gap labelled 0.087 into terms summing
  to 0.1067; the primary record for 0.087 was not found.
- **"8,156 training samples every run … records loading"** is false for the 24
  wave-27 held-out-speaker cells (6,987).
- **Figure 1 Panel D footer** keeps "The mechanism travels … 21 of 21".
- **Figure 4 caption / Appendix C** attribute the shrinking gain to the read-out;
  the numbers show the rate arm rising +0.0397 and attention moving −0.0056.
- **"Substrate left bit-identical" under hidden-shuffled** holds for the rate arm
  only; the attention arm's credit trains the substrate.
- **Cramer's ≤60% counts-only bar** is used as the no-temporal-information
  benchmark while the bin-shuffled rate arm reaches 0.69 (within-bin synchrony
  and length survive). Raised by the first audit; not fixed.
- **Statistics**: no multiplicity treatment across 42 tests; percentile
  bootstrap at n = 12 undercovers; intervals are conditional on one permutation
  realisation and one test set. "Every contrast was preregistered" is too broad:
  the 16/21 stratification, the intervals and the headroom normalisation are
  post hoc.
- **Methods gaps**: the embedding W_e, 1/√d scaling, positional frequencies,
  per-epoch minibatch reshuffle, the linear anneal, fixed-tN truncation,
  adaptation/recurrence parameters, and that test-set permutations are
  independent draws ("applied identically" misleads).
- **Leftovers**: "9 of 9 permitted pages" (`:933`); "Honesty note required in
  the main text:" (`:401`); "Two paragraphs lifted from §4.6 on 2026-09-07"
  (`:889`); "ANON" anonymiser residue; "(Appendix, Appendix J.7)";
  "finding N of §4.1" against unnumbered findings; figures citing "Table SHD-7"
  and "Table A"; "four scope limits" against six.
- **Rendering**: tables uncaptioned and unnumbered; Appendix A table ~9 pt
  over the text block; figures placed far from first citation, Figure 2 cited
  before Figure 1; arXiv ids dropped by `tmlr.bst` for `@misc`; four stub
  references; ETLP cited in prose with no reference; capitalisation
  (SuperSpike, Analyzing).
- **Dash edits**: nested citation parentheses at `:136`; four clumsy joins
  (`:405`, `:409`, `:587`, `:191`); comment date in `paper_figures.rs`.

## Nits

ρ prints −0.1430; the exact value is −1/7 = −0.1429. Two of the five
"no gain" points have intervals wholly above zero. All-caps prose in figures.
The footnote's "the author" reveals a single author.

## Standing critique (unchanged)

The comparator cannot see order by construction. The minimum answer is the
order-sensitive comparator experiment (§4.8), or leading with that limitation.

## Adopted the same day

With the author's three decisions (strip repository identifiers from the
submission; keep the anchor share with its interval; cut the secondary
programme), every blocker is fixed:

1. **Proportion**: one share at the anchor, 94.5% (95% CI 90.9–97.9%), printed
   as holding there only (second section of the 2026-09-29 amendment).
2. **Appendix E** restated under the amendment; "leading open problem" gone.
3. **Overfitting** stated once, as the lost-fit finding licenses: excluded for
   the h1024 collapse. The Figure 3 ban and its test encoded the retired
   "not excluded" and were updated with it.
4. **Initialisation** stated as the code does it: blocks start as identity
   residuals; the read-out matrix is Glorot, so logits differ at step 0.
5. **Anonymity**: the submission prints record markers only; the supplement's
   `RECORDS.md` maps them. The submission check now fails on any record
   filename, `results/` path or config hash in the PDF.

Should-fix items fixed: the Spearman explanation (now h768), both "no gain"
definitions labelled, negative shuffled gains counted (12 at e400, 5 at e100),
the h1024/L1 reversal and synchrony points flagged, Figure 2's e200 label moved
to n = 12, the depth sentence, 0.1067 in place of 0.087 in §4.8, the 8,156
claim, Figure 4 and Appendix I crediting the rate arm, hidden-shuffled
qualified, Cramer's counts-only bar distinguished from bin-shuffling, a
statistics paragraph (bootstrap coverage, conditioning, no multiplicity
correction), the post-hoc quantities named in §2.4, the methods gaps, the
leftovers (they left with the secondary programme or were rewritten), table
captions, Appendix K widths, figure order and placement, `hidelinks`, ρ as
−0.1429, and the bibliography (Yu to its published three-author version, ETLP
added, arXiv ids printed, STSC article number, capitalisation), each field
checked against Crossref.

**Refuted on measurement: figure text below 7 pt.** The pass inferred ~5.9 pt
from word boxes using the body font's box-to-size ratio. Measured in the figure
PDF itself, ArialMT at 20.97 pt gives 19.7-high boxes (ratio 0.94), so the
6.59-high boxes in the paper are **7.0 pt**, the floor the Rust test asserts.

**Still open, deliberately:**
- the order-sensitive comparator experiment (the standing critique);
- no multiplicity correction is applied, and the paper now says so;
- the ETLP sentence's reading of that paper, and the Yu journal text against
  the quotes checked on arXiv v3, were not re-read;
- Perez-Nieves et al. (2021) is in the bibliography but not cited;
- the LLM-use footnote's "the author reviewed the text" awaits the author's
  confirmation, and its singular "author" discloses a single author.
