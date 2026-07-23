#!/usr/bin/env bash
# One-shot baseline commit helper (CI-green C1 temporal G2 fixes).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

git status --short

# Stage tracked project sources; leave local agent/devcouncil state untracked.
git add \
  .github \
  .gitignore \
  Cargo.lock \
  Cargo.toml \
  README.md \
  binn-areas \
  binn-core \
  binn-data \
  binn-engine \
  binn-lab \
  binn-learn \
  results \
  scripts

git status --short

git commit -m "$(cat <<'EOF'
Baseline: C1 temporal k-WTA, ±1 reward eligibility, accounting, and CI.

Silence encoder and zero coincidence backgrounds; score membrane voltage for
k-WTA; force action spikes with ±1 reward; count synapse plasticity; isolate
RSS; causal tests; clippy/fmt/GC green on quick C1 (PILOT, valid harness).

EOF
)"

git rev-parse HEAD
git log -1 --oneline
git status --short
