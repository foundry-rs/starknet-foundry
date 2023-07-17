use std::any::Any;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use crate::scarb::StarknetContractArtifacts;
use anyhow::{anyhow, Context, Result};
use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::contract_class::{
    ContractClass as BlockifierContractClass, ContractClassV1,
};
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::{DeclareTransaction, ExecutableTransaction};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatable_starknet::constants::{
    build_block_context, build_declare_transaction, build_invoke_transaction,
    build_transaction_context, TEST_ACCOUNT_CONTRACT_ADDRESS,
};
use cheatable_starknet::state::DictStateReader;
use num_traits::{Num, ToPrimitive};
use regex::Regex;
use serde::Deserialize;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, InvokeTransaction, InvokeTransactionV1,
};
use starknet_api::{patricia_key, stark_felt, StarknetApiError};
use thiserror::Error;

use crate::vm_memory::{
    felt_from_pointer, insert_at_pointer, relocatable_from_pointer, usize_from_pointer,
    write_cheatcode_panic,
};
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::extract_relocatable;
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
            return execute_cheatcode_hint(
                vm,
                exec_scopes,
                &mut self.blockifier_state,
                selector,
                input_start,
                input_end,
                output_start,
                output_end,
                self.contracts,
            );
        }
        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            return execute_syscall(system, vm, &mut self.blockifier_state);
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
) -> Result<(), HintError> {
    let (cell, offset) = extract_buffer(system);
    let mut system_ptr = get_ptr(vm, cell, &offset)?;

    let selector = felt_from_pointer(vm, &mut system_ptr)
        .unwrap()
        .to_bytes_be();

    let gas_counter = usize_from_pointer(vm, &mut system_ptr).unwrap();
    let contract_address = felt_from_pointer(vm, &mut system_ptr).unwrap();
    let entry_point_selector = felt_from_pointer(vm, &mut system_ptr).unwrap();

    let start = relocatable_from_pointer(vm, &mut system_ptr).unwrap();
    let end = relocatable_from_pointer(vm, &mut system_ptr).unwrap();
    let calldata = read_data_from_range(vm, start, end).unwrap();

    assert_eq!(std::str::from_utf8(&selector).unwrap(), "CallContract");
    let call_result = call_contract(
        &contract_address,
        &entry_point_selector,
        &calldata,
        blockifier_state,
    )
    .unwrap_or_else(|err| panic!("Transaction execution error: {err}"));

    let (result, exit_code) = match call_result {
        CallContractOutput::Success { ret_data } => (ret_data, 0),
        CallContractOutput::Panic { panic_data } => (panic_data, 1),
    };

    insert_at_pointer(vm, &mut system_ptr, gas_counter).unwrap();
    insert_at_pointer(vm, &mut system_ptr, Felt252::from(exit_code)).unwrap();

    let mut ptr = vm.add_memory_segment();
    let start = ptr;
    for value in result {
        insert_at_pointer(vm, &mut ptr, value).unwrap();
    }
    let end = ptr;

    insert_at_pointer(vm, &mut system_ptr, start).unwrap();
    insert_at_pointer(vm, &mut system_ptr, end).unwrap();

    Ok(())
}

enum CallContractOutput {
    Success { ret_data: Vec<Felt252> },
    Panic { panic_data: Vec<Felt252> },
}

// This can mutate state, the name of the syscall is not very good
fn call_contract(
    contract_address: &Felt252,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    blockifier_state: &mut CachedState<DictStateReader>,
) -> Result<CallContractOutput> {
    let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
        contract_address.to_be_bytes(),
    )?)?);
    let entry_point_selector =
        EntryPointSelector(StarkHash::new(entry_point_selector.to_be_bytes())?);
    let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
    let calldata = Calldata(Arc::new(
        calldata
            .iter()
            .map(|data| StarkFelt::new(data.to_be_bytes()))
            .collect::<Result<Vec<_>, _>>()?,
    ));
    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: contract_address,
        caller_address: account_address,
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut resources = ExecutionResources::default();
    let account_context = build_transaction_context();
    let block_context = build_block_context();

    let mut context = EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps,
    );

    let exec_result = entry_point.execute(blockifier_state, &mut resources, &mut context);
    if let Ok(call_info) = exec_result {
        let raw_return_data = &call_info.execution.retdata.0;

        let return_data = raw_return_data
            .iter()
            .map(|data| Felt252::from_bytes_be(data.bytes()))
            .collect();

        Ok(CallContractOutput::Success {
            ret_data: return_data,
        })
    } else if let Err(EntryPointExecutionError::ExecutionFailed { error_data }) = exec_result {
        let err_data = error_data
            .iter()
            .map(|data| Felt252::from_bytes_be(data.bytes()))
            .collect();

        Ok(CallContractOutput::Panic {
            panic_data: err_data,
        })
    } else {
        panic!("Unparseable result: {exec_result:?}");
    }
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

#[allow(clippy::trivially_copy_pass_by_ref, clippy::too_many_arguments)]
fn execute_cheatcode_hint(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    blockifier_state: &mut CachedState<DictStateReader>,
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
    let inputs = read_data_from_range(vm, input_start, input_end)
        .map_err(|_| HintError::CustomHint(Box::from("Failed to read input data".to_string())))?;

    match_cheatcode_by_selector(
        vm,
        blockifier_state,
        selector,
        inputs,
        output_start,
        output_end,
        contracts,
    )
    .map_err(Into::into)
}

