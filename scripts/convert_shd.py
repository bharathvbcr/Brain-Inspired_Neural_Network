#!/usr/bin/env python3
"""Retired: use the Rust convert-shd binary instead of this h5py script.

    PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \\
      cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \\
        --cache-dir data/shd

See data/shd/README.md.
"""
from __future__ import annotations
import sys

print(
    "scripts/convert_shd.py is retired.\n"
    "Use:\n"
    '  PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \\\n'
    "    cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \\\n"
    "      --cache-dir data/shd",
    file=sys.stderr,
)
raise SystemExit(2)
