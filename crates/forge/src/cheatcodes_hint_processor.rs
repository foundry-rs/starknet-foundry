use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::scarb::StarknetContractArtifacts;
use anyhow::{anyhow, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::execution_utils::{felt_to_stark_felt, stark_felt_to_felt};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatnet::rpc::{call_contract, CallContractOutput};
use cheatnet::{
    cheatcodes::{CheatcodeError, ContractArtifacts, EnhancedHintError},
    CheatnetState,
};
use num_traits::{One, ToPrimitive};
use serde::Deserialize;
use starknet_api::core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;

use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{
    casm_run::{cell_ref_to_relocatable, extract_buffer, get_ptr},
    insert_value_to_cellref, CairoHintProcessor as OriginalCairoHintProcessor,
};

use cairo_lang_starknet::contract::starknet_keccak;
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cheatnet::cheatcodes::spy_events::SpyTarget;
use cheatnet::conversions::contract_address_from_felt;

mod file_operations;

// TODO(#41) Remove after we have a separate scarb package
impl From<&StarknetContractArtifacts> for ContractArtifacts {
    fn from(artifacts: &StarknetContractArtifacts) -> Self {
        ContractArtifacts {
            sierra: artifacts.sierra.clone(),
            casm: artifacts.casm.clone(),
        }
    }
}

pub struct CairoHintProcessor<'a> {
    pub original_cairo_hint_processor: OriginalCairoHintProcessor<'a>,
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
    pub cheatnet_state: CheatnetState,
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
            return execute_syscall(
                system,
                vm,
                &mut self.cheatnet_state,
                exec_scopes,
                hint_data,
                constants,
                &mut self.original_cairo_hint_processor,
            );
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
                self.cheatnet_state.start_roll(contract_address, value);
                Ok(())
            }
            "stop_roll" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.cheatnet_state.stop_roll(contract_address);
                Ok(())
            }
            "start_warp" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);
                let value = inputs[1].clone();
                self.cheatnet_state.start_warp(contract_address, value);
                Ok(())
            }
            "stop_warp" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.cheatnet_state.stop_warp(contract_address);
                Ok(())
            }
            "start_prank" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                let caller_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[1].clone().to_be_bytes(),
                )?)?);

                self.cheatnet_state
                    .start_prank(contract_address, caller_address);
                Ok(())
            }
            "stop_prank" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.cheatnet_state.stop_prank(contract_address);
                Ok(())
            }
            "start_mock_call" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);
                let function_name = inputs[1].clone();

                let ret_data_length = inputs[2]
                    .to_usize()
                    .expect("Missing ret_data len in inputs");

                let ret_data = inputs
                    .iter()
                    .skip(3)
                    .take(ret_data_length)
                    .cloned()
                    .collect::<Vec<_>>();

                self.cheatnet_state
                    .start_mock_call(contract_address, &function_name, &ret_data);
                Ok(())
            }
            "stop_mock_call" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);
                let function_name = inputs[1].clone();

                self.cheatnet_state
                    .stop_mock_call(contract_address, &function_name);
                Ok(())
            }
            "start_spoof" => {
                let contract_address = contract_address_from_felt(&inputs[0]);

                let version = inputs[1].is_one().then(|| inputs[2].clone());
                let account_contract_address = inputs[3].is_one().then(|| inputs[4].clone());
                let max_fee = inputs[5].is_one().then(|| inputs[6].clone());
                let transaction_hash = inputs[7].is_one().then(|| inputs[8].clone());
                let chain_id = inputs[9].is_one().then(|| inputs[10].clone());
                let nonce = inputs[11].is_one().then(|| inputs[12].clone());

                let signature_len = inputs[14]
                    .to_usize()
                    .expect("Failed to convert signature_len to usize");
                let signature = inputs[13]
                    .is_one()
                    .then(|| Vec::from(&inputs[15..(15 + signature_len)]));

                self.cheatnet_state.start_spoof(
                    contract_address,
                    version,
                    account_contract_address,
                    max_fee,
                    signature,
                    transaction_hash,
                    chain_id,
                    nonce,
                );
                Ok(())
            }
            "stop_spoof" => {
                let contract_address = contract_address_from_felt(&inputs[0]);

                self.cheatnet_state.stop_spoof(contract_address);
                Ok(())
            }
            "declare" => {
                let contract_name = inputs[0].clone();

                match self.cheatnet_state.declare(
                    &contract_name,
                    // TODO(#41) Remove after we have a separate scarb package
                    &contracts
                        .iter()
                        .map(|(k, v)| (k.clone(), ContractArtifacts::from(v)))
                        .collect(),
                ) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);

                        buffer
                            .write(Felt252::from(0))
                            .expect("Failed to insert error code");
                        buffer
                            .write(felt_class_hash)
                            .expect("Failed to insert declared contract class hash");
                        Ok(())
                    }
                    Err(CheatcodeError::Recoverable(_)) => {
                        panic!("Declare should not fail recoverably!")
                    }
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "deploy" => {
                let class_hash = inputs[0].clone();
                let class_hash = ClassHash(StarkFelt::new(class_hash.to_be_bytes()).unwrap());

                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

                match self.cheatnet_state.deploy(&class_hash, &calldata) {
                    Ok(contract_address) => {
                        let felt_contract_address: Felt252 =
                            stark_felt_to_felt(*contract_address.0.key());

                        buffer
                            .write(Felt252::from(0))
                            .expect("Failed to insert error code");
                        buffer
                            .write(felt_contract_address)
                            .expect("Failed to insert deployed contract address");
                        Ok(())
                    }
                    Err(CheatcodeError::Recoverable(panic_data)) => {
                        write_cheatcode_panic(&mut buffer, &panic_data);
                        Ok(())
                    }
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "print" => {
                print(inputs);
                Ok(())
            }
            "precalculate_address" => {
                let class_hash = inputs[0].clone();
                let class_hash = ClassHash(StarkFelt::new(class_hash.to_be_bytes()).unwrap());

                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

                let contract_address = self
                    .cheatnet_state
                    .precalculate_address(&class_hash, &calldata);

                let felt_contract_address: Felt252 = stark_felt_to_felt(*contract_address.0.key());
                buffer
                    .write(felt_contract_address)
                    .expect("Failed to insert a precalculated contract address");

                Ok(())
            }
            "get_class_hash" => {
                let contract_address = contract_address_from_felt(&inputs[0]);

                match self.cheatnet_state.get_class_hash(contract_address) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);

                        buffer
                            .write(felt_class_hash)
                            .expect("Failed to insert contract class hash");
                        Ok(())
                    }
                    Err(CheatcodeError::Recoverable(_)) => unreachable!(),
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "l1_handler_execute" => {
                let contract_address = contract_address_from_felt(&inputs[0]);
                let function_name = inputs[1].clone();
                let from_address = inputs[2].clone();
                let fee = inputs[3].clone();
                let payload_length: usize = inputs[4]
                    .clone()
                    .to_usize()
                    .expect("Payload length is expected to fit into usize type");

                let payload = Vec::from(&inputs[5..inputs.len()]);

                match self.cheatnet_state.l1_handler_execute(
                    contract_address,
                    &function_name,
                    &from_address,
                    &fee,
                    &payload,
                ) {
                    Ok(()) => Ok(()),
                    Err(CheatcodeError::Recoverable(panic_data)) => {
                        write_cheatcode_panic(&mut buffer, &panic_data);
                        Ok(())
                    }
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "parse_txt" => {
                let file_path = inputs[0].clone();
                let parsed_content = file_operations::parse_txt(&file_path)?;
                buffer
                    .write_data(parsed_content.iter())
                    .expect("Failed to insert file content to memory");
                Ok(())
            }
            "parse_json" => todo!(),
            "spy_events" => {
                let spy_on = match inputs.len() {
                    0 => unreachable!("Serialized enum should always be longer than 0"),
                    1 => SpyTarget::All,
                    2 => SpyTarget::One(contract_address_from_felt(&inputs[1])),
                    _ => {
                        let addresses_length = inputs[1].to_usize().unwrap();
                        let addresses = Vec::from(&inputs[2..(2 + addresses_length)])
                            .iter()
                            .map(contract_address_from_felt)
                            .collect();

                        SpyTarget::Multiple(addresses)
                    }
                };

                let id = self.cheatnet_state.spy_events(spy_on);
                buffer
                    .write(Felt252::from(id))
                    .expect("Failed to insert spy id");
                Ok(())
            }
            "fetch_events" => {
                let id = &inputs[0];
                let (emitted_events_len, serialized_events) = self.cheatnet_state.fetch_events(id);

                buffer
                    .write(Felt252::from(emitted_events_len))
                    .expect("Failed to insert serialized events length");
                for felt in serialized_events {
                    buffer
                        .write(felt)
                        .expect("Failed to insert serialized events");
                }
                Ok(())
            }
            "event_name_hash" => {
                let name = inputs[0].clone();
                let hash = starknet_keccak(as_cairo_short_string(&name).unwrap().as_bytes());

                buffer
                    .write(Felt252::from(hash))
                    .expect("Failed to insert event name hash");
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
    cheatnet_state: &mut CheatnetState,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &Box<dyn Any>,
    constants: &HashMap<String, Felt252>,
    original_cairo_hint_processor: &mut OriginalCairoHintProcessor,
) -> Result<(), HintError> {
    let (cell, offset) = extract_buffer(system);
    let system_ptr = get_ptr(vm, cell, &offset)?;

    // We peek into memory to check the selector
    let selector = DeprecatedSyscallSelector::try_from(felt_to_stark_felt(
        &vm.get_integer(system_ptr).unwrap(),
    ))?;

    match selector {
        DeprecatedSyscallSelector::CallContract => {
            execute_call_contract(MemBuffer::new(vm, system_ptr), cheatnet_state)?;
            Ok(())
        }
        DeprecatedSyscallSelector::Keccak => {
            original_cairo_hint_processor.execute_hint(vm, exec_scopes, hint_data, constants)
        }
        _ => Err(HintError::CustomHint(Box::from(
            "starknet syscalls (other than CallContract and Keccak) cannot be used in tests"
                .to_string(),
        ))),
    }
}

fn execute_call_contract(
    mut buffer: MemBuffer,
    cheatnet_state: &mut CheatnetState,
) -> Result<(), HintError> {
    let _selector = buffer.next_felt252().unwrap();
    let gas_counter = buffer.next_usize().unwrap();

    let contract_address = buffer.next_felt252().unwrap().into_owned();
    let contract_address = ContractAddress(
        PatriciaKey::try_from(
            StarkFelt::new(contract_address.to_be_bytes()).expect("Felt conversion failed"),
        )
        .expect("PatriciaKey failed"),
    );

    let entry_point_selector = buffer.next_felt252().unwrap().into_owned();

    let calldata = buffer.next_arr().unwrap();

    let call_result = call_contract(
        &contract_address,
        &entry_point_selector,
        &calldata,
        cheatnet_state,
    )
    .unwrap_or_else(|err| panic!("Transaction execution error: {err}"));

    let (result, exit_code) = match call_result {
        CallContractOutput::Success { ret_data } => (ret_data, 0),
        CallContractOutput::Panic { panic_data } => (panic_data, 1),
        CallContractOutput::Error { msg } => return Err(HintError::CustomHint(Box::from(msg))),
    };

    buffer.write(gas_counter).unwrap();
    buffer.write(Felt252::from(exit_code)).unwrap();

    buffer.write_arr(result.iter()).unwrap();
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

fn write_cheatcode_panic(buffer: &mut MemBuffer, panic_data: &[Felt252]) {
    buffer.write(1).expect("Failed to insert err code");
    buffer
        .write(panic_data.len())
        .expect("Failed to insert panic_data len");
    buffer
        .write_data(panic_data.iter())
        .expect("Failed to insert error in memory");
}
