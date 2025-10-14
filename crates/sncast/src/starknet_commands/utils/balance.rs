use anyhow::{Result, anyhow};
use clap::Args;
use sncast::get_block_id;
use sncast::helpers::rpc::RpcArgs;
use sncast::helpers::token::Token;
use sncast::response::balance::BalanceResponse;
use sncast::response::errors::StarknetCommandError;
use starknet::{
    core::{types::FunctionCall, utils::get_selector_from_name},
    providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport},
};
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone)]
#[group(id = "TokenIdentifier", multiple = false)]
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
            Token::Strk.contract_address()
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

    let res = provider.call(call, block_id).await.map_err(|err| {
        StarknetCommandError::ProviderError(
            sncast::response::errors::SNCastProviderError::UnknownError(err.into()),
        )
    })?;

    let res = res
        .iter()
        .map(|val| u128::from_str_radix(&val.to_string(), 16).expect("Failed to parse u128"))
        .collect::<Vec<u128>>();

    let (low, high) = match res.as_slice() {
        [low, high] => (*low, *high),
        _ => {
            return Err(StarknetCommandError::UnknownError(anyhow!(
                "Balance response should contain exactly two u128 values"
            )));
        }
    };

    let token = match (
        balance.token_identifier.token,
        balance.token_identifier.token_address,
    ) {
        (Some(tok), None) => Some(tok),
        (None, Some(_)) => None,
        (None, None) => Some(Token::default()),
        (Some(_), Some(_)) => unreachable!(
            "Clap should ensure that only one of `--token` or `--token-address` is provided"
        ),
    };

    Ok(BalanceResponse {
        account_address,
        balance: (low, high),
        token,
    })
}
