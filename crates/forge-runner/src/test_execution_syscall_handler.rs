use std::any::Any;
use std::collections::HashMap;
use std::convert::Into;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::execution_utils::{
    felt_to_stark_felt, stark_felt_from_ptr, stark_felt_to_felt, ReadOnlySegment,
};
use blockifier::execution::syscalls::hint_processor::{read_felt_array, SyscallExecutionError};
use blockifier::execution::syscalls::{
    SyscallRequest, SyscallResponse, SyscallResponseWrapper, SyscallResult,
};
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatnet::cheatcodes::deploy::{deploy, deploy_at, DeployCallPayload};
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::execution::cheatable_syscall_handler::CheatableSyscallHandler;
use cheatnet::rpc::{call_contract, CallContractFailure, CallContractOutput, CallContractResult};
use cheatnet::state::{BlockifierState, CheatTarget, CheatnetState};
use conversions::StarknetConversions;
use num_traits::{One, ToPrimitive};
use scarb_artifacts::StarknetContractArtifacts;
use serde::Deserialize;

use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{
    extract_buffer, extract_relocatable, get_ptr, vm_get_range, MemBuffer,
};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{casm_run::cell_ref_to_relocatable, insert_value_to_cellref};
use starknet_api::core::ContractAddress;

use crate::test_execution_syscall_handler::file_operations::string_into_felt;
use cairo_lang_starknet::contract::starknet_keccak;
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cheatnet::cheatcodes::spy_events::SpyTarget;
use cheatnet::execution::cheated_syscalls::SingleSegmentResponse;
use cheatnet::execution::contract_execution_syscall_handler::{
    print, ContractExecutionSyscallHandler,
};
use starknet::signers::SigningKey;

mod file_operations;

pub struct TestExecutionState<'a> {
    pub environment_variables: &'a HashMap<String, String>,
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
}

// This hint processor provides an implementation logic for functions from snforge_std library.
// If cannot execute a hint it falls back to the CheatableSyscallHandler
pub struct TestExecutionSyscallHandler<'a> {
    pub child: ContractExecutionSyscallHandler<'a>,
    pub test_execution_state: &'a mut TestExecutionState<'a>,
    // we need to keep a copy of hints as SyscallHintProcessor keeps it as private
    pub hints: &'a HashMap<String, Hint>,
    pub run_resources: RunResources,
}

impl<'a> TestExecutionSyscallHandler<'a> {
    pub fn wrap(
        child: ContractExecutionSyscallHandler<'a>,
        test_execution_state: &'a mut TestExecutionState<'a>,
        hints: &'a HashMap<String, Hint>,
    ) -> Self {
        TestExecutionSyscallHandler {
            child,
            test_execution_state,
            hints,
            run_resources: RunResources::default(),
        }
    }
}

// crates/blockifier/src/execution/syscalls/hint_processor.rs:472 (ResourceTracker for SyscallHintProcessor)
impl ResourceTracker for TestExecutionSyscallHandler<'_> {
    fn consumed(&self) -> bool {
        self.child.child.child.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.child
            .child
            .child
            .context
            .vm_run_resources
            .consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.child
            .child
            .child
            .context
            .vm_run_resources
            .get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.child
            .child
            .child
            .context
            .vm_run_resources
            .run_resources()
    }
}

impl HintProcessorLogic for TestExecutionSyscallHandler<'_> {
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
                self.test_execution_state.contracts,
                self.test_execution_state.environment_variables,
            );
        }
        if let Some(Hint::Starknet(StarknetHint::SystemCall { system })) = maybe_extended_hint {
            return execute_syscall(
                system,
                vm,
                exec_scopes,
                hint_data,
                constants,
                &mut self.child.child,
            );
        }
        self.child
            .child
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
        Ok(Box::new(self.hints[hint_code].clone()))
    }
}

