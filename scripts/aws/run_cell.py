#!/usr/bin/env python3
"""Run one planned cell: initialise, then train.

Every flag is derived from the plan entry, so the cell id and the command line
cannot drift apart. Anything the plan does not specify is left at the
instrument's own default rather than restated here.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

#: Matches every other helper in this directory; `test_campaign_tooling.py`
#: asserts the copies agree, and caught this file introducing 60.
AWS_TIMEOUT_S = 300


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cell_id")
    parser.add_argument("--plan", default="cells.json")
    parser.add_argument("--bucket", default=os.environ.get("BUCKET"),
                        help="re-read the published queue from here when the "
                             "cell is absent from the local plan")
    parser.add_argument("--work", required=True)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--threads", default="16")
    parser.add_argument("--events", default="data/shd/events")
    args = parser.parse_args()

    # `--plan` is the copy fetched once at boot. `claim_next.py` re-reads the
    # PUBLISHED queue on every claim, so a wave appended mid-campaign is claimed
    # here and then rejected by a plan that has never heard of it. That is not
    # hypothetical: on 2026-08-27 wave 20 was appended, all eighty cells were
    # claimed, and all eighty failed instantly with "is not in the plan" --
    # consuming their claims, so they were dead rather than retried.
    #
    # Fixing claim_next alone was half a fix. Both readers of the queue have to
    # agree on which queue they are reading.
    plan = {c["id"]: c for c in json.load(open(args.plan))}
    if args.cell_id not in plan and args.bucket:
        try:
            fetched = subprocess.run(
                ["aws", "s3", "cp", f"s3://{args.bucket}/input/cells.json", "-"],
                capture_output=True, text=True, timeout=AWS_TIMEOUT_S)
            if fetched.returncode == 0:
                published = {c["id"]: c for c in json.loads(fetched.stdout)}
                if args.cell_id in published:
                    print(f"run_cell: {args.cell_id} is absent from {args.plan} "
                          f"but present in the published queue; using the "
                          f"published entry", file=sys.stderr)
                    plan = published
        except (subprocess.TimeoutExpired, ValueError, OSError) as exc:
            print(f"run_cell: could not re-read the published queue ({exc})",
                  file=sys.stderr)
    if args.cell_id not in plan:
        raise SystemExit(f"{args.cell_id} is not in the plan")
    spec = plan[args.cell_id]

    work = Path(args.work)
    work.mkdir(parents=True, exist_ok=True)
    weights, orders = work / "w.bin", work / "o.bin"
    env = dict(os.environ, BINN_CAMPAIGN_AUTHORIZED="1", RAYON_NUM_THREADS=args.threads)

    init = [args.binary, "init",
            "--n-inputs", str(spec["n_inputs"]), "--hidden", str(spec["hidden"]),
            "--classes", "20", "--seed", str(spec["seed"]),
            "--epochs", str(spec["epochs"]), "--n-train", str(spec["n_train"]),
            "--arm", spec["arm"], "--weights", str(weights), "--orders", str(orders)]
    if spec["attn_dim"]:
        init += ["--attn-dim", str(spec["attn_dim"]), "--attn-layers", str(spec["attn_layers"])]
    subprocess.run(init, check=True, env=env)

    train = [args.binary, "train-cell",
             "--train-events", f"{args.events}/train.events",
             "--test-events", f"{args.events}/test.events",
             "--contract", spec["contract"], "--geometry", spec["geometry"],
             "--weights", str(weights), "--orders", str(orders),
             "--epochs", str(spec["epochs"]), "--arm", spec["arm"],
             "--out", str(work / "cell.json"),
             # Provenance label. The seed that determined the run was consumed by
             # `init`; without this the cell records every parameter EXCEPT which
             # seed produced it, and the filename becomes the only witness.
             # Ignored by binaries predating the flag, so a pinned older binary
             # still runs unchanged.
             "--seed", str(spec["seed"])]
    if spec["temporal"] != "intact":
        train += ["--temporal", spec["temporal"], "--temporal-seed", str(spec["temporal_seed"])]
    if spec["surrogate_scale"] is not None:
        train += ["--surrogate-scale", str(spec["surrogate_scale"])]
    if spec["clip_grad_norm"] is not None:
        train += ["--clip-grad-norm", str(spec["clip_grad_norm"])]
    subprocess.run(train, check=True, env=env)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
