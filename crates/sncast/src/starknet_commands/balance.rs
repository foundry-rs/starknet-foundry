use anyhow::{Error, Result};
use clap::Args;
use primitive_types::U256;
use sncast::get_block_id;
use sncast::helpers::rpc::RpcArgs;
use sncast::helpers::token::Token;
use sncast::response::balance::BalanceResponse;
use sncast::response::errors::SNCastProviderError;
use sncast::response::errors::StarknetCommandError;
use starknet_rust::{
    core::{types::FunctionCall, utils::get_selector_from_name},
    providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport},
};
use starknet_types_core::felt::Felt;
#[derive(Args, Debug, Clone)]
#[group(multiple = false)]
pub struct TokenIdentifier {
    /// Symbol of the token to check the balance for.
    /// Supported tokens are: strk, eth.
    /// Defaults to strk.
    #[arg(value_enum, short = 't', long)]
    pub token: Option<Token>,

    /// Token contract address to check the balance for. Token needs to be compatible with ERC-20 standard.
    #[arg(short = 'd', long)]
    pub token_address: Option<Felt>,
}

impl TokenIdentifier {
    pub fn contract_address(&self) -> Felt {
        if let Some(addr) = self.token_address {
            addr
        } else if let Some(tok) = self.token {
            tok.contract_address()
        } else {
            // Both token and token address are optional, hence we cannot have
            // default value for token at clap level.
            Token::default().contract_address()
        }
    }

    pub fn token_suffix(&self) -> Option<Token> {
        match (self.token, self.token_address) {
            (Some(token), None) => Some(token),
            (None, Some(_)) => None,
            (None, None) => Some(Token::default()),
            (Some(_), Some(_)) => unreachable!(
                "Clap should ensure that only one of `--token` or `--token-address` is provided"
            ),
        }
    }
}

#[derive(Args, Debug)]
#[command(about = "Fetch balance of the account for specified token")]
pub struct Balance {
    #[command(flatten)]
    pub token_identifier: TokenIdentifier,

    /// Block identifier on which balance should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn balance(
    account_address: Felt,
    provider: &JsonRpcClient<HttpTransport>,
    balance: &Balance,
) -> Result<BalanceResponse> {
    let call = FunctionCall {
        contract_address: balance.token_identifier.contract_address(),
        entry_point_selector: get_selector_from_name("balance_of").expect("Failed to get selector"),
        calldata: vec![account_address],
    };
    let block_id = get_block_id(&balance.block_id)?;

    let res = provider
        .call(call, block_id)
        .await
        .map_err(|err| StarknetCommandError::ProviderError(SNCastProviderError::from(err)))?;

    let token = &balance.token_identifier.token_suffix();
    let balance = erc20_balance_to_u256(&res)?;

    Ok(BalanceResponse {
        balance,
        token: *token,
    })
}

fn erc20_balance_to_u256(balance: &[Felt]) -> Result<U256, Error> {
    if balance.len() != 2 {
        return Err(anyhow::anyhow!(
            "Balance response should contain exactly two values"
        ));
    }

    let low: u128 = balance[0].to_string().parse()?;
    let high: u128 = balance[1].to_string().parse()?;

    let mut bytes = [0u8; 32];
    bytes[0..16].copy_from_slice(&low.to_le_bytes());
    bytes[16..32].copy_from_slice(&high.to_le_bytes());

    Ok(U256::from_little_endian(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitive_types::U256;
    use starknet_rust::macros::felt;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_happy_case() {
        let balance = vec![
            Felt::from_hex("0x1").unwrap(),
            Felt::from_hex("0x0").unwrap(),
        ];
        let result = erc20_balance_to_u256(&balance).unwrap();
        assert_eq!(result, U256::from(1u64));

        let balance = vec![
            Felt::from_hex("0xFFFFFFFFFFFFFFFF").unwrap(),
            Felt::from_hex("0x2").unwrap(),
        ];
        let result = erc20_balance_to_u256(&balance).unwrap();
        let expected = U256::from_dec_str("680564733841877041525364481164555130389").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_length() {
        let balance = vec![felt!("0x1"), felt!("0x0"), felt!("0x0")];
        let err = erc20_balance_to_u256(&balance).unwrap_err();
        assert!(
            err.to_string()
                .contains("Balance response should contain exactly two values"),
            "Unexpected error: {err}"
        );

        let balance = vec![Felt::from_hex("0x1").unwrap()];
        let err = erc20_balance_to_u256(&balance).unwrap_err();
        assert!(
            err.to_string()
                .contains("Balance response should contain exactly two values"),
            "Unexpected error: {err}"
        );
    }
}
