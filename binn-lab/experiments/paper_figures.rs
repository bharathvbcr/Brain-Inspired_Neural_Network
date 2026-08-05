//! Generate camera-ready paper figures (plotters; requires `--features plots`).
//!
//! ```text
//! cargo run --locked --release -p binn-lab --features plots --bin paper-figures -- \
//!   --out results/runs/2026-07-23-paper-hard-both/figures
//! ```

use std::path::PathBuf;

fn main() {
    let mut out = PathBuf::from("results/runs/2026-07-23-paper-hard-both/figures");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => {
                out = PathBuf::from(args.next().expect("--out needs a path"));
            }
            "-h" | "--help" => {
                eprintln!(
                    "paper-figures — write figM / fig1 / fig3 / graphical_abstract\n\
                     Options:\n  --out DIR   output directory (default: camp figures/)"
                );
                return;
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }
    match binn_lab::paper_figures::generate_all(&out) {
        Ok(paths) => {
            println!("wrote {} files under {}", paths.len(), out.display());
            for p in paths {
                println!("  {}", p.display());
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
