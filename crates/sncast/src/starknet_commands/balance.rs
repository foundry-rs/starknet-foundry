use anyhow::{Result, anyhow};
use clap::Args;
use sncast::get_block_id;
use sncast::helpers::rpc::RpcArgs;
use sncast::helpers::token::Token;
use sncast::response::balance::BalanceResponse;
use sncast::response::errors::SNCastProviderError;
use sncast::response::errors::StarknetCommandError;
use starknet::{
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

    /// Token contract address to check the balance for.
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
            Token::Strk.contract_address()
        }
    }

    pub fn displayed_token(&self) -> Option<Token> {
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
) -> Result<BalanceResponse, StarknetCommandError> {
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
    let res: Result<Vec<u128>, _> = res
        .iter()
        .map(|val| {
            u128::from_str_radix(&val.to_string(), 16)
                .map_err(|_| anyhow::anyhow!("Failed to parse balance as u128"))
        })
        .collect();

    let (low, high) = match res?.as_slice() {
        [low, high] => (*low, *high),
        _ => {
            return Err(StarknetCommandError::UnknownError(anyhow!(
                "Balance response should contain exactly two u128 values"
            )));
        }
    };

    Ok(BalanceResponse {
        balance: (low, high),
        token: balance.token_identifier.displayed_token(),
    })
}
