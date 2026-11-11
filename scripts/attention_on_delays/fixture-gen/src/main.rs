//! Rust side of the PyTorch attention read-out parity test (prereg
//! `PREREG_2026-11-08_ATTENTION_ON_THE_DELAY_SUBSTRATE.md` §3).
//!
//! Writes one JSON fixture computed by `binn_learn::shd_attention` itself:
//!
//! - the read-out's initial weights from `AttentionParams::deterministic` at the
//!   registered shape (hidden 256, 20 classes, d_model 32, 4 blocks) and the
//!   series' first seed, so the Python initialiser can be checked bit for bit;
//! - a second weight set with a non-zero `W_o` in every block, because at
//!   initialisation `W_o = 0` and the blocks are identity residuals, which a
//!   parity test on initial weights alone would never exercise;
//! - the positional code at T_h = 124 and 100 (`positional_code`);
//! - spike trains `[T_h, 256]`, including one at the post-dropout value
//!   `1/(1-p)` rather than 1.0, and one presentation permutation;
//! - `pooled` and `a = W_a pooled` for every case, from `attention_forward_presented`.
//!
//! Every float is written as little-endian f32 bytes in hex, so nothing is lost
//! to decimal formatting (scripts/gate_e_recurrent_parity.py records how `{:.9}`
//! broke bit comparison there).
//!
//! usage: attention-parity-fixture <out.json> <binn-commit>

use std::fmt::Write as _;

use binn_learn::shd_attention::{
    attention_forward_presented, attention_logits, positional_code, AttentionConfig,
    AttentionParams, ATTENTION_MAX_CYCLES, ATTENTION_MIN_CYCLES,
};
use binn_learn::PortableRng;

const HIDDEN: usize = 256;
const CLASSES: usize = 20;
const D_MODEL: usize = 32;
const LAYERS: usize = 4;
/// The series' first seed (prereg §4).
const SEED: u64 = 5_170_001;
/// The pinned reference's dropout probability (`config.py`, `dropout_p`).
const DROPOUT_P: f32 = 0.4;
/// Stream key for the fixture's own `W_o` draws. Outside the 0x01..0x05 keys
/// `AttentionParams::deterministic` uses, so it cannot alias any of them.
const W_O_KEY: u64 = 0x4154_544E_0000_0006;
/// Stream key for the fixture's spike trains and permutation.
const SPIKE_KEY: u64 = 0x4154_544E_0000_0007;
/// Firing probability of the synthetic spike trains, near the substrate's rate.
const FIRE_P: f32 = 0.08;

fn hex(values: &[f32]) -> String {
    let mut out = String::with_capacity(values.len() * 8);
    for value in values {
        for byte in value.to_le_bytes() {
            write!(out, "{byte:02x}").expect("write to String");
        }
    }
    out
}

fn params_json(params: &AttentionParams) -> String {
    let blocks: Vec<String> = params
        .blocks
        .iter()
        .map(|block| {
            format!(
                "{{\"w_q\":\"{}\",\"w_k\":\"{}\",\"w_v\":\"{}\",\"w_o\":\"{}\"}}",
                hex(&block.w_q),
                hex(&block.w_k),
                hex(&block.w_v),
                hex(&block.w_o)
            )
        })
        .collect();
    format!(
        "{{\"w_e\":\"{}\",\"blocks\":[{}],\"w_a\":\"{}\"}}",
        hex(&params.w_e),
        blocks.join(","),
        hex(&params.w_a)
    )
}

/// `TAU * cycles` per sin/cos pair, computed the way `attention_forward_presented`
/// computes it. Diagnostic: it lets a positional-code mismatch be traced to the
/// frequency ladder or to `sin`/`cos`.
fn tau_cycles() -> Vec<f32> {
    let pairs = D_MODEL / 2;
    let ratio = ATTENTION_MAX_CYCLES / ATTENTION_MIN_CYCLES;
    (0..pairs)
        .map(|pair| {
            let exponent = pair as f32 / (pairs - 1) as f32;
            std::f32::consts::TAU * (ATTENTION_MIN_CYCLES * ratio.powf(exponent))
        })
        .collect()
}

