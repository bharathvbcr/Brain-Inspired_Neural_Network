#!/bin/sh
# usage: abba_base.sh <variant> <outdir>  h512 channels-700 fixed-t500 base cell, 2 epochs
set -eu
V=$1; P=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad; W=$P/spike/s1/$2
R=/Users/bharath/Code/research/BINN
mkdir -p "$W"
i=0
for tag in r4 var var r4 r4 var var r4; do
  i=$((i + 1))
  bin=$P/prof/bins/r4; extra=""; [ $tag = var ] && { bin=$P/prof/bins/spike-fma; extra="--numerics $V"; }
  uptime | sed 's/.*load averages*: */load /' > "$W/$i.$tag.load"
  nice -n 5 /usr/bin/time -l -o "$W/$i.$tag.time" $bin train-cell --train-events $R/data/shd/events/train.events --test-events $R/data/shd/events/test.events --contract fixed-t500 --geometry channels-700 --weights $R/results/shd_instrument_v4/initialization/n700-h512-s5170001.weights --orders $R/results/shd_instrument_v4/initialization/n8156-e100-s5170001.orders --epochs 2 --out "$W/$i.$tag.json" --save-final-weights "$W/$i.$tag.weights" $extra > "$W/$i.$tag.log" 2>&1
  sha=$(shasum -a 256 "$W/$i.$tag.weights" | cut -d' ' -f1)
  wall=$(python3 -I -c "import json,sys; print(json.load(open(sys.argv[1]))['wall_secs'])" "$W/$i.$tag.json")
  ins=$(awk '/instructions retired/ {print $1}' "$W/$i.$tag.time")
  cyc=$(awk '/cycles elapsed/ {print $1}' "$W/$i.$tag.time")
  usr=$(awk '/ user / {for (k = 1; k <= NF; k++) if ($k == "user") print $(k - 1)}' "$W/$i.$tag.time")
  echo "$i $tag wall=$wall user=$usr instructions=$ins cycles=$cyc sha=$sha $(cat "$W/$i.$tag.load")"
done
