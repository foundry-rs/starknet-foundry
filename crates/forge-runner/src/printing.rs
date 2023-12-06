use crate::{test_case_summary::TestCaseSummary, test_crate_summary::AnyTestCaseSummary};
use console::style;

pub(crate) fn print_test_result(any_test_result: &AnyTestCaseSummary) {
    match any_test_result {
        AnyTestCaseSummary::Fuzzing(test_result) => {
            if let TestCaseSummary::Skipped { .. } = test_result {
                return;
            }

            let result_header = match test_result {
                TestCaseSummary::Passed { .. } => format!("[{}]", style("PASS").green()),
                TestCaseSummary::Failed { .. } => format!("[{}]", style("FAIL").red()),
                TestCaseSummary::Ignored { .. } => format!("[{}]", style("IGNORE").yellow()),
                TestCaseSummary::Skipped { .. } => {
                    unreachable!()
                }
            };

            let result_name = match test_result {
                TestCaseSummary::Ignored { name }
                | TestCaseSummary::Failed { name, .. }
                | TestCaseSummary::Passed { name, .. } => name,
                TestCaseSummary::Skipped {} => {
                    unreachable!()
                }
            };

            let result_message = match test_result {
                TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
                TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
                _ => String::new(),
            };

            let mut fuzzer_report = String::new();
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

            let block_number_message = match test_result.latest_block_number() {
                None => String::new(),
                Some(latest_block_number) => {
                    format!("\nNumber of the block used for fork testing = {latest_block_number}")
                }
            };

            let mut gas_usage = String::new();

            if let Some(result) = test_result.gas_usage() {
                gas_usage = format!(", gas: {result}");
            }

            println!("{result_header} {result_name}{fuzzer_report}{gas_usage}{block_number_message}{result_message}");
        }
        AnyTestCaseSummary::Single(test_result)  => {
            if let TestCaseSummary::Skipped { .. } = test_result {
                return;
            }

            let result_header = match test_result {
                TestCaseSummary::Passed { .. } => format!("[{}]", style("PASS").green()),
                TestCaseSummary::Failed { .. } => format!("[{}]", style("FAIL").red()),
                TestCaseSummary::Ignored { .. } => format!("[{}]", style("IGNORE").yellow()),
                TestCaseSummary::Skipped { .. } => {
                    unreachable!()
                }
            };

            let result_name = match test_result {
                TestCaseSummary::Ignored { name }
                | TestCaseSummary::Failed { name, .. }
                | TestCaseSummary::Passed { name, .. } => name,
                TestCaseSummary::Skipped {} => {
                    unreachable!()
                }
            };

            let result_message = match test_result {
                TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
                TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
                _ => String::new(),
            };
        }
    }
}
