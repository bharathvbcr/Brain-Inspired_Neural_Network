"""Per-cell validity and stability, with one owner.

# Why this file exists

Three copies of this rule were in the tree, and they had already drifted:

  * ``scripts/aws/analyse_campaign.py`` checked the temporal audit.
  * ``scripts/azure/analyse.py`` checked ``mechanical_status`` and validated
    field *types*; the AWS pair did neither.
  * ``scripts/aws/analyse_wave8.py`` — the copy imported by wave 9 **and**
    wave 10 — checked neither.

Wave 9's whole purpose was the bin-shuffled arm, and it was analysed through
the copy with no temporal check at all. The claim "every ``w9shf`` cell passes
the temporal audit" is true, but only because the Rust instrument hard-errors
upstream when a manipulation fails its audit; nothing in the analysis path
verified it. A gate that is satisfied by an invariant it does not check is the
same shape as a gate that cannot fail.

# What is new here, beyond unifying

Two checks that no copy had:

  * **The cell ran the condition the plan asked for.** A cell whose
    ``temporal_condition`` disagrees with its spec is currently scored as
    though it were the arm it claims to be.
  * **Gradient-norm sanity.** Every existing gate is blind to magnitude, so a
    ``rec+alif`` cell peaking at 3.93e33 — five orders from f32 overflow, and
    described in ``AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md`` §6 as
    "numerically marginal" — passes them all. "Numerically sick" and "genuinely
    worse" are then indistinguishable, on exactly the arms a cross-architecture
    wave adds.

# Why magnitude warns and never voids

An earlier draft of this file voided a cell whose peak norm came within five
orders of f32 overflow, taking ``AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md``
§6's "~5 orders from f32 overflow" as a bar. Its own test caught the problem:
that amendment's run peaked at **3.93e33** and was reported as a *result* — the
registered expectation MET, ``non_finite_events`` 0, loss falling — described as
marginal, not discarded. A bar at 3.4e33 would therefore have retroactively
voided a published run.

Retroactively voiding published cells is a re-scoring, not a hardening, and it
would need its own registration. So magnitude never voids here. What voids is
what already voided: a norm that is not representable at all. Magnitude is
reported, at two tiers, and what to do about it stays with the reader.

``WARN_GRADIENT_NORM`` is empirical. Over the 624 cells of
``shd_attention_campaign_v1`` and ``_v2`` that carry per-epoch norms, the
largest observed maximum is **1.13e8** (``ff+fixed+attn`` at h1024) and the
median is 7.70. Per arm and width:

    ff+fixed+attn h1024  n=24   max=1.133e+08  median=1.226e+03
    ff+fixed+attn h128   n=324  max=7.354e+02  median=1.001e+01
    ff+fixed+attn h512   n=24   max=3.424e+02  median=3.679e+01
    ff+fixed+attn h256   n=12   max=1.485e+02  median=8.776e+01
    ff+fixed      h128   n=192  max=1.676e+00  median=5.943e-01

1e9 therefore sits above every cell in the record: turning this on voids
nothing retroactively and re-scores no published number. It is a **warning**,
not a void, for the same reason — a cell above it is reported so a reader can
see the arm was straining, and the decision about what that means stays with
the person reading, not with this file.
"""

from __future__ import annotations

import math

# Number of SHD classes. A cell that predicted fewer has collapsed.
EXPECTED_CLASSES = 20
# Above this share on one class, the readout is a constant predictor wearing an
# accuracy.
MAJORITY_MAX = 0.30
SILENT_MAX = 0.95
SATURATED_MAX = 0.05
# A "shuffle" that relocated less than half its entries did not shuffle.
RELOCATED_MIN = 0.5
# Registered floor on a full shuffle's mean displacement, as a fraction of the
# sequence length: half the T/3 a uniform permutation gives in expectation.
FULL_SHUFFLE_DISPLACEMENT_FLOOR = 1.0 / 6.0
# Slack on the window operator's relocated band; a uniform permutation of w
# items leaves one fixed point in expectation, so the fraction concentrates on
# 1 - 1/w.
WINDOW_RELOCATED_SLACK = 0.15
# Width of the binomial band on dropout retention, in standard deviations.
DROPOUT_BAND_SIGMAS = 5.0

