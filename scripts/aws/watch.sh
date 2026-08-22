#!/usr/bin/env bash
# Emit one line per change in campaign state. Every terminal condition is
# covered: progress, failures, a stall, the fleet dying, and completion. A
# monitor that only reported success would be silent through a crashloop, and
# silence would look exactly like "still running".
set -uo pipefail
B="${1:?bucket}"
count() { aws s3 ls "s3://$B/$1" 2>/dev/null | grep -c "$2" | tr -d ' \n'; }
# The plan can grow mid-campaign - a wave registered while the fleet runs - so
# the target is re-read every cycle, not captured once at startup. Reading it
# once has the same failure as hardcoding it: COMPLETE fires at the old total and
# the watch stops with cells still queued. It has now done so twice.
plan_total() {
  aws s3 cp "s3://$B/input/cells.json" - 2>/dev/null \
    | python3 -c "import json,sys;print(len(json.load(sys.stdin)))" 2>/dev/null
}
TOTAL=$(plan_total); TOTAL=${TOTAL:-0}
# Seed the gate cache from what already exists and summarise it in one line.
# Announcing every known gate individually on each restart turned a restart into
# six notifications that say nothing new; only gates first seen while watching
# are worth an event of their own.
seen_gates=""
gate_summary=""
for key in $(aws s3 ls "s3://$B/gates/" 2>/dev/null | grep '\.json' | awk '{print $4}'); do
  seen_gates="$seen_gates $key"
  verdict=$(aws s3 cp "s3://$B/gates/$key" - 2>/dev/null \
    | python3 -c "import json,sys;d=json.load(sys.stdin);print(d['instance'][:11]+'='+d['cross_machine_gate_f'])" 2>/dev/null)
  gate_summary="$gate_summary ${verdict}"
done
prev_g=$(count gates/ '\.json'); prev_g=${prev_g:-0}
echo "watching ${TOTAL} planned cells; every 20 cells, immediately on failure. gates:${gate_summary:- none}"
prev_g=0; prev_r=-1; prev_f=-1; prev_milestone=-1; quiet=0
while true; do
  g=$(count gates/ '\.json'); g=${g:-0}
  r=$(count results/ '\.json'); r=${r:-0}
  f=$(count failures/ '\.log'); f=${f:-0}
  alive=$(aws ec2 describe-instances --region us-east-1 \
    --filters Name=tag:Project,Values=binn-campaign \
              Name=instance-state-name,Values=pending,running \
    --query 'length(Reservations[].Instances[])' --output text 2>/dev/null)
  alive=${alive:-0}
  fresh=$(plan_total)
  if [ -n "$fresh" ] && [ "$fresh" != "$TOTAL" ]; then
    echo "plan changed: ${TOTAL} -> ${fresh} cells"
    TOTAL=$fresh
  fi
  # Only announce gate files not seen before. Re-printing every gate whenever a
  # new instance joins turned one event into four.
  if [ "$g" -gt "$prev_g" ]; then
    for key in $(aws s3 ls "s3://$B/gates/" | grep '\.json' | awk '{print $4}'); do
      case " $seen_gates " in
        *" $key "*) ;;
        *)
          echo "GATE $(aws s3 cp "s3://$B/gates/$key" - 2>/dev/null | tr -d '\n')"
          seen_gates="$seen_gates $key"
          ;;
      esac
    done
    prev_g=$g
  fi
  # Report progress at milestones, not per cell: 420 cells would otherwise emit
  # 420 notifications and bury the events that actually need a decision. Any
  # change in the FAILURE count still reports immediately, at any size - a
  # failure is always worth interrupting for.
  milestone=$(( r / 20 ))
  if [ "$f" != "$prev_f" ] || [ "$milestone" != "$prev_milestone" ]; then
    echo "progress ${r}/${TOTAL} results, ${f} failures, ${alive} instance(s)"
    prev_milestone=$milestone; prev_f=$f; quiet=0
  elif [ "$r" != "$prev_r" ]; then
    prev_r=$r; quiet=0
  else
    quiet=$((quiet + 1))
    # Silence is not a stall. A single attention cell at e400 runs for hours, so
    # "no new results in 25 minutes" is the normal state for most of this
    # campaign. Asking whether the workers are ALIVE is the check that actually
    # distinguishes a long cell from a dead fleet - so ask, rather than infer
    # from silence and cry wolf every 25 minutes.
    if [ "$quiet" -ge 25 ]; then
      quiet=0
      claims=$(count claims/ '')
      workers=0
      for id in $(aws ec2 describe-instances --region us-east-1 \
            --filters Name=tag:Project,Values=binn-campaign \
                      Name=instance-state-name,Values=running \
            --query 'Reservations[].Instances[].InstanceId' --output text 2>/dev/null); do
        cmd=$(aws ssm send-command --region us-east-1 --instance-ids "$id" \
              --document-name AWS-RunShellScript \
              --parameters 'commands=["pgrep -fc \"shd-instrument train-cell\" || echo 0"]' \
              --query 'Command.CommandId' --output text 2>/dev/null) || continue
        sleep 6
        n=$(aws ssm get-command-invocation --region us-east-1 --command-id "$cmd" \
            --instance-id "$id" --query StandardOutputContent --output text 2>/dev/null | tr -d ' \n')
        workers=$(( workers + ${n:-0} ))
      done
      if [ "${workers:-0}" -eq 0 ]; then
        echo "STALLED ${r}/${TOTAL}: NO WORKER PROCESSES on ${alive} running instance(s), ${claims} claims held. This needs a hand."
      else
        echo "quiet ${r}/${TOTAL}, ${f} failures, ${workers} workers busy on long cells, ${claims} claims held - healthy"
      fi
    fi
  fi
  # A diverged cell never produces a result - the instrument refuses to emit a
  # cell containing a non-finite value - so completion is results PLUS
  # divergences. Testing results alone means a campaign with any divergence can
  # never report COMPLETE and instead sits at the finish line looking stalled.
  settled=$(( r + f ))
  [ "$TOTAL" -gt 0 ] && [ "$settled" -ge "$TOTAL" ] && {
    echo "COMPLETE ${r} results + ${f} diverged = ${settled}/${TOTAL}"; break; }
  [ "$alive" = "0" ] && [ "$g" -gt 0 ] && { echo "FLEET GONE ${r}/${TOTAL} done, ${f} failures"; break; }
  sleep 60
done
