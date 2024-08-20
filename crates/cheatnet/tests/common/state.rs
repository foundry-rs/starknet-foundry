use blockifier::state::cached_state::CachedState;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::ExtendedStateReader;
use shared::test_utils::node_url::node_rpc_url;
use starknet_api::block::BlockNumber;

pub fn create_cached_state() -> CachedState<ExtendedStateReader> {
    CachedState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(),
        fork_state_reader: None,
    })
}

pub fn create_fork_cached_state(cache_dir: &str) -> CachedState<ExtendedStateReader> {
    create_fork_cached_state_at(54_060, cache_dir)
}

pub fn create_fork_cached_state_at(
    block_number: u64,
    cache_dir: &str,
) -> CachedState<ExtendedStateReader> {
    let node_url = node_rpc_url();
    CachedState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(),
        fork_state_reader: Some(
            ForkStateReader::new(node_url, BlockNumber(block_number), cache_dir.into()).unwrap(),
        ),
    })
}