F32_MAX = 3.4028234663852886e38
# Above every cell in the current record; see the module docstring.
WARN_GRADIENT_NORM = 1e9
# The regime the 2026-08-05 amendment calls numerically marginal: within five
# orders of f32 overflow. Warned about in stronger terms, still not a void.
MARGINAL_GRADIENT_NORM = F32_MAX / 1e5


def _number(cell: dict, field: str, problems: list[str]):
    """Fetch a numeric field, or record why it could not be read.

    A missing or non-numeric field is a problem in its own right. Reading it as
    absent — or letting a ``KeyError`` escape into a caller that treats the cell
    as missing rather than invalid — is how a broken cell becomes a silent one.
    """
    if field not in cell:
        problems.append(f"{field} missing")
        return None
    value = cell[field]
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        problems.append(f"{field}={value!r} is not a number")
        return None
    # NaN and infinity must be rejected here rather than left to the callers.
    # Every gate below is a comparison, and every comparison against NaN is
    # false, so a NaN sails through `>= MAJORITY_MAX`, `> SILENT_MAX` and
    # `> SATURATED_MAX` alike — a cell whose diagnostics are all NaN would be
    # scored as a clean one. `json.loads` accepts the bare `NaN` token the Rust
    # instrument would emit for a 0/0 denominator, so this is reachable without
    # any parse error to warn of it.
    if not math.isfinite(value):
        problems.append(f"{field}={value!r} is not finite")
        return None
    return value


def _finite_max(values) -> tuple[float | None, int]:
    """Largest finite entry of a per-epoch trace, and how many were not finite.

    JSON has no literal for infinity, so a non-finite norm arrives as ``null``
    or as a string. Both are counted rather than skipped.
    """
    if not isinstance(values, list) or not values:
        return None, 0
    finite = []
    non_finite = 0
    for value in values:
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            non_finite += 1
            continue
        if value != value or value in (float("inf"), float("-inf")):
            non_finite += 1
            continue
        finite.append(value)
    return (max(finite) if finite else None), non_finite


def validity_problems(cell: dict, spec: dict | None = None) -> list[str]:
    """Everything that disqualifies a cell from being scored.

    ``spec`` is the plan entry when the caller has one. It is optional because
    two of the three call sites did not have one, and the checks that do not
    need it must run regardless — that is what let the wave-8 copy drop the
    temporal audit in the first place.
    """
    problems: list[str] = []

    status = cell.get("mechanical_status")
    if status != "COMPLETE":
        problems.append(f"mechanical_status={status!r}")

    events = _number(cell, "non_finite_events", problems)
    if events is not None and events != 0:
        problems.append(f"non_finite_events={events}")

    classes = _number(cell, "classes_predicted", problems)
    if classes is not None and classes != EXPECTED_CLASSES:
        problems.append(f"classes_predicted={classes}")

    majority = _number(cell, "majority_prediction", problems)
    if majority is not None and majority >= MAJORITY_MAX:
        problems.append(f"majority_prediction={majority:.3f}")

    silent = _number(cell, "silent_fraction", problems)
    if silent is not None and silent > SILENT_MAX:
        problems.append(f"silent_fraction={silent:.3f}")

    saturated = _number(cell, "saturated_fraction", problems)
    if saturated is not None and saturated > SATURATED_MAX:
        problems.append(f"saturated_fraction={saturated:.3f}")

    # `accuracy` is the field every published number is derived from, and it
    # was the one field no gate read. A cell could carry NaN, null, 2.0 or the
    # string "0.83" and be scored as valid. Every cell that fails this today
    # (317 of 1671 archived, all `null`) is already voided by
    # `mechanical_status`, so this voids nothing that was previously scored.
    accuracy = _number(cell, "accuracy", problems)
    if accuracy is not None and not 0.0 <= accuracy <= 1.0:
        problems.append(f"accuracy={accuracy} is outside [0, 1]")

    # `non_finite_forward` counts test samples whose logits left f32's range.
    # `non_finite_events` beside it does NOT see them: it counts gradient and
    # update excursions during training, so until 2026-08-29 a cell could score
    # a minority of poisoned forwards into `accuracy` and report zero of
    # everything. `argmax` orders by `total_cmp`, under which NaN outranks every
    # real, so such a sample does not abstain -- it predicts, and is counted.
    #
    # Absence is NOT a problem here, unlike every other field above, and the
    # asymmetry is deliberate: the 861 archived cells predate the guard, and
    # running them through `_number` would void the entire corpus at a stroke.
    # What absence means is "this cell was produced before the forward was
    # checked", which is a provenance fact rather than a defect -- and
    # `pre_guard_cells()` below exists so that fact stays countable instead of
    # becoming a silent hole.
    if "non_finite_forward" in cell:
        forward = _number(cell, "non_finite_forward", problems)
        if forward is not None and forward != 0:
            problems.append(f"non_finite_forward={forward}")

    problems.extend(_temporal_problems(cell, spec))
    problems.extend(_plan_agreement_problems(cell, spec))
    problems.extend(_gradient_problems(cell))
    return problems


