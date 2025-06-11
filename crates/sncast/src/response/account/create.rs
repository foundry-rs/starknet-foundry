use crate::response::cast_message::SncastMessage;
use crate::{
    helpers::block_explorer::LinkProvider,
    response::{command::CommandResponse, explorer_link::OutputLink},
};
use conversions::padded_felt::PaddedFelt;
use conversions::string::IntoPaddedHexStr;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::{Serialize, Serializer};
use serde_json::Value;

fn as_str<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

#[derive(Serialize, Debug, Clone)]
pub struct AccountCreateResponse {
    pub address: PaddedFelt,
    #[serde(serialize_with = "as_str")]
    pub estimated_fee: u128,
    pub add_profile: Option<String>,
    pub message: String,
}

impl CommandResponse for AccountCreateResponse {}

impl Message for SncastMessage<AccountCreateResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account created")
            .blank_line()
            .field(
                "Address",
                &self.command_response.address.into_padded_hex_str(),
            )
            .field(
                "Estimated Fee",
                &self.command_response.estimated_fee.to_string(),
            )
            .if_some(
                self.command_response.add_profile.as_ref(),
                |builder, profile| builder.field("Add Profile", profile),
            )
            .blank_line()
            .text_field(&self.command_response.message)
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}

impl OutputLink for AccountCreateResponse {
    const TITLE: &'static str = "account creation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("account: {}", provider.contract(self.address))
    }
}
