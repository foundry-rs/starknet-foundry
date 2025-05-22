use conversions::padded_felt::PaddedFelt;
use serde::{Serialize, Serializer};
use starknet_types_core::felt::Felt;

use crate::{
    helpers::block_explorer::LinkProvider,
    response::{command::CommandResponse, explorer_link::OutputLink},
};

#[derive(Clone)]
pub struct Decimal(pub u64);

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

fn serialize_as_decimal<S>(value: &Felt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:#}"))
}

#[derive(Serialize, Debug, Clone)]
pub struct AccountCreateResponse {
    pub address: PaddedFelt,
    #[serde(serialize_with = "serialize_as_decimal")]
    pub max_fee: Felt,
    pub add_profile: String,
    pub message: String,
}

impl CommandResponse for AccountCreateResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for CastMessage<AccountCreateResponse> {}

impl OutputLink for AccountCreateResponse {
    const TITLE: &'static str = "account creation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("account: {}", provider.contract(self.address))
    }
}
