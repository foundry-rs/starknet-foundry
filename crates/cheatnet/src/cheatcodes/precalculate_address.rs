use crate::constants::TEST_ACCOUNT_CONTRACT_ADDRESS;

use crate::CheatnetState;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::Calldata;

use starknet_api::patricia_key;

impl CheatnetState {
    #[must_use]
    pub fn precalculate_address(
        &self,
        class_hash: &ClassHash,
        calldata: &[Felt252],
    ) -> ContractAddress {
        let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
        let salt = self.get_salt();

        let execute_calldata = create_execute_calldata(calldata);
        calculate_contract_address(salt, *class_hash, &execute_calldata, account_address).unwrap()
    }
}

fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    Calldata(calldata.into())
}
