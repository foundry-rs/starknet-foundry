use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use camino::Utf8PathBuf;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::{ExtendedStateReader, CheatnetState, BlockifierState};
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;


#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state() -> (BlockifierState, CheatnetState) {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");

    let blockifier_state = BlockifierState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: None,
    });

    let cheatnet_state = CheatnetState::new();

    (blockifier_state, cheatnet_state)
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_fork_state() -> (BlockifierState, CheatnetState) {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4";

    let blockifier_state = BlockifierState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: Some(ForkStateReader::new(node_url, BlockId::Tag(Latest), None)),
    });
    let cheatnet_state = CheatnetState::new();

    (blockifier_state, cheatnet_state)
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_fork_state_at(block_id: BlockId, cache_dir: &str) -> (BlockifierState, CheatnetState) {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4";

    let blockifier_state =  BlockifierState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: Some(ForkStateReader::new(node_url, block_id, Some(cache_dir))),
    });
    let cheatnet_state = CheatnetState::new();

    (blockifier_state, cheatnet_state)
}
