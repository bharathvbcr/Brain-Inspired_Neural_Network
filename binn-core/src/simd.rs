//! SIMD cell math — delegated to [`sparsl`] so BINN and sparsl cannot diverge
//! on `tau == 0` refusal, Tick→f32 exactness, or lane width.

pub use sparsl::{scalar_leak_integrate, simd_leak_integrate, LANES};

#[cfg(test)]
mod tests {
    use super::{scalar_leak_integrate, simd_leak_integrate, LANES};
    use crate::rng::Rng;
    use crate::time::Tick;

    const ATOL: f32 = 1e-6;

    fn assert_close(a: &[f32], b: &[f32], atol: f32) {
        assert_eq!(a.len(), b.len());
        for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
            let err = (x - y).abs();
            assert!(
                err <= atol,
                "mismatch at {i}: simd={x} scalar={y} err={err} atol={atol}"
            );
        }
    }

    fn random_inputs(n: usize, seed: u64) -> (Vec<f32>, Vec<f32>, Vec<f32>, Tick) {
        let mut rng = Rng::new(seed);
        let v: Vec<f32> = (0..n).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
        let input: Vec<f32> = (0..n).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
        let tau: Vec<f32> = (0..n).map(|_| 0.5 + rng.next_f32() * 4.0).collect();
        let dt = (1 + rng.gen_index(4)) as Tick;
        (v, input, tau, dt)
    }

    #[test]
    fn simd_matches_scalar_random() {
        for &n in &[0, 1, 7, 8, 9, 15, 16, 17, 63, 64, 65, 1024, 1024 + 3] {
            let (v0, input, tau, dt) = random_inputs(n, 0x51_4D_44_00 + n as u64);
            let mut v_simd = v0.clone();
            let mut v_scalar = v0;
            simd_leak_integrate(&mut v_simd, &input, &tau, dt);
            scalar_leak_integrate(&mut v_scalar, &input, &tau, dt);
            assert_close(&v_simd, &v_scalar, ATOL);
        }
    }

    #[test]
    fn simd_matches_scalar_many_seeds() {
        for seed in 0..32u64 {
            let n = 257;
            let (v0, input, tau, dt) = random_inputs(n, seed ^ 0xB177_C0DE);
            let mut v_simd = v0.clone();
            let mut v_scalar = v0;
            simd_leak_integrate(&mut v_simd, &input, &tau, dt);
            scalar_leak_integrate(&mut v_scalar, &input, &tau, dt);
            assert_close(&v_simd, &v_scalar, ATOL);
        }
    }

    #[test]
    fn lanes_constant_is_eight() {
        assert_eq!(LANES, 8);
    }

    #[test]
    #[should_panic(expected = "input length must match v")]
    fn rejects_input_len_mismatch() {
        let mut v = [0.0f32; 4];
        simd_leak_integrate(&mut v, &[0.0; 3], &[1.0; 4], 1);
    }

    #[test]
    #[should_panic(expected = "tau[0] must be finite and non-zero")]
    fn simd_refuses_tau_zero() {
        let mut v = [0.0f32; 1];
        simd_leak_integrate(&mut v, &[1.0], &[0.0], 1);
    }

    #[test]
    #[should_panic(expected = "exceeds 2^24")]
    fn simd_refuses_tick_past_exact_f32() {
        let mut v = [0.0f32; 1];
        simd_leak_integrate(&mut v, &[1.0], &[1.0], (1 << 24) + 1);
    }
}
