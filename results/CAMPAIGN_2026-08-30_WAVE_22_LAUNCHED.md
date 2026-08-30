# Campaign — wave 22 launched

**Launched:** 2026-08-30 05:44 UTC.
**Registered:** [`PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md),
committed in `7fb7a70` and amended in `9d5bc5d` — **both before this launch**,
which is the ordering that carries the epistemic weight and is attested by git
history rather than by mtimes.
**Analyser:** `scripts/aws/analyse_wave22.py`, frozen in the same commits, and
verified to report `0 of 12 points evaluable` against an empty corpus — the
correct pre-launch state.

---

## What is running

| | |
|---|---|
| plan | 504 cells — 288 attention, 216 rate |
| fleet | 2 × `c7g.16xlarge`, `i-03fbc9c8bd557f748` and `i-09e51d77bb6fbebd5` |
| bucket | `binn-campaign-v2-511192439661-us-east-1` |
| estimated | $124 spot; the estimator over-predicts ~3×, so expect $35–45 |

## Why a new bucket

The campaign bucket `binn-campaign-511192439661-us-east-1` holds **765 results**
and pins binary `22d97c51ab02…`, which produced the entire archived corpus.
`scripts/aws/bootstrap.sh` makes a published pin **mandatory**, so running wave
22 on the guarded binary would have meant replacing that pin in place — and a
future re-run of any earlier wave from that bucket would then silently use a
different binary. That is the "one label, two experiments" failure the
matched-architecture hash retirement was about.

A second bucket leaves the old pin untouched. It starts with **no** pin, so the
first instance builds and publishes the guarded binary, and every later instance
downloads exactly that. The corpus was copied server-side from the old bucket,
so `train.events` and `test.events` are byte-identical to what every previous
wave ran against.

## The first launch failed, and the launcher was the reason

The first attempt (`i-01c9dc277aa06dc28`, `i-0a5a0ed4dbdfadccd`) came up unable
to fetch `bootstrap.sh`. S3 answered **403**, the instance wrote the XML error
body to `/tmp/bootstrap.sh`, and bash tried to execute it:

    /tmp/bootstrap.sh: line 1: `<?xml version="1.0" encoding="UTF-8"?>`

cloud-init finished in **7.6 seconds** and two `c7g.16xlarge` sat running and
idle until they were terminated by hand.

**The cause was `launch.py::ensure_role`.** It returned as soon as the instance
profile existed — correct for the profile, wrong for the IAM policy attached to
it. The policy names a bucket, and it went on naming
`binn-campaign-511192439661-us-east-1`, the bucket it was first created for.
Confirmed against the live policy before changing anything. So the deliberate
choice to use a second bucket was exactly what tripped it.

Nothing in the launch output said anything was wrong. It printed
`instance profile binn-campaign-worker exists` and then `2 instance(s)`, and
both were true. **That is the failure mode this repository exists to hunt: a
step that could not do its job reporting the same thing as one that did.**

Fixed — `put-role-policy` overwrites by name and is idempotent, so it is applied
on every launch and the policy stays scoped to exactly one bucket. Three tests
in `test_campaign_tooling.py::WorkerRolePolicyTest` pin it, negative-tested by
restoring the early return. The relaunch printed the line that had been missing:
`role policy re-scoped to binn-campaign-v2-...`.

## The binary this wave runs

Built from the source at `a5e671f`, which carries the forward-finiteness guard
([`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md)).
**Every cell of this wave is checked for a non-finite evaluation forward**, which
no archived cell was — and three of the twelve operating points are at h1024,
the configuration with peak gradient norms of 4.9e32.

Gate F passed on that source before launch: **21 distinct cells, bit-identical,
zero failures**, across six contracts, two widths and three arms including
`ff+fixed+attn` and `ff+alif`. That is measured on macOS/aarch64; the
Linux/Graviton build's equivalence to `22d97c51ab02…` is **inferred, not
measured**, and no verdict in this wave rests on it — the wave is self-contained
precisely so it does not have to.

## Monitoring

```bash
python3 scripts/aws/watch_campaign.py --bucket binn-campaign-v2-511192439661-us-east-1
```

```bash
python3 scripts/aws/collect.py --bucket binn-campaign-v2-511192439661-us-east-1
```

Teardown, which must be run once the queue drains — spot instances do not stop
themselves if the drain logic is not reached:

```bash
python3 scripts/aws/teardown.py --bucket binn-campaign-v2-511192439661-us-east-1
```

## What must not happen to this wave

1. **No point is re-run to improve its verdict**, and no seed is added beyond
   twelve. The stopping rule is 504 cells, once.
2. **No verdict is transcribed except from the frozen analyser.**
   `scripts/check_verdicts_transcribed.py` cross-checks whatever is written up.
3. **The archived intact halves are not used for any verdict.** They may be
   compared against the new ones descriptively, as a reproduction observation
   that crosses two binaries by construction and carries no hypothesis.
4. A point whose cells fail `scripts/cell_validity.py` is **reported with its
   exclusion count, not topped up**.
