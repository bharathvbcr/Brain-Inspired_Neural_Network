//! Advanced SHD research benchmark sweep runner across learning rules & population scales.

#[derive(Clone, Debug)]
pub struct ShdSweepResult {
    pub hidden_size: usize,
    pub arm_name: String,
    pub mean_accuracy: f32,
    pub std_accuracy: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ShdSweepReport {
    pub results: Vec<ShdSweepResult>,
}

impl ShdSweepReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_result(
        &mut self,
        hidden_size: usize,
        arm_name: &str,
        mean_accuracy: f32,
        std_accuracy: f32,
    ) {
        self.results.push(ShdSweepResult {
            hidden_size,
            arm_name: arm_name.to_string(),
            mean_accuracy,
            std_accuracy,
        });
    }

    pub fn render_markdown(&self) -> String {
        let mut out = String::from("# Advanced SHD Research Sweep Report\n\n");
        out.push_str("| Hidden Size | Arm | Mean Acc | Std Dev |\n");
        out.push_str("|---|---|---:|---:|\n");
        for r in &self.results {
            out.push_str(&format!(
                "| {} | {} | {:.4} | {:.4} |\n",
                r.hidden_size, r.arm_name, r.mean_accuracy, r.std_accuracy
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shd_sweep_report_rendering() {
        let mut r = ShdSweepReport::new();
        r.add_result(512, "Straight-Through k-WTA + Margin", 0.4250, 0.0120);
        let md = r.render_markdown();
        assert!(md.contains("Straight-Through k-WTA"));
        assert!(md.contains("512"));
    }
}
