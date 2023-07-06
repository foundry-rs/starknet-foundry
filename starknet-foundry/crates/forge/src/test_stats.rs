use cairo_lang_runner::RunResultValue;

#[derive(Default, Clone, Copy)]
pub struct TestsStats {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl TestsStats {
    pub fn update(&mut self, run_result: &RunResultValue) {
        match run_result {
            RunResultValue::Success(_) => {
                self.passed += 1;
            }
            RunResultValue::Panic(_) => {
                self.failed += 1;
            }
        }
    }
}
