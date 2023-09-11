use crate::constants::build_block_context;
use crate::CheatnetState;
use anyhow::{anyhow, Context, Result};
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
use cairo_lang_starknet::contract::starknet_keccak;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee, TransactionHash};

use super::{CheatcodeError, EnhancedHintError};
use crate::panic_data::try_extract_panic_data;
use crate::state::ExtendedStateReader;

impl CheatnetState {
    pub fn l1_handler_execute(
        &mut self,
        contract_address: ContractAddress,
        function_name: &Felt252,
        from_address: &Felt252,
        paid_fee_on_l1: &Felt252,
        payload: &[Felt252],
    ) -> Result<(), CheatcodeError> {
        let blockifier_state: &mut CachedState<ExtendedStateReader> = &mut self.blockifier_state;

        let block_context = build_block_context();

        let selector = Felt252::try_from(starknet_keccak(&function_name.to_bytes_be()))
            .context("Computing selector from short string failed")
            .map_err::<EnhancedHintError, _>(From::from)?;

        let entry_point_selector = EntryPointSelector(felt_to_stark_felt(&selector));

        let mut calldata: Vec<StarkFelt> = vec![felt_to_stark_felt(from_address)];
        calldata.extend(
            payload
                .iter()
                .map(felt_to_stark_felt)
                .collect::<Vec<StarkFelt>>(),
        );

        let calldata = Calldata(calldata.into());

        let fee = fee_from_felt252(paid_fee_on_l1)
            .context("Converting fee felt252 into Fee failed")
            .map_err::<EnhancedHintError, _>(From::from)?;

        let tx = L1HandlerTransaction {
            tx: starknet_api::transaction::L1HandlerTransaction {
                contract_address,
                entry_point_selector,
                calldata,
                ..Default::default()
            },
            tx_hash: TransactionHash::default(),
            paid_fee_on_l1: fee,
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
                let reason = if let TransactionExecutionError::ExecutionError(
                    EntryPointExecutionError::ExecutionFailed { error_data: felts },
                ) = err
                {
                    felts_as_str(&felts)
                } else {
                    format!("{err:?}")
                };

                // TODO: may we need a EnhancedHintError::Execution/Blockifier?
                // Or is it VirtualMachineError expected here?
                Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(
                    anyhow!(reason),
                )))
            }
        }
    }
}

/// Converts a felt252 into Fee.
fn fee_from_felt252(fee: &Felt252) -> Result<Fee> {
    // cairo-felt is not including leading 0 when using `to_bytes_be`.
    let mut fee_bytes = fee.to_bytes_be();
    if fee_bytes.len() > 16 {
        return Err(anyhow!("Felt252 value too large for u128 value"));
    }

    if fee_bytes.len() < 16 {
        let leading_zeros = vec![0; 16 - fee_bytes.len()];
        fee_bytes.splice(0..0, leading_zeros);
    }

    // Unwrap here because we ensured above that we always have a 16 bytes buffer.
    Ok(Fee(u128::from_be_bytes(
        fee_bytes[0..16].try_into().unwrap(),
    )))
}
