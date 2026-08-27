# The manuscript's numbers are checked, at three named tiers

**2026-08-27.** `scripts/check_every_number.py` now sweeps
[`PAPER_DRAFT.md`](PAPER_DRAFT.md). Until today it did not, and said so.

## What the hole was

The sweep's whole design is to invert the question: rather than checking a
curated list of numbers someone thought to name, it takes *every* four-decimal
number in a document and asks whether the cells can produce it. Fourteen wave
results are swept that way. The manuscript — the one artefact anyone outside
this repository will read — was not.

The reason was real. `PAPER_DRAFT.md` draws on corpora the script does not load:
the July C1 / Gate G2 track, the matched-architecture re-run, the A6
ceiling-health sweep, and one published literature figure. A cell-only sweep
reports 47 of its 119 numbers as unexplained when they are simply sourced
elsewhere, and 47 false alarms is a check nobody will keep running.

So the limit was announced instead. The announcement recorded that all of the
unexplained values had been **traced on 2026-08-27 and each found in another
document under `results/`** — by hand, once, with the result written in prose.
That is document-to-document agreement rather than derivation, and it decays
silently: a number can drift, or its source can be deleted, and the prose still
reads the same.

## What replaced it

Every number in the manuscript now lands in one of three **named** tiers, and
the tiers are printed separately because they are not the same evidence.

| tier | what it means | count |
|---|---|---:|
| **A** | derived from the cells, exactly as a wave result is | **72** |
| **B** | named in `ELSEWHERE` with its derivation stated there | **7** |
| **C** | traced to one named primary record that still contains it | **40** |

Tier C is a table of `(value, record, what the number is there)` — 40 entries,
each naming a machine-written run record: `c1_sfb.md`, `c1_dfa_live.md`,
`matched_rerun_2026-08-25/c1_matched-rl_feedforward.md`,
`a6_ceiling_health_2026-08-19/a6_report.md`, and so on. Each was chosen by
reading the paper's own sentence for that value, not by taking whichever
document happened to contain a matching string — `0.0737` also occurs in
[`MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md`](MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md)
as the accuracy `0.073763251`, which has nothing to do with the gap lower bound
the paper quotes.

**What tier C establishes:** the value is still present in a specific named
record, that record still exists, and the record is not one of the paper's own
downstream artefacts. A paper number cannot drift from its source, and a source
cannot be deleted, without the gate failing.

**What tier C does not establish:** that every occurrence of that value in the
manuscript refers to that record. The table is keyed by value, and `0.5000`
occurs eight times in the paper meaning chance, an EventProp FAIL, and a
broadcast arm. The run prints this caveat every time, immediately under the
counts, so the three numbers cannot be read as three flavours of the same
strength.

## The rules, and the evidence that each can fail

Every rule was broken and the break confirmed to fire, then reverted.

| rule broken | fired |
|---|---|
| a paper number in no tier | `1 number(s) in PAPER_DRAFT.md with no tier at all: 0.4321` |
| an entry naming a document that does not exist | `0.6437 is traced to results/c1_no_such_file.md, which does not exist` |
| a source that no longer carries its value | `0.2567 … which no longer contains it — the paper and its source have drifted apart` |
| an entry citing the paper's own table | `… which is one of the paper's own artefacts; that is the claim written twice` |
| an entry whose value has left the manuscript | `PAPER_SOURCES still names 0.4321 …; delete the entry` |
| the same value entered twice | `PAPER_SOURCES names 0.7262 twice` |
| the manuscript shrinking below its floor | `only 118 distinct numbers found …, below the floor` |
| the wave glob narrowing | `15 wave-result documents matched, below the floor of 99` |

Nineteen tests in `scripts/test_paper_number_sweep.py` hold these, driving
`sweep_paper` against a scripted manuscript and source tree rather than only
against the committed record.

## A floor that could not fire

`MIN_DOCUMENTS = 14` was declared on 2026-08-24, with a comment saying it
existed so a narrowed pattern could not sweep fewer documents and still report
success. **Nothing read it.** The guard beside it tested `if not DOCUMENTS`,
which can only fire when the glob matches nothing at all — so dropping from
fifteen documents to one would have passed. It is now enforced, and it was
briefly defined twice today, which is its own version of the same defect:
breaking the first definition changed nothing, because the second overwrote it.
A test asserts one definition and one reader.

## What this does not fix, stated because the count above will be read as strong

The sweep prints its own coincidence rate, and it has eroded. **A random
four-decimal value in [0, 1] now matches one of the 3,686 derivable quantities
31.0% of the time** — up from the 8.6% recorded when the design was chosen,
because the corpora have grown to 97 configurations and the derived set with
them. Tier A is therefore about three times better than chance, not the near
certainty "derived from the cells" sounds like; of the 72 tier-A numbers,
roughly 22 would match something by accident.

That is a property of the wave sweep too, and it has been true of every "every
number follows from the cells" line this script has printed since the corpora
grew. It is not repaired here. It is measured, printed on every run, and named
as the next piece of work.

## Files

- `scripts/check_every_number.py` — the sweep, `PAPER_SOURCES`, `PAPER_SIDE`,
  and both enforced floors
- `scripts/test_paper_number_sweep.py` — 19 tests
- `scripts/test_campaign_tooling.py` — the class asserting the old announcement
  is retired, replaced by one asserting the announcement did not outlive the
  limit it announced
