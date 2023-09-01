use camino::Utf8PathBuf;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::forking::worker::Worker;
use cheatnet::state::CustomStateReader;
use cheatnet::CheatnetState;
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    CheatnetState::new(CustomStateReader(Box::new(build_testing_state(
        &predeployed_contracts,
    ))))
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_fork_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url =
        std::env::var("CHEATNET_RPC_URL").expect("CHEATNET_RPC_URL must be set in the .env file");

    CheatnetState::new(CustomStateReader(Box::new(ForkStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        worker: Worker::new(&node_url, BlockId::Tag(Latest)),
    })))
}
