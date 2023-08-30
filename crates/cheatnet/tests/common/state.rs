use camino::Utf8PathBuf;
use cheatnet::CheatnetState;
use dotenv::dotenv;
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
    dotenv().ok();
    let node_url = std::env::var("CHEATNET_RPC_URL")
        .expect("CHEATNET_RPC_URL must be set in the .env file");

    CheatnetState::new(
        &predeployed_contracts,
        Some((&node_url, BlockId::Tag(Latest))),
    )
}
