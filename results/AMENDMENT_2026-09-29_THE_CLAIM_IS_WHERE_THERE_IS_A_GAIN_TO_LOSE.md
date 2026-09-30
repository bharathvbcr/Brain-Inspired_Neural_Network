# Amendment 2026-09-29 — the claim holds where there is a gain to lose

**What changed.** `PAPER_DRAFT.md` stated that the difference-in-differences
"clears its +0.03 bar at all twenty-one operating points" and read that as the
read-out's order-dependence being "present everywhere". The first half is true.
The second does not follow from it, and the manuscript no longer says it.

The claim is now: **wherever the read-out has a gain to lose, that gain depends
on temporal order** — at all twenty-one points after 100 epochs, and at sixteen
of twenty-one after 400. The author approved the change on 2026-09-29, after
the audit that found it
([`AUDIT_2026-09-29_THE_MANUSCRIPT_READ_AS_A_REVIEWER_WOULD.md`](AUDIT_2026-09-29_THE_MANUSCRIPT_READ_AS_A_REVIEWER_WOULD.md)).

**Why.** DiD = gain(intact) − gain(shuffled). Where the intact gain is near
zero, a positive DiD records only that the attention arm trains *below* the
rate arm once order is destroyed. It does not record the read-out buying
anything order-dependent. Recomputed from the cells with paired bootstrap
intervals by `scripts/did_confidence_intervals.py`
([`DID_INTERVALS_2026-09-29.md`](DID_INTERVALS_2026-09-29.md)):

- **At 400 epochs**, five points have an intact gain below +0.03:
  - one block at h256 (+0.0150), h512 (+0.0043) and h1024 (−0.0159);
  - one block on `channels-700` (+0.0243);
  - four blocks at h1024 (−0.1618).

  Their shuffled gains run from −0.2740 to −0.0608. Two more points, h768 with
  four blocks and h1024 with three, have gains above the bar whose intervals
  cross it.
- **At 100 epochs**, every intact-gain interval lies above +0.03; the smallest
  lower bound is +0.034.

**What did not change.** No number was re-measured, no bar was moved, and every
registered verdict stands as registered. The anchor's DiD (+0.1205, 32 of 32)
and the `hidden-shuffled` DiD (+0.1159, 12 of 12) are untouched. What changed is
what the verdicts are read as licensing.

**Consequences in the manuscript.**
- §3.5 and the abstract restate the claim.
- The five points are reported as a limitation, and so are their negative
  shuffled gains, which are unexplained. A positional code fitting a fixed
  per-sample permutation is one untested account.
- §4.8 no longer calls the gain/DiD dissociation this paper's "leading open
  problem". The five points without a gain account for much of it: there the
  DiD is carried by the shuffled gain.
- §3.5 no longer cites h512 and h1024 with one block as evidence that the
  mechanism "recurs where nothing collapses". Their intact gains are +0.0043
  and −0.0159.
- Appendix K prints every point with its intervals. A test fails if the
  appendix and the script's output differ.
