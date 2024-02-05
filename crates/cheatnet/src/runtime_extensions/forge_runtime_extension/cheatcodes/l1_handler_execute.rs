use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_l1_handler, CallResult,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use crate::state::BlockifierState;
use blockifier::abi::abi_utils::starknet_keccak;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

impl BlockifierState<'_> {
    pub fn l1_handler_execute(
        &mut self,
        runtime_state: &mut RuntimeState,
        contract_address: ContractAddress,
        function_name: &Felt252,
        from_address: &Felt252,
        payload: &[Felt252],
    ) -> CallResult {
        let selector = starknet_keccak(&function_name.to_bytes_be());

        let mut calldata = vec![from_address.clone()];
        calldata.extend_from_slice(payload);

        call_l1_handler(
            self,
            runtime_state,
            &contract_address,
            &selector,
            calldata.as_slice(),
        )
    }
}
