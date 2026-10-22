use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust::core::types::ContractClass;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Clone, Serialize)]
pub struct ClassResponse(pub ContractClass);

impl SncastCommandMessage for ClassResponse {
    fn text(&self) -> String {
        match serde_json::to_string_pretty(self) {
            Ok(res) => res,
            Err(e) => OutputBuilder::new()
                .error_message(&format!("Failed to serialize response: {e}"))
                .build(),
        }
    }
}
