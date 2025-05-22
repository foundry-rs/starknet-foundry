use console::style;
use forge_runner::package_tests::TestTargetLocation;
use forge_runner::{test_case_summary::AnyTestCaseSummary, test_target_summary::TestTargetSummary};
use foundry_ui::Ui;
use foundry_ui::components::TypedMessage;
use starknet_api::block::BlockNumber;
use std::collections::HashMap;
use url::Url;

pub(crate) fn print_collected_tests_count(tests_num: usize, package_name: &str) {
    let plain_text = format!("\n\nCollected {tests_num} test(s) from {package_name} package");
    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_running_tests(test_target_location: TestTargetLocation, tests_num: usize) {
    let dir_name = match test_target_location {
        TestTargetLocation::Lib => "src",
        TestTargetLocation::Tests => "tests",
    };
    let plain_text = format!("Running {tests_num} test(s) from {dir_name}/");

    println!("{}", style(plain_text).bold());
}

// TODO(#2574): Bring back "filtered out" number in tests summary when running with `--exact` flag
pub(crate) fn print_test_summary(
    summaries: &[TestTargetSummary],
    filtered: Option<usize>,
    ui: &Ui,
) {
    let passed: usize = summaries.iter().map(TestTargetSummary::count_passed).sum();
    let failed: usize = summaries.iter().map(TestTargetSummary::count_failed).sum();
    let skipped: usize = summaries.iter().map(TestTargetSummary::count_skipped).sum();
    let ignored: usize = summaries.iter().map(TestTargetSummary::count_ignored).sum();

    if let Some(filtered) = filtered {
        ui.print(&TypedMessage::styled(
            "Tests",
            &format!(
                "{}: {} passed, {} failed, {} skipped, {} ignored, {} filtered out",
                style("Tests").bold(),
                passed,
                failed,
                skipped,
                ignored,
                filtered
            ),
            "bold",
        ));
    } else {
        ui.print(&TypedMessage::styled(
            "Tests",
            &format!(
                "{}: {} passed, {} failed, {} skipped, {} ignored",
                style("Tests").bold(),
                passed,
                failed,
                skipped,
                ignored
            ),
            "bold",
        ));
    }
}

pub fn print_failures(all_failed_tests: &[AnyTestCaseSummary], ui: &Ui) {
    if all_failed_tests.is_empty() {
        return;
    }
    let failed_tests_names = all_failed_tests
        .iter()
        .map(|any_test_case_summary| any_test_case_summary.name().unwrap());

    ui.print(&"\nFailures:");
    for name in failed_tests_names {
        ui.print(&format!("    {name}"));
    }
}

#[expect(clippy::implicit_hasher)]
pub fn print_latest_blocks_numbers(
    url_to_latest_block_number_map: &HashMap<Url, BlockNumber>,
    ui: &Ui,
) {
    if !url_to_latest_block_number_map.is_empty() {
        ui.print(&"");
    }
    for (url, latest_block_number) in url_to_latest_block_number_map {
        ui.print(&format!(
            "Latest block number = {latest_block_number} for url = {url}",
        ));
    }
}
