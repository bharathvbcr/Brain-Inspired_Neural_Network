//! GC3: same seed ⇒ identical encoding / stream fingerprint (U12).

use binn_data::{
    ClassIncConfig, ClassIncrementalStream, Encoder, LatencyEncoder, Metrics, Sample, SynthConfig,
    SyntheticStream, TemporalClassification, WorkCosts, WorkCounters,
};

fn encode_fingerprint(seed: u64) -> u64 {
    let mut stream = SyntheticStream::new(SynthConfig::toy(seed));
    let enc = LatencyEncoder::new(4, 16, 0);
    // FNV-1a over encoded events.
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for _ in 0..32 {
        let sample = stream.next_sample();
        for ev in enc.encode(&sample) {
            hash ^= ev.t;
            hash = hash.wrapping_mul(0x100_0000_01b3);
            hash ^= ev.cell as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }
    hash ^= enc.info_loss().to_bits() as u64;
    hash = hash.wrapping_mul(0x100_0000_01b3);
    hash
}

#[test]
fn same_seed_identical_encoding_hash() {
    let seed = 0xB177_C0DE_0000_0012;
    let h1 = encode_fingerprint(seed);
    let h2 = encode_fingerprint(seed);
    assert_eq!(
        h1, h2,
        "same seed must yield identical encoding fingerprint"
    );

    let h_other = encode_fingerprint(seed ^ 0x9E37_79B9_7F4A_7C15);
    assert_ne!(h1, h_other, "different seeds must diverge");
}

#[test]
fn dataset_config_fingerprint_stable() {
    let a = TemporalClassification::toy(42);
    let b = TemporalClassification::toy(42);
    assert_eq!(a.config_fingerprint(), b.config_fingerprint());
}

#[test]
fn work_per_accuracy_not_linear_activity_estimate() {
    let counts = WorkCounters {
        source_spikes: 100,
        synaptic_deliveries: 800,
        cell_updates: 800,
        plasticity_updates: 0,
    };
    let w = Metrics::work_per_accuracy(counts, WorkCosts::unit(), 1.0);
    assert!((w - 1700.0).abs() < 1e-9);
    let naive = Metrics::naive_linear_activity_work(100.0 * 8.0 / 0.02, 0.02);
    assert!((naive - 800.0).abs() < 1e-9);
    assert!(w > naive);
}

#[test]
fn encoder_info_loss_reported() {
    let enc = LatencyEncoder::new(2, 8, 0);
    let loss = enc.info_loss();
    assert!(loss.is_finite());
    assert!((0.0..1.0).contains(&loss) || (loss - 1.0).abs() < 1e-6);
    // Smoke: encode does not panic.
    let _ = enc.encode(&Sample::from_values(vec![0.25, 0.75]));
}

#[test]
fn class_incremental_no_task_ids_and_no_raw_buffer() {
    let mut stream = ClassIncrementalStream::new(ClassIncConfig::quick(5));
    let ex = stream.next_train().expect("example");
    // Learner API: only sequence + label (no task_id field on the type).
    let _ = (ex.sequence, ex.label);
    assert_eq!(stream.phase(), 0);
    // Probes are regenerated, not retained as a learner replay buffer.
    assert_eq!(stream.probe_class(0).len(), stream.config().test_per_class);
}

#[test]
fn forgetting_metric_smoke() {
    assert!((Metrics::forgetting(0.8, 0.4) - 0.5).abs() < 1e-12);
    assert_eq!(Metrics::forgetting(0.0, 0.1), 0.0);
}
