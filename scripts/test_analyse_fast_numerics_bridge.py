"""Tests for the frozen fast-numerics bridge analyser, on synthetic cell corpora.

Stdlib unittest, CPU only. Every verdict path has a case that reaches it, each
registered refusal has a case that triggers it, and the equivalence bars are
exercised at, just inside and just outside their boundary with per-seed deltas
that are all equal, so the bootstrap interval collapses to a point.

Run: python3 scripts/test_analyse_fast_numerics_bridge.py
"""

from __future__ import annotations

import copy
import importlib.util
import io
import json
import random
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import analyse_fast_numerics_bridge as an  # noqa: E402

PLATFORM = "macOS 27.2 (26C5042a)"
BASE = {("rate", "intact"): 0.671, ("rate", "bin-shuffled"): 0.659,
        ("attn", "intact"): 0.821, ("attn", "bin-shuffled"): 0.683}
#: Per-seed spread shared by both modes, so a blas-minus-exact delta is the offset alone.
JITTER = {s: 0.002 * ((i % 4) - 1.5) for i, s in enumerate(an.SEEDS)}


def template(kind: str, cond: str, seed: int, acc: float) -> dict:
    audit = ({"samples": 0, "counts_preserved": True, "relocated_fraction": 0.0,
              "mean_bin_displacement": 0.0, "occupied_bins_before": 0.0,
              "occupied_bins_after": 0.0}
             if cond == "intact" else
             {"samples": 10420, "counts_preserved": True, "relocated_fraction": 0.997,
              "mean_bin_displacement": 109.2, "occupied_bins_before": 302.9,
              "occupied_bins_after": 302.9})
    cell = {
        **an.FIXED, "arm": an.ARM[kind], "accuracy": float(acc), "classes_predicted": 20,
        "majority_prediction": 0.09, "mean_firing_rate": 0.13, "silent_fraction": 0.0,
        "saturated_fraction": 0.0, "mean_loss": 0.17 + seed * 1e-9, "mean_gradient_norm": 1.7,
        "mean_update_rms": 0.0003, "non_finite_events": 0, "non_finite_forward": 0,
        "clipped_steps": 0, "unclippable_steps": 0, "clip_sample_grad_norm": None,
        "clipped_samples": 0, "temporal_condition": cond, "temporal_audit": audit,
        "epoch_mean_loss": [2.8, 2.2], "epoch_mean_gradient_norm": [0.4, 2.3],
        "epoch_max_gradient_norm": [1.7, 6.3], "epoch_max_gradient_step": [30.0, 20.0],
        "tail_loss_improvement": -0.14, "mechanical_status": "COMPLETE",
        "scientific_status": "CELL_PASS", "wall_secs": 100.0,
        "emitted_unix_s": 1788432604, "emitted_utc": "2026-11-12T10:50:04Z", "seed": seed,
    }
    if kind == "attn":
        cell.update(an.ATTN_SHAPE)
    return cell


def as_blas(cell: dict, platform: str = PLATFORM) -> dict:
    return {**cell, "numerics": "blas", "numerics_platform": platform}


def as_exact(cell: dict) -> dict:
    return {**cell, "numerics": "exact"}


def acc_of(kind: str, cond: str, seed: int, offset: float = 0.0) -> float:
    return BASE[(kind, cond)] + JITTER[seed] + offset


