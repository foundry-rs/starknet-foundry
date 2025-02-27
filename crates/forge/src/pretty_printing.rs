use anyhow::Error;
use console::style;
use forge_runner::package_tests::TestTargetLocation;
use forge_runner::{test_case_summary::AnyTestCaseSummary, test_target_summary::TestTargetSummary};
use starknet_api::block::BlockNumber;
use std::collections::HashMap;
use url::Url;

pub fn print_error_message(error: &Error) {
    let error_tag = style("ERROR").red();
    println!("[{error_tag}] {error:#}");
}

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
pub(crate) fn print_test_summary(summaries: &[TestTargetSummary], filtered: Option<usize>) {
    let passed: usize = summaries.iter().map(TestTargetSummary::count_passed).sum();
    let failed: usize = summaries.iter().map(TestTargetSummary::count_failed).sum();
    let skipped: usize = summaries.iter().map(TestTargetSummary::count_skipped).sum();
    let ignored: usize = summaries.iter().map(TestTargetSummary::count_ignored).sum();
    let excluded: usize = summaries
        .iter()
        .map(TestTargetSummary::count_excluded)
        .sum();

    if let Some(filtered) = filtered {
        println!(
            "{}: {} passed, {} failed, {} skipped, {} ignored, {} excluded, {} filtered out",
            style("Tests").bold(),
            passed,
            failed,
            skipped,
            ignored,
            excluded,
            filtered
        );
    } else {
        println!(
            "{}: {} passed, {} failed, {} skipped, {} ignored, other filtered out",
            style("Tests").bold(),
            passed,
            failed,
            skipped,
            ignored
        );
    }
}

pub(crate) fn print_test_seed(seed: u64) {
    println!("{}: {seed}", style("Fuzzer seed").bold());
}

pub fn print_failures(all_failed_tests: &[AnyTestCaseSummary]) {
    if all_failed_tests.is_empty() {
        return;
    }
    let failed_tests_names = all_failed_tests
        .iter()
        .map(|any_test_case_summary| any_test_case_summary.name().unwrap());

    println!("\nFailures:");
    for name in failed_tests_names {
        println!("    {name}");
    }
}

#[allow(clippy::implicit_hasher)]
pub fn print_latest_blocks_numbers(url_to_latest_block_number_map: &HashMap<Url, BlockNumber>) {
    if !url_to_latest_block_number_map.is_empty() {
        println!();
    }
    for (url, latest_block_number) in url_to_latest_block_number_map {
        println!("Latest block number = {latest_block_number} for url = {url}");
    }
}
