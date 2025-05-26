use std::collections::HashMap;

use console::style;
use forge_runner::{
    package_tests::TestTargetLocation, test_case_summary::AnyTestCaseSummary,
    test_target_summary::TestTargetSummary,
};
use foundry_ui::Message;
use serde::Serialize;
#[derive(Serialize)]
pub struct TestsRun {
    test_target_location: TestTargetLocation,
    tests_num: usize,
}

impl TestsRun {
    #[must_use]
    pub fn new(test_target_location: TestTargetLocation, tests_num: usize) -> Self {
        Self {
            test_target_location,
            tests_num,
        }
    }
}

impl Message for TestsRun {
    fn text(&self) -> String {
        let dir_name = match self.test_target_location {
            TestTargetLocation::Lib => "src",
            TestTargetLocation::Tests => "tests",
        };
        let plain_text = format!("Running {} test(s) from {}/", self.tests_num, dir_name);
        style(plain_text).bold().to_string()
    }
}

#[derive(Serialize)]
pub struct CollectedTestsCount {
    pub tests_num: usize,
    pub package_name: String,
}

impl Message for CollectedTestsCount {
    fn text(&self) -> String {
        let full = format!(
            "\n\nCollected {} test(s) from {} package",
            self.tests_num, self.package_name
        );
        style(full).bold().to_string()
    }
}

// TODO(#2574): Bring back "filtered out" number in tests summary when running with `--exact` flag
#[derive(Serialize)]
pub struct TestsSummary {
    passed: usize,
    failed: usize,
    skipped: usize,
    ignored: usize,
    filtered: Option<usize>,
}

impl TestsSummary {
    pub fn new(summaries: &[TestTargetSummary], filtered: Option<usize>) -> Self {
        let passed = summaries.iter().map(TestTargetSummary::count_passed).sum();
        let failed = summaries.iter().map(TestTargetSummary::count_failed).sum();
        let skipped = summaries.iter().map(TestTargetSummary::count_skipped).sum();
        let ignored = summaries.iter().map(TestTargetSummary::count_ignored).sum();

        Self {
            passed,
            failed,
            skipped,
            ignored,
            filtered,
        }
    }
}

impl Message for TestsSummary {
    fn text(&self) -> String {
        if let Some(filtered) = self.filtered {
            format!(
                "{}: {} passed, {} failed, {} skipped, {} ignored, {filtered} filtered out",
                style("Tests").bold(),
                self.passed,
                self.failed,
                self.skipped,
                self.ignored
            )
        } else {
            format!(
                "{}: {} passed, {} failed, {} skipped, {} ignored, other filtered out",
                style("Tests").bold(),
                self.passed,
                self.failed,
                self.skipped,
                self.ignored
            )
        }
    }
}

#[derive(Serialize)]
pub struct TestsFailureSummary {
    pub failed_test_names: Vec<String>,
}

impl TestsFailureSummary {
    #[must_use]
    pub fn new(all_failed_tests: &[AnyTestCaseSummary]) -> Self {
        let failed_test_names = all_failed_tests
            .iter()
            .map(|any_test_case_summary| any_test_case_summary.name().unwrap().to_string())
            .collect();

        Self { failed_test_names }
    }
}

impl Message for TestsFailureSummary {
    fn text(&self) -> String {
        if self.failed_test_names.is_empty() {
            return String::new();
        }

        let mut failures = "\nFailures:".to_string();
        for name in &self.failed_test_names {
            failures = format!("{failures}\n    {name}");
        }
        style(failures).bold().to_string()
    }
}

#[derive(Serialize)]
pub struct LatestBlocksNumbers {
    url_to_latest_block_number_map: HashMap<url::Url, starknet_api::block::BlockNumber>,
}

impl LatestBlocksNumbers {
    #[must_use]
    pub fn new(
        url_to_latest_block_number_map: HashMap<url::Url, starknet_api::block::BlockNumber>,
    ) -> Self {
        Self {
            url_to_latest_block_number_map,
        }
    }
}

impl Message for LatestBlocksNumbers {
    fn text(&self) -> String {
        if self.url_to_latest_block_number_map.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        for (url, latest_block_number) in &self.url_to_latest_block_number_map {
            output =
                format!("{output}Latest block number = {latest_block_number} for url = {url}\n",);
        }
        style(output).bold().to_string()
    }
}
