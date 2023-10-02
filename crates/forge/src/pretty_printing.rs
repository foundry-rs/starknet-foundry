use crate::test_case_summary::TestCaseSummary;
use crate::{TestCrateSummary, TestCrateType};
use anyhow::{Error, Result};
use console::style;
use starknet::core::types::MaybePendingBlockWithTxs::{Block, PendingBlock};
use starknet::core::types::{BlockId, BlockWithTxs, PendingBlockWithTxs};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use tokio::runtime::Runtime;
use url::Url;

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

pub(crate) fn print_test_result(
    test_result: &TestCaseSummary,
    fuzzer_runs: Option<u32>,
) -> Result<()> {
    let result_header = match test_result {
        TestCaseSummary::Passed { .. } => format!("[{}]", style("PASS").green()),
        TestCaseSummary::Failed { .. } => format!("[{}]", style("FAIL").red()),
        TestCaseSummary::Skipped { .. } => format!("[{}]", style("SKIP").yellow()),
    };

    let result_name = test_result.name();

    let result_message = match test_result {
        TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
        TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
        _ => String::new(),
    };
    let fork_params = test_result.fork_params();
    let block_message = get_block_message(fork_params)?;

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

    println!("{result_header} {result_name}{fuzzer_runs}{block_message}{result_message}");
    Ok(())
}

fn get_block_message(fork_params: Option<&(String, BlockId)>) -> Result<String> {
    let result = if let Some((url, block_id)) = fork_params {
        match block_id {
            BlockId::Tag(_) => {
                let client = JsonRpcClient::new(HttpTransport::new(
                    Url::parse(url).unwrap_or_else(|_| panic!("Could not parse the {url} URL.")),
                ));
                let runtime = Runtime::new().expect("Could not instantiate Runtime");
                match runtime.block_on(client.get_block_with_txs(block_id))? {
                    Block(BlockWithTxs {
                        block_number,
                        block_hash,
                        ..
                    }) => format!("\nBlock number: {block_number}, block hash: {block_hash}"),
                    PendingBlock(PendingBlockWithTxs { parent_hash, .. }) => {
                        format!("\nBlock parent hash: {parent_hash}")
                    }
                }
            }
            _ => String::new(),
        }
    } else {
        String::new()
    };

    Ok(result)
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
