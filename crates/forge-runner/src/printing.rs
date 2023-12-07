use crate::{test_case_summary::TestCaseSummary, test_crate_summary::AnyTestCaseSummary};
use console::style;

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


pub(crate) fn print_test_result(any_test_result: &AnyTestCaseSummary) {
    let result_header = result_header(any_test_result);
    let result_name = any_test_result.name().unwrap();
    let result_msg = any_test_result.msg().ok_or(Some(String::new())).unwrap();
    let mut fuzzer_report = String::new();
    if let AnyTestCaseSummary::Fuzzing(test_result) = any_test_result {
        if let Some(runs) = test_result.runs() {
            fuzzer_report = {
                if matches!(test_result, TestCaseSummary::Failed { .. }) {
                    let arguments = test_result.arguments();
                    format!(" (fuzzer runs = {runs}, arguments = {arguments:?})")
                } else {
                    format!(" (fuzzer runs = {runs})")
                }
            }
        };
    }
    let block_number_message = match any_test_result.latest_block_number() {
        None => String::new(),
        Some(latest_block_number) => {
            format!("\nNumber of the block used for fork testing = {latest_block_number}")
        }
    };
    let mut gas_usage = String::new();
    match any_test_result {
        AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { gas_info, .. }) => {
            gas_usage = format!("(max: ~{}, min: ~{})", gas_info.max, gas_info.min);
        }
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
            gas_usage = format!(", gas: {gas_info}");
        }
        _ => {}
    }
    println!("{result_header} {result_name}{fuzzer_report}{gas_usage}{block_number_message}{result_msg}");
}
