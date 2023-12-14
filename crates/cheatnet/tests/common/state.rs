use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use blockifier::state::state_api::State;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::{BlockifierState, CheatnetState, ExtendedStateReader};
use starknet::core::types::BlockTag::Latest;
use starknet::core::types::{BlockId, MaybePendingBlockWithTxHashes};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_api::block::BlockNumber;
use tokio::runtime::Handle;
use url::Url;

pub fn create_cached_state() -> CachedState<ExtendedStateReader> {
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: None,
        },
        GlobalContractCache::default(),
    )
}

async fn get_latest_block_number(url: &Url) -> BlockNumber {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    match Handle::current()
        .spawn(async move { client.get_block_with_tx_hashes(BlockId::Tag(Latest)).await })
        .await
        .unwrap()
    {
        Ok(MaybePendingBlockWithTxHashes::Block(block)) => BlockNumber(block.block_number),
        _ => panic!("Could not get the latest block number"),
    }
}

pub async fn create_fork_cached_state(cache_dir: &str) -> CachedState<ExtendedStateReader> {
    let node_url = "http://188.34.188.184:9545/rpc/v0.5".parse().unwrap();
    let block_num = get_latest_block_number(&node_url).await;
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(node_url, block_num, cache_dir)),
        },
        GlobalContractCache::default(),
    )
}

pub fn create_fork_cached_state_at(
    block_number: BlockNumber,
    cache_dir: &str,
) -> CachedState<ExtendedStateReader> {
    let node_url = "http://188.34.188.184:9545/rpc/v0.5".parse().unwrap();
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(node_url, block_number, cache_dir)),
        },
        GlobalContractCache::default(),
    )
}

pub fn create_cheatnet_state(state: &mut dyn State) -> (BlockifierState, CheatnetState) {
    let blockifier_state = BlockifierState::from(state);
    let cheatnet_state = CheatnetState::default();
    (blockifier_state, cheatnet_state)
}
