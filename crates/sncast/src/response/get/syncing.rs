use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust::core::types::SyncStatus;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Debug)]
pub struct SyncingResponse(pub Option<SyncStatus>);

impl SncastCommandMessage for SyncingResponse {
    fn text(&self) -> String {
        let status = if self.0.is_some() {
            "Syncing"
        } else {
            "Not syncing"
        };

        OutputBuilder::new()
            .success_message("Syncing status retrieved")
            .blank_line()
            .field("Status", status)
            .if_some(self.0.as_ref(), |b, status| {
                b.padded_felt_field("Starting Block Hash", &status.starting_block_hash)
                    .field(
                        "Starting Block Number",
                        &status.starting_block_num.to_string(),
                    )
                    .padded_felt_field("Current Block Hash", &status.current_block_hash)
                    .field(
                        "Current Block Number",
                        &status.current_block_num.to_string(),
                    )
                    .padded_felt_field("Highest Block Hash", &status.highest_block_hash)
                    .field(
                        "Highest Block Number",
                        &status.highest_block_num.to_string(),
                    )
            })
            .build()
    }
}

impl Serialize for SyncingResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct SyncingResponseSerialize<'a> {
            syncing: bool,
            #[serde(flatten)]
            status: &'a Option<SyncStatus>,
        }

        SyncingResponseSerialize {
            syncing: self.0.is_some(),
            status: &self.0,
        }
        .serialize(serializer)
    }
}
