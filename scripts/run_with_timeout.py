#!/usr/bin/env python3
"""Portable process-group timeout for macOS/Linux overnight jobs."""

from __future__ import annotations

import os
import signal
import subprocess
import sys


TIMEOUT_EXIT = 124
TERMINATION_GRACE_SECONDS = 10


def terminate_group(proc: subprocess.Popen[bytes], sig: signal.Signals) -> None:
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        pass


def main() -> int:
    if len(sys.argv) < 3:
        print(
            "usage: run_with_timeout.py SECONDS COMMAND [ARG ...]",
            file=sys.stderr,
        )
        return 2

    try:
        timeout_seconds = float(sys.argv[1])
    except ValueError:
        print(f"invalid timeout: {sys.argv[1]!r}", file=sys.stderr)
        return 2
    if timeout_seconds <= 0:
        print("timeout must be positive", file=sys.stderr)
        return 2

    try:
        proc = subprocess.Popen(sys.argv[2:], start_new_session=True)
    except FileNotFoundError:
        print(f"command not found: {sys.argv[2]}", file=sys.stderr)
        return 127

    try:
        return proc.wait(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        terminate_group(proc, signal.SIGTERM)
        try:
            proc.wait(timeout=TERMINATION_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            terminate_group(proc, signal.SIGKILL)
            proc.wait()
        return TIMEOUT_EXIT
    except KeyboardInterrupt:
        terminate_group(proc, signal.SIGINT)
        try:
            return proc.wait(timeout=TERMINATION_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            terminate_group(proc, signal.SIGKILL)
            proc.wait()
            return 130


if __name__ == "__main__":
    raise SystemExit(main())
