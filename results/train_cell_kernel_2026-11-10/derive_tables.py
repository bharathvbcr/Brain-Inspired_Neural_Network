"""Recompute every number in README.md from the raw files beside it.

    python3 -I results/train_cell_kernel_2026-11-10/derive_tables.py

Reads `head/sample.txt` (raw `/usr/bin/sample` output) and the `abba3/`, `abba4/`
`/usr/bin/time -l` files; prints the figures the README quotes. Nothing is
copied from a log's own summary line.
"""

import glob
import os
import re
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ORACLE = "9c364bd9441d1406cd97aa15078a83c1042b93180963f6bf159652528df512d3"


def sample_shares() -> None:
    lines = (HERE / "head" / "sample.txt").read_text(errors="replace").split("\n")
    i = next(k for k, line in enumerate(lines) if line.startswith("Sort by top of stack"))
    j = next(k for k, line in enumerate(lines) if line.startswith("Binary Images"))
    rows = []
    for line in lines[i + 1:j]:
        m = re.match(r"\s+(\S.*?)\s+\(in [^)]*\)\s+(\d+)\s*$", line)
        if m:
            rows.append((m.group(1), int(m.group(2))))

    def get(name: str, exact: bool = True) -> int:
        return sum(n for s, n in rows if (s == name if exact else name in s))

    total = sum(n for _, n in rows)
    grad = get("shd_attention18attention_gradient", False)
    fwd = get("27attention_forward_presented", False)
    expf = get("expf") + get("DYLD-STUB$$expf")
    idle = get("__psynch_cvwait")
    spin = get("swtch_pri")
    print(f"sample.txt: {len(rows)} top-of-stack rows, {total} samples")
    print(f"  attention_gradient           {grad:7d}")
    print(f"  attention_forward_presented  {fwd:7d}")
    print(f"  expf (+ DYLD-STUB$$expf)     {expf:7d}")
    print(f"  __psynch_cvwait (idle)       {idle:7d}")
    print(f"  swtch_pri (rayon spin yield) {spin:7d}")
    for label, denom in (("all samples", total),
                         ("minus cvwait", total - idle),
                         ("minus cvwait and swtch_pri", total - idle - spin)):
        print(f"  denominator = {label}: {denom}")
        for name, n in (("gradient", grad), ("forward", fwd), ("gradient+forward", grad + fwd),
                        ("expf", expf), ("cvwait", idle)):
            print(f"    {name:18s} {100 * n / denom:6.2f}%")


def abba_table(directory: str) -> None:
    sha_by_run = {}
    for line in (HERE / directory / "weights.sha256").read_text().splitlines():
        sha, name = line.split()
        sha_by_run[name.removesuffix(".weights")] = sha
    rows = defaultdict(list)
    for path in sorted(glob.glob(str(HERE / directory / "*.time"))):
        run = os.path.basename(path).removesuffix(".time")
        index, tag = run.split(".", 1)
        text = Path(path).read_text()
        instructions = int(re.search(r"(\d+)\s+instructions retired", text).group(1))
        cycles = int(re.search(r"(\d+)\s+cycles elapsed", text).group(1))
        user = float(re.search(r"([\d.]+)\s+user", text).group(1))
        rows[tag].append((int(index), instructions, cycles, user, sha_by_run[run] == ORACLE))
    print(f"{directory}:")
    for tag, runs in rows.items():
        print(f"  {tag:8s} n={len(runs)} oracle={all(r[4] for r in runs)} "
              f"instructions min={min(r[1] for r in runs):.4e} "
              f"cycles min={min(r[2] for r in runs):.4e} max={max(r[2] for r in runs):.4e} "
              f"user min={min(r[3] for r in runs):.2f}s max={max(r[3] for r in runs):.2f}s "
              f"runs={[r[0] for r in runs]}")


if __name__ == "__main__":
    sample_shares()
    abba_table("abba3")
    abba_table("abba4")