fn spikes(rng: &mut PortableRng, t_steps: usize, value: f32) -> Vec<f32> {
    (0..t_steps * HIDDEN)
        .map(|_| if rng.uniform(0.0, 1.0) < FIRE_P { value } else { 0.0 })
        .collect()
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: attention-parity-fixture <out.json> <binn-commit>".into());
    }
    let (out_path, commit) = (&args[1], &args[2]);

    let config = AttentionConfig::new(D_MODEL, LAYERS)?;
    let init = AttentionParams::deterministic(HIDDEN, CLASSES, config, SEED)?;
    let mut trained = init.clone();
    let limit = (6.0_f32 / (D_MODEL + D_MODEL) as f32).sqrt();
    for (layer, block) in trained.blocks.iter_mut().enumerate() {
        let mut rng = PortableRng::new(SEED ^ W_O_KEY ^ ((layer as u64 + 1) << 32));
        for weight in &mut block.w_o {
            *weight = rng.uniform(-limit, limit);
        }
    }
    let param_sets = [("init", &init), ("wo", &trained)];

    // Post-dropout value: spikingjelly's Dropout scales kept entries by 1/(1-p).
    let dropout_value = 1.0_f32 / (1.0_f32 - DROPOUT_P);

    let mut pos_json = Vec::new();
    let mut sample_json = Vec::new();
    let mut case_json = Vec::new();
    for t_steps in [124_usize, 100] {
        let mut pos = vec![0.0_f32; t_steps * D_MODEL];
        for t in 0..t_steps {
            positional_code(t, t_steps, &mut pos[t * D_MODEL..(t + 1) * D_MODEL]);
        }
        pos_json.push(format!("\"{t_steps}\":\"{}\"", hex(&pos)));

        let mut rng = PortableRng::new(SEED ^ SPIKE_KEY ^ ((t_steps as u64) << 32));
        let binary = spikes(&mut rng, t_steps, 1.0);
        let scaled = spikes(&mut rng, t_steps, dropout_value);
        let silent = vec![0.0_f32; t_steps * HIDDEN];
        let mut order: Vec<usize> = (0..t_steps).collect();
        rng.shuffle(&mut order);
        let samples = [("binary", &binary), ("dropout", &scaled), ("silent", &silent)];
        let encoded: Vec<String> = samples
            .iter()
            .map(|(name, values)| format!("\"{name}\":\"{}\"", hex(values)))
            .collect();
        let order_list: Vec<String> = order.iter().map(usize::to_string).collect();
        sample_json.push(format!(
            "\"{t_steps}\":{{{},\"order\":[{}]}}",
            encoded.join(","),
            order_list.join(",")
        ));

        let cases: [(&str, &str, bool); 6] = [
            ("init", "silent", false),
            ("init", "binary", false),
            ("wo", "binary", false),
            ("wo", "dropout", false),
            ("wo", "binary", true),
            ("wo", "silent", false),
        ];
        for (params_name, sample_name, presented) in cases {
            let params = param_sets
                .iter()
                .find(|(name, _)| *name == params_name)
                .map(|(_, p)| *p)
                .expect("known parameter set");
            let values = samples
                .iter()
                .find(|(name, _)| *name == sample_name)
                .map(|(_, v)| *v)
                .expect("known sample");
            let presentation = presented.then_some(order.as_slice());
            let cache = attention_forward_presented(params, values, t_steps, presentation)?;
            let logits = attention_logits(params, cache.pooled());
            case_json.push(format!(
                "{{\"t_steps\":{t_steps},\"params\":\"{params_name}\",\"sample\":\"{sample_name}\",\"presented\":{presented},\"pooled\":\"{}\",\"a\":\"{}\"}}",
                hex(cache.pooled()),
                hex(&logits)
            ));
        }
    }

    let json = format!(
        "{{\"schema\":\"attention-parity-fixture-v1\",\"binn_commit\":\"{commit}\",\"source\":\"binn-learn/src/shd_attention.rs\",\
\"hidden\":{HIDDEN},\"n_classes\":{CLASSES},\"d_model\":{D_MODEL},\"layers\":{LAYERS},\"seed\":{SEED},\
\"dropout_p\":{DROPOUT_P},\"fire_p\":{FIRE_P},\"w_o_key\":{W_O_KEY},\"spike_key\":{SPIKE_KEY},\
\"tau_cycles\":\"{}\",\"params\":{{\"init\":{},\"wo\":{}}},\"pos\":{{{}}},\"samples\":{{{}}},\"cases\":[{}]}}\n",
        hex(&tau_cycles()),
        params_json(&init),
        params_json(&trained),
        pos_json.join(","),
        sample_json.join(","),
        case_json.join(",")
    );
    std::fs::write(out_path, json).map_err(|error| format!("{out_path}: {error}"))
}
