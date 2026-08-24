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

    problems.extend(_temporal_problems(cell, spec))
    problems.extend(_gradient_problems(cell))
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

    audit = cell.get("temporal_audit")
    if not isinstance(audit, dict):
        problems.append("temporal_audit missing for a manipulated cell")
        return problems
    if audit.get("counts_preserved") is not True:
        problems.append("counts not preserved")
    relocated = audit.get("relocated_fraction")
    if not isinstance(relocated, (int, float)) or isinstance(relocated, bool):
        problems.append(f"relocated_fraction={relocated!r} is not a number")
    elif relocated < RELOCATED_MIN:
        problems.append(f"relocated_fraction={relocated:.3f}")
    return problems


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
            f"gradient clipping bound: {clipped_steps} batch step(s), "
            f"{clipped_samples} sample(s) — not comparable to an unclipped arm"
        )
    unclippable = cell.get("unclippable_steps") or 0
    if unclippable:
        warnings.append(f"{unclippable} step(s) had an unrepresentable gradient norm")
    return warnings
