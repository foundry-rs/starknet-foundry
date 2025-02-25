use crate::CheatnetState;
use crate::runtime_extensions::common::create_execute_calldata;
use conversions::IntoConv;
use runtime::starknet::constants::TEST_ADDRESS;
use starknet_api::core::{ClassHash, ContractAddress, calculate_contract_address};
use starknet_types_core::felt::Felt;

impl CheatnetState {
    #[must_use]
    pub fn precalculate_address(
        &self,
        class_hash: &ClassHash,
        calldata: &[Felt],
    ) -> ContractAddress {
        let salt = self.get_salt();

        let execute_calldata = create_execute_calldata(calldata);
        let deployer_address = Felt::from_hex(TEST_ADDRESS).unwrap();
        calculate_contract_address(
            salt,
            *class_hash,
            &execute_calldata,
            deployer_address.into_(),
        )
        .unwrap()
    }
}
