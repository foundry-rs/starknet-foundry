use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use cheatnet::state::{CheatnetState, ExtendedStateReader};
use starknet_api::block::BlockNumber;

pub fn create_cached_state() -> CachedState<ExtendedStateReader> {
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: None,
        },
        GlobalContractCache::default(),
    )
}

pub fn create_fork_cached_state(cache_dir: &str) -> CachedState<ExtendedStateReader> {
    let node_url = "http://188.34.188.184:9545/rpc/v0_6".parse().unwrap();
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(
                node_url,
                BlockNumber(320_000),
                cache_dir,
            )),
        },
        GlobalContractCache::default(),
    )
}

pub fn create_fork_cached_state_at(
    block_number: BlockNumber,
    cache_dir: &str,
) -> CachedState<ExtendedStateReader> {
    let node_url = "http://188.34.188.184:9545/rpc/v0_6".parse().unwrap();
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(node_url, block_number, cache_dir)),
        },
        GlobalContractCache::default(),
    )
}

pub fn build_runtime_state(cheatnet_state: &mut CheatnetState) -> RuntimeState {
    RuntimeState { cheatnet_state }
}
