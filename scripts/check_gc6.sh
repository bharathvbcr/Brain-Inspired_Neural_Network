#!/usr/bin/env bash
# GC6: no unsafe without a documented invariant (doc comment immediately above).
# Clippy also runs with -D warnings in CI.
#
# Scope is BINN-owned source only. `patches/` is where a crate vendored via
# [patch.crates-io] would live (none since 2026-09-29, when the unused `block`
# patch was deleted); BINN does not author such crates and annotating one would
# be a fork, not a fix. The `block` patch's 30 `unsafe` sites once drowned the
# real signal, which is that BINN itself contains no `unsafe` at all.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail=0
# Run the search first and check how it ended. `|| true` on the pipeline
# swallowed everything: ripgrep exits 1 for "searched, found nothing" and 2 for
# "could not search", and a missing binary exits 127 — so `rg: command not
# found` printed "GC6 PASS: no undocumented unsafe" after reading no files.
set +e
rg_hits="$(rg -n --glob '*.rs' --glob '!patches/**' -e '\bunsafe\b' . </dev/null)"
rg_rc=$?
set -e
if [[ $rg_rc -gt 1 ]]; then
  echo "GC6 CANNOT RUN: the search failed (rg exit $rg_rc). GC6 read nothing."
  echo "Install ripgrep, or GC6 is not looking at any source at all."
  exit 1
fi

while IFS= read -r hit; do
  # A here-string over an empty variable still yields one empty line.
  [[ -n "$hit" ]] || continue
  [[ -z "$hit" ]] && continue
  f="${hit%%:*}"
  rest="${hit#*:}"
  lineno="${rest%%:*}"
  src_line="$(sed -n "${lineno}p" "$f")"
  # Skip comment-only mentions of the word unsafe. Strip the comment part rather
  # than discarding the whole line: matching `*//*` against the whole line
  # skipped any line containing `//` anywhere, so appending a trailing comment
  # to a real `unsafe` block hid it from the gate entirely. Verified: a planted
  # `unsafe { }` passed GC6 with a trailing comment and failed without one.
  code_part="${src_line%%//*}"
  case "$code_part" in
    *unsafe*) ;;
    *) continue ;;
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
done <<< "$rg_hits"

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
echo "GC6 PASS: no undocumented unsafe"
