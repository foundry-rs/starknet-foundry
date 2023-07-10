use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::contract_class::{
    ContractClass as BlockifierContractClass, ContractClassV1,
};
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
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatable_starknet::constants::{
    build_block_context, build_declare_transaction, build_invoke_transaction,
    TEST_ACCOUNT_CONTRACT_ADDRESS,
};
use cheatable_starknet::rpc::call_contract;
use cheatable_starknet::state::DictStateReader;
use num_traits::{Num, ToPrimitive, FromPrimitive};
use serde::Deserialize;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, InvokeTransaction, InvokeTransactionV1,
};
use cairo_felt_blockifier::Felt252 as blockifier_Felt252;
use starknet_api::{patricia_key, stark_felt, StarknetApiError};
use thiserror::Error;

use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{
    casm_run::{cell_ref_to_relocatable, extract_buffer, get_ptr},
    insert_value_to_cellref, CairoHintProcessor as OriginalCairoHintProcessor,
};
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};

pub struct CairoHintProcessor<'a> {
    pub original_cairo_hint_processor: OriginalCairoHintProcessor<'a>,
    pub blockifier_state: Option<CachedState<DictStateReader>>,
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
        let blockifier_state = self
            .blockifier_state
            .as_mut()
            .expect("Blockifier state is needed for executing hints");
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
                blockifier_state,
                selector,
                input_start,
                input_end,
                output_start,
                output_end,
            );
        }
        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            return execute_syscall(system, vm, blockifier_state);
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

fn convert_to_blockifier_felt(val: Felt252) -> blockifier_Felt252 {
    let v = val.to_i64().unwrap();
    blockifier_Felt252::from_i64(v).unwrap() // TODO incorrect conversion
}
fn convert_from_blockifier_felt(val: blockifier_Felt252) -> Felt252 {
    let v = val.to_i64().unwrap();
    Felt252::from_i64(v).unwrap() // TODO incorrect conversion
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

    let calldata_blockifier: Vec<blockifier_Felt252> = calldata.into_iter().map(|v| convert_to_blockifier_felt(v)).collect();
    assert_eq!(std::str::from_utf8(&selector).unwrap(), "CallContract");
    let result_blockfier: Vec<blockifier_Felt252> = call_contract(
        &convert_to_blockifier_felt(contract_address),
        &convert_to_blockifier_felt(entry_point_selector),
        &calldata_blockifier,
        blockifier_state,
    )
    .unwrap();
    let result: Vec<Felt252> = result_blockfier.into_iter().map(|v| convert_from_blockifier_felt(v)).collect();

    insert_at_pointer(vm, &mut system_ptr, gas_counter).unwrap();
    insert_at_pointer(vm, &mut system_ptr, Felt252::from(0)).unwrap();

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
        "declare_cairo0" => todo!(),
        "declare" => declare(vm, blockifier_state, &inputs, &mut result_segment_ptr),
        "deploy" => deploy(vm, blockifier_state, &inputs, &mut result_segment_ptr),
        "print" => print(inputs),
        _ => Err(anyhow!("Unknown cheatcode selector: {selector}")).map_err(Into::into),
    }?;

    let result_end = result_segment_ptr;
    insert_value_to_cellref!(vm, output_start, result_start)?;
    insert_value_to_cellref!(vm, output_end, result_end)?;

    Ok(())
}

fn print(inputs: Vec<Felt252>) -> Result<(), EnhancedHintError> {
    for value in inputs {
        if let Some(short_string) = as_cairo_short_string(&value) {
            println!("original value: [{value}], converted to a string: [{short_string}]",);
        } else {
            println!("original value: [{value}]");
        }
    }
    Ok(())
}

