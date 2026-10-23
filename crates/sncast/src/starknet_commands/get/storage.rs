use std::{process::ExitCode, str::FromStr};

use crate::starknet_commands::utils::felt_or_id::ContractAddress;
use anyhow::{Context, Result};
use clap::Args;
use sncast::{
    get_block_id,
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{get::storage::StorageResponse, ui::UI},
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

    /// Address of the contract (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short = 'd', long)]
    pub contract_address: ContractAddress,

    /// Block identifier from which storage value should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    /// Storage key. Either a Felt252 (hex, decimal) or a plaintext
    pub key: String,

    /// Fetch last block that modified slot under `key`
    #[arg(short = 'l', long)]
    pub include_last_block: bool,
}

pub async fn storage(storage: Storage, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let flags = get_flags(&storage);
    let contract_address = storage.contract_address.resolve(&config)?;
    let key = get_key(&storage.key)?;
    let block_id = get_block_id(&storage.block_id)?;

    let provider = storage.rpc.get_provider(&config, ui).await?;

    let res = provider
        .get_storage_at(contract_address, key, block_id, Some(&flags))
        .await?;

    Ok(process_command_result(
        "get storage",
        Ok(StorageResponse::new(storage.key, res)),
        ui,
        None,
    ))
}

fn get_flags(storage: &Storage) -> Vec<StorageResponseFlag> {
    let mut res = vec![];

    if storage.include_last_block {
        res.push(StorageResponseFlag::IncludeLastUpdateBlock);
    }

    res
}

fn get_key(key: &str) -> Result<Felt> {
    if let Ok(key) = Felt::from_str(key) {
        return Ok(key);
    }

    get_selector_from_name(key).context("failed to calculate selector from short string")
}
