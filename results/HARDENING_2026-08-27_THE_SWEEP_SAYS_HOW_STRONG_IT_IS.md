# The number sweep now says which generator reached each number, and stops calling an empty document a pass

**2026-08-27**, immediately after
[the manuscript sweep](HARDENING_2026-08-27_THE_MANUSCRIPT_IS_SWEPT.md), whose
closing section named this as the next piece of work.

## The single rate was hiding an uneven set

`scripts/check_every_number.py` asks whether the cells can produce every
four-decimal number in a result document, and prints its own coincidence rate
so a clean pass can be read against the rate at which any number would pass.
That rate had reached **31.0%**. At that density "every number follows from the
cells" is about three times better than chance and reads like a proof.

The set is not uniformly dense, and splitting it by generator shows where the
density is:

| generator | what it is | quantities | a random 4dp value matches |
|---|---|---:|---:|
| `arm` | one configuration's own mean, extremes, headroom, or a per-seed value | 679 | **6.7%** |
| `paired` | anything over two comparable arms' shared seeds | 2,720 | **22.4%** |
| `pooled` | a cross-arm difference or ratio over each arm's own seeds | 287 | **1.9%** |

Essentially all of it is `paired`, and `paired` grew because the corpora did:
97 valid configurations now yield **1,354 comparable pairs**, each contributing
a paired mean, its headroom, a ratio, a mean gain and a per-seed gain per shared
seed.

Each number is now credited to the **first** generator that reaches it, and each
document reports the split:

```
  [ok  ] RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md    arm 36  paired 10
  [ok  ] RESULT_2026-08-23_W13_RECURRENT_STABILITY.md          arm 12
  [ok  ] RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD  arm 12  paired 21  pooled 1
```

W13's twelve numbers are each a configuration's own statistic, checked against a
679-value set. W15–17's twenty-one paired numbers were checked against a set
that would accept one value in four by accident. Both used to print the same
word.

**This does not make the check stronger.** It makes the run say where its
strength is, so a reader can weight a document's clean pass by what actually
cleared it. Reducing the paired set would mean dropping generators the documents
genuinely quote — W11 quotes per-seed gains — and scoping the set per document
was tried and abandoned: four wave results name no width, contract or geometry
at all, so the scope would have been derived from absent metadata.

## A document that printed `ok` while checking nothing

[`RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md`](RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md)
is 93 lines long and contains **no four-decimal number at all**. It has printed
`[ok  ]` for as long as this sweep has existed — the same mark as a document
whose forty-six numbers were each recomputed from cells — and was counted among
the "14 swept wave results" in the closing claim.

That is the defect this repository hunts, inside the script whose purpose is to
hunt it: a check that could not run reporting the same result as a check that
ran and passed. It now prints

```
  [none] RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md   NOTHING TO CHECK: no four-decimal number here
```

is named in a line of its own before the closing claim, and is excluded from the
count — **13 swept wave results, not 14.** The wave itself is unaffected: it
reports a qualitative result (the recurrent arm does not complete) and had no
numbers to quote. The defect was the sweep's, not the document's.

## Negative tests

Both changes were broken and the break confirmed to fire.

| break | caught by |
|---|---|
| an empty document falls back to reporting a pass | `test_the_empty_document_is_marked_apart_from_a_pass`, `test_it_is_excluded_from_the_closing_claim` |
| generators overlap, so per-document counts exceed the numbers checked | `test_the_generators_are_disjoint` |

`scripts/test_paper_number_sweep.py` is now 28 tests. It also pins the ordering
claim — `arm` must stay sparser than `paired` — because if that inverts, the
tier names are telling the reader the opposite of the truth, and every tier must
be non-empty, since a permanently empty tier's printed coincidence rate is
reassurance about nothing.

## Files

- `scripts/check_every_number.py` — `TIERS`, `derivable` returning a tier map,
  `explain`, the per-document split, and the `[none]` branch
- `scripts/test_paper_number_sweep.py` — 28 tests
