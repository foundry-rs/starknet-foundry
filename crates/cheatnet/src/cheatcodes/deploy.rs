use crate::constants::TEST_ACCOUNT_CONTRACT_ADDRESS;
use crate::{cheatcodes::EnhancedHintError, CheatnetState};
use anyhow::Result;
use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::execution_utils::{felt_to_stark_felt, stark_felt_to_felt};
use std::sync::Arc;

use blockifier::state::state_api::StateReader;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use starknet::core::utils::get_selector_from_name;

use starknet_api::core::{
    calculate_contract_address, ClassHash, ContractAddress, EntryPointSelector, PatriciaKey,
};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_api::{patricia_key, stark_felt};

use super::CheatcodeError;
use crate::conversions::{felt_from_short_string, field_element_to_felt252};
use crate::rpc::{call_contract, CallContractOutput};

impl CheatnetState {
    pub fn deploy_at(
        &mut self,
        class_hash: &ClassHash,
        calldata: &[Felt252],
        salt: ContractAddressSalt,
        contract_address: ContractAddress,
    ) -> Result<ContractAddress, CheatcodeError> {
        // Deploy a contract using syscall deploy.
        let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
        let entry_point_selector = selector_from_name("deploy_contract");

        if self
            .blockifier_state
            .state
            .address_to_class_hash
            .get(&contract_address)
            .is_some()
        {
            return Err(CheatcodeError::Unrecoverable(EnhancedHintError::from(
                CustomHint(Box::from("Address is already taken")),
            )));
        }

        let contract_class = self
            .blockifier_state
            .get_compiled_contract_class(class_hash)
            .map_err::<EnhancedHintError, _>(From::from)?;
        if contract_class.constructor_selector().is_none() && !calldata.is_empty() {
            return Err(CheatcodeError::Recoverable(vec![felt_from_short_string(
                "No constructor in contract",
            )]));
        }

        let execute_calldata = create_execute_calldata(
            calldata,
            class_hash,
            &account_address,
            &entry_point_selector,
            &salt,
        );

        let call_result = call_contract(
            &account_address,
            &field_element_to_felt252(&get_selector_from_name("__execute__").unwrap()),
            execute_calldata.as_slice(),
            self,
        )
        .unwrap_or_else(|err| panic!("Deploy txn failed: {err}"));

        match call_result {
            CallContractOutput::Success { .. } => {
                self.blockifier_state
                    .state
                    .address_to_class_hash
                    .insert(contract_address, *class_hash);

                Ok(contract_address)
            }
            CallContractOutput::Panic { panic_data } => {
                Err(CheatcodeError::Recoverable(panic_data))
            }
            CallContractOutput::Error { msg } => Err(CheatcodeError::Unrecoverable(
                EnhancedHintError::from(CustomHint(Box::from(msg))),
            )),
        }
    }

    pub fn deploy(
        &mut self,
        class_hash: &ClassHash,
        calldata: &[Felt252],
    ) -> Result<ContractAddress, CheatcodeError> {
        let salt = self.get_salt();
        let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));

        let contract_address = calculate_contract_address(
            salt,
            *class_hash,
            &Calldata(Arc::new(calldata.iter().map(felt_to_stark_felt).collect())),
            account_address,
        )
        .unwrap();

        self.increment_deploy_salt_base();

        self.deploy_at(class_hash, calldata, salt, contract_address)
    }
}

fn create_execute_calldata(
    calldata: &[Felt252],
    class_hash: &ClassHash,
    account_address: &ContractAddress,
    entry_point_selector: &EntryPointSelector,
    salt: &ContractAddressSalt,
) -> Vec<Felt252> {
    let calldata_len = u128::try_from(calldata.len()).unwrap();
    let mut execute_calldata = vec![
        *account_address.0.key(),      // Contract address.
        entry_point_selector.0,        // EP selector.
        stark_felt!(calldata_len + 3), // Calldata length.
        class_hash.0,                  // Calldata: class_hash.
        salt.0,                        // Contract_address_salt.
        stark_felt!(calldata_len),     // Constructor calldata length.
    ];
    let mut calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    execute_calldata.append(&mut calldata);
    return execute_calldata
        .iter()
        .map(|sf| stark_felt_to_felt(*sf))
        .collect();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn execute_calldata() {
        let calldata = create_execute_calldata(
            &[Felt252::from(100), Felt252::from(200)],
            &ClassHash(StarkFelt::from(123_u32)),
            &ContractAddress::try_from(StarkFelt::from(111_u32)).unwrap(),
            &EntryPointSelector(StarkFelt::from(222_u32)),
            &ContractAddressSalt(StarkFelt::from(333_u32)),
        );
        assert_eq!(
            calldata,
            vec![
                Felt252::from(111_u32),
                Felt252::from(222_u32),
                Felt252::from(5_u32),
                Felt252::from(123_u32),
                Felt252::from(333_u32),
                Felt252::from(2_u32),
                Felt252::from(100_u32),
                Felt252::from(200_u32),
            ]
        );
    }

    #[test]
    fn execute_calldata_no_entrypoint_calldata() {
        let calldata = create_execute_calldata(
            &[],
            &ClassHash(StarkFelt::from(123_u32)),
            &ContractAddress::try_from(StarkFelt::from(111_u32)).unwrap(),
            &EntryPointSelector(StarkFelt::from(222_u32)),
            &ContractAddressSalt(StarkFelt::from(333_u32)),
        );
        assert_eq!(
            calldata,
            vec![
                Felt252::from(111_u32),
                Felt252::from(222_u32),
                Felt252::from(3_u32),
                Felt252::from(123_u32),
                Felt252::from(333_u32),
                Felt252::from(0_u32),
            ]
        );
    }
}
