#!/bin/zsh
set -e
cd /Users/bharath/Code/parameter_golf/binn
RUNDIR=results/runs/2026-07-24-shd-full
START_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "start=$START_UTC" > "$RUNDIR/c1_shd_full_start.txt"
echo "restart=1" >> "$RUNDIR/c1_shd_full_start.txt"
echo "pid=$$" >> "$RUNDIR/c1_shd_full_start.txt"
echo "cmd=./target/release/c1 --shd-full --out results/c1_shd_full.md" >> "$RUNDIR/c1_shd_full_start.txt"
/usr/bin/time -p ./target/release/c1 --shd-full --out results/c1_shd_full.md \
  > "$RUNDIR/c1_shd_full_scientific.log" 2>&1
EC=$?
{
  echo "exit=$EC"
  date -u +%Y-%m-%dT%H:%M:%SZ
  echo DONE
} >> "$RUNDIR/c1_shd_full_end.txt"
cp -f results/c1_shd_full.md "$RUNDIR/c1_shd_full.md" 2>/dev/null || true
exit $EC
