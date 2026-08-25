//! Spiking Heidelberg Digits (SHD) offline loader — passthrough spike frames.
//!
//! **No tonic / HDF5 runtime dependency.** Official SHD is converted offline
//! into dense `[T × N_IN]` spike bins + labels under `binn/data/shd/` (gitignored
//! blobs). CI uses a tiny fixture with the same on-disk format.
//!
//! Does **not** use [`crate::encoder::LatencyEncoder`] — events are already
//! time-binned spike frames.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Canonical SHD channel count (cochlear bins).
pub const SHD_N_IN: usize = 700;
/// Default time bins for calibration cache (truncate/pad offline).
pub const SHD_DEFAULT_T: usize = 100;
/// Spoken-digit classes (German + English → 20-way).
pub const SHD_N_CLASSES: usize = 20;
/// Chance accuracy for 20-way classification.
pub const SHD_CHANCE: f32 = 1.0 / 20.0;

/// Magic for the BINN SHD bin cache (`BINNSHD1`).
const MAGIC: &[u8; 8] = b"BINNSHD1";

/// One passthrough SHD example: dense spike frame `[T × N_IN]` row-major + label.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdSample {
    /// Flat `[t * n_in + c]` spike counts / binary occupancy in `[0, 1]`.
    pub frames: Vec<f32>,
    pub t: usize,
    pub n_in: usize,
    pub label: u32,
}

impl ShdSample {
    /// Frame row at time `t` (length `n_in`).
    pub fn frame(&self, t: usize) -> &[f32] {
        let start = t * self.n_in;
        &self.frames[start..start + self.n_in]
    }
}

/// Split of SHD examples.
#[derive(Clone, Debug, PartialEq)]
pub struct ShdSplit {
    pub train: Vec<ShdSample>,
    pub test: Vec<ShdSample>,
    pub n_in: usize,
    pub t: usize,
    pub n_classes: usize,
    /// True when loaded from the CI / smoke fixture (not full SHD).
    pub fixture: bool,
}

