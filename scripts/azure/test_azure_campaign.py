#!/usr/bin/env python3
"""Mechanical tests for the Azure campaign plan and sharding."""

from __future__ import annotations

import unittest
from pathlib import Path

from scripts.aws.plan_cells import estimated_seconds
from scripts.azure.launch import DEFAULT_HOURS, DEFAULT_SKU
from scripts.azure.plan_cells import BUDGETS, CONTRACTS, NODE_COUNT, WIDTHS, planned_cells
from scripts.azure.run_shard import (
    ScheduleError,
    shard,
    simulate_schedule,
    thread_count_for_cell,
)


class PlanTests(unittest.TestCase):
    def test_plan_is_injective_and_has_registered_size(self) -> None:
        cells = planned_cells()
        self.assertEqual(len(cells), 252)
        self.assertEqual(len({cell["id"] for cell in cells}), 252)

    def test_every_confirmatory_pair_has_twelve_shared_seeds(self) -> None:
        cells = planned_cells()
        for hidden in WIDTHS:
            budgets = BUDGETS if hidden == 128 else (400,)
            for epochs in budgets:
                control = {c["seed"] for c in cells if c["wave"] == "az8wid"
                           and c["hidden"] == hidden and c["epochs"] == epochs
                           and c["arm"] == "ff+fixed"}
                treatment = {c["seed"] for c in cells if c["wave"] == "az8wid"
                             and c["hidden"] == hidden and c["epochs"] == epochs
                             and c["arm"] == "ff+fixed+attn"}
                self.assertEqual(control, treatment)
                self.assertEqual(len(control), 12)

    def test_contract_axis_is_complete(self) -> None:
        cells = planned_cells()
        observed = {c["contract"] for c in cells if c["hidden"] == 128
                    and c["geometry"] == "adjacent-sum-5"}
        self.assertEqual(observed, set(CONTRACTS))

    def test_shards_are_disjoint_and_complete(self) -> None:
        cells = planned_cells()
        shards = [shard(cells, index, NODE_COUNT) for index in range(NODE_COUNT)]
        self.assertEqual(
            [len(entries) for entries in shards],
            [63, 63, 63, 63],
        )
        ids = [entry["id"] for entries in shards for entry in entries]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual(set(ids), {entry["id"] for entry in cells})

    def test_plan_is_longest_processing_time_ordered(self) -> None:
        estimates = [estimated_seconds(cell) for cell in planned_cells()]
        self.assertEqual(estimates, sorted(estimates, reverse=True))

    def test_cluster_defaults_fill_the_approved_regional_quota(self) -> None:
        self.assertEqual(NODE_COUNT, 4)
        self.assertEqual(DEFAULT_SKU, "Standard_D64als_v7")
        self.assertEqual(DEFAULT_HOURS, 19.0)

    def test_bootstrap_requires_azure_local_quorum_despite_cross_platform_fail(self) -> None:
        bootstrap = Path("scripts/azure/bootstrap.sh").read_text()
        self.assertIn('--expected-cross-platform-status FAIL', bootstrap)
        self.assertIn('--expected-gate-cells 3', bootstrap)
        self.assertIn('Azure-local Gate F quorum PASS', bootstrap)
        self.assertIn('did not form within 30 minutes', bootstrap)
        self.assertIn('deallocate_all || true', bootstrap)
        self.assertNotIn('if [[ "$GATE_STATUS" != PASS ]]; then', bootstrap)

    def test_launch_pins_gate_reports_to_frozen_source(self) -> None:
        launcher = Path("scripts/azure/launch.py").read_text()
        self.assertIn('"SOURCE_SHA256": source_sha', launcher)
        self.assertIn('"scientific_fields_must_be_bit_identical": True', launcher)

    def test_role_lookup_bypasses_graph_for_fresh_managed_identity(self) -> None:
        launcher = Path("scripts/azure/launch.py").read_text()
        self.assertIn('"--assignee-object-id", principal', launcher)
        self.assertIn('"--fill-principal-name", "false"', launcher)

    def test_vmss_disable_overprovision_uses_current_cli_switch_form(self) -> None:
        launcher = Path("scripts/azure/launch.py").read_text()
        self.assertIn('"--disable-overprovision", "--admin-username"', launcher)
        self.assertNotIn('"--disable-overprovision", "true"', launcher)

    def test_bootstrap_derives_numeric_shard_from_imds_resource_id(self) -> None:
        bootstrap = Path("scripts/azure/bootstrap.sh").read_text()
        self.assertIn("metadata 'instance/compute/resourceId'", bootstrap)
        self.assertIn('NODE_INDEX="${INSTANCE_RESOURCE_ID##*/}"', bootstrap)
        self.assertNotIn("metadata 'instance/compute/instanceId'", bootstrap)

    def test_nat_egress_is_explicit_and_attached_only_to_worker_subnet(self) -> None:
        launcher = Path("scripts/azure/launch.py").read_text()
        self.assertIn('parser.add_argument("--enable-outbound-nat", action="store_true")', launcher)
        self.assertIn('"public-ip", "create"', launcher)
        self.assertIn('"--allocation-method", "Static", "--sku", "Standard"', launcher)
        self.assertIn('"nat", "gateway", "create"', launcher)
        self.assertIn('"subnet", "update"', launcher)
        self.assertIn('"--nat-gateway", NAT_GATEWAY', launcher)
        self.assertIn('"--default-outbound", "false"', launcher)

    def test_bootstrap_does_not_source_rustup_env_with_unset_home(self) -> None:
        bootstrap = Path("scripts/azure/bootstrap.sh").read_text()
        self.assertIn('BINN_CARGO_HOME=/root/.cargo', bootstrap)
        self.assertIn('export PATH="$BINN_CARGO_HOME/bin:$PATH"', bootstrap)
        self.assertNotIn('source "$BINN_CARGO_HOME/env"', bootstrap)

    def test_only_registered_wide_attention_cells_receive_eight_threads(self) -> None:
        normal = {"hidden": 1024, "attn_dim": 32}
        wide = {"hidden": 1024, "attn_dim": 64}
        self.assertEqual(thread_count_for_cell(normal, 4, 8), 4)
        self.assertEqual(thread_count_for_cell(wide, 4, 8), 8)

    def test_resource_scheduler_fills_but_never_oversubscribes_d64(self) -> None:
        cells = ([{"id": f"normal-{index}", "hidden": 128, "attn_dim": 32}
                  for index in range(12)]
                 + [{"id": f"wide-{index}", "hidden": 1024, "attn_dim": 64}
                    for index in range(2)])
        schedule = simulate_schedule(
            cells,
            host_cores=64,
            max_concurrency=16,
            normal_threads=4,
            wide_threads=8,
            duration_seconds=lambda _cell, _threads: 10.0,
        )
        self.assertEqual(schedule["peak_reserved_cores"], 64)
        self.assertLessEqual(schedule["peak_reserved_cores"], 64)
        self.assertLessEqual(schedule["peak_running_cells"], 16)

    def test_resource_scheduler_rejects_a_cell_larger_than_the_host(self) -> None:
        with self.assertRaises(ScheduleError):
            simulate_schedule(
                [{"id": "wide", "hidden": 1024, "attn_dim": 64}],
                host_cores=4,
                max_concurrency=1,
                normal_threads=4,
                wide_threads=8,
                duration_seconds=lambda _cell, _threads: 1.0,
            )


if __name__ == "__main__":
    unittest.main()
