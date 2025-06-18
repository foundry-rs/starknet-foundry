use conversions::padded_felt::PaddedFelt;
use serde::{Serialize, Serializer};

use crate::{
    helpers::block_explorer::LinkProvider,
    response::{command::CommandResponse, explorer_link::OutputLink},
};

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

// TODO(#3391): Update text output to be more user friendly
// impl Message for SncastMessage<AccountCreateResponse> {}

impl OutputLink for AccountCreateResponse {
    const TITLE: &'static str = "account creation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("account: {}", provider.contract(self.address))
    }
}
