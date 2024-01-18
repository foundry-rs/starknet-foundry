use crate::compiled_raw::CrateLocation;
use anyhow::Error;
use console::style;
use forge_runner::{test_case_summary::AnyTestCaseSummary, test_crate_summary::TestCrateSummary};
use starknet_api::block::BlockNumber;
use std::collections::HashMap;

pub fn print_warning(error: &Error) {
    let warning_tag = style("WARNING").color256(11);
    println!("[{warning_tag}] {error}");
}

pub fn print_error_message(error: &Error) {
    let error_tag = style("ERROR").red();
    println!("[{error_tag}] {error}");
}

pub(crate) fn print_collected_tests_count(tests_num: usize, package_name: &str) {
    let plain_text = format!("\n\nCollected {tests_num} test(s) from {package_name} package");
    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_running_tests(test_crate_file: CrateLocation, tests_num: usize) {
    let dir_name = match test_crate_file {
        CrateLocation::Lib => "src",
        CrateLocation::Tests => "tests",
    };
    let plain_text = format!("Running {tests_num} test(s) from {dir_name}/");

    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_test_summary(summaries: &[TestCrateSummary], filtered: usize) {
    let passed: usize = summaries.iter().map(TestCrateSummary::count_passed).sum();
    let failed: usize = summaries.iter().map(TestCrateSummary::count_failed).sum();
    let skipped: usize = summaries.iter().map(TestCrateSummary::count_skipped).sum();
    let ignored: usize = summaries.iter().map(TestCrateSummary::count_ignored).sum();

    println!(
        "{}: {} passed, {} failed, {} skipped, {} ignored, {} filtered out",
        style("Tests").bold(),
        passed,
        failed,
        skipped,
        ignored,
        filtered,
    );
}

pub(crate) fn print_test_seed(seed: u64) {
    println!("{}: {seed}", style("Fuzzer seed").bold());
}

pub fn print_failures(all_failed_tests: &[AnyTestCaseSummary]) {
    if all_failed_tests.is_empty() {
        return;
    }
    let failed_tests_names: Vec<&String> = all_failed_tests
        .iter()
        .map(|any_test_case_summary| any_test_case_summary.name().unwrap())
        .collect();

    println!("\nFailures:");
    for name in failed_tests_names {
        println!("    {name}");
    }
}

#[allow(clippy::implicit_hasher)]
pub fn print_latest_blocks_numbers(url_to_latest_block_number_map: &HashMap<String, BlockNumber>) {
    if !url_to_latest_block_number_map.is_empty() {
        println!();
    }
    for (url, latest_block_number) in url_to_latest_block_number_map {
        println!("Latest block number = {latest_block_number} for url = {url}");
    }
}