fn declare(
    vm: &mut VirtualMachine,
    blockifier_state: &mut CachedState<DictStateReader>,
    inputs: &[Felt252],
    result_segment_ptr: &mut Relocatable,
) -> Result<(), EnhancedHintError> {
    let contract_value = inputs[0].clone();

    let contract_value_as_short_str = as_cairo_short_string(&contract_value)
        .context("Converting contract name to short string failed")?;
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?
        .join("target/dev");

    let mut paths = fs::read_dir(&current_dir)
        .context("Failed to read ./target/dev, scarb build probably failed")?;

    let starknet_artifacts_entry = &paths
        .find_map(|path| match path {
            Ok(path) => {
                let name = path.file_name().into_string().ok()?;
                name.contains("starknet_artifacts").then_some(path)
            }
            Err(_) => None,
        })
        .context("Failed to find starknet_artifacts.json file")?;
    let starknet_artifacts =
        fs::read_to_string(starknet_artifacts_entry.path()).context(format!(
            "Failed to read {:?} contents",
            starknet_artifacts_entry.file_name()
        ))?;
    let starknet_artifacts: ScarbStarknetArtifacts =
        serde_json::from_str(starknet_artifacts.as_str()).context(format!(
            "Failed to parse {:?} contents",
            starknet_artifacts_entry.file_name()
        ))?;

    let sierra_path = starknet_artifacts
        .contracts
        .iter()
        .find_map(|contract| {
            if contract.contract_name == contract_value_as_short_str {
                return Some(contract.artifacts.sierra.clone());
            }
            None
        })
        .context(format!(
            "Failed to find contract {contract_value_as_short_str} in starknet_artifacts.json"
        ))?;
    let sierra_path = current_dir.join(sierra_path);

    let file = fs::File::open(&sierra_path)
        .context(format!("Failed to open file at path = {:?}", &sierra_path))?;
    let sierra_contract_class: ContractClass =
        serde_json::from_reader(&file).context("File to parse json from file = {file:?}")?;

    let casm_contract_class = CasmContractClass::from_contract_class(sierra_contract_class, true)
        .context("Sierra to casm failed")?;
    let casm_serialized = serde_json::to_string_pretty(&casm_contract_class)
        .context("Failed to serialize contract to casm")?;

    let contract_class = ContractClassV1::try_from_json_string(&casm_serialized)
        .context("Failed to read contract class from json")?;
    let contract_class = BlockifierContractClass::V1(contract_class);

    // TODO(#2134) replace this. Hash should be calculated in the correct manner. This is just a workaround.
    let mut hasher = DefaultHasher::new();
    casm_serialized.hash(&mut hasher);
    let class_hash = hasher.finish();
    let class_hash = ClassHash(stark_felt!(class_hash));

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
    .unwrap_or_else(|err| panic!("Unable to build transaction {:?}", err));

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

fn deploy(
    vm: &mut VirtualMachine,
    blockifier_state: &mut CachedState<DictStateReader>,
    inputs: &[Felt252],
    result_segment_ptr: &mut Relocatable,
) -> Result<(), EnhancedHintError> {
    let _contract_address = inputs[0].clone();
    // TODO(#1991) deploy should fail if contract address provided doesn't match calculated
    //  or not accept this address as argument at all.
    let class_hash = inputs[1].clone();

    let calldata_length = inputs[2].to_usize().unwrap();
    let mut calldata = vec![];
    for felt in inputs.iter().skip(3).take(calldata_length) {
        calldata.push(felt.clone());
    }

    // Deploy a contract using syscall deploy.
    let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
    let block_context = build_block_context();
    let entry_point_selector = selector_from_name("deploy_contract");
    let salt = ContractAddressSalt::default();
    let class_hash = ClassHash(StarkFelt::new(class_hash.to_be_bytes()).unwrap());

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
    let tx_result = account_tx
        .execute(blockifier_state, &block_context)
        .context("Failed to execute deploy transaction")?;
    let return_data = tx_result
        .execute_call_info
        .context("Failed to get execution data from method")?
        .execution
        .retdata;
    let contract_address = return_data
        .0
        .get(0)
        .context("Failed to get contract_address from return_data")?;
    let contract_address = Felt252::from_bytes_be(contract_address.bytes());

    // TODO(#2152): in case of error, consider filling the panic data instead of packing in rust
    insert_at_pointer(vm, result_segment_ptr, Felt252::from(0)).unwrap();
    insert_at_pointer(vm, result_segment_ptr, contract_address).unwrap();

    Ok(())
}

// TODO(#2164): remove this when extract_relocatable is pub in cairo
fn extract_relocatable(
    vm: &VirtualMachine,
    buffer: &ResOperand,
) -> Result<Relocatable, VirtualMachineError> {
    let (base, offset) = extract_buffer(buffer);
    get_ptr(vm, base, &offset)
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
    vm: &mut VirtualMachine,
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

fn insert_at_pointer<T: Into<MaybeRelocatable>>(
    vm: &mut VirtualMachine,
    ptr: &mut Relocatable,
    value: T,
) -> Result<()> {
    vm.insert_value(*ptr, value)?;
    *ptr += 1;
    Ok(())
}

fn usize_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<usize> {
    let gas_counter = vm
        .get_integer(*ptr)?
        .to_usize()
        .ok_or_else(|| anyhow!("Failed to convert to usize"))?;
    *ptr += 1;
    Ok(gas_counter)
}

fn relocatable_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<Relocatable> {
    let start = vm.get_relocatable(*ptr)?;
    *ptr += 1;
    Ok(start)
}

fn felt_from_pointer(vm: &mut VirtualMachine, ptr: &mut Relocatable) -> Result<Felt252> {
    let entry_point_selector = vm.get_integer(*ptr)?.into_owned();
    *ptr += 1;
    Ok(entry_point_selector)
}

fn felt252_from_hex_string(value: &str) -> Result<Felt252> {
    let stripped_value = value.replace("0x", "");
    Felt252::from_str_radix(&stripped_value, 16)
        .map_err(|_| anyhow!("Failed to convert value = {value} to Felt252"))
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use cairo_felt::Felt252;

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
}
