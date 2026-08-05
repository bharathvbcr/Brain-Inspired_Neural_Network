use binn_hybrid_learn::fnv1a64;

pub const HYBRID_PROTOCOL_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq)]
pub struct HybridProtocol {
    pub quick: bool,
    pub development_seeds: usize,
    pub held_out_seeds: usize,
    pub train_examples: usize,
    pub test_examples: usize,
    pub c1_accuracy_floor: f32,
    pub c3_accuracy_floor: f32,
    pub c3_min_teacher_depth: usize,
    pub c3_min_student_depth: usize,
    pub min_gap_closed: f32,
    pub confidence_z: f32,
    pub learning_rate: f32,
}

impl HybridProtocol {
    pub fn quick() -> Self {
        Self {
            quick: true,
            development_seeds: 3,
            held_out_seeds: 3,
            train_examples: 240,
            test_examples: 160,
            c1_accuracy_floor: 0.65,
            c3_accuracy_floor: 0.65,
            c3_min_teacher_depth: 6,
            c3_min_student_depth: 6,
            min_gap_closed: 0.50,
            confidence_z: 1.96,
            learning_rate: 0.035,
        }
    }

    pub fn scientific() -> Self {
        Self {
            quick: false,
            development_seeds: 5,
            held_out_seeds: 20,
            train_examples: 2_000,
            test_examples: 500,
            ..Self::quick()
        }
    }

    pub fn hash(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&HYBRID_PROTOCOL_VERSION.to_le_bytes());
        bytes.push(u8::from(self.quick));
        for value in [
            self.development_seeds,
            self.held_out_seeds,
            self.train_examples,
            self.test_examples,
            self.c3_min_teacher_depth,
            self.c3_min_student_depth,
        ] {
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        for value in [
            self.c1_accuracy_floor,
            self.c3_accuracy_floor,
            self.min_gap_closed,
            self.confidence_z,
            self.learning_rate,
        ] {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        fnv1a64(&bytes)
    }

    pub fn hash_string(&self) -> String {
        format!(
            "binn-hybrid-h0-v{HYBRID_PROTOCOL_VERSION}-{:016x}",
            self.hash()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_hash_changes_with_semantics() {
        let a = HybridProtocol::quick();
        let mut b = a.clone();
        b.train_examples += 1;
        assert_ne!(a.hash(), b.hash());
    }
}
