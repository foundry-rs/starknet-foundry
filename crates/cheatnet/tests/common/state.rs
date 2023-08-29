use camino::Utf8PathBuf;
use cheatnet::CheatnetState;
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    CheatnetState::new(&predeployed_contracts, None)
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_fork_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    CheatnetState::new(
        &predeployed_contracts,
        Some(("http://188.34.188.184:9545/rpc/v0.4", BlockId::Tag(Latest))),
    )
}
