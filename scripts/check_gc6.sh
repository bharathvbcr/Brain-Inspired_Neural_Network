#!/usr/bin/env bash
# GC6: no unsafe without a documented invariant (doc comment immediately above).
# Clippy also runs with -D warnings in CI.
#
# Scope is BINN-owned source only. `patches/` holds vendored crates pulled in
# via [patch.crates-io]; BINN does not author them and annotating a third-party
# crate would be a fork, not a fix. Their 30 `unsafe` sites were drowning the
# real signal, which is that BINN itself contains no `unsafe` at all.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail=0
while IFS= read -r hit; do
  [[ -z "$hit" ]] && continue
  f="${hit%%:*}"
  rest="${hit#*:}"
  lineno="${rest%%:*}"
  src_line="$(sed -n "${lineno}p" "$f")"
  # Skip comment-only mentions of the word unsafe.
  case "$src_line" in
    *//*) continue ;;
  esac
  prev=""
  i=$((lineno - 1))
  while [[ "$i" -ge 1 ]]; do
    prev="$(sed -n "${i}p" "$f")"
    if [[ -n "${prev// }" ]]; then
      break
    fi
    i=$((i - 1))
  done
  case "$prev" in
    *SAFETY:*|*invariant*) ;;
    *)
      echo "GC6 FAIL: $f:$lineno unsafe without documented invariant/SAFETY comment"
      fail=1
      ;;
  esac
done < <(rg -n --glob '*.rs' --glob '!patches/**' -e '\bunsafe\b' . || true)

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
echo "GC6 PASS: no undocumented unsafe"
