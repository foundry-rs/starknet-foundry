use camino::Utf8PathBuf;
use cheatnet::CheatnetState;

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state() -> CheatnetState {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    CheatnetState::new(&predeployed_contracts, false)
}
