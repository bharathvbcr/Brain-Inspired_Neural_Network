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