def pre_guard_cells(cells) -> int:
    """How many cells predate the forward-finiteness guard.

    A cell without `non_finite_forward` was produced before the evaluation
    forward was checked for finiteness, so its `accuracy` carries one fewer
    guarantee than a cell recorded after 2026-08-29. That is not a defect and
    it does not void anything -- but it must be countable, because a check that
    could not run must never be reported as a check that ran and passed.
    """
    return sum(1 for cell in cells if "non_finite_forward" not in cell)


def temporal_invariants(condition: str) -> dict | None:
    """What one operator promises, keyed by the label a cell records.

    Duplicates ``TemporalCondition::invariants`` in
    ``binn-learn/src/shd_temporal.rs`` rather than reading it out of the cell,
    for the same reason ``TRAIN_BATCH_SIZE`` is duplicated: the binary is pinned
    across every wave of this campaign and must not be rebuilt to improve a
    diagnostic, and a gate that checked the binary's own claim about itself
    would be a check that cannot fail.
    ``test_campaign_tooling.py`` asserts the two tables agree.

    Returns ``None`` for a label this file does not know, which the caller
    reports as a problem -- an unrecognised operator must never be waved
    through as "no invariants to check".
    """
    if condition == "intact":
        return {
            "relocated": (0.0, 0.0),
            "max_displacement": 0.0,
            "min_mean_displacement_fraction": None,
            "hidden_permutation_relocated": (0.0, 0.0),
            "retention": ("exact",),
        }
    if condition in ("bin-shuffled", "channel-shuffled", "reversed"):
        return {
            "relocated": (RELOCATED_MIN, 1.0),
            "max_displacement": None,
            "min_mean_displacement_fraction": FULL_SHUFFLE_DISPLACEMENT_FLOOR,
            "hidden_permutation_relocated": (0.0, 0.0),
            "retention": ("exact",),
        }
    if condition == "hidden-shuffled":
        # Every input-side statistic reads exactly as `intact` does, by design.
        # `hidden_permutation_relocated` is the only field that separates the
        # two, which is why the band on it is the whole gate here.
        return {
            "relocated": (0.0, 0.0),
            "max_displacement": 0.0,
            "min_mean_displacement_fraction": None,
            "hidden_permutation_relocated": (RELOCATED_MIN, 1.0),
            "retention": ("exact",),
        }
    if condition.startswith("window-shuffled-w"):
        try:
            window = int(condition[len("window-shuffled-w"):])
        except ValueError:
            return None
        if window < 2:
            return None
        return {
            "relocated": (max(0.0, 1.0 - 1.0 / window - WINDOW_RELOCATED_SLACK), 1.0),
            "max_displacement": float(window - 1),
            "min_mean_displacement_fraction": None,
            "hidden_permutation_relocated": (0.0, 0.0),
            "retention": ("exact",),
        }
    if condition.startswith("spike-dropout-p"):
        try:
            percent = int(condition[len("spike-dropout-p"):])
        except ValueError:
            return None
        if not 1 <= percent <= 99:
            return None
        return {
            "relocated": (0.0, 0.0),
            "max_displacement": 0.0,
            "min_mean_displacement_fraction": None,
            "hidden_permutation_relocated": (0.0, 0.0),
            "retention": ("binomial", 1.0 - percent / 100.0, DROPOUT_BAND_SIGMAS),
        }
    return None


