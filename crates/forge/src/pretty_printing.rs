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

#[expect(clippy::implicit_hasher)]
pub fn print_latest_blocks_numbers(
    url_to_latest_block_number_map: &HashMap<Url, BlockNumber>,
    ui: &UI,
) {
    if !url_to_latest_block_number_map.is_empty() {
        ui.print(&"");
    }
    for (url, latest_block_number) in url_to_latest_block_number_map {
        ui.print(&format!(
            "Latest block number = {latest_block_number} for url = {url}"
        ));
    }
}
