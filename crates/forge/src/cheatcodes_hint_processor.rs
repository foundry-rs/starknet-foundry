use std::any::Any;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crate::scarb::StarknetContractArtifacts;
use anyhow::{anyhow, Context, Result};
use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::contract_class::{
    ContractClass as BlockifierContractClass, ContractClassV1,
};
use blockifier::execution::entry_point::CallInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::{
    DeclareTransaction, ExecutableTransaction, InvokeTransaction,
};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatable_starknet::constants::{
    build_block_context, build_declare_transaction, build_invoke_transaction,
    TEST_ACCOUNT_CONTRACT_ADDRESS,
};
use cheatable_starknet::rpc::{call_contract, CallContractOutput, CheatedState};
use cheatable_starknet::state::DictStateReader;
use num_traits::{Num, ToPrimitive};
use regex::Regex;
use serde::Deserialize;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, InvokeTransactionV1, TransactionHash,
};
use starknet_api::{patricia_key, stark_felt, StarknetApiError};
use thiserror::Error;

use crate::vm_memory::write_cheatcode_panic;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{
    casm_run::{cell_ref_to_relocatable, extract_buffer, get_ptr},
    insert_value_to_cellref, CairoHintProcessor as OriginalCairoHintProcessor,
};
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use starknet::core::types::contract::CompiledClass;

pub struct CairoHintProcessor<'a> {
    pub original_cairo_hint_processor: OriginalCairoHintProcessor<'a>,
    pub blockifier_state: CachedState<DictStateReader>,
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
    pub cheated_state: CheatedState,
}

impl ResourceTracker for CairoHintProcessor<'_> {
    fn consumed(&self) -> bool {
        self.original_cairo_hint_processor.run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.original_cairo_hint_processor
            .run_resources
            .consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.original_cairo_hint_processor
            .run_resources
            .get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.original_cairo_hint_processor
            .run_resources
            .run_resources()
    }
}

trait ForgeHintProcessor {
    fn start_roll(
        &mut self,
        contract_address: ContractAddress,
        block_number: Felt252,
    ) -> Result<(), EnhancedHintError>;

    fn start_warp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: Felt252,
    ) -> Result<(), EnhancedHintError>;

    fn start_prank(
        &mut self,
        caller_address: ContractAddress,
        target_contract_address: ContractAddress,
    ) -> Result<(), EnhancedHintError>;

    fn stop_roll(&mut self, contract_address: ContractAddress) -> Result<(), EnhancedHintError>;

    fn stop_warp(&mut self, contract_address: ContractAddress) -> Result<(), EnhancedHintError>;

    fn stop_prank(&mut self, contract_address: ContractAddress) -> Result<(), EnhancedHintError>;
}

impl HintProcessorLogic for CairoHintProcessor<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();
        if let Some(Hint::Starknet(StarknetHint::Cheatcode {
            selector,
            input_start,
            input_end,
            output_start,
            output_end,
        })) = maybe_extended_hint
        {
            return self.execute_cheatcode_hint(
                vm,
                exec_scopes,
                selector,
                input_start,
                input_end,
                output_start,
                output_end,
                self.contracts,
            );
        }
        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            return execute_syscall(system, vm, &mut self.blockifier_state, &self.cheated_state);
        }
        self.original_cairo_hint_processor
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    /// Trait function to store hint in the hint processor by string.
    fn compile_hint(
        &self,
        hint_code: &str,
        _ap_tracking_data: &ApTracking,
        _reference_ids: &HashMap<String, usize>,
        _references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        Ok(Box::new(
            self.original_cairo_hint_processor.string_to_hint[hint_code].clone(),
        ))
    }
}

impl ForgeHintProcessor for CairoHintProcessor<'_> {
    fn start_roll(
        &mut self,
        contract_address: ContractAddress,
        block_number: Felt252,
    ) -> Result<(), EnhancedHintError> {
        self.cheated_state
            .rolled_contracts
            .insert(contract_address, block_number);
        Ok(())
    }
    fn start_warp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: Felt252,
    ) -> Result<(), EnhancedHintError> {
        self.cheated_state
            .warped_contracts
            .insert(contract_address, timestamp);

        Ok(())
    }

    fn start_prank(
        &mut self,
        contract_address: ContractAddress,
        caller_address: ContractAddress,
    ) -> Result<(), EnhancedHintError> {
        self.cheated_state
            .pranked_contracts
            .insert(contract_address, caller_address);
        Ok(())
    }

    fn stop_roll(&mut self, contract_address: ContractAddress) -> Result<(), EnhancedHintError> {
        self.cheated_state
            .rolled_contracts
            .remove(&contract_address);
        Ok(())
    }

    fn stop_warp(&mut self, contract_address: ContractAddress) -> Result<(), EnhancedHintError> {
        self.cheated_state
            .warped_contracts
            .remove(&contract_address);
        Ok(())
    }

    fn stop_prank(&mut self, contract_address: ContractAddress) -> Result<(), EnhancedHintError> {
        self.cheated_state
            .pranked_contracts
            .remove(&contract_address);
        Ok(())
    }
}

