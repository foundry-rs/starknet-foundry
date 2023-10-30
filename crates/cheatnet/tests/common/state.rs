use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use blockifier::state::state_api::State;
use camino::Utf8PathBuf;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::{BlockifierState, CheatnetState, ExtendedStateReader};
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;

#[allow(clippy::module_name_repetitions)]
pub fn create_cached_state() -> CachedState<ExtendedStateReader> {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(&predeployed_contracts),
            fork_state_reader: None,
        },
        GlobalContractCache::default(),
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn create_fork_cached_state() -> CachedState<ExtendedStateReader> {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4".parse().unwrap();
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(&predeployed_contracts),
            fork_state_reader: Some(ForkStateReader::new(node_url, BlockId::Tag(Latest), None)),
        },
        GlobalContractCache::default(),
    )
}

pub fn create_fork_cached_state_at(
    block_id: BlockId,
    cache_dir: &str,
) -> CachedState<ExtendedStateReader> {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4".parse().unwrap();
    CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(&predeployed_contracts),
            fork_state_reader: Some(ForkStateReader::new(node_url, block_id, Some(cache_dir))),
        },
        GlobalContractCache::default(),
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state(state: &mut dyn State) -> (BlockifierState, CheatnetState) {
    let blockifier_state = BlockifierState::from(state);
    let cheatnet_state = CheatnetState::default();
    (blockifier_state, cheatnet_state)
}
