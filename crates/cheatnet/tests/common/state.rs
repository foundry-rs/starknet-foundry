use blockifier::state::cached_state::{
    CachedState, GlobalContractCache, GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST,
};
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::ExtendedStateReader;
use starknet_api::block::BlockNumber;

pub fn create_cached_state() -> CachedState<ExtendedStateReader> {
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: None,
        },
        GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
    )
}

pub fn create_fork_cached_state(cache_dir: &str) -> CachedState<ExtendedStateReader> {
    create_fork_cached_state_at(54_060, cache_dir)
}

pub fn create_fork_cached_state_at(
    block_number: u64,
    cache_dir: &str,
) -> CachedState<ExtendedStateReader> {
    let node_url = "http://188.34.188.184:7070/rpc/v0_7".parse().unwrap();
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(
                ForkStateReader::new(node_url, BlockNumber(block_number), cache_dir).unwrap(),
            ),
        },
        GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
    )
}
