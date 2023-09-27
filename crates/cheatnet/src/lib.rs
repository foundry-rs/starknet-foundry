use crate::state::ExtendedStateReader;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::ContractAddressSalt;
use state::CheatcodeState;

pub mod cheatcodes;
pub mod constants;
pub mod execution;
pub mod forking;
pub mod panic_data;
pub mod rpc;
pub mod state;

pub struct CheatnetState {
    cheatcode_state: CheatcodeState,
    pub blockifier_state: CachedState<ExtendedStateReader>,
    pub deploy_salt_base: u32,
}

impl CheatnetState {
    #[must_use]
    pub fn new(state: ExtendedStateReader) -> Self {
        CheatnetState {
            cheatcode_state: CheatcodeState::new(),
            blockifier_state: CachedState::new(state, GlobalContractCache::default()),
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
