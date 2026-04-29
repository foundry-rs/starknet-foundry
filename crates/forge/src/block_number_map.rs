use anyhow::{Result, anyhow};
use conversions::{IntoConv, string::IntoHexStr};
use starknet_api::block::BlockNumber;
use starknet_rust::{
    core::types::{BlockId, MaybePreConfirmedBlockWithTxHashes},
    providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport},
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::runtime::Handle;
use url::Url;

/// Caches block numbers fetched from RPC nodes, shared across concurrent config passes.
#[derive(Default)]
pub struct BlockNumberMap {
    url_to_latest_block_number: Mutex<HashMap<Url, BlockNumber>>,
    url_and_hash_to_block_number: Mutex<HashMap<(Url, Felt), BlockNumber>>,
}

impl BlockNumberMap {
    pub async fn get_latest_block_number(&self, url: Url) -> Result<BlockNumber> {
        // Release lock before awaiting.
        {
            let map = self.url_to_latest_block_number.lock().unwrap();
            if let Some(&block_number) = map.get(&url) {
                return Ok(block_number);
            }
        }

        let fetched = fetch_latest_block_number(url.clone()).await?;

        // or_insert avoids overwriting if a concurrent task raced us.
        let mut map = self.url_to_latest_block_number.lock().unwrap();
        Ok(*map.entry(url).or_insert(fetched))
    }

    pub async fn get_block_number_for_hash(&self, url: Url, hash: Felt) -> Result<BlockNumber> {
        {
            let map = self.url_and_hash_to_block_number.lock().unwrap();
            if let Some(&block_number) = map.get(&(url.clone(), hash)) {
                return Ok(block_number);
            }
        }

        let fetched = fetch_block_number_for_hash(url.clone(), hash).await?;

        let mut map = self.url_and_hash_to_block_number.lock().unwrap();
        Ok(*map.entry((url, hash)).or_insert(fetched))
    }

    #[must_use]
    pub fn get_url_to_latest_block_number(&self) -> HashMap<Url, BlockNumber> {
        self.url_to_latest_block_number.lock().unwrap().clone()
    }
}

async fn fetch_latest_block_number(url: Url) -> Result<BlockNumber> {
    let client = JsonRpcClient::new(HttpTransport::new(url));

    Ok(Handle::current()
        .spawn(async move { client.block_number().await })
        .await?
        .map(BlockNumber)?)
}

async fn fetch_block_number_for_hash(url: Url, block_hash: Felt) -> Result<BlockNumber> {
    let client = JsonRpcClient::new(HttpTransport::new(url));

    let hash = BlockId::Hash(block_hash.into_());

    match Handle::current()
        .spawn(async move { client.get_block_with_tx_hashes(hash).await })
        .await?
    {
        Ok(MaybePreConfirmedBlockWithTxHashes::Block(block)) => Ok(BlockNumber(block.block_number)),
        _ => Err(anyhow!(
            "Could not get the block number for block with hash 0x{}",
            block_hash.into_hex_string()
        )),
    }
}
