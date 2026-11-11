//! `--numerics exact|blas`: which arithmetic the attention read-out's matrix
//! products use.
//!
//! * [`Numerics::Exact`] is the registered kernel, bit-identical to every
//!   recorded cell. It is the default everywhere and nothing in this module is
//!   reached on that path.
//! * [`Numerics::Blas`] runs the read-out's matmuls through Accelerate's
//!   `cblas_sgemm`. It is a **different numerics** — a different summation
//!   order — so a blas cell is a member of a different corpus and is never
//!   paired against an exact one. Arms without attention never call into this
//!   module, so their cells are byte-identical in both modes.
//!
//! # No process globals
//!
//! The mode is a value on [`crate::AttentionParams`], read by the forward and
//! backward. Cargo runs tests on parallel threads in one process, so a global
//! switch would let one test's mode leak into another's; the spike that this is
//! ported from used one, and this is the reason it was not merged as it was.
//!
//! # The `accelerate` feature
//!
//! Accelerate is a macOS system framework, not a crate. It is linked only when
//! the `accelerate` cargo feature is on **and** the target is macOS. In every
//! other build [`Numerics::Blas`] is refused by [`Numerics::require_available`]
//! with [`BLAS_UNAVAILABLE`]; it never falls back to the exact path silently.

/// The refusal for a build that cannot run [`Numerics::Blas`]. The wording is
/// part of the contract with the launchers, which look for it.
pub const BLAS_UNAVAILABLE: &str =
    "--numerics blas needs a build with the accelerate feature on macOS";

/// Which arithmetic the attention read-out uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Numerics {
    /// The registered kernel. Bit-identical to every recorded cell.
    #[default]
    Exact,
    /// Accelerate `cblas_sgemm` for the read-out's matmuls.
    Blas,
}

impl Numerics {
    /// `exact` or `blas`, nothing else. Case-sensitive on purpose: the label is
    /// written into the cell record and read back by key.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "exact" => Ok(Self::Exact),
            "blas" => Ok(Self::Blas),
            other => Err(format!("--numerics must be exact or blas, got {other:?}")),
        }
    }

    /// The `numerics` field of a cell record.
    pub fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Blas => "blas",
        }
    }

    /// Whether this build links Accelerate.
    pub const fn blas_compiled() -> bool {
        cfg!(all(feature = "accelerate", target_os = "macos"))
    }

    /// `Ok` unless the mode needs Accelerate and this build has none.
    pub fn require_available(self) -> Result<(), String> {
        if self == Self::Blas && !Self::blas_compiled() {
            return Err(BLAS_UNAVAILABLE.to_string());
        }
        Ok(())
    }

    /// The value of an optional `--numerics` flag: absent means `exact`, an
    /// unknown value is refused, and `blas` is refused in a build without
    /// Accelerate.
    pub fn from_flag(flag: Option<&str>) -> Result<Self, String> {
        let numerics = match flag {
            None => Self::Exact,
            Some(value) => Self::parse(value)?,
        };
        numerics.require_available()?;
        Ok(numerics)
    }
}

/// `"macOS <ProductVersion> (<BuildVersion>)"`, read through `sysctlbyname`
/// (`kern.osproductversion`, `kern.osversion`) with no subprocess. This is
/// recorded in a blas cell because Accelerate's arithmetic is part of the OS:
/// the same binary can produce different bits after a system update.
#[cfg(all(feature = "accelerate", target_os = "macos"))]
pub fn platform_label() -> Result<String, String> {
    Ok(format!(
        "macOS {} ({})",
        blas::sysctl_string("kern.osproductversion")?,
        blas::sysctl_string("kern.osversion")?
    ))
}

