use crate::helpers::token::TokenUnit;
use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use primitive_types::U256;
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug)]
pub struct BalanceResponse {
    pub balance: U256,
    pub token_unit: Option<TokenUnit>,
}

impl SncastCommandMessage for BalanceResponse {
    fn text(&self) -> String {
        let balance_str = if let Some(token_unit) = self.token_unit {
            format!("{} {token_unit}", self.balance)
        } else {
            self.balance.to_string()
        };

        styling::OutputBuilder::new()
            .field("Balance", &balance_str)
            .build()
    }
}

// We need custom serialization because U256's default serialization is hex string
impl Serialize for BalanceResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("BalanceResponse", 2)?;
        // Default U256 serialization uses hex string, we want decimal string
        s.serialize_field("balance", &self.balance.to_string())?;
        s.serialize_field("token_unit", &self.token_unit)?;
        s.end()
    }
}
