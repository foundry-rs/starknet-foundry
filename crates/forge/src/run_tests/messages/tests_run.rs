use console::style;
use forge_runner::package_tests::TestTargetLocation;
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
pub struct TestsRunMessage {
    test_target_location: TestTargetLocation,
    tests_num: usize,
}

impl TestsRunMessage {
    #[must_use]
    pub fn new(test_target_location: TestTargetLocation, tests_num: usize) -> Self {
        Self {
            test_target_location,
            tests_num,
        }
    }
}

impl Message for TestsRunMessage {
    fn text(&self) -> String {
        let dir_name = match self.test_target_location {
            TestTargetLocation::Lib => "src",
            TestTargetLocation::Tests => "tests",
        };
        let plain_text = format!("Running {} test(s) from {}/", self.tests_num, dir_name);
        style(plain_text).bold().to_string()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
