use crate::forking::state::ForkStateReader;
use crate::forking::worker::Worker;
use crate::state::CustomStateReader;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use camino::Utf8PathBuf;
use constants::build_testing_state;
use starknet::core::types::BlockId;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::ContractAddressSalt;
use state::CheatcodeState;

pub mod cheatcodes;
pub mod constants;
pub mod contract_print;
pub mod conversions;
pub mod forking;
pub mod panic_data;
pub mod rpc;
pub mod state;

pub struct CheatnetState {
    cheatcode_state: CheatcodeState,
    blockifier_state: CachedState<CustomStateReader>,
    pub deploy_salt_base: u32,
}

impl CheatnetState {
    #[must_use]
    pub fn new(predeployed_contracts: &Utf8PathBuf, rpc_config: Option<(&str, BlockId)>) -> Self {
        let dict_state_reader = build_testing_state(predeployed_contracts);
        CheatnetState {
            cheatcode_state: CheatcodeState::new(),
            blockifier_state: CachedState::new(
                if let Some((url, block_id)) = rpc_config {
                    CustomStateReader::ForkStateReader(ForkStateReader {
                        dict_state_reader,
                        worker: Worker::new(url, block_id),
                    })
                } else {
                    CustomStateReader::DictStateReader(dict_state_reader)
                },
                GlobalContractCache::default(),
            ),
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