class Built:
    """One synthetic experiment: the three directories' records, mutable before writing."""

    def __init__(self, offsets=None, per_seed=None):
        """`offsets[(kind, cond)]` shifts the blas accuracy; `per_seed[(kind, cond)][i]`
        adds a seed-specific shift. The rate arm is never shifted unless a test says so."""
        offsets = offsets or {}
        per_seed = per_seed or {}
        self.exact: dict = {}
        self.blas: dict = {}
        self.det: dict = {}
        for kind, cond, seed in an.registered_keys():
            e = template(kind, cond, seed, acc_of(kind, cond, seed))
            shift = offsets.get((kind, cond), 0.0)
            shift += per_seed.get((kind, cond), {}).get(seed, 0.0)
            b = template(kind, cond, seed, acc_of(kind, cond, seed, shift))
            self.exact[(kind, cond, seed)] = as_exact(e)
            self.blas[(kind, cond, seed)] = as_blas(b)
        for seed in an.DET_SEEDS:
            main = self.blas[("attn", "intact", seed)]
            for t in an.DET_THREADS:
                for r in an.DET_RERUNS:
                    self.det[(seed, t, r)] = {**main, "wall_secs": 50.0 + t + r,
                                              "emitted_unix_s": 1788432999 + t + r}

    def write(self, root: Path) -> tuple[Path, Path, Path]:
        dirs = []
        for name, cells in (("exact", self.exact), ("blas", self.blas), ("det", self.det)):
            d = root / name
            d.mkdir()
            for i, (key, rec) in enumerate(sorted(cells.items(), key=lambda kv: str(kv[0]))):
                suffix = f"__t{key[1]}__r{key[2]}" if name == "det" else ""
                (d / f"c{i:03d}{suffix}.json").write_text(json.dumps(rec))
            dirs.append(d)
        return tuple(dirs)


def run(built: Built, mutate=None) -> tuple[int, str, dict]:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        exact, blas, det = built.write(root)
        if mutate:
            mutate(exact, blas, det)
        out_json = root / "verdicts.json"
        out_md = root / "table.md"
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = an.main([str(exact), str(blas), "--determinism", str(det),
                            "--json", str(out_json), "--markdown", str(out_md)])
        data = json.loads(out_json.read_text()) if out_json.exists() else {}
        data["_markdown"] = out_md.read_text() if out_md.exists() else ""
    return code, buf.getvalue(), data


def drop_file(directory: Path, pattern: str) -> None:
    victims = sorted(directory.glob(pattern))
    victims[0].unlink()


class TemplateIsValid(unittest.TestCase):
    def test_synthetic_cells_pass_the_repository_validity_gate(self):
        problems = [an.validity_problems(template(k, c, 5170001, 0.7))
                    for k in an.KINDS for c in an.CONDITIONS]
        self.assertEqual(problems, [[], [], [], []])


class RegisteredConstants(unittest.TestCase):
    def test_bars_and_design_are_the_registered_ones(self):
        self.assertEqual((an.LEVEL_BAR, an.DELTA_G, an.DELTA_DID, an.ALPHA, an.RESAMPLES),
                         (0.03, 0.015, 0.02, 0.05, 10_000))
        self.assertEqual((len(an.SEEDS), an.SEEDS[0], an.SEEDS[-1]), (12, 5170001, 5170012))
        self.assertEqual((an.DET_SEEDS, an.DET_THREADS, an.DET_RERUNS),
                         ((5170001, 5170002), (1, 16), (1, 2)))
        self.assertEqual(len(an.registered_keys()), 48)

    def test_fleet_pins_reproduce_the_published_gain_and_did(self):
        f = an.FLEET
        self.assertEqual(round(f["G"], 4), 0.1499)
        self.assertEqual(round(f["DiD"], 4), 0.1260)
        self.assertAlmostEqual(f["G"], f["attn_intact"] - f["rate_intact"], delta=2e-9)
        self.assertAlmostEqual(
            f["DiD"], f["G"] - (f["attn_shuffled"] - f["rate_shuffled"]), delta=2e-9)

    def test_clock_fields_are_the_fixture_tool_s(self):
        spec = importlib.util.spec_from_file_location(
            "train_cell_fixtures_for_parity", Path(__file__).resolve().parent / "train_cell_fixtures.py")
        fixtures = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = fixtures
        spec.loader.exec_module(fixtures)
        self.assertEqual(an.CLOCK_FIELDS, fixtures.NONDETERMINISTIC_CELL_FIELDS)


