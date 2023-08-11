use crate::constants::build_block_context;
use crate::state::DictStateReader;
use crate::CheatnetState;
use anyhow::Result;
use blockifier::execution::{
    errors::EntryPointExecutionError,
    execution_utils::{felt_to_stark_felt, felts_as_str},
};
use blockifier::state::cached_state::CachedState;
use blockifier::transaction::{
    errors::TransactionExecutionError,
    transactions::{ExecutableTransaction, L1HandlerTransaction},
};
use cairo_felt::Felt252;

use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee, TransactionHash};

use super::CheatcodeError;
use crate::panic_data::try_extract_panic_data;

impl CheatnetState {
    pub fn l1_handler_call(
        &mut self,
        contract_address: ContractAddress,
        selector: &Felt252,
        from_address: &Felt252,
        payload: &[Felt252],
    ) -> Result<(), CheatcodeError> {
        let blockifier_state: &mut CachedState<DictStateReader> = &mut self.blockifier_state;

        let block_context = build_block_context();

        let entry_point_selector = EntryPointSelector(felt_to_stark_felt(selector));

        let mut calldata: Vec<StarkFelt> = vec![felt_to_stark_felt(from_address)];
        calldata.extend(
            payload
                .iter()
                .map(felt_to_stark_felt)
                .collect::<Vec<StarkFelt>>(),
        );

        let calldata = Calldata(calldata.into());

        let tx = L1HandlerTransaction {
            tx: starknet_api::transaction::L1HandlerTransaction {
                contract_address,
                entry_point_selector,
                calldata,
                ..Default::default()
            },
            // TODO: Is tx_hash default always valid?
            tx_hash: TransactionHash::default(),
            // TODO: In this context, is u128::MAX a good choice? May it be configurable?
            paid_fee_on_l1: Fee(u128::MAX),
        };

        match tx.execute(blockifier_state, &block_context, true, true) {
            Ok(exec_info) => {
                if let Some(revert_error) = &exec_info.revert_error {
                    let extracted_panic_data = try_extract_panic_data(revert_error)
                        .expect("L1Transaction revert error: {revert_error}");

                    return Err(CheatcodeError::Recoverable(extracted_panic_data));
                }
                Ok(())
            }
            Err(err) => {
                if let TransactionExecutionError::ExecutionError(
                    EntryPointExecutionError::ExecutionFailed { error_data: felts },
                ) = err
                {
                    let reason: String = felts_as_str(&felts);
                    panic!("L1Transaction error: {reason:?}");
                } else {
                    panic!("L1Transaction error: {err:?}");
                }
            }
        }
    }
}