def _audit_number(audit: dict, field: str, problems: list[str]):
    value = audit.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        problems.append(f"temporal_audit.{field}={value!r} is not a number")
        return None
    return float(value)


#: Cell fields a plan entry may pin, and the plan key that pins each.
#:
#: `temporal` is handled separately because it also carries an audit. These are
#: the fields a cell records from something other than its own command line, so
#: a plan and a cell can disagree about them **without anything erroring**.
#:
#: `attn_readout` is the case that motivated this. It is read from the weight
#: file, and `train-cell` has no `--attn-readout` flag to cross-check against --
#: unlike `--arm`, which it does accept and does compare. A plan that asked for
#: a `qk-norm` cell and pointed at a default weight file would therefore produce
#: a perfectly healthy `default` cell, and it would be scored as the qk-norm arm
#: by a wave whose whole point was that the two are different instruments.
PLAN_PINNED_FIELDS = {
    "arm": "arm",
    "attn_readout": "attn_readout",
    "attn_dim": "attn_dim",
    "attn_layers": "attn_layers",
    "tau_m": "tau_m",
    "val_speakers": "val_speakers",
}


def _plan_agreement_problems(cell: dict, spec: dict | None) -> list[str]:
    """A cell that ran something other than what the plan asked for.

    Only compares keys the plan actually carries **with a value**. A plan that
    does not pin a field is not asserting anything about it, and inventing an
    expectation would void cells for a field their wave never registered.

    ``None`` counts as not pinned, and that is not a nicety. The campaign's plan
    files write ``null`` for every field that does not apply to a cell --
    ``surrogate_scale``, ``clip_grad_norm``, and ``attn_dim`` on a rate arm. Read
    as a pin, that last one voided **886 archived cells** on the first run of
    this check: their plans say ``attn_dim: null`` and the cells, correctly,
    record no ``attn_dim`` at all.
    """
    if spec is None:
        return []
    problems: list[str] = []
    for plan_key, cell_key in PLAN_PINNED_FIELDS.items():
        if spec.get(plan_key) is None:
            continue
        expected = spec[plan_key]
        if cell_key not in cell:
            problems.append(
                f"the plan pins {plan_key}={expected!r} and the cell records no {cell_key}"
            )
            continue
        observed = cell[cell_key]
        # Floats are compared with a tolerance because a plan writes 10.05 and a
        # cell records the f32 that value rounds to.
        if isinstance(expected, float) or isinstance(observed, float):
            try:
                if abs(float(observed) - float(expected)) > 1e-4:
                    problems.append(
                        f"{cell_key}={observed!r} but the plan asked for {expected!r}"
                    )
            except (TypeError, ValueError):
                problems.append(
                    f"{cell_key}={observed!r} is not comparable to the plan's {expected!r}"
                )
        elif str(observed) != str(expected):
            problems.append(f"{cell_key}={observed!r} but the plan asked for {expected!r}")
    return problems


