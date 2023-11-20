use crate::CrateLocation;
use anyhow::Error;
use console::style;
use forge_runner::test_case_summary::TestCaseSummary;
use forge_runner::test_crate_summary::TestCrateSummary;

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

pub fn print_failures(all_failed_tests: &[TestCaseSummary]) {
    if all_failed_tests.is_empty() {
        return;
    }

    let failed_tests_names: Vec<&String> = all_failed_tests
        .iter()
        .map(|test_case_summary| match test_case_summary {
            TestCaseSummary::Failed { name, .. } => name,
            TestCaseSummary::Passed { .. }
            | TestCaseSummary::Ignored { .. }
            | TestCaseSummary::Skipped {} => unreachable!(),
        })
        .collect();

    println!("\nFailures:");
    for name in failed_tests_names {
        println!("    {name}");
    }
}