#[allow(unused, clippy::too_many_lines, clippy::trivially_copy_pass_by_ref)]
fn match_cheatcode_by_selector(
    vm: &mut VirtualMachine,
    blockifier_state: &mut CachedState<DictStateReader>,
    selector: &str,
    inputs: Vec<Felt252>,
    output_start: &CellRef,
    output_end: &CellRef,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<(), EnhancedHintError> {
    let mut result_segment_ptr = vm.add_memory_segment();
    let result_start = result_segment_ptr;

    match selector {
        "prepare" => todo!(),
        "start_roll" => todo!(),
        "stop_roll" => todo!(),
        "start_warp" => todo!(),
        "stop_warp" => todo!(),
        "start_prank" => todo!(),
        "stop_prank" => todo!(),
        "mock_call" => todo!(),
        "declare" => declare(
            vm,
            blockifier_state,
            &inputs,
            &mut result_segment_ptr,
            contracts,
        ),
        "deploy" => deploy(vm, blockifier_state, &inputs, &mut result_segment_ptr),
        "print" => {
            print(inputs);
            Ok(())
        }
        _ => Err(anyhow!("Unknown cheatcode selector: {selector}")).map_err(Into::into),
    }?;

    let result_end = result_segment_ptr;
    insert_value_to_cellref!(vm, output_start, result_start)?;
    insert_value_to_cellref!(vm, output_end, result_end)?;

    Ok(())
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
    vm: &mut VirtualMachine,
    blockifier_state: &mut CachedState<DictStateReader>,
    inputs: &[Felt252],
    result_segment_ptr: &mut Relocatable,
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

    let class_hash = get_class_hash(casm_serialized.as_str());

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
        contract_class,
    )
    .unwrap_or_else(|err| panic!("Unable to build transaction {err:?}"));

    let account_tx = AccountTransaction::Declare(tx);
    let block_context = build_block_context();
    let _tx_result = account_tx
        .execute(blockifier_state, &block_context)
        .context("Failed to execute declare transaction")?;
    // result_segment.
    let felt_class_hash = felt252_from_hex_string(&class_hash.to_string()).unwrap();

    insert_at_pointer(vm, result_segment_ptr, Felt252::from(0))?;
    insert_at_pointer(vm, result_segment_ptr, felt_class_hash)?;

    Ok(())
}

fn get_class_hash(casm_contract: &str) -> ClassHash {
    let compiled_class = serde_json::from_str::<CompiledClass>(casm_contract).unwrap();
    let class_hash = compiled_class.class_hash().unwrap();
    let class_hash = StarkFelt::new(class_hash.to_bytes_be()).unwrap();
    ClassHash(class_hash)
}

fn deploy(
    vm: &mut VirtualMachine,
    blockifier_state: &mut CachedState<DictStateReader>,
    inputs: &[Felt252],
    result_segment_ptr: &mut Relocatable,
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
            vm,
            result_segment_ptr,
            vec![felt_from_short_string("No constructor in contract")],
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
    let account_tx =
        AccountTransaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 { nonce, ..tx }));

    let tx_info = account_tx
        .execute(blockifier_state, &block_context)
        .unwrap_or_else(|e| panic!("Unparseable transaction error: {e:?}"));

    if let Some(CallInfo { execution, .. }) = tx_info.execute_call_info {
        let contract_address = execution
            .retdata
            .0
            .get(0)
            .expect("Failed to get contract_address from return_data");
        let contract_address = Felt252::from_bytes_be(contract_address.bytes());

        insert_at_pointer(vm, result_segment_ptr, 0).expect("Failed to insert error code");
        insert_at_pointer(vm, result_segment_ptr, contract_address)
            .expect("Failed to insert deployed contract address");
    } else {
        let revert_error = tx_info
            .revert_error
            .expect("Unparseable tx info, {tx_info:?}");
        let extracted_panic_data = try_extract_panic_data(&revert_error)
            .expect("Unparseable error message, {revert_error}");

        write_cheatcode_panic(vm, result_segment_ptr, extracted_panic_data);
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

fn read_data_from_range(
    vm: &VirtualMachine,
    mut start: Relocatable,
    end: Relocatable,
) -> Result<Vec<Felt252>> {
    let mut calldata: Vec<Felt252> = vec![];
    while start != end {
        let value = felt_from_pointer(vm, &mut start)?;
        calldata.push(value);
    }
    Ok(calldata)
}

fn felt252_from_hex_string(value: &str) -> Result<Felt252> {
    let stripped_value = value.replace("0x", "");
    Felt252::from_str_radix(&stripped_value, 16)
        .map_err(|_| anyhow!("Failed to convert value = {value} to Felt252"))
}

#[cfg(test)]
mod test {
    use cairo_felt::Felt252;
    use std::path::Path;

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
        let casm_contract_path = Path::new("./tests/data/example.casm");
        let expected_class_hash =
            "0x3eb55a3f9f7485408838b08067c3b0f5d72523c525f568b04627464f5464749";

        let casm_contract_definition = std::fs::read_to_string(casm_contract_path).unwrap();
        let actual_class_hash = get_class_hash(casm_contract_definition.as_str());
        assert_eq!(
            actual_class_hash,
            ClassHash(stark_felt!(expected_class_hash))
        );
    }
}
