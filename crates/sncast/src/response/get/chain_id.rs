use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust_crypto::Felt;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Serialize)]
pub struct ChainIdResponse {
    pub chain_id: Felt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_name: Option<String>,
}

impl SncastCommandMessage for ChainIdResponse {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .success_message("Chain ID retrieved")
            .blank_line()
            .felt_field("Chain ID", &self.chain_id);

        let builder = if let Some(name) = &self.chain_name {
            builder.field("Chain Name", name)
        } else {
            builder
        };

        builder.build()
    }
}
