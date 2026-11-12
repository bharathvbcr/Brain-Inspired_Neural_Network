#!/bin/sh
# Independent check of the Accelerate spike: interleaved ABBA of r4 (exact) vs
# spike-blas (--numerics blas) on the 2-epoch d32/L4 oracle cell, at a fixed
# RAYON_NUM_THREADS. usage: abba_threads.sh <threads> <outdir>
set -eu
T=$1
S=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad
EV=/Users/bharath/Code/research/BINN/data/shd/events
W=$S/spike/$2
mkdir -p "$W"
"$S/prof/bins/r4" init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs 2 \
  --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 \
  --weights "$W/init.weights" --orders "$W/orders.bin" > /dev/null 2>&1
i=0
for tag in r4 blas blas r4 r4 blas blas r4; do
  i=$((i + 1))
  bin=$S/prof/bins/r4; extra=""
  [ "$tag" = blas ] && bin=$S/prof/bins/spike-blas && extra="--numerics blas"
  load=$(uptime | sed 's/.*load averages*: *//')
  RAYON_NUM_THREADS=$T /usr/bin/time -l -o "$W/$i.$tag.time" "$bin" train-cell \
    --train-events "$EV/train.events" --test-events "$EV/test.events" \
    --contract published-2ms --geometry adjacent-sum-5 --weights "$W/init.weights" --orders "$W/orders.bin" \
    --epochs 2 --arm ff+fixed+attn --out "$W/$i.$tag.json" --seed 5170001 \
    --train-speakers "$EV/train.speakers" --val-speakers 3,6 \
    --save-final-weights "$W/$i.$tag.weights" $extra > "$W/$i.$tag.log" 2>&1
  sha=$(shasum -a 256 "$W/$i.$tag.weights" | cut -c1-16)
  real=$(awk '/ real / {print $1}' "$W/$i.$tag.time")
  usr=$(awk '/ user / {for (k = 1; k <= NF; k++) if ($k == "user") print $(k - 1)}' "$W/$i.$tag.time")
  cyc=$(awk '/cycles elapsed/ {print $1}' "$W/$i.$tag.time")
  echo "$i $tag threads=$T real=$real user=$usr cycles=$cyc sha=$sha load $load"
done