def _temporal_problems(cell: dict, spec: dict | None) -> list[str]:
    """The manipulation ran, did what it claims, and is the one that was asked for."""
    problems: list[str] = []
    condition = cell.get("temporal_condition")
    if condition is None:
        problems.append("temporal_condition missing")
        return problems

    # A cell that ran a different condition from its plan entry is not the arm
    # it will be scored as. No previous copy compared these.
    if spec is not None and "temporal" in spec and spec["temporal"] != condition:
        problems.append(
            f"temporal_condition={condition!r} but the plan asked for {spec['temporal']!r}"
        )

    if condition == "intact":
        return problems

    invariants = temporal_invariants(condition)
    if invariants is None:
        problems.append(f"temporal_condition={condition!r} has no registered invariants")
        return problems

    audit = cell.get("temporal_audit")
    if not isinstance(audit, dict):
        problems.append("temporal_audit missing for a manipulated cell")
        return problems

    samples = audit.get("samples")
    if isinstance(samples, int) and not isinstance(samples, bool) and samples == 0:
        problems.append("temporal_audit covers zero samples - the manipulation never ran")
        return problems

    low, high = invariants["relocated"]
    relocated = _audit_number(audit, "relocated_fraction", problems)
    if relocated is not None and not low <= relocated <= high:
        problems.append(
            f"relocated_fraction={relocated:.3f} outside {condition}'s band [{low:.3f}, {high:.3f}]"
        )

    low, high = invariants["hidden_permutation_relocated"]
    if "hidden_permutation_relocated" in audit:
        permuted = _audit_number(audit, "hidden_permutation_relocated", problems)
        if permuted is not None and not low <= permuted <= high:
            problems.append(
                f"hidden_permutation_relocated={permuted:.3f} outside {condition}'s "
                f"band [{low:.3f}, {high:.3f}]"
            )
    elif high > 0.0:
        # `hidden-shuffled` is invisible in every other field, so a cell that
        # predates the field cannot be checked at all -- and unlike the other
        # clauses, there is nothing left over. Reported as a problem rather than
        # skipped: a check that could not run must not read as one that passed.
        problems.append(
            f"{condition} needs hidden_permutation_relocated and the cell has none; "
            "nothing else in the audit distinguishes it from intact"
        )

    # Fields added with the operator table. A cell recorded before them cannot
    # be checked against the clauses that need them, and saying so is not the
    # same as passing: `temporal_fields_absent` counts those cells for the
    # caller, exactly as `non_finite_forward` is counted.
    bound = invariants["max_displacement"]
    if bound is not None and "max_bin_displacement" in audit:
        observed = _audit_number(audit, "max_bin_displacement", problems)
        if observed is not None and observed > bound:
            problems.append(
                f"max_bin_displacement={observed:.1f} past {condition}'s bound of {bound:.1f}"
            )

    fraction = invariants["min_mean_displacement_fraction"]
    if fraction is not None and "mean_steps" in audit:
        steps = _audit_number(audit, "mean_steps", problems)
        mean_displacement = _audit_number(audit, "mean_bin_displacement", problems)
        if steps is not None and mean_displacement is not None:
            floor = fraction * steps
            if mean_displacement < floor:
                problems.append(
                    f"mean_bin_displacement={mean_displacement:.3f} below {condition}'s "
                    f"floor of {floor:.3f} on a {steps:.1f}-bin sequence"
                )

    rule = invariants["retention"]
    if rule[0] == "exact":
        if audit.get("counts_preserved") is not True:
            problems.append("counts not preserved")
    else:
        _, survival, sigmas = rule
        if audit.get("counts_preserved") is True:
            problems.append(f"{condition} preserved counts - it deleted nothing")
        if "count_before" in audit and "count_after" in audit:
            before = _audit_number(audit, "count_before", problems)
            after = _audit_number(audit, "count_after", problems)
            if before is not None and after is not None:
                if before <= 0.0:
                    problems.append(f"{condition} saw no spikes to delete")
                else:
                    observed = after / before
                    deviation = math.sqrt(survival * (1.0 - survival) / before) * sigmas
                    if abs(observed - survival) > deviation:
                        problems.append(
                            f"retained {observed:.6f} of {before:.0f} spikes; "
                            f"{condition} registers {survival:.6f} +/- {deviation:.6f}"
                        )
    return problems


