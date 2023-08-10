use crate::constants::TEST_ACCOUNT_CONTRACT_ADDRESS;

use crate::{cheatcodes::EnhancedHintError, CheatedState};
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::MemBuffer;
use num_traits::cast::ToPrimitive;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{Calldata, ContractAddressSalt};

use starknet_api::patricia_key;

impl CheatedState {
    pub fn precalculate_address(
        &self,
        buffer: &mut MemBuffer,
        inputs: &[Felt252],
    ) -> Result<(), EnhancedHintError> {
        let class_hash = inputs[0].clone();

        let account_address: ContractAddress =
            ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
        println!("{}", *account_address.0.key());
        let class_hash = ClassHash(StarkFelt::new(class_hash.to_be_bytes()).unwrap());
        let next_deploy_id = self.next_deploy_id;
        println!("Next deploy id will be {}", next_deploy_id);
        let salt = ContractAddressSalt::default();
        let calldata_length = inputs[1].to_usize().unwrap();

        let mut calldata = vec![];
        for felt in inputs.iter().skip(2).take(calldata_length) {
            calldata.push(felt.clone());
        }

        let execute_calldata = create_execute_calldata(&calldata);
        let contract_address =
            calculate_contract_address(salt, class_hash, &execute_calldata, account_address)
                .unwrap();

        let contract_address = Felt252::from_bytes_be((*contract_address.0.key()).bytes());

        buffer
            .write(Felt252::from(0))
            .expect("Failed to insert error code");

        buffer
            .write(contract_address)
            .expect("Failed to insert declared contract class hash");
        Ok(())
    }
}

fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let mut execute_calldata = vec![];
    let mut calldata: Vec<StarkFelt> = calldata
        .iter()
        .map(|data| StarkFelt::new(data.to_be_bytes()).unwrap())
        .collect();
    execute_calldata.append(&mut calldata);
    Calldata(execute_calldata.into())
}
