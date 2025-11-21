use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::HashMap, fmt::Write};

#[derive(Serialize)]
pub struct LatestBlocksNumbersMessage {
    url_to_latest_block_number_map: HashMap<url::Url, starknet_api::block::BlockNumber>,
}

impl LatestBlocksNumbersMessage {
    #[must_use]
    pub fn new(
        url_to_latest_block_number_map: HashMap<url::Url, starknet_api::block::BlockNumber>,
    ) -> Self {
        Self {
            url_to_latest_block_number_map,
        }
    }
}

impl Message for LatestBlocksNumbersMessage {
    fn text(&self) -> String {
        let mut output = String::new();
        output = format!("{output}\n");

        for (url, latest_block_number) in &self.url_to_latest_block_number_map {
            let _ = writeln!(
                &mut output,
                "Latest block number = {latest_block_number} for url = {url}"
            );
        }

        output
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
