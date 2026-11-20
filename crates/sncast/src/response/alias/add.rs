use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust_crypto::Felt;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Serialize)]
pub struct AliasAddMessage {
    pub name: String,
    pub profile: String,
    pub value: Felt,
}

impl SncastCommandMessage for AliasAddMessage {
    fn text(&self) -> String {
        OutputBuilder::new()
            .success_message("Alias added")
            .blank_line()
            .field("Profile", &self.profile)
            .felt_field(&self.name, &self.value)
            .build()
    }
}
