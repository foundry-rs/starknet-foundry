use crate::response::cast_message::SncastCommandMessage;
use conversions::serde::serialize::CairoSerialize;
use conversions::string::IntoHexStr;
use foundry_ui::styling;
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, CairoSerialize, Clone)]
pub struct NonceResponse {
    pub nonce: Felt,
}

impl SncastCommandMessage for NonceResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Nonce retrieved")
            .blank_line()
            .field("Nonce", &self.nonce.into_hex_string())
            .build()
    }
}
