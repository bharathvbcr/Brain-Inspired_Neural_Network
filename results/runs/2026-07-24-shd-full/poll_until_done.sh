#!/bin/zsh
RUNDIR=/Users/bharath/Code/parameter_golf/binn/results/runs/2026-07-24-shd-full
POLLLOG=$RUNDIR/poll.log
: > "$POLLLOG"
while true; do
  ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  C1_PID=$(cat $RUNDIR/c1.pid 2>/dev/null)
  if [ -z "$C1_PID" ] || ! kill -0 $C1_PID 2>/dev/null; then
    NEW=$(pgrep -n -f './target/release/c1 --shd-full --out results/c1_shd_full.md' || true)
    if [ -n "$NEW" ]; then
      echo "$NEW" > $RUNDIR/c1.pid
      C1_PID=$NEW
    else
      echo "PROCESS_EXITED $ts" >> "$POLLLOG"
      break
    fi
  fi
  etime=$(ps -p $C1_PID -o etime= 2>/dev/null | tr -d ' ')
  cpu=$(ps -p $C1_PID -o %cpu= 2>/dev/null | tr -d ' ')
  rss=$(ps -p $C1_PID -o rss= 2>/dev/null | tr -d ' ')
  log_lines=$(wc -l < $RUNDIR/c1_shd_full_scientific.log 2>/dev/null | tr -d ' ')
  out=0; [ -f /Users/bharath/Code/parameter_golf/binn/results/c1_shd_full.md ] && out=1
  end=0; [ -f $RUNDIR/c1_shd_full_end.txt ] && end=1
  echo "$ts alive pid=$C1_PID etime=$etime cpu=$cpu rss_kb=$rss log_lines=$log_lines out=$out end=$end" >> "$POLLLOG"
  sleep 300
done
{
  echo '---END FILE---'
  cat $RUNDIR/c1_shd_full_end.txt 2>/dev/null || echo '(no end)'
  echo '---LOG TAIL---'
  tail -n 120 $RUNDIR/c1_shd_full_scientific.log
  echo '---OUT---'
  cat /Users/bharath/Code/parameter_golf/binn/results/c1_shd_full.md 2>/dev/null || echo '(no out)'
} >> "$POLLLOG"