impl TestExecutionSyscallHandler<'_> {
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
        environment_variables: &HashMap<String, String>,
    ) -> Result<(), HintError> {
        // Parse the selector.
        let selector = &selector.value.to_bytes_be().1;
        let selector = std::str::from_utf8(selector).map_err(|_| {
            CustomHint(Box::from(
                "Failed to parse the  cheatcode selector".to_string(),
            ))
        })?;

        // Extract the inputs.
        let input_start = extract_relocatable(vm, input_start)?;
        let input_end = extract_relocatable(vm, input_end)?;
        let inputs = vm_get_range(vm, input_start, input_end)
            .map_err(|_| CustomHint(Box::from("Failed to read input data".to_string())))?;

        self.match_cheatcode_by_selector(
            vm,
            selector,
            inputs,
            output_start,
            output_end,
            contracts,
            environment_variables,
        )
        .map_err(Into::into)
    }

    #[allow(
        unused,
        clippy::too_many_lines,
        clippy::trivially_copy_pass_by_ref,
        clippy::too_many_arguments
    )]
    fn match_cheatcode_by_selector(
        &mut self,
        vm: &mut VirtualMachine,
        selector: &str,
        inputs: Vec<Felt252>,
        output_start: &CellRef,
        output_end: &CellRef,
        contracts: &HashMap<String, StarknetContractArtifacts>,
        environment_variables: &HashMap<String, String>,
    ) -> Result<(), EnhancedHintError> {
        let mut buffer = MemBuffer::new_segment(vm);
        let result_start = buffer.ptr;

        match selector {
            "start_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let block_number = inputs.last().unwrap().clone();

                self.child
                    .child
                    .cheatnet_state
                    .start_roll(target, block_number);
                Ok(())
            }
            "stop_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.child.child.cheatnet_state.stop_roll(target);
                Ok(())
            }
            "start_warp" => {
                // The last element in `inputs` should be the timestamp in all cases
                let warp_timestamp = inputs.last().unwrap().clone();

                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                self.child
                    .child
                    .cheatnet_state
                    .start_warp(target, warp_timestamp);

                Ok(())
            }
            "stop_warp" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.child.child.cheatnet_state.stop_warp(target);
                Ok(())
            }
            "start_elect" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let sequencer_address = inputs.last().unwrap().to_contract_address();

                self.child
                    .child
                    .cheatnet_state
                    .start_elect(target, sequencer_address);
                Ok(())
            }
            "stop_elect" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.child.child.cheatnet_state.stop_elect(target);
                Ok(())
            }
            "start_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                // The last element in `inputs` should be the contract address in all cases
                let caller_address = inputs.last().unwrap().to_contract_address();

                self.child
                    .child
                    .cheatnet_state
                    .start_prank(target, caller_address);
                Ok(())
            }
            "stop_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.child.child.cheatnet_state.stop_prank(target);
                Ok(())
            }
            "start_mock_call" => {
                let contract_address = inputs[0].to_contract_address();
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

                self.child.child.cheatnet_state.start_mock_call(
                    contract_address,
                    &function_name,
                    &ret_data,
                );
                Ok(())
            }
            "stop_mock_call" => {
                let contract_address = inputs[0].to_contract_address();
                let function_name = inputs[1].clone();

                self.child
                    .child
                    .cheatnet_state
                    .stop_mock_call(contract_address, &function_name);
                Ok(())
            }
            "start_spoof" => {
                let (target, inputs_start) = deserialize_cheat_target(&inputs);

                // We check for 1s - because of serialization from tx_info.cairo::option_as_tuple
                let version = inputs[inputs_start]
                    .is_one()
                    .then(|| inputs[inputs_start + 1].clone());
                let account_contract_address = inputs[inputs_start + 2]
                    .is_one()
                    .then(|| inputs[inputs_start + 3].clone());
                let max_fee = inputs[inputs_start + 4]
                    .is_one()
                    .then(|| inputs[inputs_start + 5].clone());
                let transaction_hash = inputs[inputs_start + 6]
                    .is_one()
                    .then(|| inputs[inputs_start + 7].clone());
                let chain_id = inputs[inputs_start + 8]
                    .is_one()
                    .then(|| inputs[inputs_start + 9].clone());
                let nonce = inputs[inputs_start + 10]
                    .is_one()
                    .then(|| inputs[inputs_start + 11].clone());

                let signature_len = inputs[inputs_start + 13]
                    .to_usize()
                    .expect("Failed to convert signature_len to usize");
                let signature = inputs[inputs_start + 12].is_one().then(|| {
                    Vec::from(&inputs[inputs_start + 14..(inputs_start + 14 + signature_len)])
                });

                self.child.child.cheatnet_state.start_spoof(
                    target,
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
                let (target, _) = deserialize_cheat_target(&inputs);

                self.child.child.cheatnet_state.stop_spoof(target);
                Ok(())
            }
            "declare" => {
                let contract_name = inputs[0].clone();
                let mut blockifier_state = BlockifierState::from(self.child.child.child.state);

                match blockifier_state.declare(&contract_name, contracts) {
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
                let class_hash = inputs[0].to_class_hash();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
                let mut blockifier_state = BlockifierState::from(self.child.child.child.state);

                handle_deploy_result(
                    deploy(
                        &mut blockifier_state,
                        self.child.child.cheatnet_state,
                        &class_hash,
                        &calldata,
                    ),
                    &mut buffer,
                )
            }
            "deploy_at" => {
                let class_hash = inputs[0].to_class_hash();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
                let contract_address = inputs[2 + calldata_length].to_contract_address();

                let mut blockifier_state = BlockifierState::from(self.child.child.child.state);

                handle_deploy_result(
                    deploy_at(
                        &mut blockifier_state,
                        self.child.child.cheatnet_state,
                        &class_hash,
                        &calldata,
                        contract_address,
                    ),
                    &mut buffer,
                )
            }
            "print" => {
                print(inputs);
                Ok(())
            }
            "precalculate_address" => {
                let class_hash = inputs[0].to_class_hash();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

                let contract_address = self
                    .child
                    .child
                    .cheatnet_state
                    .precalculate_address(&class_hash, &calldata);

                let felt_contract_address = contract_address.to_felt252();
                buffer
                    .write(felt_contract_address)
                    .expect("Failed to insert a precalculated contract address");

                Ok(())
            }
            "var" => {
                let name = inputs[0].clone();
                let name = as_cairo_short_string(&name).unwrap_or_else(|| {
                    panic!("Failed to parse var argument = {name} as short string")
                });

                let env_var = environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = string_into_felt(env_var)
                    .with_context(|| format!("Failed to parse value = {env_var} to felt"))?;

                buffer
                    .write(parsed_env_var)
                    .expect("Failed to insert parsed env var");
                Ok(())
            }
            "get_class_hash" => {
                let contract_address = inputs[0].to_contract_address();

                let mut blockifier_state = BlockifierState::from(self.child.child.child.state);

                match blockifier_state.get_class_hash(contract_address) {
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
                let contract_address = inputs[0].to_contract_address();
                let function_name = inputs[1].clone();
                let from_address = inputs[2].clone();
                let payload_length: usize = inputs[3]
                    .clone()
                    .to_usize()
                    .expect("Payload length is expected to fit into usize type");

                let payload = Vec::from(&inputs[4..inputs.len()]);

                let mut blockifier_state = BlockifierState::from(self.child.child.child.state);

                match blockifier_state
                    .l1_handler_execute(
                        self.child.child.cheatnet_state,
                        contract_address,
                        &function_name,
                        &from_address,
                        &payload,
                    )
                    .result
                {
                    CallContractResult::Success { .. } => {
                        buffer.write(0);
                        Ok(())
                    }
                    CallContractResult::Failure(CallContractFailure::Panic { panic_data }) => {
                        write_cheatcode_panic(&mut buffer, &panic_data);
                        Ok(())
                    }
                    CallContractResult::Failure(CallContractFailure::Error { msg }) => Err(
                        EnhancedHintError::from(HintError::CustomHint(Box::from(msg))),
                    ),
                }
            }
            "read_txt" => {
                let file_path = inputs[0].clone();
                let parsed_content = file_operations::read_txt(&file_path)?;
                buffer
                    .write_data(parsed_content.iter())
                    .expect("Failed to insert file content to memory");
                Ok(())
            }
            "read_json" => {
                let file_path = inputs[0].clone();
                let parsed_content = file_operations::read_json(&file_path)?;
                buffer
                    .write_data(parsed_content.iter())
                    .expect("Failed to insert file content to memory");
                Ok(())
            }
            "spy_events" => {
                let spy_on = match inputs.len() {
                    0 => unreachable!("Serialized enum should always be longer than 0"),
                    1 => SpyTarget::All,
                    2 => SpyTarget::One(inputs[1].to_contract_address()),
                    _ => {
                        let addresses_length = inputs[1].to_usize().unwrap();
                        let addresses = Vec::from(&inputs[2..(2 + addresses_length)])
                            .iter()
                            .map(Felt252::to_contract_address)
                            .collect();

                        SpyTarget::Multiple(addresses)
                    }
                };

                let id = self.child.child.cheatnet_state.spy_events(spy_on);
                buffer
                    .write(Felt252::from(id))
                    .expect("Failed to insert spy id");
                Ok(())
            }
            "fetch_events" => {
                let id = &inputs[0];
                let (emitted_events_len, serialized_events) =
                    self.child.child.cheatnet_state.fetch_events(id);

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
            "generate_ecdsa_keys" => {
                let key_pair = SigningKey::from_random();

                buffer
                    .write(key_pair.secret_scalar().to_felt252())
                    .expect("Failed to insert private key");
                buffer
                    .write(key_pair.verifying_key().scalar().to_felt252())
                    .expect("Failed to insert public key");
                Ok(())
            }
            "get_public_key" => {
                let private_key = inputs[0].clone();
                let key_pair = SigningKey::from_secret_scalar(private_key.to_field_element());

                buffer
                    .write(key_pair.verifying_key().scalar().to_felt252())
                    .expect("Failed to insert public key");

                Ok(())
            }
            "ecdsa_sign_message" => {
                let private_key = inputs[0].clone();
                let message_hash = inputs[1].clone();

                let key_pair = SigningKey::from_secret_scalar(private_key.to_field_element());

                if let Ok(signature) = key_pair.sign(&message_hash.to_field_element()) {
                    buffer.write(0).expect("Failed to insert exit code");
                    buffer
                        .write(signature.r.to_felt252())
                        .expect("Failed to insert signature r");
                    buffer
                        .write(signature.s.to_felt252())
                        .expect("Failed to insert signature s");
                } else {
                    buffer.write(1).expect("Failed to insert exit code");
                    buffer
                        .write("message_hash out of range".to_string().to_felt252())
                        .expect("Failed to insert error message");
                }

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

fn handle_deploy_result(
    deploy_result: Result<DeployCallPayload, CheatcodeError>,
    buffer: &mut MemBuffer,
) -> Result<(), EnhancedHintError> {
    match deploy_result {
        Ok(deploy_payload) => {
            let felt_contract_address: Felt252 = deploy_payload.contract_address.to_felt252();

            buffer
                .write(Felt252::from(0))
                .expect("Failed to insert error code");
            buffer
                .write(felt_contract_address)
                .expect("Failed to insert deployed contract address");
            Ok(())
        }
        Err(CheatcodeError::Recoverable(panic_data)) => {
            write_cheatcode_panic(buffer, &panic_data);
            Ok(())
        }
        Err(CheatcodeError::Unrecoverable(err)) => Err(err),
    }
}
// Returns the tuple (target, n read elements)
fn deserialize_cheat_target(inputs: &[Felt252]) -> (CheatTarget, usize) {
    // First element encodes the variant of CheatTarget
    match inputs[0].to_u8() {
        Some(0) => (CheatTarget::All, 1),
        Some(1) => (CheatTarget::One(inputs[1].to_contract_address()), 2),
        Some(2) => {
            let n_targets = inputs[1].to_usize().unwrap();
            let contract_addresses: Vec<_> = inputs[2..2 + n_targets]
                .iter()
                .map(Felt252::to_contract_address)
                .collect();
            (CheatTarget::Multiple(contract_addresses), 2 + n_targets)
        }
        _ => unreachable!("Invalid CheatTarget variant"),
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
    exec_scopes: &mut ExecutionScopes,
    hint_data: &Box<dyn Any>,
    constants: &HashMap<String, Felt252>,
    cheatable_syscall_handler: &mut CheatableSyscallHandler,
) -> Result<(), HintError> {
    let (cell, offset) = extract_buffer(system);
    let system_ptr = get_ptr(vm, cell, &offset)?;

    cheatable_syscall_handler
        .child
        .verify_syscall_ptr(system_ptr)?;

    // We peek into memory to check the selector
    let selector = DeprecatedSyscallSelector::try_from(felt_to_stark_felt(
        &vm.get_integer(cheatable_syscall_handler.child.syscall_ptr)
            .unwrap(),
    ))?;

    match selector {
        DeprecatedSyscallSelector::CallContract => {
            let call_args =
                CallContractArgs::read(vm, &mut cheatable_syscall_handler.child.syscall_ptr)?;

            let mut blockifier_state = BlockifierState::from(cheatable_syscall_handler.child.state);
            let call_result = execute_call_contract(
                &mut blockifier_state,
                cheatable_syscall_handler.cheatnet_state,
                &call_args,
            );
            write_call_contract_response(cheatable_syscall_handler, vm, &call_args, call_result)?;
            Ok(())
        }
        DeprecatedSyscallSelector::ReplaceClass => Err(HintError::CustomHint(Box::from(
            "Replace class can't be used in tests".to_string(),
        ))),
        _ => cheatable_syscall_handler.execute_hint(vm, exec_scopes, hint_data, constants),
    }
}

struct CallContractArgs {
    _selector: Felt252,
    gas_counter: u64,
    contract_address: ContractAddress,
    entry_point_selector: Felt252,
    calldata: Vec<Felt252>,
}

impl SyscallRequest for CallContractArgs {
    fn read(vm: &VirtualMachine, ptr: &mut Relocatable) -> SyscallResult<CallContractArgs> {
        let selector = stark_felt_from_ptr(vm, ptr)?.to_felt252();
        let gas_counter = stark_felt_from_ptr(vm, ptr)?.to_felt252().to_u64().unwrap();

        let contract_address = stark_felt_from_ptr(vm, ptr)?.to_contract_address();
        let entry_point_selector = stark_felt_from_ptr(vm, ptr)?.to_felt252();

        let calldata = read_felt_array::<SyscallExecutionError>(vm, ptr)?
            .iter()
            .map(StarknetConversions::to_felt252)
            .collect();

        Ok(CallContractArgs {
            _selector: selector,
            gas_counter,
            contract_address,
            entry_point_selector,
            calldata,
        })
    }
}

fn write_call_contract_response(
    cheatable_syscall_handler: &mut CheatableSyscallHandler,
    vm: &mut VirtualMachine,
    call_args: &CallContractArgs,
    call_output: CallContractOutput,
) -> Result<(), HintError> {
    let response_wrapper: SyscallResponseWrapper<SingleSegmentResponse> = match call_output.result {
        CallContractResult::Success { ret_data, .. } => {
            let memory_segment_start_ptr = cheatable_syscall_handler
                .child
                .read_only_segments
                .allocate(vm, &ret_data.iter().map(Into::into).collect())?;

            SyscallResponseWrapper::Success {
                gas_counter: call_args.gas_counter,
                response: SingleSegmentResponse {
                    segment: ReadOnlySegment {
                        start_ptr: memory_segment_start_ptr,
                        length: ret_data.len(),
                    },
                },
            }
        }
        CallContractResult::Failure(failure_type) => match failure_type {
            CallContractFailure::Panic { panic_data, .. } => SyscallResponseWrapper::Failure {
                gas_counter: call_args.gas_counter,
                error_data: panic_data
                    .iter()
                    .map(StarknetConversions::to_stark_felt)
                    .collect(),
            },
            CallContractFailure::Error { msg, .. } => return Err(CustomHint(Box::from(msg))),
        },
    };

    response_wrapper.write(vm, &mut cheatable_syscall_handler.child.syscall_ptr)?;

    Ok(())
}

fn execute_call_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    call_args: &CallContractArgs,
) -> CallContractOutput {
    call_contract(
        blockifier_state,
        cheatnet_state,
        &call_args.contract_address,
        &call_args.entry_point_selector,
        &call_args.calldata,
    )
    .unwrap_or_else(|err| panic!("Transaction execution error: {err}"))
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
