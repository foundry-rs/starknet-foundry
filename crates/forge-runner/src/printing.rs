use crate::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
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
        if let Some(runs) = test_result.runs() {
            fuzzer_report = {
                if matches!(test_result, TestCaseSummary::Failed { .. }) {
                    let arguments = test_result.arguments();
                    Some(format!(
                        " (fuzzer runs = {runs}, arguments = {arguments:?})"
                    ))
                } else {
                    Some(format!(" (fuzzer runs = {runs})"))
                }
            }
        };
    }
    let fuzzer_report = fuzzer_report.unwrap_or_else(String::new);

    let block_number_message = match any_test_result.latest_block_number() {
        None => String::new(),
        Some(latest_block_number) => {
            format!("\nNumber of the block used for fork testing = {latest_block_number}")
        }
    };
    let gas_usage = match any_test_result {
        AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { gas_info, .. }) => {
            let mean = gas_info
                .mean
                .map(|mean| format!("mean = ~{mean}"))
                .unwrap_or_default();
            let std_deviation = gas_info
                .std_deviation
                .map(|std_dev| format!("std deviation = ~{std_dev}"))
                .unwrap_or_default();

            format!(
                ", (max = ~{}, min = ~{}{})",
                gas_info.max,
                gas_info.min,
                [mean, std_deviation].join(", ")
            )
        }
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
            format!(", gas = ~{gas_info}")
        }
        _ => String::new(),
    };
    println!(
        "{result_header} {result_name}{fuzzer_report}{gas_usage}{block_number_message}{result_msg}"
    );
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
