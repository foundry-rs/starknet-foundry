use blockifier::state::cached_state::CachedState;
use camino::Utf8PathBuf;
use constants::build_testing_state;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::ContractAddressSalt;
use state::{CheatcodeState, DictStateReader};

pub mod cheatcodes;
pub mod constants;
pub mod contract_print;
pub mod conversions;
pub mod panic_data;
pub mod rpc;
pub mod state;

pub struct CheatnetState {
    cheatcode_state: CheatcodeState,
    blockifier_state: CachedState<DictStateReader>,
    pub deploy_salt_base: u32,
}

impl CheatnetState {
    #[must_use]
    pub fn new(predeployed_contracts: &Utf8PathBuf) -> Self {
        CheatnetState {
            cheatcode_state: CheatcodeState::new(),
            blockifier_state: build_testing_state(predeployed_contracts),
            deploy_salt_base: 0,
        }
    }

    pub fn increment_deploy_salt_base(&mut self) {
        self.deploy_salt_base += 1;
    }

    #[must_use]
    pub fn get_salt(&self) -> ContractAddressSalt {
        ContractAddressSalt(StarkFelt::from(self.deploy_salt_base))
    }
}
