use camino::Utf8PathBuf;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::ExtendedStateReader;
use cheatnet::CheatnetState;
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");

    CheatnetState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: None,
    })
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_fork_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4";

    CheatnetState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: Some(ForkStateReader::new(node_url, BlockId::Tag(Latest), None)),
    })
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_fork_state_at(block_id: BlockId, cache_dir: &str) -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4";

    CheatnetState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: Some(ForkStateReader::new(node_url, block_id, Some(cache_dir))),
    })
}
