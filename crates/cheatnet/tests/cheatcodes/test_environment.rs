use crate::common::{call_contract, deploy_wrapper};
use crate::common::{deploy_contract, felt_selector_from_name, state::create_cached_state};
use blockifier::state::cached_state::CachedState;
use cairo_vm::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CheatnetState, ExtendedStateReader};
use starknet_api::core::ClassHash;
use starknet_api::core::ContractAddress;

pub struct TestEnvironment {
    pub cached_state: CachedState<ExtendedStateReader>,
    pub cheatnet_state: CheatnetState,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let cached_state = create_cached_state();

        Self {
            cached_state,
            cheatnet_state: CheatnetState::default(),
        }
    }

    pub fn declare(&mut self, contract_name: &str, contracts_data: &ContractsData) -> ClassHash {
        declare(&mut self.cached_state, contract_name, contracts_data).unwrap()
    }

    pub fn deploy(&mut self, contract_name: &str, calldata: &[Felt252]) -> ContractAddress {
        deploy_contract(
            &mut self.cached_state,
            &mut self.cheatnet_state,
            contract_name,
            calldata,
        )
    }

    pub fn deploy_wrapper(
        &mut self,
        class_hash: &ClassHash,
        calldata: &[Felt252],
    ) -> ContractAddress {
        deploy_wrapper(
            &mut self.cached_state,
            &mut self.cheatnet_state,
            class_hash,
            calldata,
        )
        .unwrap()
    }

    pub fn call_contract(
        &mut self,
        contract_address: &ContractAddress,
        selector: &str,
        calldata: &[Felt252],
    ) -> CallResult {
        call_contract(
            &mut self.cached_state,
            &mut self.cheatnet_state,
            contract_address,
            felt_selector_from_name(selector),
            calldata,
        )
    }

    pub fn precalculate_address(
        &mut self,
        class_hash: &ClassHash,
        calldata: &[u128],
    ) -> ContractAddress {
        let calldata = calldata
            .iter()
            .map(|x| Felt252::from(*x))
            .collect::<Vec<_>>();
        self.cheatnet_state
            .precalculate_address(class_hash, &calldata)
    }
}
