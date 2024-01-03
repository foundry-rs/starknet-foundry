use crate::test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary};
use console::style;

pub(crate) fn print_test_result(any_test_result: &AnyTestCaseSummary) {
    if any_test_result.is_skipped() {
        return;
    }
    let result_header = result_header(any_test_result);
    let result_name = any_test_result.name().unwrap();

    let result_msg = result_message(any_test_result);

    let mut fuzzer_report = None;
    if let AnyTestCaseSummary::Fuzzing(test_result) = any_test_result {
        fuzzer_report = match test_result {
            TestCaseSummary::Passed {
                test_statistics: FuzzingStatistics { runs },
                gas_info,
                ..
            } => Some(format!(
                " (runs: {runs}, gas: {{max: ~{}, min: ~{}, mean: ~{:.2}, std deviation: ~{:.2}}})",
                gas_info.max, gas_info.min, gas_info.mean, gas_info.std_deviation
            )),
            TestCaseSummary::Failed {
                arguments,
                test_statistics: FuzzingStatistics { runs },
                ..
            } => Some(format!(" (runs: {runs}, arguments: {arguments:?})")),
            _ => None,
        };
    }
    let fuzzer_report = fuzzer_report.unwrap_or_else(String::new);

    let gas_usage = match any_test_result {
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
            format!(" (gas: ~{gas_info})")
        }
        _ => String::new(),
    };
    println!("{result_header} {result_name}{fuzzer_report}{gas_usage}{result_msg}");
}

fn result_message(any_test_result: &AnyTestCaseSummary) -> String {
    if let Some(msg) = any_test_result.msg() {
        if any_test_result.is_passed() {
            return format!("\n\nSuccess data:{msg}");
        }
        if any_test_result.is_failed() {
            return format!("\n\nFailure data:{msg}");
        }
    }
    String::new()
}

fn result_header(any_test_result: &AnyTestCaseSummary) -> String {
    if any_test_result.is_passed() {
        return format!("[{}]", style("PASS").green());
    }
    if any_test_result.is_failed() {
        return format!("[{}]", style("FAIL").red());
    }
    if any_test_result.is_ignored() {
        return format!("[{}]", style("IGNORE").yellow());
    }
    unreachable!()
}
