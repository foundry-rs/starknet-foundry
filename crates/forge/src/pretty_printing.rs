use crate::test_case_summary::TestCaseSummary;
use crate::{TestCrateSummary, TestCrateType};
use anyhow::Error;
use console::style;

pub fn print_error_message(error: &Error) {
    let error_tag = style("ERROR").red();
    println!("[{error_tag}] {error}");
}

pub(crate) fn print_collected_tests_count(tests_num: usize, package_name: &str) {
    let plain_text = format!("\n\nCollected {tests_num} test(s) from {package_name} package");
    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_running_tests(test_crate_file: TestCrateType, tests_num: usize) {
    let dir_name = match test_crate_file {
        TestCrateType::Lib => "src",
        TestCrateType::Tests => "tests",
    };
    let plain_text = format!("Running {tests_num} test(s) from {dir_name}/");

    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_test_summary(summaries: &[TestCrateSummary]) {
    let passed: usize = summaries.iter().map(TestCrateSummary::count_passed).sum();
    let skipped: usize = summaries.iter().map(TestCrateSummary::count_skipped).sum();
    let failed: usize = summaries.iter().map(TestCrateSummary::count_failed).sum();

    println!(
        "{}: {} passed, {} failed, {} skipped",
        style("Tests").bold(),
        passed,
        failed,
        skipped,
    );
}

pub(crate) fn print_test_seed(seed: u64) {
    println!("{}: {seed}", style("Fuzzer seed").bold());
}

pub(crate) fn print_test_result(test_result: &TestCaseSummary, fuzzer_runs: Option<u32>) {
    let result_header = match test_result {
        TestCaseSummary::Passed { .. } => format!("[{}]", style("PASS").green()),
        TestCaseSummary::Failed { .. } => format!("[{}]", style("FAIL").red()),
        TestCaseSummary::Skipped { .. } => format!("[{}]", style("SKIP").yellow()),
    };

    let result_name = match test_result {
        TestCaseSummary::Skipped { name }
        | TestCaseSummary::Failed { name, .. }
        | TestCaseSummary::Passed { name, .. } => name,
    };

    let result_message = match test_result {
        TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
        TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
        _ => String::new(),
    };

    let fuzzer_runs = match fuzzer_runs {
        None => String::new(),
        Some(runs) => {
            if matches!(test_result, TestCaseSummary::Failed { .. }) {
                let arguments = test_result.arguments();
                format!(" (fuzzer runs = {runs}, arguments = {arguments:?})")
            } else {
                format!(" (fuzzer runs = {runs})")
            }
        }
    };

    println!("{result_header} {result_name}{fuzzer_runs}{result_message}");
}

pub fn print_failures(all_failed_tests: &[TestCaseSummary]) {
    if all_failed_tests.is_empty() {
        return;
    }
    let failed_tests_names: Vec<&String> = all_failed_tests
        .iter()
        .map(|test_case_summary| match test_case_summary {
            TestCaseSummary::Passed { name, .. }
            | TestCaseSummary::Failed { name, .. }
            | TestCaseSummary::Skipped { name, .. } => name,
        })
        .collect();

    println!("\nFailures:");
    for name in failed_tests_names {
        println!("    {name}");
    }
}
