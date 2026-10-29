use std::process::ExitCode;

use crate::starknet_commands::utils::felt_or_id::ContractAddress;
use anyhow::{Context, Result};
use clap::Args;
use sncast::{
    get_block_id,
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{
        errors::{StarknetCommandError, handle_starknet_command_error},
        get::storage::StorageResponse,
        ui::UI,
    },
};
use starknet_rust::{
    core::{types::StorageResponseFlag, utils::get_selector_from_name},
    providers::Provider,
};
use starknet_rust_crypto::Felt;

#[derive(Args, Debug)]
pub struct Storage {
    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(flatten)]
    pub key: KeySelector,

    /// Address of the contract (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short = 'd', long)]
    pub contract_address: ContractAddress,

    /// Block identifier from which storage value should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    /// Fetch last block that modified slot under `key`
    #[arg(short = 'l', long)]
    pub include_last_block: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct KeySelector {
    /// Storage key as a Felt in hexadecimal (0x-prefixed) or decimal representation.
    #[arg(short, long)]
    pub felt_key: Option<Felt>,

    /// Storage key as a string. Its selector is used as the storage address.
    #[arg(short, long)]
    pub key: Option<String>,
}

impl KeySelector {
    fn resolve(&self) -> Result<Felt> {
        match (&self.key, &self.felt_key) {
            (Some(key), None) => get_selector_from_name(key)
                .context("failed to calculate selector from short string"),
            (None, Some(felt)) => Ok(*felt),
            _ => unreachable!("Either `felt` or `key` is `Some`"),
        }
    }

    fn key(self) -> String {
        match (self.key, self.felt_key) {
            (Some(key), None) => key,
            (None, Some(felt)) => felt.to_hex_string(),
            _ => unreachable!("Either `felt` or `key` is `Some`"),
        }
    }
}

pub async fn storage(storage: Storage, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let flags = get_flags(&storage);
    let contract_address = storage.contract_address.resolve(&config)?;
    let key = storage.key.resolve()?;
    let block_id = get_block_id(&storage.block_id)?;

    let provider = storage.rpc.get_provider(&config, ui).await?;

    let result = provider
        .get_storage_at(contract_address, key, block_id, Some(&flags))
        .await
        .map(|res| StorageResponse::new(storage.key.key(), key, res))
        .map_err(|e| StarknetCommandError::ProviderError(e.into()))
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get storage", result, ui, None))
}

fn get_flags(storage: &Storage) -> Vec<StorageResponseFlag> {
    let mut res = vec![];

    if storage.include_last_block {
        res.push(StorageResponseFlag::IncludeLastUpdateBlock);
    }

    res
}
