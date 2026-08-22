#!/usr/bin/env bash
# Add campaign capacity as fast as the account allows.
#
# A quota raise does not appear in EC2 the moment the support case closes:
# Service Quotas reported 300 while RunInstances still refused at 128, then
# accepted 32 more and refused the next 8. So the ceiling is discovered by
# asking, not by reading - try the largest instance that could fit, and on
# refusal step down the size ladder.
#
# Emits one line per change. Exits when the target is reached.
set -uo pipefail
PLAN="${1:?plan}"
TARGET_VCPU="${2:-288}"
LADDER="c7g.16xlarge:64 c7g.8xlarge:32 c7g.4xlarge:16 c7g.2xlarge:8"

vcpus_running() {
  local total=0
  for t in $(aws ec2 describe-instances --region us-east-1 \
        --filters Name=tag:Project,Values=binn-campaign \
                  Name=instance-state-name,Values=pending,running \
        --query 'Reservations[].Instances[].InstanceType' --output text 2>/dev/null); do
    case "$t" in
      *.16xlarge) total=$((total + 64));; *.8xlarge) total=$((total + 32));;
      *.4xlarge)  total=$((total + 16));; *.2xlarge) total=$((total + 8));;
      *) total=$((total + 4));;
    esac
  done
  echo "$total"
}

while true; do
  have=$(vcpus_running)
  if [ "$have" -ge "$TARGET_VCPU" ]; then
    echo "AT TARGET: ${have} vCPU running"
    exit 0
  fi
  added=0
  for entry in $LADDER; do
    type="${entry%%:*}"; size="${entry##*:}"
    [ $(( have + size )) -gt "$TARGET_VCPU" ] && continue
    if python3 scripts/aws/launch.py --plan "$PLAN" --count 1 --skip-inputs \
         --instance-type "$type" >/tmp/scale.log 2>&1; then
      echo "added ${type} (+${size} vCPU) -> $(vcpus_running) vCPU running"
      added=1
      break
    fi
  done
  [ "$added" -eq 0 ] && echo "held at ${have} vCPU; account ceiling not yet raised further"
  sleep 240
done
