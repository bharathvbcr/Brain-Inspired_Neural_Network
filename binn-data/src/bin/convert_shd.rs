//! Offline SHD HDF5 → BINNSHD1 converter (replaces `scripts/convert_shd.py`).
//!
//! Build / run (requires system libhdf5, e.g. `brew install hdf5`):
//! ```text
//! PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
//!   cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
//!     --cache-dir data/shd
//! ```

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use flate2::read::GzDecoder;
use hdf5_metno::types::VarLenArray;
use hdf5_metno::File as H5File;

use binn_data::{SHD_DEFAULT_T, SHD_N_CLASSES, SHD_N_IN};

const MAGIC: &[u8; 8] = b"BINNSHD1";

const MIRRORS: &[&str] = &[
    "https://zenkelab.org/datasets",
    "https://compneuro.net/datasets",
];

fn main() {
    if let Err(e) = run(std::env::args().skip(1).collect()) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }
}

fn run(argv: Vec<String>) -> Result<(), String> {
    let args = parse_args(&argv)?;
    fs::create_dir_all(&args.cache_dir).map_err(|e| e.to_string())?;

    let train_h5 = if args.skip_download {
        let p = args.cache_dir.join("shd_train.h5");
        if !p.is_file() {
            return Err(format!("missing {}", p.display()));
        }
        p
    } else {
        ensure_h5("train", &args.cache_dir, args.force_download)?
    };
    let test_h5 = if args.skip_download {
        let p = args.cache_dir.join("shd_test.h5");
        if !p.is_file() {
            return Err(format!("missing {}", p.display()));
        }
        p
    } else {
        ensure_h5("test", &args.cache_dir, args.force_download)?
    };

    let train_bin = args.cache_dir.join("train.bin");
    let test_bin = args.cache_dir.join("test.bin");
    println!(
        "converting train: {} → {}",
        train_h5.display(),
        train_bin.display()
    );
    // One time axis for the whole dataset. Both splits are scanned in full
    // before either is written, and the wider of the two horizons is used for
    // both — otherwise a model is trained on one time base and evaluated on
    // another, which is what the shipped caches actually did.
    let train_horizon = split_horizon(&train_h5, args.max_train)?;
    let test_horizon = split_horizon(&test_h5, args.max_test)?;
    let raw_horizon = train_horizon.max(test_horizon);
    let t_max = raw_horizon.clamp(T_MAX_FLOOR, T_MAX_CEILING);
    println!(
        "time horizon: train {train_horizon:.6}s, test {test_horizon:.6}s -> \
         shared {t_max:.6}s ({:.2} bins/s)",
        args.t as f64 / t_max
    );
    if (raw_horizon - t_max).abs() > f64::EPSILON {
        println!(
            "  NOTE: the observed horizon {raw_horizon:.6}s was clamped into \
             [{T_MAX_FLOOR}, {T_MAX_CEILING}]; events past {t_max:.6}s fold into \
             the final bin."
        );
    }
    let n_train = convert_split(
        &train_h5,
        &train_bin,
        args.t,
        args.n_in,
        args.max_train,
        t_max,
    )?;
    println!(
        "converting test:  {} → {}",
        test_h5.display(),
        test_bin.display()
    );
    let n_test = convert_split(&test_h5, &test_bin, args.t, args.n_in, args.max_test, t_max)?;

    // The horizon is not recoverable from the cache: the BINNSHD1 header
    // carries only (n, T, N_IN). Recording it beside the files is what lets a
    // consumer tell whether two splits share a time axis at all.
    let manifest = args.cache_dir.join("binning.json");
    fs::write(
        &manifest,
        format!(
            "{{\"schema\":\"binnshd-binning-v1\",\"t_bins\":{},\"n_in\":{},\
             \"t_max_s\":{:.9},\"train_horizon_s\":{:.9},\"test_horizon_s\":{:.9},\
             \"bins_per_s\":{:.9},\"n_train\":{},\"n_test\":{}}}\n",
            args.t,
            args.n_in,
            t_max,
            train_horizon,
            test_horizon,
            args.t as f64 / t_max,
            n_train,
            n_test
        ),
    )
    .map_err(|e| format!("write {}: {e}", manifest.display()))?;
    println!("wrote {}", manifest.display());

    let (nt, tt, nin) = verify_bin(&train_bin, args.n_in, args.t)?;
    let (ne, te, nie) = verify_bin(&test_bin, args.n_in, args.t)?;
    if (nt, tt, nin) != (n_train, args.t, args.n_in) || (ne, te, nie) != (n_test, args.t, args.n_in)
    {
        return Err("verify mismatch after convert".into());
    }

    let fixture_marker = args.cache_dir.join("FIXTURE");
    if fixture_marker.is_file() {
        let _ = fs::remove_file(&fixture_marker);
    }

    println!(
        "OK: wrote {} ({n_train} samples) and {} ({n_test} samples); T={} N_IN={} classes={}",
        train_bin.display(),
        test_bin.display(),
        args.t,
        args.n_in,
        SHD_N_CLASSES
    );
    println!(
        "Next:\n  \
         # capped p27 e-prop calibration (frozen; do not remassage):\n  \
         cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \\\n    \
         --out results/c1_shd.md\n  \
         # protocol-29 full-corpus + SuperSpike ceiling:\n  \
         cargo run --locked --release -p binn-lab --bin c1 -- --shd-full \\\n    \
         --out results/c1_shd_full.md\n  \
         # path-proof subset:\n  \
         cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --smoke \\\n    \
         --out results/c1_shd_full_smoke.md"
    );
    Ok(())
}