class BootstrapMatchesTheRepositoryMachinery(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location(
            "did_confidence_intervals_for_parity",
            Path(__file__).resolve().parent / "did_confidence_intervals.py")
        cls.dci = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = cls.dci
        spec.loader.exec_module(cls.dci)

    def test_percentile_interval_is_the_repository_one(self):
        draws = [random.Random(7).random() for _ in range(1000)]
        self.assertEqual(an.percentile_interval(draws, 0.10),
                         self.dci.percentile_interval(draws, 0.10))

    def test_paired_intervals_are_the_repository_bootstrap_on_the_same_stream(self):
        g = [0.004 * ((i * 7) % 5 - 2) for i in range(12)]
        d = [0.009 * ((i * 3) % 7 - 3) for i in range(12)]
        quads = {i: (g[i], 0.0, d[i]) for i in range(12)}
        theirs = self.dci.bootstrap(quads, random.Random(an.RNG_SEED), an.RESAMPLES, alpha=0.10)
        ours_g, ours_d = an.paired_intervals(g, d)
        self.assertEqual((ours_g, ours_d), (theirs["gain_intact_ci"], theirs["did_ci"]))

    def test_ninety_percent_tails_sit_at_elements_500_and_9499(self):
        self.assertEqual(an.percentile_interval(list(range(10_000)), 0.10), (500, 9499))


class EquivalenceBoundaries(unittest.TestCase):
    """Equal per-seed deltas make the interval a point, so the bar is hit exactly."""

    def interval_for(self, d_g: float, d_did: float):
        return an.paired_intervals([d_g] * 12, [d_did] * 12, resamples=200)

    def test_strict_at_the_upper_margin(self):
        ci_g, _ = self.interval_for(an.DELTA_G, 0.0)
        self.assertFalse(an.equivalent(ci_g, an.DELTA_G))

    def test_strict_at_the_lower_margin(self):
        ci_g, _ = self.interval_for(-an.DELTA_G, 0.0)
        self.assertFalse(an.equivalent(ci_g, an.DELTA_G))

    def test_just_inside_both_margins_passes(self):
        inside = an.paired_intervals([an.DELTA_G - 1e-6] * 12, [-(an.DELTA_DID - 1e-6)] * 12, resamples=200)
        self.assertEqual([an.equivalent(inside[0], an.DELTA_G), an.equivalent(inside[1], an.DELTA_DID)],
                         [True, True])

    def test_just_outside_both_margins_fails(self):
        outside = an.paired_intervals([-(an.DELTA_G + 1e-6)] * 12, [an.DELTA_DID + 1e-6] * 12, resamples=200)
        self.assertEqual([an.equivalent(outside[0], an.DELTA_G), an.equivalent(outside[1], an.DELTA_DID)],
                         [False, False])

    def test_did_margin_is_wider_than_g_s(self):
        # 0.017 is outside G's +-0.015 and inside the DiD's +-0.02.
        ci_g, ci_d = self.interval_for(0.017, 0.017)
        self.assertEqual([an.equivalent(ci_g, an.DELTA_G), an.equivalent(ci_d, an.DELTA_DID)],
                         [False, True])


class Verdicts(unittest.TestCase):
    def test_equivalent_builds_pass_every_clause(self):
        code, out, data = run(Built())
        self.assertEqual((code, data["HF-0"]["status"], data["HF-1"]["status"], data["HF-2"]["status"]),
                         (0, "PASS", "PASS", "PASS"))
        self.assertIn("Licensed sentence", out)
        self.assertEqual(data["numerics_platform"], PLATFORM)
        self.assertEqual(data["HF-2"]["G"]["delta"], 0.0)

    def test_a_small_real_offset_still_passes(self):
        # +0.004 on both attention arms: G delta +0.004, DiD delta 0.
        code, _, data = run(Built({("attn", "intact"): 0.004, ("attn", "bin-shuffled"): 0.004}))
        self.assertEqual((code, data["HF-2"]["G"]["pass"], data["HF-2"]["DiD"]["pass"]), (0, True, True))
        self.assertAlmostEqual(data["HF-2"]["G"]["delta"], 0.004, delta=1e-9)

    def test_g_just_over_the_margin_fails_hf2_and_names_g(self):
        code, out, data = run(Built({("attn", "intact"): 0.016, ("attn", "bin-shuffled"): 0.016}))
        self.assertEqual((code, data["HF-2"]["G"]["pass"], data["HF-2"]["DiD"]["pass"], data["stopped_at"]),
                         (5, False, True, "HF-2"))
        self.assertIn("NOT shown equivalent", out)

    def test_did_just_over_its_margin_fails_hf2_while_g_passes(self):
        code, _, data = run(Built({("attn", "bin-shuffled"): -0.021}))
        self.assertEqual((code, data["HF-2"]["G"]["pass"], data["HF-2"]["DiD"]["pass"]),
                         (5, True, False))

    def test_did_delta_between_the_two_margins_passes_through_the_full_read(self):
        # DiD delta +0.017 is outside G's 0.015 and inside the DiD's 0.02.
        code, _, data = run(Built({("attn", "bin-shuffled"): -0.017}))
        self.assertEqual((code, data["HF-2"]["G"]["pass"], data["HF-2"]["DiD"]["pass"]), (0, True, True))
        self.assertAlmostEqual(data["HF-2"]["DiD"]["delta"], 0.017, delta=1e-9)

    def test_high_variance_fails_equivalence_even_with_a_zero_mean(self):
        swing = {s: (0.05 if i % 2 else -0.05) for i, s in enumerate(an.SEEDS)}
        code, _, data = run(Built(per_seed={("attn", "intact"): swing}))
        self.assertEqual((code, data["HF-1"]["status"], data["HF-2"]["status"]), (5, "PASS", "FAIL"))
        self.assertAlmostEqual(data["HF-2"]["G"]["delta"], 0.0, delta=1e-9)

    def test_level_shift_over_the_bar_stops_at_hf1_and_reads_no_hf2(self):
        code, out, data = run(Built({("attn", "intact"): 0.031}))
        self.assertEqual((code, data["HF-1"]["status"], data["stopped_at"], "HF-2" in data),
                         (4, "FAIL", "HF-1", False))
        self.assertIn("self-contained waves", out)

    def test_level_shift_exactly_at_the_bar_passes_hf1(self):
        code, _, data = run(Built({("attn", "intact"): 0.03}))
        self.assertEqual((data["HF-1"]["status"], code), ("PASS", 5))  # hf1 ok, but G fails hf2

    def test_clock_fields_never_make_cells_differ(self):
        built = Built()
        built.blas[("rate", "intact", 5170003)]["wall_secs"] = 12345.0
        built.blas[("rate", "intact", 5170003)]["emitted_utc"] = "2030-01-01T00:00:00Z"
        code, _, data = run(built)
        self.assertEqual((code, data["HF-0"]["b"]["pass"]), (0, True))

    def test_markdown_carries_the_verdict_rows(self):
        _, _, data = run(Built())
        md = data["_markdown"]
        self.assertEqual([("| HF-2 G | PASS" in md), ("| HF-2 DiD | PASS" in md), ("| HF-0 | PASS" in md)],
                         [True, True, True])


class DeterminismAndRateIdentity(unittest.TestCase):
    def test_a_determinism_cell_that_differs_fails_hf0_and_names_the_witness(self):
        built = Built()
        built.det[(5170002, 16, 2)]["mean_loss"] += 1e-9
        code, out, data = run(built)
        row = data["HF-0"]["a"]["seeds"][1]
        self.assertEqual((code, data["HF-0"]["status"], row["identical"], row["differing"]),
                         (3, "FAIL", False, {"t16r2": ["mean_loss"]}))
        self.assertEqual(("HF-1" in data, data["stopped_at"]), (False, "HF-0"))
        self.assertIn("Mode withdrawn", out)

    def test_the_main_wave_cell_is_a_fifth_witness(self):
        built = Built()
        built.blas[("attn", "intact", 5170001)]["accuracy"] += 0.0004
        code, _, data = run(built)
        self.assertEqual((code, data["HF-0"]["a"]["seeds"][0]["differing"]),
                         (3, {"main": ["accuracy"]}))

    def test_a_blas_rate_cell_that_differs_fails_hf0b(self):
        built = Built()
        built.blas[("rate", "bin-shuffled", 5170009)]["mean_firing_rate"] += 1e-9
        code, _, data = run(built)
        failing = data["HF-0"]["b"]["failing"]
        self.assertEqual((code, data["HF-0"]["b"]["pass"], len(failing), failing[0]["differing"]),
                         (3, False, 1, ["mean_firing_rate"]))

    def test_a_fleet_style_exact_rate_arm_fails_the_platform_rule(self):
        # The same arm computed with another libm differs by a few ulps of accuracy.
        built = Built()
        for seed in an.SEEDS:
            built.exact[("rate", "intact", seed)]["accuracy"] += 0.0003
        code, _, data = run(built)
        self.assertEqual((code, data["HF-0"]["b"]["pass"], len(data["HF-0"]["b"]["failing"])),
                         (3, False, 12))

    def test_hf0_failure_is_read_even_when_later_cells_are_missing(self):
        built = Built()
        built.det[(5170001, 1, 1)]["mean_loss"] += 1e-9
        code, _, data = run(built, lambda e, b, d: drop_file(b, "c*.json"))
        self.assertEqual(code, 3)  # not 2: HF-0 does not need the attention cells


class Incomplete(unittest.TestCase):
    def test_empty_corpora_are_incomplete_not_a_verdict(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name in ("e", "b", "d"):
                Path(tmp, name).mkdir()
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = an.main([f"{tmp}/e", f"{tmp}/b", "--determinism", f"{tmp}/d"])
        self.assertEqual(code, 2)
        self.assertIn("INCOMPLETE", buf.getvalue())

    def test_a_missing_seed_in_the_blas_corpus_gives_no_hf1_or_hf2(self):
        built = Built()
        del built.blas[("attn", "bin-shuffled", 5170012)]
        code, out, data = run(built)
        self.assertEqual((code, data["HF-0"]["status"], data["HF-1"]["status"], "HF-2" in data,
                          data["complete"]), (2, "PASS", "NOT EVALUABLE", False, False))
        self.assertIn("INCOMPLETE", out)
        self.assertIn("5170012", data["HF-1"]["missing"][0])

    def test_a_missing_seed_in_the_exact_corpus_is_named(self):
        built = Built()
        del built.exact[("attn", "intact", 5170003)]
        code, _, data = run(built)
        self.assertEqual((code, data["HF-1"]["missing"]), (2, ["exact ('attn', 'intact', 5170003)"]))

    def test_a_missing_determinism_cell_makes_hf0_not_evaluable(self):
        built = Built()
        del built.det[(5170002, 16, 1)]
        code, out, data = run(built)
        self.assertEqual((code, data["HF-0"]["status"], data["HF-0"]["missing"], "HF-1" in data),
                         (2, "NOT EVALUABLE", ["determinism (5170002, 16, 1)"], False))
        self.assertIn("INCOMPLETE", out)


class Refusals(unittest.TestCase):
    def invalid(self, built: Built, mutate=None) -> str:
        code, out, data = run(built, mutate)
        self.assertEqual((code, data), (1, {"_markdown": ""}))
        return out

    def test_a_blas_cell_in_the_exact_corpus(self):
        built = Built()
        built.exact[("attn", "intact", 5170004)] = built.blas[("attn", "intact", 5170004)]
        self.assertIn("mixed numerics", self.invalid(built))

    def test_an_exact_cell_in_the_blas_corpus(self):
        built = Built()
        built.blas[("rate", "intact", 5170004)] = built.exact[("rate", "intact", 5170004)]
        self.assertIn("mixed numerics", self.invalid(built))

    def test_an_unknown_numerics_value(self):
        built = Built()
        built.blas[("rate", "intact", 5170004)]["numerics"] = "fma"
        # cell_validity refuses an unregistered mode before the corpus check
        # runs; either refusal is an INVALID, never a verdict.
        self.assertIn("numerics='fma' is not one of exact, blas", self.invalid(built))

    def test_mixed_platforms_within_the_blas_corpus(self):
        built = Built()
        built.blas[("rate", "intact", 5170004)]["numerics_platform"] = "macOS 27.3 (26D1)"
        self.assertIn("mixed numerics_platform", self.invalid(built))

    def test_mixed_platforms_between_the_blas_and_determinism_corpora(self):
        built = Built()
        built.det[(5170001, 1, 1)]["numerics_platform"] = "macOS 27.3 (26D1)"
        self.assertIn("mixed numerics_platform", self.invalid(built))

    def test_a_blas_cell_with_no_platform(self):
        built = Built()
        del built.blas[("rate", "intact", 5170004)]["numerics_platform"]
        self.assertIn("numerics_platform", self.invalid(built))

    def test_an_exact_cell_carrying_a_platform(self):
        built = Built()
        built.exact[("rate", "intact", 5170004)]["numerics_platform"] = PLATFORM
        self.assertIn("carries numerics_platform", self.invalid(built))

    def test_a_duplicate_cell(self):
        def dup(exact, blas, det):
            src = sorted(exact.glob("c*.json"))[0]
            (exact / "copy.json").write_text(src.read_text())
        self.assertIn("duplicate cell", self.invalid(Built(), dup))

    def test_an_unregistered_seed(self):
        built = Built()
        built.exact[("rate", "intact", 5170004)]["seed"] = 5170013
        self.assertIn("not a registered seed", self.invalid(built))

    def test_a_hidden_shuffled_cell(self):
        built = Built()
        built.blas[("attn", "bin-shuffled", 5170004)]["temporal_condition"] = "hidden-shuffled"
        self.assertIn("hidden-shuffled", self.invalid(built))

    def test_a_cell_from_another_operating_point(self):
        built = Built()
        built.exact[("rate", "intact", 5170004)]["hidden"] = 256
        self.assertIn("hidden = 256", self.invalid(built))

    def test_a_cell_failing_the_repository_validity_gate(self):
        built = Built()
        built.blas[("attn", "intact", 5170004)]["non_finite_events"] = 3
        self.assertIn("cell_validity", self.invalid(built))

    def test_a_determinism_file_without_thread_and_rerun_tokens(self):
        def rename(exact, blas, det):
            sorted(det.glob("c*.json"))[0].rename(det / "plain.json")
        self.assertIn("__t<threads>__r<rerun>", self.invalid(Built(), rename))

    def test_a_determinism_cell_at_an_unregistered_thread_count(self):
        def rename(exact, blas, det):
            sorted(det.glob("c*.json"))[0].rename(det / "x__t3__r1.json")
        self.assertIn("is not registered", self.invalid(Built(), rename))

    def test_a_determinism_cell_that_is_not_attention_intact(self):
        built = Built()
        built.det[(5170001, 1, 1)] = {**built.blas[("rate", "intact", 5170001)]}
        self.assertIn("determinism cells are attention intact", self.invalid(built))

    def test_a_missing_directory(self):
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = an.main(["/nonexistent/exact", "/nonexistent/blas", "--determinism", "/nonexistent/det"])
        self.assertEqual((code, "is not a directory" in buf.getvalue()), (1, True))

    def test_an_unreadable_record(self):
        def corrupt(exact, blas, det):
            sorted(blas.glob("c*.json"))[0].write_text("{not json")
        self.assertIn("unreadable", self.invalid(Built(), corrupt))

    def test_the_whole_fleet_directory_cannot_be_passed_as_exact(self):
        # A fleet cell has the right fields but is not one of the registered cells' numerics
        # twin; here an extra record from another wave's operating point stands in for it.
        def pollute(exact, blas, det):
            other = copy.deepcopy(json.loads(sorted(exact.glob("c*.json"))[0].read_text()))
            other["contract"] = "published-10ms"
            (exact / "w22cov__other.json").write_text(json.dumps(other))
        self.assertIn("contract", self.invalid(Built(), pollute))


if __name__ == "__main__":
    unittest.main()
