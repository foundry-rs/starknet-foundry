use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust::core::types::GetStorageAtResult;
use starknet_rust_crypto::Felt;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Serialize)]
pub struct StorageResponse {
    pub key: String,
    pub storage_address: Felt,
    pub value: Felt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_update_block: Option<u64>,
}

impl SncastCommandMessage for StorageResponse {
    fn text(&self) -> String {
        OutputBuilder::new()
            .success_message("Value retrieved")
            .blank_line()
            .field("Key", &self.key)
            .felt_field("Storage address", &self.storage_address)
            .felt_field("Value", &self.value)
            .if_some(self.last_update_block.as_ref(), |builder, block| {
                builder.field("Last update block", &block.to_string())
            })
            .build()
    }
}

impl StorageResponse {
    #[must_use]
    pub fn new(key: String, storage_address: Felt, result: GetStorageAtResult) -> Self {
        match result {
            GetStorageAtResult::Value(value) => Self {
                key,
                storage_address,
                value,
                last_update_block: None,
            },
            GetStorageAtResult::ValueWithMetadata(res) => Self {
                key,
                storage_address,
                value: res.value,
                last_update_block: Some(res.last_update_block),
            },
        }
    }
}