/// Resolve cache directory: `$BINN_SHD_DIR` or `<crate>/../data/shd` relative to CWD.
pub fn default_shd_dir() -> PathBuf {
    if let Ok(p) = std::env::var("BINN_SHD_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from("data/shd")
}

/// Load a cached split. Prefers `train.bin`/`test.bin`; falls back to fixture.
/// When `max_train` / `max_test` are `Some(n)` with `n > 0`, only the first `n`
/// samples of each split are decoded (same prefix as an in-memory truncate).
pub fn load_shd_split(dir: &Path) -> Result<ShdSplit, String> {
    load_shd_split_capped(dir, None, None)
}

/// Like [`load_shd_split`] with optional per-split decode caps.
pub fn load_shd_split_capped(
    dir: &Path,
    max_train: Option<usize>,
    max_test: Option<usize>,
) -> Result<ShdSplit, String> {
    let train_path = dir.join("train.bin");
    let test_path = dir.join("test.bin");
    if train_path.is_file() && test_path.is_file() {
        warn_if_axis_unverifiable(dir);
        let train = read_bin_capped(&train_path, max_train)?;
        let test = read_bin_capped(&test_path, max_test)?;
        let (n_in, t, n_classes) = dims_from(&train, &test)?;
        return Ok(ShdSplit {
            train,
            test,
            n_in,
            t,
            n_classes,
            fixture: dir.join("FIXTURE").is_file()
                || dir.file_name().is_some_and(|n| n == "fixture"),
        });
    }
    Err(format!(
        "SHD cache not found under {} (expected train.bin + test.bin). \
         See binn/data/shd/README.md for offline conversion; or use the CI fixture.",
        dir.display()
    ))
}

/// Say so when a cache cannot be shown to put both splits on one time axis.
///
/// The BINNSHD1 header carries only `(n, T, N_IN)`, so the time horizon a cache
/// was binned against is not recoverable from the files. `convert-shd` used to
/// derive that horizon *per split*, from a scan of the first 256 samples, and
/// the two splits came out different: 1.167969s for train against 1.148438s for
/// test on the shipped cache — 85.62 against 87.07 bins/s, so a spike at the
/// same instant landed in a different bin depending on which split it came
/// from. The converter now derives one horizon for both and records it in
/// `binning.json`.
///
/// A cache without that file predates the fix and may carry the defect. This
/// warns rather than refuses, because refusing would stop four experiments that
/// are mid-flight on the existing cache; the point is that "not checked" must
/// not read as "checked and fine".
fn warn_if_axis_unverifiable(dir: &Path) {
    if dir.join("binning.json").is_file() {
        return;
    }
    if dir.join("FIXTURE").is_file() || dir.file_name().is_some_and(|n| n == "fixture") {
        return;
    }
    eprintln!(
        "warning: {} has no binning.json, so it cannot be shown that train and \
         test share a time axis. Caches written before the shared-horizon fix \
         binned the two splits separately. Re-run `convert-shd` to settle it.",
        dir.display()
    );
}

/// Load the repo CI fixture (tiny SHD-format bins).
pub fn load_fixture() -> Result<ShdSplit, String> {
    let candidates = [
        PathBuf::from("data/shd/fixture"),
        PathBuf::from("binn/data/shd/fixture"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/shd/fixture"),
    ];
    for dir in &candidates {
        if dir.join("train.bin").is_file() {
            let mut split = load_shd_split(dir)?;
            split.fixture = true;
            return Ok(split);
        }
    }
    // Synthesize an in-memory fixture if on-disk missing (keeps unit tests green).
    Ok(synthesize_fixture(32, 16, 20, 24, 8, 0x51D_F187))
}

/// Write a split to `dir/{train,test}.bin` (+ optional FIXTURE marker).
pub fn write_split(dir: &Path, split: &ShdSplit) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    write_bin(&dir.join("train.bin"), &split.train)?;
    write_bin(&dir.join("test.bin"), &split.test)?;
    if split.fixture {
        fs::write(dir.join("FIXTURE"), b"ci-fixture\n").map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Deterministic tiny fixture matching the on-disk format (not scientific SHD).
pub fn synthesize_fixture(
    n_in: usize,
    t: usize,
    n_classes: usize,
    n_train: usize,
    n_test: usize,
    seed: u64,
) -> ShdSplit {
    let mut rng = seed;
    let mut next = || {
        rng = rng.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0x5171);
        rng
    };
    let mut mk = |n: usize| -> Vec<ShdSample> {
        (0..n)
            .map(|_| {
                let label = (next() as usize % n_classes) as u32;
                let mut frames = vec![0.0f32; t * n_in];
                // Class-conditioned sparse spikes (passthrough-like).
                for _ in 0..(t / 4).max(1) {
                    let tt = (next() as usize) % t;
                    let c = (label as usize * 3 + (next() as usize % 5)) % n_in;
                    frames[tt * n_in + c] = 1.0;
                }
                ShdSample {
                    frames,
                    t,
                    n_in,
                    label,
                }
            })
            .collect()
    };
    ShdSplit {
        train: mk(n_train),
        test: mk(n_test),
        n_in,
        t,
        n_classes,
        fixture: true,
    }
}

fn dims_from(train: &[ShdSample], test: &[ShdSample]) -> Result<(usize, usize, usize), String> {
    let sample = train
        .first()
        .or_else(|| test.first())
        .ok_or_else(|| "empty SHD split".to_string())?;
    let n_in = sample.n_in;
    let t = sample.t;
    let mut max_label = 0u32;
    for s in train.iter().chain(test.iter()) {
        if s.n_in != n_in || s.t != t {
            return Err("inconsistent SHD frame dims".into());
        }
        max_label = max_label.max(s.label);
    }
    Ok((n_in, t, (max_label as usize + 1).max(SHD_N_CLASSES)))
}

fn write_bin(path: &Path, samples: &[ShdSample]) -> Result<(), String> {
    let mut f = File::create(path).map_err(|e| e.to_string())?;
    f.write_all(MAGIC).map_err(|e| e.to_string())?;
    write_u32(&mut f, samples.len() as u32)?;
    if let Some(s) = samples.first() {
        write_u32(&mut f, s.t as u32)?;
        write_u32(&mut f, s.n_in as u32)?;
    } else {
        write_u32(&mut f, 0)?;
        write_u32(&mut f, 0)?;
    }
    for s in samples {
        write_u32(&mut f, s.label)?;
        for &v in &s.frames {
            write_f32(&mut f, v)?;
        }
    }
    Ok(())
}

fn read_bin_capped(path: &Path, max_samples: Option<usize>) -> Result<Vec<ShdSample>, String> {
    use std::io::BufReader;
    let mut f = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
    let mut magic = [0u8; 8];
    f.read_exact(&mut magic).map_err(|e| e.to_string())?;
    if &magic != MAGIC {
        return Err(format!("bad SHD magic in {}", path.display()));
    }
    let n_file = read_u32_io(&mut f)?;
    let t = read_u32_io(&mut f)? as usize;
    let n_in = read_u32_io(&mut f)? as usize;
    let n = match max_samples {
        Some(m) if m > 0 => (n_file as usize).min(m),
        _ => n_file as usize,
    };
    let frame_len = t.checked_mul(n_in).ok_or("frame overflow")?;
    let mut out = Vec::with_capacity(n);
    let mut frame_bytes = vec![0u8; frame_len * 4];
    for _ in 0..n {
        let label = read_u32_io(&mut f)?;
        f.read_exact(&mut frame_bytes).map_err(|e| e.to_string())?;
        let mut frames = Vec::with_capacity(frame_len);
        for chunk in frame_bytes.chunks_exact(4) {
            frames.push(f32::from_bits(u32::from_le_bytes(
                chunk.try_into().unwrap(),
            )));
        }
        out.push(ShdSample {
            frames,
            t,
            n_in,
            label,
        });
    }
    Ok(out)
}

fn read_u32_io(f: &mut impl Read) -> Result<u32, String> {
    let mut b = [0u8; 4];
    f.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(b))
}

fn write_u32(f: &mut File, v: u32) -> Result<(), String> {
    f.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())
}

fn write_f32(f: &mut File, v: f32) -> Result<(), String> {
    f.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn roundtrip_bin_format() {
        let split = synthesize_fixture(16, 8, 20, 4, 2, 42);
        let dir = temp_dir().join(format!("binn_shd_rt_{}", std::process::id()));
        write_split(&dir, &split).unwrap();
        let loaded = load_shd_split(&dir).unwrap();
        assert_eq!(loaded.train.len(), 4);
        assert_eq!(loaded.test.len(), 2);
        assert_eq!(loaded.n_in, 16);
        assert_eq!(loaded.t, 8);
        assert!(loaded.fixture);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_fixture_always_works() {
        let split = load_fixture().unwrap();
        assert!(split.fixture);
        assert_eq!(split.n_classes, 20);
        assert!(!split.train.is_empty());
    }
}
