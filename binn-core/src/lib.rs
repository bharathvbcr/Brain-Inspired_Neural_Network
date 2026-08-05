//! BINN L2 numeric core.

pub mod buffer;
pub mod metal_backend;
pub mod rng;
pub mod scan;
pub mod simd;
pub mod sparse;
pub mod time;

pub use buffer::Buffer;
#[cfg(feature = "gpu")]
pub use metal_backend::MetalGpuContext;
pub use metal_backend::{
    benchmarkable_backends, Backend, BackendUnavailable, SpmvBackend, SpmvBackendConfig,
    METAL_GPU_DISPATCH_IMPLEMENTED,
};
pub use rng::Rng;
pub use scan::{assoc_scan, assoc_scan_chunked, assoc_scan_sequential, State, DEFAULT_CHUNK_SIZE};
pub use simd::{scalar_leak_integrate, simd_leak_integrate, LANES};
pub use sparse::{Csc, Csr, CsrError};
pub use time::Tick;

#[cfg(test)]
mod determinism_gc3 {
    //! GC3: same seed ⇒ identical output hash (U01).

    use crate::{Buffer, Csr, Rng, Tick};

    /// Fold RNG / Buffer / Csr state into a single fingerprint.
    fn fingerprint(seed: u64) -> u64 {
        let mut rng = Rng::new(seed);
        // FNV-1a 64-bit
        let mut hash = 0xcbf2_9ce4_8422_2325u64;

        for _ in 0..256 {
            hash ^= rng.next_u64();
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }

        let buf = Buffer::from_fn(32, |_| rng.next_u64());
        for &v in buf.as_slice() {
            hash ^= v;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }

        let nrows = 8usize;
        let mut adj = vec![Vec::new(); nrows];
        for row in adj.iter_mut() {
            let deg = rng.gen_index(4);
            for _ in 0..deg {
                row.push(rng.gen_index(nrows) as u32);
            }
        }
        let csr = Csr::from_adjacency(&adj);
        for (r, c) in csr.edges() {
            hash ^= ((r as u64) << 32) | (c as u64);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }

        let ticks: Buffer<Tick> = Buffer::from_fn(8, |_| rng.next_u64());
        for &t in ticks.as_slice() {
            hash ^= t;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }

        hash
    }

    #[test]
    fn gc3_same_seed_identical_output_hash() {
        let seed = 0xB177_C0DE_0000_0001;
        let h1 = fingerprint(seed);
        let h2 = fingerprint(seed);
        assert_eq!(h1, h2, "same seed must yield identical fingerprint");

        let h_other = fingerprint(seed ^ 0x9E37_79B9_7F4A_7C15);
        assert_ne!(
            h1, h_other,
            "different seeds must not collide for this stream"
        );
    }
}