def temporal_fields_absent(cells: list[dict]) -> int:
    """How many manipulated cells predate the per-operator audit fields.

    A cell without `max_bin_displacement` / `mean_steps` / `count_before` was
    produced before the operator table existed, so the displacement and
    retention clauses above could not run on it. That is not a defect and voids
    nothing, but a check that could not run must never be counted as a check
    that ran and passed.
    """
    needed = ("max_bin_displacement", "mean_steps", "count_before", "count_after",
              "hidden_permutation_relocated")
    absent = 0
    for cell in cells:
        if cell.get("temporal_condition") in (None, "intact"):
            continue
        audit = cell.get("temporal_audit")
        if not isinstance(audit, dict) or any(field not in audit for field in needed):
            absent += 1
    return absent


def _gradient_problems(cell: dict) -> list[str]:
    """Magnitude, which every previous copy of this gate ignored."""
    problems: list[str] = []
    trace = cell.get("epoch_max_gradient_norm")
    if trace is None:
        # Older cells predate the per-epoch traces. Absence is reported so it
        # cannot be mistaken for a clean check, but it does not void a cell
        # that could not have carried the field.
        return problems
    largest, non_finite = _finite_max(trace)
    if non_finite:
        problems.append(f"epoch_max_gradient_norm has {non_finite} non-finite entries")
    if largest is None:
        problems.append("epoch_max_gradient_norm has no finite entry")
    # Magnitude alone does not void; see the module docstring.
    return problems


#: Mirrors `let batch_size = 256usize;` in
#: `binn-lab/experiments/shd_instrument.rs`. Duplicated rather than emitted by
#: the binary because the binary is pinned across every wave of this campaign
#: and must not be rebuilt to improve a diagnostic. `test_campaign_tooling.py`
#: asserts the two stay equal.
TRAIN_BATCH_SIZE = 256


def total_optimiser_steps(cell: dict) -> int | None:
    """How many optimiser steps a cell ran, or None if it does not say.

    A clipped-step count without its denominator is unreadable: "bound on 96
    steps" is either negligible or pervasive depending on a number the reader
    does not have. Reported alongside rather than instead of the count, because
    a derived percentage that silently assumed a batch size would be worse than
    no percentage at all.
    """
    epochs = cell.get("epochs")
    n_train = cell.get("n_train")
    if not isinstance(epochs, int) or not isinstance(n_train, int):
        return None
    if epochs <= 0 or n_train <= 0:
        return None
    return epochs * -(-n_train // TRAIN_BATCH_SIZE)


def _of_total(count: int, total: int | None) -> str:
    """`96 of 12,800 (0.75%)`, or just `96` when the cell does not carry the
    fields the denominator needs. Never invents a denominator."""
    if not total:
        return str(count)
    return f"{count:,} of {total:,} ({100 * count / total:.2f}%)"


def stability_warnings(cell: dict) -> list[str]:
    """Non-voiding notes a report must print.

    Separate from :func:`validity_problems` because these do not disqualify a
    cell — they are the difference between "this arm scored lower" and "this
    arm scored lower while straining", which a mean alone cannot say.
    """
    warnings: list[str] = []
    largest, _ = _finite_max(cell.get("epoch_max_gradient_norm"))
    if largest is not None and largest > MARGINAL_GRADIENT_NORM:
        warnings.append(
            f"peak gradient norm {largest:.3e} is within five orders of f32 "
            f"overflow — the numerically marginal regime of "
            f"AMENDMENT_2026-08-05; an accuracy from this cell says as much "
            f"about the arithmetic as about the arm"
        )
    elif largest is not None and largest > WARN_GRADIENT_NORM:
        warnings.append(
            f"peak gradient norm {largest:.3e} exceeds every cell in the "
            f"recorded campaign (max 1.13e8)"
        )
    clipped_steps = cell.get("clipped_steps") or 0
    clipped_samples = cell.get("clipped_samples") or 0
    if clipped_steps or clipped_samples:
        warnings.append(
            f"gradient clipping bound: "
            f"{_of_total(clipped_steps, total_optimiser_steps(cell))} batch step(s), "
            f"{clipped_samples} sample(s) — not comparable to an unclipped arm"
        )
    unclippable = cell.get("unclippable_steps") or 0
    if unclippable:
        warnings.append(f"{unclippable} step(s) had an unrepresentable gradient norm")
    return warnings
