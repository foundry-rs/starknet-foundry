use crate::compiled_raw::RawForkParams;
use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use conversions::IntoConv;
use forge_runner::compiled_runnable::ValidatedForkConfig;
use num_bigint::BigInt;
use starknet::core::types::BlockTag::Latest;
use starknet::core::types::{BlockId, MaybePendingBlockWithTxHashes};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_api::block::BlockNumber;
use std::collections::HashMap;
use tokio::runtime::Handle;
use url::Url;

#[derive(Default)]
pub struct BlockNumberMap {
    url_to_latest_block_number: HashMap<String, BlockNumber>,
    url_and_hash_to_block_number: HashMap<(String, Felt252), BlockNumber>,
}

impl BlockNumberMap {
    fn add_latest_block_number(&mut self, url: String, latest_block_number: BlockNumber) {
        self.url_to_latest_block_number
            .insert(url, latest_block_number);
    }

    fn add_block_number_for_hash(&mut self, url: String, hash: Felt252, block_number: BlockNumber) {
        self.url_and_hash_to_block_number
            .insert((url, hash), block_number);
    }

    fn get_latest_block_number(&mut self, url: &String) -> Option<&BlockNumber> {
        self.url_to_latest_block_number.get(url)
    }

    fn get_block_number_for_hash(&mut self, url: String, hash: Felt252) -> Option<&BlockNumber> {
        self.url_and_hash_to_block_number.get(&(url, hash))
    }

    #[must_use]
    pub fn get_url_to_latest_block_number(&self) -> &HashMap<String, BlockNumber> {
        &self.url_to_latest_block_number
    }

    pub async fn validated_fork_config_from_fork_params(
        &mut self,
        fork_params_string: &RawForkParams,
    ) -> Result<ValidatedForkConfig> {
        let url_str = fork_params_string.url.clone();
        let url = fork_params_string.url.parse()?;
        let block_number = match fork_params_string.block_id_type.to_lowercase().as_str() {
            "number" => BlockNumber(fork_params_string.block_id_value.parse()?),
            "hash" => {
                let block_hash =
                    Felt252::from(fork_params_string.block_id_value.parse::<BigInt>().unwrap());
                if let Some(block_number) =
                    self.get_block_number_for_hash(url_str.clone(), block_hash.clone())
                {
                    *block_number
                } else {
                    let block_number = get_block_number_from_hash(&url, &block_hash).await?;
                    self.add_block_number_for_hash(url_str, block_hash, block_number);
                    block_number
                }
            }
            "tag" => {
                assert_eq!(fork_params_string.block_id_value, "Latest");
                if let Some(block_number) = self.get_latest_block_number(&url_str) {
                    *block_number
                } else {
                    let latest_block_number = get_latest_block_number(&url).await?;
                    self.add_latest_block_number(url_str, latest_block_number);
                    latest_block_number
                }
            }
            _ => unreachable!(),
        };
        Ok(ValidatedForkConfig { url, block_number })
    }
}

async fn get_latest_block_number(url: &Url) -> Result<BlockNumber> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    match Handle::current()
        .spawn(async move { client.get_block_with_tx_hashes(BlockId::Tag(Latest)).await })
        .await?
    {
        Ok(MaybePendingBlockWithTxHashes::Block(block)) => Ok(BlockNumber(block.block_number)),
        _ => Err(anyhow!("Could not get the latest block number".to_string())),
    }
}

async fn get_block_number_from_hash(url: &Url, block_hash: &Felt252) -> Result<BlockNumber> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let hash = BlockId::Hash((*block_hash).clone().into_());
    match Handle::current()
        .spawn(async move { client.get_block_with_tx_hashes(hash).await })
        .await?
    {
        Ok(MaybePendingBlockWithTxHashes::Block(block)) => Ok(BlockNumber(block.block_number)),
        _ => Err(anyhow!(format!(
            "Could not get the block number for block with hash 0x{}",
            block_hash.to_str_radix(16)
        ))),
    }
}
