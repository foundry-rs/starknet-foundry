use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, Clone)]
pub struct NonceResponse {
    pub nonce: Felt,
}

impl SncastCommandMessage for NonceResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Nonce retrieved")
            .blank_line()
            .field("Nonce", &self.nonce.to_string())
            .build()
    }
}
