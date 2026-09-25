use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust::core::types::SyncStatus;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Debug, Serialize)]
pub struct SyncingResponse {
    pub syncing: bool,
    #[serde(flatten)]
    pub status: Option<SyncStatus>,
}

impl SncastCommandMessage for SyncingResponse {
    fn text(&self) -> String {
        let status = if self.syncing {
            "Syncing"
        } else {
            "Not Syncing"
        };

        let builder = OutputBuilder::new()
            .success_message("Syncing status retrieved")
            .blank_line()
            .field("Status", status);

        let builder = if let Some(status) = self.status.as_ref() {
            builder
                .felt_field("Starting Block Hash", &status.starting_block_hash)
                .field("Starting Block Num", &status.starting_block_num.to_string())
                .felt_field("Current Block Hash", &status.current_block_hash)
                .field("Current Block Num", &status.current_block_num.to_string())
                .felt_field("Highest Block Hash", &status.highest_block_hash)
                .field("Highest Block Num", &status.highest_block_num.to_string())
        } else {
            builder
        };

        builder.build()
    }
}
