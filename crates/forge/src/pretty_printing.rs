use forge_runner::test_case_summary::AnyTestCaseSummary;
use foundry_ui::{UI, components::typed::LabeledMessage};
use starknet_api::block::BlockNumber;
use std::collections::HashMap;
use url::Url;

pub(crate) fn print_test_seed(seed: u64, ui: &UI) {
    ui.print(&LabeledMessage::styled(
        "Fuzzer seed",
        &seed.to_string(),
        "bold",
    ));
}

pub fn print_failures(all_failed_tests: &[AnyTestCaseSummary]) {
    if all_failed_tests.is_empty() {
        return;
    }
    let failed_tests_names = all_failed_tests
        .iter()
        .map(|any_test_case_summary| any_test_case_summary.name().unwrap());

    println!("\nFailures:");
    for name in failed_tests_names {
        println!("    {name}");
    }
}

#[expect(clippy::implicit_hasher)]
pub fn print_latest_blocks_numbers(url_to_latest_block_number_map: &HashMap<Url, BlockNumber>) {
    if !url_to_latest_block_number_map.is_empty() {
        println!();
    }
    for (url, latest_block_number) in url_to_latest_block_number_map {
        println!("Latest block number = {latest_block_number} for url = {url}");
    }
}
