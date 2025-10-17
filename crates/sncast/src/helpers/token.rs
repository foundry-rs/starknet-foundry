use clap::ValueEnum;
use serde::Serialize;
use starknet::macros::felt;
use starknet_types_core::felt::Felt;

const STRK_CONTRACT_ADDRESS: Felt =
    felt!("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d");

const ETH_CONTRACT_ADDRESS: Felt =
    felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7");

#[derive(Default, Serialize, Clone, Copy, Debug, ValueEnum, strum_macros::Display)]
// Both serde and strum enums need to have proper
// casing configuration in clap and for serialization.
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Token {
    #[default]
    Strk,
    Eth,
}

impl Token {
    #[must_use]
    pub fn contract_address(&self) -> Felt {
        match self {
            Token::Strk => STRK_CONTRACT_ADDRESS,
            Token::Eth => ETH_CONTRACT_ADDRESS,
        }
    }
}