/// Single-precision GEMM and the per-thread threading pin.
///
/// Without the feature (or off macOS) the types still exist so the call sites
/// compile, but [`Numerics::require_available`] refuses before any of them can
/// run, and [`sgemm`] panics if reached.
pub mod blas {
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    use std::cell::Cell;
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    use std::ffi::{c_char, c_int, c_void, CString};

    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    #[link(name = "Accelerate", kind = "framework")]
    extern "C" {
        // vecLib.framework/Headers/cblas.h:610 (SDK under `xcrun --show-sdk-path`):
        //   void cblas_sgemm(enum CBLAS_ORDER, enum CBLAS_TRANSPOSE TransA,
        //     enum CBLAS_TRANSPOSE TransB, int M, int N, int K, float alpha,
        //     const float *A, int lda, const float *B, int ldb, float beta,
        //     float *C, int ldc);
        // The enums are C `int`-sized.
        fn cblas_sgemm(
            order: c_int,
            trans_a: c_int,
            trans_b: c_int,
            m: c_int,
            n: c_int,
            k: c_int,
            alpha: f32,
            a: *const f32,
            lda: c_int,
            b: *const f32,
            ldb: c_int,
            beta: f32,
            c: *mut f32,
            ldc: c_int,
        );
        // vecLib.framework/Headers/thread_api.h:34 (macOS 15.0+):
        //   int BLASSetThreading(const enum BLAS_THREADING threading);
        // "This setting is per thread, saved in a thread local variable."
        // Returns 0 on success, -1 when the option is unsupported.
        // `enum BLAS_THREADING : unsigned int`, thread_api.h:14.
        fn BLASSetThreading(threading: u32) -> c_int;
        // thread_api.h:45: `enum BLAS_THREADING BLASGetThreading()`, this thread's setting.
        #[cfg(test)]
        fn BLASGetThreading() -> u32;
    }

    // libSystem; <sys/sysctl.h>:810 in the same SDK:
    //   int sysctlbyname(const char *, void *oldp, size_t *oldlenp,
    //                    void *newp, size_t newlen);
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    extern "C" {
        fn sysctlbyname(
            name: *const c_char,
            oldp: *mut c_void,
            oldlenp: *mut usize,
            newp: *mut c_void,
            newlen: usize,
        ) -> c_int;
    }

    // cblas.h:34 `enum CBLAS_ORDER {CblasRowMajor=101, CblasColMajor=102}`.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    const CBLAS_ROW_MAJOR: c_int = 101;
    // cblas.h:35 `enum CBLAS_TRANSPOSE {CblasNoTrans=111, CblasTrans=112, ...}`.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    const CBLAS_NO_TRANS: c_int = 111;
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    const CBLAS_TRANS: c_int = 112;
    // thread_api.h:16-17: MULTI_THREADED = 0, SINGLE_THREADED = 1.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    const BLAS_THREADING_SINGLE_THREADED: u32 = 1;

    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    thread_local! {
        /// Set once this thread has been pinned to single-threaded BLAS.
        /// `BLASSetThreading` is per thread, so every rayon worker has to make
        /// the call itself; a call on the main thread pins only the main thread.
        static PINNED: Cell<bool> = const { Cell::new(false) };
    }