#[derive(Debug)]
struct Args {
    cache_dir: PathBuf,
    t: usize,
    n_in: usize,
    max_train: Option<usize>,
    max_test: Option<usize>,
    force_download: bool,
    skip_download: bool,
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut cache_dir = PathBuf::from("data/shd");
    let mut t = SHD_DEFAULT_T;
    let mut n_in = SHD_N_IN;
    let mut max_train = None;
    let mut max_test = None;
    let mut force_download = false;
    let mut skip_download = false;
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--cache-dir" => {
                i += 1;
                cache_dir = PathBuf::from(argv.get(i).ok_or("--cache-dir needs value")?);
            }
            "--t" => {
                i += 1;
                t = argv
                    .get(i)
                    .ok_or("--t needs value")?
                    .parse()
                    .map_err(|e| format!("--t: {e}"))?;
            }
            "--n-in" => {
                i += 1;
                n_in = argv
                    .get(i)
                    .ok_or("--n-in needs value")?
                    .parse()
                    .map_err(|e| format!("--n-in: {e}"))?;
            }
            "--max-train" => {
                i += 1;
                max_train = Some(
                    argv.get(i)
                        .ok_or("--max-train needs value")?
                        .parse()
                        .map_err(|e| format!("--max-train: {e}"))?,
                );
            }
            "--max-test" => {
                i += 1;
                max_test = Some(
                    argv.get(i)
                        .ok_or("--max-test needs value")?
                        .parse()
                        .map_err(|e| format!("--max-test: {e}"))?,
                );
            }
            "--force-download" => force_download = true,
            "--skip-download" => skip_download = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown arg: {other}")),
        }
        i += 1;
    }
    Ok(Args {
        cache_dir,
        t,
        n_in,
        max_train,
        max_test,
        force_download,
        skip_download,
    })
}

fn print_help() {
    eprintln!(
        "convert-shd — official SHD HDF5 → BINNSHD1 dense bins\n\n\
         Options:\n\
           --cache-dir DIR     HDF5 + output bins (default: data/shd)\n\
           --t N               time bins (default {SHD_DEFAULT_T})\n\
           --n-in N            channels (default {SHD_N_IN})\n\
           --max-train N       optional train cap\n\
           --max-test N        optional test cap\n\
           --force-download    re-download even if .h5 / .h5.gz present\n\
           --skip-download     require existing shd_{{train,test}}.h5\n"
    );
}

