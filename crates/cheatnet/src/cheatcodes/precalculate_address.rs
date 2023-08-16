use crate::constants::TEST_ACCOUNT_CONTRACT_ADDRESS;

use crate::{cheatcodes::EnhancedHintError, CheatnetState};
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::MemBuffer;
use num_traits::cast::ToPrimitive;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::Calldata;

use starknet_api::patricia_key;

impl CheatnetState {
    pub fn precalculate_address(
        &self,
        buffer: &mut MemBuffer,
        inputs: &[Felt252],
    ) -> Result<(), EnhancedHintError> {
        let class_hash = inputs[0].clone();

        let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
        let class_hash = ClassHash(StarkFelt::new(class_hash.to_be_bytes()).unwrap());
        let salt = self.get_salt();
        let calldata_length = inputs[1].to_usize().unwrap();

        let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

        let execute_calldata = create_execute_calldata(&calldata);
        let contract_address =
            calculate_contract_address(salt, class_hash, &execute_calldata, account_address)
                .unwrap();

        let contract_address = Felt252::from_bytes_be((*contract_address.0.key()).bytes());

        buffer
            .write(contract_address)
            .expect("Failed to insert declared contract class hash");
        Ok(())
    }
}

fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    Calldata(calldata.into())
}
