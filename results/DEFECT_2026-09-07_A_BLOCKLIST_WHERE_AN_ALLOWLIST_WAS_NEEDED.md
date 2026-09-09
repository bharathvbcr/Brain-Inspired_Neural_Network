# Defect — nine published reproduction commands could not run, and the gate for that passed

**Date:** 2026-09-07
**Status:** the nine commands are repaired, the gate is extended, and the
citation exposure §5 named is **closed 2026-09-09** — see §5.
**Gate:** [`scripts/test_published_hashes_resolve.py`](../scripts/test_published_hashes_resolve.py)
**Found by:** accident, while sweeping `--config-hash` values for an unrelated
measurement. Nothing was looking for it.

---

## 1. What was wrong

Nine `--config-hash` arguments published as reproduction steps name values the
binary refuses with `unknown ... hash`. Every one of those commands exits
non-zero before doing anything.

| document | commands | why the hash does not resolve |
|---|---:|---|
| `PAPER_SKELETON.md` §10 | 8 | minted by `--matched-forward`, never a preset |
| `REPRO_ARTIFACT_CHECKLIST.md` §A | 1 | the `ep4` preset's hash moved on 2026-08-25 |

**The eight are the more interesting half.** `--config-hash` resolves a
**preset**. `--matched-forward` then overrides the preset and mints a **new**
hash — which is not itself a preset, so `from_hash` cannot resolve it. Writing
the minted hash back into the command is not redundant; it is fatal. The flags
alone mint exactly the archived hashes, verified for all eight against
`matched_rerun_2026-08-25/`, so the repair drops the `--config-hash` argument
and **no archived hash moved and no number changed**.

## 2. The part that should be uncomfortable

The block holding those eight commands is **itself the repair of the previous
round of this defect**. It opens:

> **Those commands could not run as written.** The current hashes, per graph:

and then publishes eight commands that also could not run as written, for a
different reason. The gate written after the first round did not catch the
second, and the document that announces the fix is the document that carries the
regression.

## 3. Why the gate passed

`test_published_hashes_resolve.py` asserted, among other things:

> No `--config-hash` argument anywhere in the record names a **retired** hash.

That is a **blocklist**. It asks whether a published hash is on a list of
values somebody already knew were dead. A hash that was never a preset was never
retired either, so it is not on the list, so it passes — unseen, not tolerated.

All nine offenders were invisible to it for exactly that reason: eight were
override-minted and never presets, and the ninth belongs to a retirement that
was only half recorded (§4).

The gate now also asserts the **allowlist**: every published `--config-hash`
value must be a preset of the suite it is passed to. The preset lists are read
from the binary itself — each suite prints them when handed an unknown hash, and
exits immediately — rather than duplicated in Python, because a duplicated
allowlist drifts and a drifting allowlist is worse than none.

**Mutation-proven.** Restoring one repaired command fails it; making the preset
probe parse nothing fails it; widening the hash pattern to swallow `c1-<hex>`
documentation placeholders fails it. That third check had to be rewritten: the
first version compared scanned values against the corpus and **could not fail**,
because every placeholder in this repository lives in
`binn-lab/experiments/c1.rs` and the scan reads markdown. It is now asserted
against the pattern, where it can fail, and the docstring says why.

## 4. A retirement was recorded for one preset of four

On 2026-08-25 `MATCHED_INPUT_SCALE` and the forward graph were mixed into the
matched configs' hashes. That moved **every preset in each family**, not just
the scientific one — `--matched-arch` alone has four:

| preset | hash today |
|---|---|
| `c1-match` | `c1-match-6f6000f148f7d30c` |
| `c1-match-quick` | `c1-match-b8ef596b23868ba0` |
| `c1-match-ep4` | `c1-match-afc3f531dc910130` |
| `c1-match-ep4-quick` | `c1-match-236f33b814d923d8` |

The freeze comment in `match_config.rs` records the retirement of **one** of
them, and `freeze_blocks()` parses one `retired:`/`current:` pair per config
file, so the blocklist could never have held more. The pre-repair `quick` value
`c1-match-85e9548f0615b85a` still stands in `results/c1_match_quick.md`, and the
pre-repair `ep4` value `c1-match-b46b23549b37d90a` stood in the reviewer's
checklist until today.