fn ensure_h5(split: &str, cache_dir: &Path, force: bool) -> Result<PathBuf, String> {
    let h5 = cache_dir.join(format!("shd_{split}.h5"));
    let gz = cache_dir.join(format!("shd_{split}.h5.gz"));
    if h5.is_file() && !force {
        return Ok(h5);
    }
    if gz.is_file() && !force {
        gunzip(&gz, &h5)?;
        return Ok(h5);
    }
    let mut last_err = None;
    for base in MIRRORS {
        let url = format!("{base}/shd_{split}.h5.gz");
        match download_gz(&url, &gz) {
            Ok(()) => {
                gunzip(&gz, &h5)?;
                return Ok(h5);
            }
            Err(e) => {
                eprintln!("  mirror failed ({base}): {e}");
                last_err = Some(e);
            }
        }
    }
    Err(format!(
        "could not obtain shd_{split}.h5: {}",
        last_err.unwrap_or_else(|| "no mirrors".into())
    ))
}

fn download_gz(url: &str, dest_gz: &Path) -> Result<(), String> {
    if let Some(parent) = dest_gz.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    println!("downloading {url} → {}", dest_gz.display());
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(300))
        .user_agent("binn-shd-convert/1.0")
        .build();
    let resp = agent.get(url).call().map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    let mut out = File::create(dest_gz).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn gunzip(src_gz: &Path, dest_h5: &Path) -> Result<(), String> {
    println!("decompressing {} → {}", src_gz.display(), dest_h5.display());
    let fin = File::open(src_gz).map_err(|e| e.to_string())?;
    let mut decoder = GzDecoder::new(fin);
    let mut fout = File::create(dest_h5).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = decoder.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        fout.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Bounds on the shared time horizon. SHD samples run around 1s; anything
/// outside this is a corrupt time value rather than a longer recording.
const T_MAX_FLOOR: f64 = 0.5;
const T_MAX_CEILING: f64 = 1.5;

/// Widest spike time across a split, opening the file for the scan alone.
fn split_horizon(h5_path: &Path, max_samples: Option<usize>) -> Result<f64, String> {
    let file = H5File::open(h5_path).map_err(|e| format!("open {}: {e}", h5_path.display()))?;
    let n_all = file
        .dataset("labels")
        .map_err(|e| format!("labels: {e}"))?
        .read_1d::<i64>()
        .map_err(|e| format!("read labels: {e}"))?
        .len();
    let n = match max_samples {
        Some(m) => n_all.min(m),
        None => n_all,
    };
    if n == 0 {
        return Err(format!("empty SHD split: {}", h5_path.display()));
    }
    let times_ds = file
        .dataset("spikes/times")
        .map_err(|e| format!("spikes/times: {e}"))?;
    scan_time_horizon(&times_ds, n)
}

/// Widest spike time across every sample of a split, in seconds.
///
/// Every sample, not a capped prefix: a 256-sample scan makes the horizon
/// depend on file order, and silently differ between two splits of the same
/// dataset.
fn scan_time_horizon(times_ds: &hdf5_metno::Dataset, n: usize) -> Result<f64, String> {
    let mut widest = 0.0f64;
    for i in 0..n {
        let slice = times_ds
            .read_slice_1d::<VarLenArray<f32>, _>(i..i + 1)
            .map_err(|e| format!("times scan {i}: {e}"))?;
        if let Some(arr) = slice.get(0) {
            for &tm in arr.iter() {
                widest = widest.max(f64::from(tm));
            }
        }
    }
    Ok(widest)
}

fn convert_split(
    h5_path: &Path,
    out_bin: &Path,
    t_bins: usize,
    n_in: usize,
    max_samples: Option<usize>,
    t_max: f64,
) -> Result<usize, String> {
    let file = H5File::open(h5_path).map_err(|e| format!("open {}: {e}", h5_path.display()))?;
    let labels = file
        .dataset("labels")
        .map_err(|e| format!("labels: {e}"))?
        .read_1d::<i64>()
        .map_err(|e| format!("read labels: {e}"))?;
    let times_ds = file
        .dataset("spikes/times")
        .map_err(|e| format!("spikes/times: {e}"))?;
    let units_ds = file
        .dataset("spikes/units")
        .map_err(|e| format!("spikes/units: {e}"))?;

    let n_all = labels.len();
    let n = match max_samples {
        Some(m) => n_all.min(m),
        None => n_all,
    };
    if n == 0 {
        return Err("empty SHD split".into());
    }

    // The horizon is supplied by the caller, not derived here. Deriving it per
    // split put train and test on *different* time axes: this function is
    // called once per split, and each computed its own `t_max` from a scan of
    // the first 256 samples. Measured on the shipped caches, train came out at
    // 1.167969 s and test at 1.148438 s — 85.62 vs 87.07 bins/s, a 1.70%
    // dilation of the test axis relative to train. A model trained on one time
    // base was evaluated on another, and nothing recorded either number.

    if let Some(parent) = out_bin.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out = File::create(out_bin).map_err(|e| e.to_string())?;
    out.write_all(MAGIC).map_err(|e| e.to_string())?;
    write_u32(&mut out, n as u32)?;
    write_u32(&mut out, t_bins as u32)?;
    write_u32(&mut out, n_in as u32)?;

    for i in 0..n {
        let label = labels[i];
        if label < 0 || label as usize >= SHD_N_CLASSES {
            return Err(format!("label {label} out of range at sample {i}"));
        }
        let times = times_ds
            .read_slice_1d::<VarLenArray<f32>, _>(i..i + 1)
            .map_err(|e| format!("times[{i}]: {e}"))?;
        let units = units_ds
            .read_slice_1d::<VarLenArray<u16>, _>(i..i + 1)
            .map_err(|e| format!("units[{i}]: {e}"))?;
        let t_arr = times.get(0).map(|a| a.as_slice()).unwrap_or(&[]);
        let u_arr = units.get(0).map(|a| a.as_slice()).unwrap_or(&[]);
        let frames = spikes_to_frame(t_arr, u_arr, t_bins, n_in, t_max);
        write_u32(&mut out, label as u32)?;
        for &v in &frames {
            write_f32(&mut out, v)?;
        }
        if (i + 1) % 500 == 0 || i + 1 == n {
            println!(
                "  {}: {}/{n}",
                out_bin.file_name().unwrap().to_string_lossy(),
                i + 1
            );
        }
    }
    Ok(n)
}

fn spikes_to_frame(
    times: &[f32],
    units: &[u16],
    t_bins: usize,
    n_in: usize,
    t_max: f64,
) -> Vec<f32> {
    let mut frame = vec![0.0f32; t_bins * n_in];
    if times.is_empty() {
        return frame;
    }
    let scale = t_bins as f64 / t_max.max(1e-6);
    for (&tm, &u) in times.iter().zip(units.iter()) {
        let ch = u as usize;
        if ch >= n_in {
            continue;
        }
        let mut tb = (f64::from(tm) * scale) as isize;
        if tb < 0 {
            continue;
        }
        if tb as usize >= t_bins {
            tb = (t_bins as isize) - 1;
        }
        let idx = (tb as usize) * n_in + ch;
        if frame[idx] < 1.0 {
            frame[idx] = 1.0;
        }
    }
    frame
}

fn verify_bin(
    path: &Path,
    expect_n_in: usize,
    expect_t: usize,
) -> Result<(usize, usize, usize), String> {
    let mut f = File::open(path).map_err(|e| e.to_string())?;
    let mut magic = [0u8; 8];
    f.read_exact(&mut magic).map_err(|e| e.to_string())?;
    if &magic != MAGIC {
        return Err(format!("bad magic in {}: {:?}", path.display(), magic));
    }
    let n = read_u32(&mut f)? as usize;
    let t = read_u32(&mut f)? as usize;
    let n_in = read_u32(&mut f)? as usize;
    if t != expect_t || n_in != expect_n_in {
        return Err(format!(
            "{}: got T={t} N_IN={n_in}, expected T={expect_t} N_IN={expect_n_in}",
            path.display()
        ));
    }
    let frame_bytes = t * n_in * 4;
    let mut buf = vec![0u8; frame_bytes];
    for i in 0..n {
        let mut label_raw = [0u8; 4];
        f.read_exact(&mut label_raw)
            .map_err(|_| format!("truncated label at sample {i}"))?;
        f.read_exact(&mut buf)
            .map_err(|_| format!("truncated frames at sample {i}"))?;
    }
    let mut leftover = [0u8; 1];
    if f.read(&mut leftover).unwrap_or(0) != 0 {
        return Err(format!("trailing bytes in {}", path.display()));
    }
    Ok((n, t, n_in))
}

fn write_u32(f: &mut File, v: u32) -> Result<(), String> {
    f.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())
}
fn write_f32(f: &mut File, v: f32) -> Result<(), String> {
    f.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())
}
fn read_u32(f: &mut File) -> Result<u32, String> {
    let mut b = [0u8; 4];
    f.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The defect this file was restructured to fix.
    ///
    /// `convert_split` is called once per split. It used to derive its own
    /// `t_max` from a scan of the first 256 samples, so train and test landed
    /// on different time axes — measured on the shipped caches, 1.167969s and
    /// 1.148438s, i.e. 85.62 against 87.07 bins/s. A spike at the same instant
    /// fell in a different bin depending on which split it came from.
    #[test]
    fn one_horizon_puts_the_same_instant_in_the_same_bin() {
        let t_bins = 100;
        let n_in = 4;
        let times = [0.10_f32, 0.50, 0.90, 1.10];
        let units = [0_u16, 1, 2, 3];

        let shared = 1.167969_f64;
        let train = spikes_to_frame(&times, &units, t_bins, n_in, shared);
        let test = spikes_to_frame(&times, &units, t_bins, n_in, shared);
        assert_eq!(train, test, "one horizon must bin both splits identically");

        // And the failure it replaces: the two per-split horizons the shipped
        // caches actually used disagree on where these spikes belong.
        let old_test_horizon = 1.148438_f64;
        let divergent = spikes_to_frame(&times, &units, t_bins, n_in, old_test_horizon);
        assert_ne!(
            train, divergent,
            "these two horizons must disagree, or this test is not \
             demonstrating the defect it describes"
        );
    }

    /// The horizon scales the axis, so a wider horizon moves a spike earlier.
    #[test]
    fn a_wider_horizon_moves_a_spike_to_an_earlier_bin() {
        let (t_bins, n_in) = (100, 1);
        let times = [0.90_f32];
        let units = [0_u16];
        let narrow = spikes_to_frame(&times, &units, t_bins, n_in, 1.0);
        let wide = spikes_to_frame(&times, &units, t_bins, n_in, 1.5);
        let bin_of = |frame: &[f32]| frame.iter().position(|&v| v > 0.0).expect("one spike");
        assert!(
            bin_of(&wide) < bin_of(&narrow),
            "a wider horizon compresses the axis: wide {} vs narrow {}",
            bin_of(&wide),
            bin_of(&narrow)
        );
    }

    #[test]
    fn events_past_the_horizon_fold_into_the_final_bin() {
        // Documented rather than silently true: an event beyond `t_max` is
        // clamped, not dropped, so the last bin is an OR of the whole tail.
        let (t_bins, n_in) = (10, 1);
        let frame = spikes_to_frame(&[5.0_f32], &[0_u16], t_bins, n_in, 1.0);
        assert_eq!(frame[t_bins - 1], 1.0, "the tail folds into the last bin");
        assert_eq!(frame[..t_bins - 1].iter().sum::<f32>(), 0.0);
    }
}
