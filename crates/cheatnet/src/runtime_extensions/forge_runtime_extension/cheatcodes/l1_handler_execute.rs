use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_l1_handler, CallOutput,
};
use crate::state::{BlockifierState, CheatnetState};
use blockifier::abi::abi_utils::starknet_keccak;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

impl BlockifierState<'_> {
    pub fn l1_handler_execute(
        &mut self,
        cheatable_state: &mut CheatnetState,
        contract_address: ContractAddress,
        function_name: &Felt252,
        from_address: &Felt252,
        payload: &[Felt252],
    ) -> CallOutput {
        let selector = starknet_keccak(&function_name.to_bytes_be());

        let mut calldata = vec![from_address.clone()];
        calldata.extend_from_slice(payload);

        call_l1_handler(
            self,
            cheatable_state,
            &contract_address,
            &selector,
            calldata.as_slice(),
        )
        .expect("Calling l1 handler failed")
    }
}
