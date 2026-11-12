#!/bin/sh
# usage: abba_attn.sh <variant> <outdir>   ABBA r4 vs spike-fma --numerics <variant>; records sha rather than failing
set -eu
V=$1; P=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad; W=$P/spike/s1/$2
EV=/Users/bharath/Code/research/BINN/data/shd/events
mkdir -p "$W"
$P/prof/bins/r4 init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs 2 --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 --weights "$W/init.weights" --orders "$W/orders.bin" > /dev/null 2>&1
i=0
for tag in r4 var var r4 r4 var var r4; do
  i=$((i + 1))
  bin=$P/prof/bins/r4; extra=""; [ $tag = var ] && { bin=$P/prof/bins/spike-fma; extra="--numerics $V"; }
  uptime | sed 's/.*load averages*: */load /' > "$W/$i.$tag.load"
  nice -n 5 /usr/bin/time -l -o "$W/$i.$tag.time" $bin train-cell --train-events $EV/train.events --test-events $EV/test.events --contract published-2ms --geometry adjacent-sum-5 --weights "$W/init.weights" --orders "$W/orders.bin" --epochs 2 --arm ff+fixed+attn --out "$W/$i.$tag.json" --seed 5170001 --train-speakers $EV/train.speakers --val-speakers 3,6 --save-final-weights "$W/$i.$tag.weights" $extra > "$W/$i.$tag.log" 2>&1
  sha=$(shasum -a 256 "$W/$i.$tag.weights" | cut -d' ' -f1)
  wall=$(python3 -I -c "import json,sys; print(json.load(open(sys.argv[1]))['wall_secs'])" "$W/$i.$tag.json")
  ins=$(awk '/instructions retired/ {print $1}' "$W/$i.$tag.time")
  cyc=$(awk '/cycles elapsed/ {print $1}' "$W/$i.$tag.time")
  usr=$(awk '/ user / {for (k = 1; k <= NF; k++) if ($k == "user") print $(k - 1)}' "$W/$i.$tag.time")
  echo "$i $tag wall=$wall user=$usr instructions=$ins cycles=$cyc sha=$sha $(cat "$W/$i.$tag.load")"
done
