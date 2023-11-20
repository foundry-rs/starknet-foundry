use crate::test_case_summary::TestCaseSummary;
use console::style;

pub(crate) fn print_test_result(test_result: &TestCaseSummary) {
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

    let fuzzer_report = match test_result.runs() {
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

    let block_number_message = match test_result.latest_block_number() {
        None => String::new(),
        Some(latest_block_number) => {
            format!("\nNumber of the block used for fork testing = {latest_block_number}")
        }
    };

    let _gas_report = match test_result {
        TestCaseSummary::Passed { gas, .. } => {
            format!("\nGas used: {gas}\n")
        }
        _ => String::new(),
    };

    // uncomment this to see used gas
    // println!("{result_header} {result_name}{fuzzer_report}{block_number_message}{result_message}{_gas_report}");

    println!("{result_header} {result_name}{fuzzer_report}{block_number_message}{result_message}");
}
