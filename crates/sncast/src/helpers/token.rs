use anyhow::{Result, anyhow};
use clap::ValueEnum;
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use serde::{Deserialize, Serialize};
use starknet::macros::felt;
use starknet_types_core::felt::Felt;
use std::{fmt, str::FromStr};

const STRK_CONTRACT_ADDRESS: Felt =
    felt!("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d");

const ETH_CONTRACT_ADDRESS: Felt =
    felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7");

impl CairoSerialize for Token {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            Token::Strk => output.write_felt(0.into()),
            Token::Eth => output.write_felt(1.into()),
        }
    }
}
#[derive(Default, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Token {
    #[default]
    Strk,
    Eth,
}

impl Token {
    #[must_use]
    pub fn contract_address(self) -> Felt {
        match self {
            Token::Strk => STRK_CONTRACT_ADDRESS,
            Token::Eth => ETH_CONTRACT_ADDRESS,
        }
    }
}

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strk" => Ok(Token::Strk),
            "eth" => Ok(Token::Eth),
            account_type => Err(anyhow!("Invalid token: {account_type}")),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Strk => write!(f, "strk"),
            Token::Eth => write!(f, "eth"),
        }
    }
}
