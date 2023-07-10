use crate::test_results::{TestResult, TestSummary};
use anyhow::Error;
use camino::Utf8PathBuf;
use console::style;

pub fn print_error_message(error: &Error) {
    let error_tag = style("ERROR").red();
    println!("[{error_tag}] {error}");
}

pub fn print_collected_tests_count(tests_num: usize, tests_files_num: usize) {
    let plain_text = format!("Collected {tests_num} test(s) and {tests_files_num} test file(s)");
    println!("{}", style(plain_text).bold());
}

pub fn print_running_tests(test_file: &Utf8PathBuf, tests_num: usize) {
    let plain_text = format!("Running {tests_num} test(s) from {test_file}");
    println!("{}", style(plain_text).bold());
}

pub fn print_test_summary(test_summary: &TestSummary) {
    println!(
        "{}: {} passed, {} failed, {} skipped",
        style("Tests").bold(),
        test_summary.passed.len(),
        test_summary.failed.len(),
        test_summary.skipped.len(),
    );
}

pub fn print_test_result(test_result: &TestResult) {
    let result_header = match test_result {
        TestResult::Passed { .. } => format!("[{}]", style("PASS").green()),
        TestResult::Failed { .. } => format!("[{}]", style("FAIL").red()),
        TestResult::Skipped { .. } => format!("[{}]", style("SKIP").yellow()),
    };

    let result_name = match test_result {
        TestResult::Passed { name, .. } => name,
        TestResult::Failed { name, .. } => name,
        TestResult::Skipped { name } => name,
    };

    let result_message = match test_result {
        TestResult::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{}", msg),
        TestResult::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{}", msg),
        _ => String::new(),
    };

    println!("{result_header} {result_name}{result_message}")
}
