use camino::Utf8PathBuf;
use cheatnet::CheatnetState;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

static PREDEPLOYED_CONTRACTS: Dir = include_dir!("crates/cheatnet/predeployed-contracts");

fn load_predeployed_contracts() -> TempDir {
    let tmp_dir = tempdir().expect("Failed to create a temporary directory");
    PREDEPLOYED_CONTRACTS
        .extract(&tmp_dir)
        .expect("Failed to copy predeployed contracts to temporary directory");
    tmp_dir
}

#[allow(clippy::module_name_repetitions)]
pub fn create_cheatnet_state() -> CheatnetState {
    let predeployed_contracts_dir = load_predeployed_contracts();
    let predeployed_contracts: PathBuf = predeployed_contracts_dir.path().into();
    let predeployed_contracts = Utf8PathBuf::try_from(predeployed_contracts)
        .expect("Failed to convert path to predeployed contracts to Utf8PathBuf");

    CheatnetState::new(&predeployed_contracts)
}
