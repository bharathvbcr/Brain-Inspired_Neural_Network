# SHD offline cache (BINN)

Passthrough spike-frame cache for protocols `c1-shd-cal-*` (capped p27) and
`c1-shd-full-*` (protocol 29 full corpus + SuperSpike). **Not** LatencyEncoder.

> **Calibration status:** `BINNSHD1` is a frozen legacy format that stores
> binary occupancy. It is not used by the new instrument calibration because
> collisions cannot be recovered from it. The calibration runner derives a
> count-preserving `SHDEVT1` cache from the official HDF5 files under
> `data/shd/events/`; Python and Rust frame the same events at runtime.

## Layout

```
data/shd/
  README.md          # this file (tracked)
  fixture/           # tiny CI smoke bins (tracked)
    train.bin
    test.bin
    FIXTURE
  train.bin          # full SHD (gitignored) — official 8156 train
  test.bin           # full SHD (gitignored) — official 2264 test
  shd_train.h5[.gz]  # optional HDF5 cache (gitignored)
  shd_test.h5[.gz]
```

Binary format (`BINNSHD1`):

1. magic `BINNSHD1` (8 bytes)
2. `u32 le` sample count
3. `u32 le` T (time bins)
4. `u32 le` N_IN (channels; full SHD = 700)
5. repeated samples: `u32 le` label + `T*N_IN` little-endian `f32` frames (row-major `t, channel`)

Canonical full SHD: `N_IN=700`, `T=100` (truncate/pad offline), `n_classes=20`, chance = 1/20.

## Fetch + convert (Rust only; no Python / h5py)

`scripts/convert_shd.py` is **retired**. Use the Rust converter.

Requires system **libhdf5** (Homebrew: `brew install hdf5`). Downloads Zenke Lab / CompNeuro HDF5 if needed, writes BINNSHD1 bins:

```bash
cd binn
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
  cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
    --cache-dir data/shd
```

If HDF5 files are already present:

```bash
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
  cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
    --cache-dir data/shd --skip-download
```

Useful flags: `--t 100` (default), `--n-in 700` (default), `--max-train N`, `--max-test N`, `--force-download`.

## Run calibration (protocol 27; capped)

Scientific schedule (expects full bins; **caps** `max_train=2000` / `max_test=500` — calibration, not full-corpus SOTA). Ceiling = true e-prop (~0.09–0.10 overnight).

Default (protocol **27**, includes RL×B):

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \
  --out results/c1_shd.md
# hash: c1-shd-cal-eb3cb5d93417a638  (FROZEN — do not remassage)
```

Archived protocol-26 scientific (3 arms, no RL×B; frozen hash):

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \
  --config-hash c1-shd-cal-de44bb52bbd28fbc --out results/c1_shd.md
```

CI / smoke (no download; tracked fixture):

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal --quick \
  --out results/c1_shd.md
```

## Run full-corpus + SuperSpike ceiling (protocol 29)

New hash family `c1-shd-full-*` — **distinct from p27 and proto-135**.
Arms: broadcast ±1 / DFA / RL×B / **true SuperSpike reverse-mode BPTT**.

```bash
# Path-proof subset (400/100, 2 seeds, 8 epochs):
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --smoke \
  --out results/c1_shd_full_smoke.md
# hash: c1-shd-full-a9542a730cb22c74

# Full official splits (8156 train / 2264 test):
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full \
  --out results/c1_shd_full.md
# hash: c1-shd-full-2c93117075740ed0
```

Fixture SuperSpike path:

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --quick \
  --out results/c1_shd_full_quick.md
```

## Claims

Calibration / software-harness only — **not Gate G2**, not neuromorphic SOTA.
Do **not** mix p27 capped e-prop results with protocol-29 SuperSpike full-corpus,
or with exploratory proto-135 (5-class). Disclose wall time for full-corpus runs.
