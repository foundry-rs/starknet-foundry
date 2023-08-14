use crate::test_case_summary::TestCaseSummary;
use crate::TestFileSummary;
use anyhow::Error;
use camino::Utf8PathBuf;
use console::style;
use num_traits::ToPrimitive;

pub fn print_error_message(error: &Error) {
    let error_tag = style("ERROR").red();
    println!("[{error_tag}] {error}");
}

pub(crate) fn print_collected_tests_count(tests_num: usize, tests_files_num: usize) {
    let plain_text = format!("Collected {tests_num} test(s) and {tests_files_num} test file(s)");
    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_running_tests(test_file: &Utf8PathBuf, package_name: &str, tests_num: usize) {
    let plain_text = if test_file == "src/lib.cairo" {
        format!("Running {tests_num} test(s) from {package_name} package")
    } else {
        format!("Running {tests_num} test(s) from {test_file}")
    };

    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_test_summary(summaries: &[TestFileSummary]) {
    let passed: usize = summaries.iter().map(TestFileSummary::count_passed).sum();
    let skipped: usize = summaries.iter().map(TestFileSummary::count_skipped).sum();
    let failed: usize = summaries.iter().map(TestFileSummary::count_failed).sum();

    println!(
        "{}: {} passed, {} failed, {} skipped",
        style("Tests").bold(),
        passed,
        failed,
        skipped,
    );
}

pub(crate) fn print_test_result(test_result: &TestCaseSummary) {
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

    let gas_used = match test_result {
        TestCaseSummary::Passed {
            available_gas,
            run_result,
            ..
        }
        | TestCaseSummary::Failed {
            available_gas,
            run_result: Some(run_result),
            ..
        } if available_gas.is_some() => run_result.gas_counter.clone().and_then(|gas_counter| {
            gas_counter
                .to_usize()
                .map(|gas| available_gas.unwrap() - gas)
        }),
        _ => None,
    };

    let gas_estimation_message =
        gas_used.map_or(String::new(), |gas| format!(" (gas usage est.: {gas}) "));

    let result_message = match test_result {
        TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
        TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
        _ => String::new(),
    };

    println!("{result_header} {result_name}{gas_estimation_message}{result_message}");
}