impl CairoHintProcessor<'_> {
    #[allow(clippy::trivially_copy_pass_by_ref, clippy::too_many_arguments)]
    pub fn execute_cheatcode_hint(
        &mut self,
        vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        selector: &BigIntAsHex,
        input_start: &ResOperand,
        input_end: &ResOperand,
        output_start: &CellRef,
        output_end: &CellRef,
        contracts: &HashMap<String, StarknetContractArtifacts>,
    ) -> Result<(), HintError> {
        // Parse the selector.
        let selector = &selector.value.to_bytes_be().1;
        let selector = std::str::from_utf8(selector).map_err(|_| {
            HintError::CustomHint(Box::from(
                "Failed to parse the  cheatcode selector".to_string(),
            ))
        })?;

        // Extract the inputs.
        let input_start = extract_relocatable(vm, input_start)?;
        let input_end = extract_relocatable(vm, input_end)?;
        let inputs = vm_get_range(vm, input_start, input_end).map_err(|_| {
            HintError::CustomHint(Box::from("Failed to read input data".to_string()))
        })?;

        self.match_cheatcode_by_selector(vm, selector, inputs, output_start, output_end, contracts)
            .map_err(Into::into)
    }

    #[allow(unused, clippy::too_many_lines, clippy::trivially_copy_pass_by_ref)]
    fn match_cheatcode_by_selector(
        &mut self,
        vm: &mut VirtualMachine,
        selector: &str,
        inputs: Vec<Felt252>,
        output_start: &CellRef,
        output_end: &CellRef,
        contracts: &HashMap<String, StarknetContractArtifacts>,
    ) -> Result<(), EnhancedHintError> {
        let mut buffer = MemBuffer::new_segment(vm);
        let result_start = buffer.ptr;

        match selector {
            "prepare" => todo!(),
            "start_roll" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);
                let value = inputs[1].clone();
                self.start_roll(contract_address, value)
            }
            "stop_roll" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.stop_roll(contract_address)
            }
            "start_warp" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);
                let value = inputs[1].clone();
                self.start_warp(contract_address, value)
            }
            "stop_warp" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.stop_warp(contract_address)
            }
            "start_prank" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                let caller_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[1].clone().to_be_bytes(),
                )?)?);

                self.start_prank(contract_address, caller_address)
            }
            "stop_prank" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.stop_prank(contract_address)
            }
            "mock_call" => todo!(),
            "declare" => declare(&mut buffer, &mut self.blockifier_state, &inputs, contracts),
            "deploy" => deploy(&mut buffer, &mut self.blockifier_state, &inputs),
            "print" => {
                print(inputs);
                Ok(())
            }
            _ => Err(anyhow!("Unknown cheatcode selector: {selector}")).map_err(Into::into),
        }?;

        let result_end = buffer.ptr;
        insert_value_to_cellref!(vm, output_start, result_start)?;
        insert_value_to_cellref!(vm, output_end, result_end)?;

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScarbStarknetArtifacts {
    version: u32,
    contracts: Vec<ScarbStarknetContract>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScarbStarknetContract {
    id: String,
    package_name: String,
    contract_name: String,
    artifacts: ScarbStarknetContractArtifact,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScarbStarknetContractArtifact {
    sierra: PathBuf,
    casm: Option<PathBuf>,
}

fn execute_syscall(
    system: &ResOperand,
    vm: &mut VirtualMachine,
    blockifier_state: &mut CachedState<DictStateReader>,
    cheated_state: &CheatedState,
) -> Result<(), HintError> {
    let (cell, offset) = extract_buffer(system);
    let system_ptr = get_ptr(vm, cell, &offset)?;

    let mut buffer = MemBuffer::new(vm, system_ptr);

    let selector = buffer.next_felt252().unwrap().to_bytes_be();
    let gas_counter = buffer.next_usize().unwrap();
    let contract_address = buffer.next_felt252().unwrap().into_owned();
    let entry_point_selector = buffer.next_felt252().unwrap().into_owned();

    let calldata = buffer.next_arr().unwrap();

    assert_eq!(std::str::from_utf8(&selector).unwrap(), "CallContract");

    let call_result = call_contract(
        &contract_address,
        &entry_point_selector,
        &calldata,
        blockifier_state,
        cheated_state,
    )
    .unwrap_or_else(|err| panic!("Transaction execution error: {err}"));

    let (result, exit_code) = match call_result {
        CallContractOutput::Success { ret_data } => (ret_data, 0),
        CallContractOutput::Panic { panic_data } => (panic_data, 1),
    };

    buffer.write(gas_counter).unwrap();
    buffer.write(Felt252::from(exit_code)).unwrap();

    buffer.write_arr(result.iter()).unwrap();

    Ok(())
}

// All errors that can be thrown from the hint executor have to be added here,
// to prevent the whole runner from panicking
#[derive(Error, Debug)]
enum EnhancedHintError {
    #[error(transparent)]
    Hint(#[from] HintError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    VirtualMachine(#[from] VirtualMachineError),
    #[error(transparent)]
    Memory(#[from] MemoryError),
    #[error(transparent)]
    State(#[from] StateError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StarknetApi(#[from] StarknetApiError),
}

impl From<EnhancedHintError> for HintError {
    fn from(error: EnhancedHintError) -> Self {
        match error {
            EnhancedHintError::Hint(error) => error,
            error => HintError::CustomHint(error.to_string().into_boxed_str()),
        }
    }
}

fn print(inputs: Vec<Felt252>) {
    for value in inputs {
        if let Some(short_string) = as_cairo_short_string(&value) {
            println!("original value: [{value}], converted to a string: [{short_string}]",);
        } else {
            println!("original value: [{value}]");
        }
    }
}

fn declare(
    buffer: &mut MemBuffer,
    blockifier_state: &mut CachedState<DictStateReader>,
    inputs: &[Felt252],
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<(), EnhancedHintError> {
    let contract_value = inputs[0].clone();

    let contract_value_as_short_str = as_cairo_short_string(&contract_value)
        .context("Converting contract name to short string failed")?;
    let contract_artifact = contracts.get(&contract_value_as_short_str).ok_or_else(|| {
        anyhow!("Failed to get contract artifact for name = {contract_value_as_short_str}. Make sure starknet target is correctly defined in Scarb.toml file.")
    })?;
    let sierra_contract_class: ContractClass = serde_json::from_str(&contract_artifact.sierra)
        .with_context(|| format!("File to parse json from artifact = {contract_artifact:?}"))?;

    let casm_contract_class = CasmContractClass::from_contract_class(sierra_contract_class, true)
        .context("Sierra to casm failed")?;
    let casm_serialized = serde_json::to_string_pretty(&casm_contract_class)
        .context("Failed to serialize contract to casm")?;

    let contract_class = ContractClassV1::try_from_json_string(&casm_serialized)
        .context("Failed to read contract class from json")?;
    let contract_class = BlockifierContractClass::V1(contract_class);

    let class_hash = get_class_hash(casm_serialized.as_str())?;

    let nonce = blockifier_state
        .get_nonce_at(ContractAddress(patricia_key!(
            TEST_ACCOUNT_CONTRACT_ADDRESS
        )))
        .context("Failed to get nonce")?;

    let declare_tx = build_declare_transaction(
        nonce,
        class_hash,
        ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS)),
    );
    let tx = DeclareTransaction::new(
        starknet_api::transaction::DeclareTransaction::V2(declare_tx),
        // TODO(#358)
        TransactionHash::default(),
        contract_class,
    )
    .unwrap_or_else(|err| panic!("Unable to build transaction {err:?}"));

    let account_tx = AccountTransaction::Declare(tx);
    let block_context = build_block_context();
    let tx_result = account_tx
        // FIXME not sure we should be using true or false for these
        .execute(blockifier_state, &block_context, true, true);

    match tx_result {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!(format!("Failed to execute declare transaction:\n    {e}")).into())
        }
    };

    // result_segment.
    let felt_class_hash = felt252_from_hex_string(&class_hash.to_string()).unwrap();

    buffer
        .write(Felt252::from(0))
        .expect("Failed to insert error code");
    buffer
        .write(felt_class_hash)
        .expect("Failed to insert declared contract class hash");

    Ok(())
}

fn get_class_hash(casm_contract: &str) -> Result<ClassHash> {
    let compiled_class = serde_json::from_str::<CompiledClass>(casm_contract)?;
    let class_hash = compiled_class.class_hash()?;
    let class_hash = StarkFelt::new(class_hash.to_bytes_be())?;
    Ok(ClassHash(class_hash))
}

fn deploy(
    buffer: &mut MemBuffer,
    blockifier_state: &mut CachedState<DictStateReader>,
    inputs: &[Felt252],
) -> Result<(), EnhancedHintError> {
    // TODO(#1991) deploy should fail if contract address provided doesn't match calculated
    //  or not accept this address as argument at all.
    let class_hash = inputs[0].clone();

    let calldata_length = inputs[1].to_usize().unwrap();
    let mut calldata = vec![];
    for felt in inputs.iter().skip(2).take(calldata_length) {
        calldata.push(felt.clone());
    }

    // Deploy a contract using syscall deploy.
    let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
    let block_context = build_block_context();
    let entry_point_selector = selector_from_name("deploy_contract");
    let salt = ContractAddressSalt::default();
    let class_hash = ClassHash(StarkFelt::new(class_hash.to_be_bytes()).unwrap());

    let contract_class = blockifier_state.get_compiled_contract_class(&class_hash)?;
    if contract_class.constructor_selector().is_none() && !calldata.is_empty() {
        write_cheatcode_panic(
            buffer,
            vec![felt_from_short_string("No constructor in contract")].as_slice(),
        );
        return Ok(());
    }

    let execute_calldata = create_execute_calldata(
        &calldata,
        &class_hash,
        &account_address,
        &entry_point_selector,
        &salt,
    );

    let nonce = blockifier_state
        .get_nonce_at(account_address)
        .context("Failed to get nonce")?;
    let tx = build_invoke_transaction(execute_calldata, account_address);
    let tx = InvokeTransactionV1 { nonce, ..tx };
    let account_tx = AccountTransaction::Invoke(InvokeTransaction {
        tx: starknet_api::transaction::InvokeTransaction::V1(tx),
        tx_hash: TransactionHash::default(), // TODO(#358): Check if this is legit
    });

    let tx_info = account_tx
        .execute(blockifier_state, &block_context, true, true)
        .unwrap_or_else(|e| panic!("Unparseable transaction error: {e:?}"));

    if let Some(CallInfo { execution, .. }) = tx_info.execute_call_info {
        let contract_address = execution
            .retdata
            .0
            .get(0)
            .expect("Failed to get contract_address from return_data");
        let contract_address = Felt252::from_bytes_be(contract_address.bytes());

        buffer
            .write(Felt252::from(0))
            .expect("Failed to insert error code");
        buffer
            .write(contract_address)
            .expect("Failed to insert deployed contract address");
    } else {
        let revert_error = tx_info
            .revert_error
            .expect("Unparseable tx info, {tx_info:?}");
        let extracted_panic_data = try_extract_panic_data(&revert_error)
            .expect("Unparseable error message, {revert_error}");

        write_cheatcode_panic(buffer, extracted_panic_data.as_slice());
    }
    Ok(())
}

fn felt_from_short_string(short_str: &str) -> Felt252 {
    return Felt252::from_bytes_be(short_str.as_bytes());
}

fn try_extract_panic_data(err: &str) -> Option<Vec<Felt252>> {
    let re = Regex::new(r#"(?m)^Got an exception while executing a hint: Custom Hint Error: Execution failed\. Failure reason: "(.*)"\.$"#)
        .expect("Could not create panic_data matching regex");

    if let Some(captures) = re.captures(err) {
        if let Some(panic_data_match) = captures.get(1) {
            if panic_data_match.as_str().is_empty() {
                return Some(vec![]);
            }
            let panic_data_felts: Vec<Felt252> = panic_data_match
                .as_str()
                .split(", ")
                .map(felt_from_short_string)
                .collect();

            return Some(panic_data_felts);
        }
    }
    None
}

// Should this function panic?
fn create_execute_calldata(
    calldata: &[Felt252],
    class_hash: &ClassHash,
    account_address: &ContractAddress,
    entry_point_selector: &EntryPointSelector,
    salt: &ContractAddressSalt,
) -> Calldata {
    let calldata_len = u128::try_from(calldata.len()).unwrap();
    let mut execute_calldata = vec![
        *account_address.0.key(),      // Contract address.
        entry_point_selector.0,        // EP selector.
        stark_felt!(calldata_len + 3), // Calldata length.
        class_hash.0,                  // Calldata: class_hash.
        salt.0,                        // Contract_address_salt.
        stark_felt!(calldata_len),     // Constructor calldata length.
    ];
    let mut calldata: Vec<StarkFelt> = calldata
        .iter()
        .map(|data| StarkFelt::new(data.to_be_bytes()).unwrap())
        .collect();
    execute_calldata.append(&mut calldata);
    Calldata(execute_calldata.into())
}

fn felt252_from_hex_string(value: &str) -> Result<Felt252> {
    let stripped_value = value.replace("0x", "");
    Felt252::from_str_radix(&stripped_value, 16)
        .map_err(|_| anyhow!("Failed to convert value = {value} to Felt252"))
}

#[cfg(test)]
mod test {
    use assert_fs::fixture::PathCopy;
    use std::sync::Arc;

    use cairo_felt::Felt252;
    use std::process::Command;

    use super::*;

    #[test]
    fn felt_2525_from_prefixed_hex() {
        assert_eq!(
            felt252_from_hex_string("0x1234").unwrap(),
            Felt252::from(0x1234)
        );
    }

    #[test]
    fn felt_2525_from_non_prefixed_hex() {
        assert_eq!(
            felt252_from_hex_string("1234").unwrap(),
            Felt252::from(0x1234)
        );
    }

    #[test]
    fn felt_252_err_on_failed_conversion() {
        let result = felt252_from_hex_string("yyyy");
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Failed to convert value = yyyy to Felt252");
    }

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
            Calldata(Arc::new(vec![
                StarkFelt::from(111_u32),
                StarkFelt::from(222_u32),
                StarkFelt::from(5_u32),
                StarkFelt::from(123_u32),
                StarkFelt::from(333_u32),
                StarkFelt::from(2_u32),
                StarkFelt::from(100_u32),
                StarkFelt::from(200_u32),
            ]))
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
            Calldata(Arc::new(vec![
                StarkFelt::from(111_u32),
                StarkFelt::from(222_u32),
                StarkFelt::from(3_u32),
                StarkFelt::from(123_u32),
                StarkFelt::from(333_u32),
                StarkFelt::from(0_u32),
            ]))
        );
    }

    #[test]
    fn string_extracting_panic_data() {
        let cases: [(&str, Option<Vec<Felt252>>); 4] = [
            (
                "Beginning of trace\nGot an exception while executing a hint: Custom Hint Error: Execution failed. Failure reason: \"PANIK, DAYTA\".\n
                 End of trace", 
                Some(vec![Felt252::from(344_693_033_291_u64), Felt252::from(293_154_149_441_u64)])
            ),
            (
                "Got an exception while executing a hint: Custom Hint Error: Execution failed. Failure reason: \"AYY, LMAO\".", 
                Some(vec![Felt252::from(4_282_713_u64), Felt252::from(1_280_131_407_u64)])
            ),
            (
                "Got an exception while executing a hint: Custom Hint Error: Execution failed. Failure reason: \"\".", 
                Some(vec![])
            ),
            ("Custom Hint Error: Invalid trace: \"PANIC, DATA\"", None)
        ];

        for (str, expected) in cases {
            assert_eq!(try_extract_panic_data(str), expected);
        }
    }

    #[test]
    fn parsing_felt_from_short_string() {
        let cases = [
            ("", Felt252::from(0)),
            ("{", Felt252::from(123)),
            ("PANIK", Felt252::from(344_693_033_291_u64)),
        ];

        for (str, felt_res) in cases {
            assert_eq!(felt_from_short_string(str), felt_res);
        }
    }

    #[test]
    fn class_hash_correct() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();

        Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();

        let temp_dir_path = temp.path();

        // expected_class_hash computed with
        // https://github.com/software-mansion/starknet.py/blob/cea191679cbdd2726ca7989f3a7662dee6ea43ca/starknet_py/tests/e2e/docs/guide/test_cairo1_contract.py#L29-L36
        let cases = [
            (
                // TODO verify calculation of this
                "0x00b0e07d0ab5d68a22072cd5f35f39335d0dcbf1a28fb92820bd5d547c497f33",
                "target/dev/simple_package_ERC20.casm.json",
            ),
            (
                // TODO verify calculation of this
                "0x02ff90d068517ba09883a50339d55cc8a678a60e1526032ee0a899ed219f44e7",
                "target/dev/simple_package_HelloStarknet.casm.json",
            ),
        ];

        for (expected_class_hash, casm_contract_path) in cases {
            let casm_contract_path = temp_dir_path.join(casm_contract_path);
            let casm_contract_path = casm_contract_path.as_path();

            let casm_contract_definition = std::fs::read_to_string(casm_contract_path)
                .unwrap_or_else(|_| panic!("Failed to read file: {casm_contract_path:?}"));
            let actual_class_hash = get_class_hash(casm_contract_definition.as_str()).unwrap();
            assert_eq!(
                actual_class_hash,
                ClassHash(stark_felt!(expected_class_hash))
            );
        }
    }
}