    /// Pin the calling thread's BLAS to one thread, once.
    ///
    /// Why single-threaded: the instrument already parallelises across samples
    /// on every core, so a multithreaded GEMM inside each sample would
    /// oversubscribe. It is also what makes the result independent of the
    /// worker count and of what else is running — one thread, one summation
    /// order.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    fn pin_this_thread() {
        PINNED.with(|pinned| {
            if pinned.get() {
                return;
            }
            // The call takes an enum by value and no pointers, and only sets
            // this thread's threading model.
            // SAFETY: no pointer crosses the boundary and the argument is a valid enum value.
            let status = unsafe { BLASSetThreading(BLAS_THREADING_SINGLE_THREADED) };
            assert_eq!(
                status, 0,
                "BLASSetThreading(SINGLE_THREADED) returned {status}: this macOS cannot \
                 pin Accelerate to one thread, so a blas cell would not be reproducible"
            );
            pinned.set(true);
        });
    }

    /// This thread's current BLAS threading model (0 multi, 1 single).
    #[cfg(all(test, feature = "accelerate", target_os = "macos"))]
    pub(super) fn current_threading() -> u32 {
        // SAFETY: no arguments and no pointers; it only reads this thread's setting.
        unsafe { BLASGetThreading() }
    }

    /// A string-valued sysctl, e.g. `kern.osversion`.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    pub(super) fn sysctl_string(name: &str) -> Result<String, String> {
        let key = CString::new(name).map_err(|_| format!("sysctl name {name:?} has a NUL"))?;
        let mut length: usize = 0;
        // `key` is a NUL-terminated string that outlives the call; a null
        // `oldp` with a valid `oldlenp` asks only for the required size.
        // SAFETY: every pointer is valid for the call and `oldp` is null, so nothing is written.
        let status = unsafe {
            sysctlbyname(
                key.as_ptr(),
                std::ptr::null_mut(),
                &mut length,
                std::ptr::null_mut(),
                0,
            )
        };
        if status != 0 || length == 0 {
            return Err(format!("sysctl {name} could not be sized"));
        }
        let mut buffer = vec![0u8; length];
        // `buffer` is `length` writable bytes and `length` is passed as its
        // capacity; `key` is as above.
        // SAFETY: the kernel cannot write past `buffer`, whose capacity is `length`.
        let status = unsafe {
            sysctlbyname(
                key.as_ptr(),
                buffer.as_mut_ptr().cast::<c_void>(),
                &mut length,
                std::ptr::null_mut(),
                0,
            )
        };
        if status != 0 {
            return Err(format!("sysctl {name} could not be read"));
        }
        buffer.truncate(length);
        while buffer.last() == Some(&0) {
            buffer.pop();
        }
        String::from_utf8(buffer).map_err(|_| format!("sysctl {name} is not UTF-8"))
    }

    /// `C[m,n] = op(A) * op(B)`, row-major, `alpha = 1`, `beta = 0`, where
    /// `op(A)` is `[m,k]` and `op(B)` is `[k,n]`; a `trans_*` flag means the
    /// operand is stored transposed (`A` as `[k,m]`, `B` as `[n,k]`).
    ///
    /// Every extent is checked against the slice before the foreign call, in
    /// release builds too: the bounds are what make the call sound.
    #[allow(clippy::too_many_arguments)]
    pub fn sgemm(
        trans_a: bool,
        trans_b: bool,
        m: usize,
        n: usize,
        k: usize,
        a: &[f32],
        lda: usize,
        b: &[f32],
        ldb: usize,
        c: &mut [f32],
        ldc: usize,
    ) {
        if m == 0 || n == 0 {
            return;
        }
        if k == 0 {
            for row in 0..m {
                c[row * ldc..row * ldc + n].fill(0.0);
            }
            return;
        }
        let (a_rows, a_cols) = if trans_a { (k, m) } else { (m, k) };
        let (b_rows, b_cols) = if trans_b { (n, k) } else { (k, n) };
        assert!(
            lda >= a_cols && a.len() >= (a_rows - 1) * lda + a_cols,
            "sgemm: A [{a_rows},{a_cols}] lda {lda} does not fit a slice of {}",
            a.len()
        );
        assert!(
            ldb >= b_cols && b.len() >= (b_rows - 1) * ldb + b_cols,
            "sgemm: B [{b_rows},{b_cols}] ldb {ldb} does not fit a slice of {}",
            b.len()
        );
        assert!(
            ldc >= n && c.len() >= (m - 1) * ldc + n,
            "sgemm: C [{m},{n}] ldc {ldc} does not fit a slice of {}",
            c.len()
        );
        #[cfg(all(feature = "accelerate", target_os = "macos"))]
        {
            let extent =
                |value: usize| c_int::try_from(value).expect("sgemm extent does not fit a C int");
            pin_this_thread();
            // The three asserts above establish that `a`, `b` and `c` each
            // cover every element the call reads or writes for the stated
            // extents and leading dimensions; `c` is a unique borrow, so
            // nothing aliases the output; every extent was converted to
            // `c_int` without truncation.
            // SAFETY: bounds of `a`, `b` and `c` are asserted, `c` is unaliased, extents fit `c_int`.
            unsafe {
                cblas_sgemm(
                    CBLAS_ROW_MAJOR,
                    if trans_a { CBLAS_TRANS } else { CBLAS_NO_TRANS },
                    if trans_b { CBLAS_TRANS } else { CBLAS_NO_TRANS },
                    extent(m),
                    extent(n),
                    extent(k),
                    1.0,
                    a.as_ptr(),
                    extent(lda),
                    b.as_ptr(),
                    extent(ldb),
                    0.0,
                    c.as_mut_ptr(),
                    extent(ldc),
                );
            }
        }
        #[cfg(not(all(feature = "accelerate", target_os = "macos")))]
        {
            let _ = (trans_a, trans_b, a, b, c);
            panic!("{}", super::BLAS_UNAVAILABLE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numerics_defaults_to_exact_and_refuses_unknown_values() {
        assert_eq!(Numerics::default(), Numerics::Exact);
        assert_eq!(Numerics::from_flag(None), Ok(Numerics::Exact));
        assert_eq!(Numerics::from_flag(Some("exact")), Ok(Numerics::Exact));
        for bad in ["", "BLAS", "Exact", "fast", "blas ", "accelerate"] {
            let error = Numerics::from_flag(Some(bad)).expect_err("unknown value");
            assert!(
                error.contains("must be exact or blas"),
                "{bad:?} gave {error:?}"
            );
        }
    }

    #[test]
    fn labels_round_trip_through_parse() {
        for numerics in [Numerics::Exact, Numerics::Blas] {
            assert_eq!(Numerics::parse(numerics.label()), Ok(numerics));
        }
    }

    /// In a build without Accelerate `blas` is refused with the contract
    /// message rather than falling back, from both entry points.
    #[cfg(not(all(feature = "accelerate", target_os = "macos")))]
    #[test]
    fn blas_without_the_feature_is_refused_with_the_contract_message() {
        assert!(!Numerics::blas_compiled());
        let expected = "--numerics blas needs a build with the accelerate feature on macOS";
        assert_eq!(Numerics::from_flag(Some("blas")), Err(expected.to_string()));
        assert_eq!(
            Numerics::Blas.require_available(),
            Err(expected.to_string())
        );
        assert_eq!(Numerics::Exact.require_available(), Ok(()));
    }

    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    #[test]
    fn blas_with_the_feature_is_accepted() {
        assert!(Numerics::blas_compiled());
        assert_eq!(Numerics::from_flag(Some("blas")), Ok(Numerics::Blas));
    }

    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    #[test]
    fn the_platform_label_names_this_macos() {
        let label = platform_label().expect("sysctl");
        let (version, build) = label
            .strip_prefix("macOS ")
            .and_then(|rest| rest.strip_suffix(')'))
            .and_then(|rest| rest.split_once(" ("))
            .expect("macOS <version> (<build>)");
        assert!(
            version.chars().all(|c| c.is_ascii_digit() || c == '.') && version.contains('.'),
            "product version {version:?}"
        );
        assert!(!build.is_empty() && build.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    /// `sgemm` against an f64 triple loop, all four transpose combinations and
    /// shapes that are not multiples of any register block.
    ///
    /// Tolerance: for a length-`k` dot product evaluated in f32 in any order,
    /// `|computed - exact| <= gamma_k * sum |a_i b_i|` with `gamma_k = k u /
    /// (1 - k u)` and `u = 2^-24` (Higham, *Accuracy and Stability of Numerical
    /// Algorithms*, eq. 3.5 and the inner-product bound of section 3.1). The
    /// bound holds for every summation order, so it does not depend on how
    /// Accelerate blocks the loop. It is doubled here to leave room for the
    /// operands being rounded to f32 from the f64 generator.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    #[test]
    fn sgemm_matches_an_f64_reference_within_the_inner_product_bound() {
        let shapes = [
            (1, 1, 1),
            (3, 5, 7),
            (32, 32, 32),
            (33, 17, 61),
            (90, 32, 90),
            (7, 130, 3),
        ];
        let unit = f64::powi(2.0, -24);
        let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            ((seed >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
        };
        let mut worst_ratio = 0.0_f64;
        let mut cases = 0usize;
        for (m, n, k) in shapes {
            for trans_a in [false, true] {
                for trans_b in [false, true] {
                    let a: Vec<f32> = (0..m * k).map(|_| next() as f32).collect();
                    let b: Vec<f32> = (0..k * n).map(|_| next() as f32).collect();
                    let lda = if trans_a { m } else { k };
                    let ldb = if trans_b { k } else { n };
                    let mut c = vec![f32::NAN; m * n];
                    blas::sgemm(trans_a, trans_b, m, n, k, &a, lda, &b, ldb, &mut c, n);
                    let at = |row: usize, col: usize| {
                        f64::from(if trans_a {
                            a[col * lda + row]
                        } else {
                            a[row * lda + col]
                        })
                    };
                    let bt = |row: usize, col: usize| {
                        f64::from(if trans_b {
                            b[col * ldb + row]
                        } else {
                            b[row * ldb + col]
                        })
                    };
                    for row in 0..m {
                        for col in 0..n {
                            let mut exact = 0.0_f64;
                            let mut magnitude = 0.0_f64;
                            for inner in 0..k {
                                let term = at(row, inner) * bt(inner, col);
                                exact += term;
                                magnitude += term.abs();
                            }
                            let bound =
                                2.0 * (k as f64 * unit) / (1.0 - k as f64 * unit) * magnitude;
                            let error = (f64::from(c[row * n + col]) - exact).abs();
                            assert!(
                                error <= bound,
                                "({m},{n},{k}) ta={trans_a} tb={trans_b} [{row},{col}]: \
                                 error {error:e} exceeds bound {bound:e}"
                            );
                            if bound > 0.0 {
                                worst_ratio = worst_ratio.max(error / bound);
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
        assert!(cases > 1000, "only {cases} elements were checked");
        assert!(
            worst_ratio > 0.0,
            "every element was exact; the bound was never exercised"
        );
    }

    /// The pin is per thread, so a worker that never made the call would run
    /// Accelerate's own threading. A fresh thread starts multi-threaded, and
    /// is single-threaded after its first `sgemm` and not before.
    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    #[test]
    fn the_first_sgemm_on_a_thread_pins_that_thread_to_one_blas_thread() {
        let (before, after_empty, after) = std::thread::spawn(|| {
            let before = blas::current_threading();
            // An empty product returns before the pin: it must not count.
            blas::sgemm(false, false, 0, 0, 0, &[], 1, &[], 1, &mut [], 1);
            let after_empty = blas::current_threading();
            let a = [1.0_f32, 2.0, 3.0, 4.0];
            let mut c = [0.0_f32; 4];
            blas::sgemm(false, false, 2, 2, 2, &a, 2, &a, 2, &mut c, 2);
            (before, after_empty, blas::current_threading())
        })
        .join()
        .expect("worker");
        assert_eq!((before, after_empty, after), (0, 0, 1));
    }

    #[cfg(all(feature = "accelerate", target_os = "macos"))]
    #[test]
    fn sgemm_with_a_zero_contraction_writes_zeros_and_respects_ldc() {
        let a: Vec<f32> = Vec::new();
        let b: Vec<f32> = Vec::new();
        let mut c = vec![7.0_f32; 3 * 5];
        // ldc 5 over n 2: the columns beyond n belong to someone else.
        blas::sgemm(false, false, 3, 2, 0, &a, 1, &b, 2, &mut c, 5);
        let expected: Vec<f32> = (0..15).map(|i| if i % 5 < 2 { 0.0 } else { 7.0 }).collect();
        assert_eq!(c, expected);
    }
}