The allowlist makes this moot for **commands**: a stale preset hash fails it
whether or not anyone recorded the retirement. As of 2026-09-09 the parser
holds every pair, and **twelve** retirements are recorded where four were.

## 5. The citation exposure, and how it was closed

**Stated open on 2026-09-07.** The allowlist reads `--config-hash` *arguments*.
A stale preset hash sitting in a **table** — `c1-match-b46b23549b37d90a` as the
label of "Break-it v22 undertrain FAIL" — is not a command and was not checked
by it. Assertion (3) would have caught it, but only for hashes on the
blocklist, and these were exactly the ones the blocklist never held.

**Closed 2026-09-09, and not by enumerating them.** Writing the missing hashes
into a list would have closed this instance and left the identical hole open
for the next retirement — the same shape as the defect itself. The set is
**derived** instead:

> Every `config hash:` header in the record names either a hash its suite still
> accepts or a hash that suite's freeze comment records as retired.

Every C1 report writes that header, so a preset that moves and is not recorded
fails the moment its report is written. The live half is read from the binary,
which is the authority on what `from_hash` resolves; the retired half is read
from the freeze comments, which is what goes stale and is now what gets caught.

**`freeze_blocks()` was the reason it could not.** It used `re.search`, so it
held one `retired:` line per file — a parser shaped so it *cannot* hold the
whole answer, which is the same defect class as a check that cannot fail. It is
`re.findall` now, and the comments carry **twelve** retirements against four:

| config | retired | which |
|---|---:|---|
| `match_config.rs` | 4 | scientific, quick, `ep4`, `ep4-quick` |
| `dfa_match_config.rs` | 2 | scientific, quick |
| `rl_match_config.rs` | 4 | v12 scientific + quick; **v11** scientific + quick |
| `eventprop_match_config.rs` | 2 | scientific, quick |

The two v11 values were retired by **supersession** rather than by the
2026-08-25 hash change — `rl_graded` as primary failed and v12 replaced it. Both
reasons are recorded, because a citation check cannot tell them apart and should
not try: each names a hash `from_hash` refuses, and citing either as a live
result is one defect with one shape.

**One exemption, named and tested.** `results/matched_rerun_2026-08-25/` holds
the `--matched-forward` reruns, whose headers are override-minted rather than
retired — never presets, and still reproducible today from the flags. The
exemption is a directory rather than eight hashes, and a test asserts it is
*load-bearing* (something in there really is override-minted) and *narrow*
(something in there really is a live preset, so it is not a blanket hole, and
nothing in it is also on the blocklist).

**What the check then found.** Three paper-side tables citing
`c1-match-b46b23549b37d90a` with no marker — `PAPER_METRICS_FULL.md`,
`PAPER_RESULTS_TABLE.md`, `PUBLISHABLE_CLAIMS.md`. They are **marked, not
replaced**: substituting the current `ep4` hash `c1-match-afc3f531dc910130`
would attach a July number to a config that never produced it, and the
undertrain check was not among the eight cells re-run on 2026-08-25.

**Mutation-proven, five ways.** Restoring `re.search` fails two tests; deleting
one `retired:` line fails the same two; making the header regex match nothing
fails the scan's own negative control; widening the exemption to all of
`results/` fails the narrowness test. The fifth was not staged — unmarking those
three rows *was* the pre-fix state, and the check failed on it before they were
marked.

**What is still not checked.** Only the four matched families. `c1x-*`,
`c1-micro-*` and `c1-mac-probe-*` headers are outside `SUITE_OF` and outside
this; extending it means giving each family its suite flag, which is mechanical
and was not done here. No number depends on it, and `check_every_number.py`
continues to trace every paper number to a source that still contains it.

## 6. What this cost, and what it did not

**Did not:** move a hash, change a number, or touch a result. Every repair
either deleted a `--config-hash` argument whose flags already mint the right
value, or replaced a stale preset hash with the current one. The paper's numbers
are unaffected and `check_every_number.py` passes unchanged.

**Did:** publish, for some period, nine reproduction steps that exit with
`unknown hash` — in `PAPER_SKELETON.md` and in the checklist a reviewer is meant
to run. The first round of this defect was found on 2026-08-29 by reading; the
second was found on 2026-09-07 by accident. Neither was found by a test, which
is the reason the allowlist exists now.
