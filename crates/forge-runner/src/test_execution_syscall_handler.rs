use std::any::Any;
use std::collections::HashMap;
use std::convert::Into;
use std::path::PathBuf;

use anyhow::{Context, Result};
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
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
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

use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{
    extract_buffer, get_ptr, MemBuffer,
};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{casm_run::cell_ref_to_relocatable, insert_value_to_cellref};
use starknet_api::core::ContractAddress;

use crate::runtime::{RuntimeExtension, ExtensionLogic, CheatcodeHadlingResult};
use crate::test_execution_syscall_handler::file_operations::string_into_felt;
use cairo_lang_starknet::contract::starknet_keccak;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use cheatnet::cheatcodes::spy_events::SpyTarget;
use cheatnet::execution::cheated_syscalls::SingleSegmentResponse;
use cheatnet::execution::contract_execution_syscall_handler::{
    print, ContractExecutionSyscallHandler,
};
use starknet::signers::SigningKey;

mod file_operations;


// This runtime extenxion provides an implementation logic for functions from snforge_std library.
impl<'a> ExtensionLogic for RuntimeExtension<TestExecutionState, ContractExecutionSyscallHandler<'a>> {
    type Runtime = ContractExecutionSyscallHandler<'a>;

    fn get_extended_runtime_mut(&mut self,) -> &mut ContractExecutionSyscallHandler<'a> {
        &mut self.extended_runtime
    }

    fn get_extended_runtime(&self,) -> &ContractExecutionSyscallHandler<'a> {
        &self.extended_runtime
    }

    fn handle_cheatcode(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
        vm: &mut VirtualMachine,
        output_start: &CellRef,
        output_end: &CellRef,) -> Result<CheatcodeHadlingResult, EnhancedHintError> {
        let mut buffer = MemBuffer::new_segment(vm);
        let result_start = buffer.ptr;

        let res = match selector {
            "start_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let block_number = inputs.last().unwrap().clone();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_roll(target, block_number);
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "stop_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime.child.cheatnet_state.stop_roll(target);
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "start_warp" => {
                // The last element in `inputs` should be the timestamp in all cases
                let warp_timestamp = inputs.last().unwrap().clone();

                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_warp(target, warp_timestamp);

                Ok(CheatcodeHadlingResult::Result(()))
            }
            "stop_warp" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime.child.cheatnet_state.stop_warp(target);
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "start_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                // The last element in `inputs` should be the contract address in all cases
                let caller_address = inputs.last().unwrap().to_contract_address();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_prank(target, caller_address);
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "stop_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime.child.cheatnet_state.stop_prank(target);
                Ok(CheatcodeHadlingResult::Result(()))
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

                self.extended_runtime.child.cheatnet_state.start_mock_call(
                    contract_address,
                    &function_name,
                    &ret_data,
                );
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "stop_mock_call" => {
                let contract_address = inputs[0].to_contract_address();
                let function_name = inputs[1].clone();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .stop_mock_call(contract_address, &function_name);
                Ok(CheatcodeHadlingResult::Result(()))
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

                self.extended_runtime.child.cheatnet_state.start_spoof(
                    target,
                    version,
                    account_contract_address,
                    max_fee,
                    signature,
                    transaction_hash,
                    chain_id,
                    nonce,
                );
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "stop_spoof" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime.child.cheatnet_state.stop_spoof(target);
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "declare" => {
                let contract_name = inputs[0].clone();
                let mut blockifier_state = BlockifierState::from(self.extended_runtime.child.child.state);

                match blockifier_state.declare(&contract_name, &self.extension_state.contracts) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);

                        buffer
                            .write(Felt252::from(0))
                            .expect("Failed to insert error code");
                        buffer
                            .write(felt_class_hash)
                            .expect("Failed to insert declared contract class hash");
                        Ok(CheatcodeHadlingResult::Result(()))
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
                let mut blockifier_state = BlockifierState::from(self.extended_runtime.child.child.state);

                handle_deploy_result(
                    deploy(
                        &mut blockifier_state,
                        self.extended_runtime.child.cheatnet_state,
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

                let mut blockifier_state = BlockifierState::from(self.extended_runtime.child.child.state);

                handle_deploy_result(
                    deploy_at(
                        &mut blockifier_state,
                        self.extended_runtime.child.cheatnet_state,
                        &class_hash,
                        &calldata,
                        contract_address,
                    ),
                    &mut buffer,
                )
            }
            "print" => {
                print(inputs);
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "precalculate_address" => {
                let class_hash = inputs[0].to_class_hash();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

                let contract_address = self
                    .extended_runtime
                    .child
                    .cheatnet_state
                    .precalculate_address(&class_hash, &calldata);

                let felt_contract_address = contract_address.to_felt252();
                buffer
                    .write(felt_contract_address)
                    .expect("Failed to insert a precalculated contract address");

                Ok(CheatcodeHadlingResult::Result(()))
            }
            "var" => {
                let name = inputs[0].clone();
                let name = as_cairo_short_string(&name).unwrap_or_else(|| {
                    panic!("Failed to parse var argument = {name} as short string")
                });

                let env_var = self.extension_state.environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = string_into_felt(env_var)
                    .with_context(|| format!("Failed to parse value = {env_var} to felt"))?;

                buffer
                    .write(parsed_env_var)
                    .expect("Failed to insert parsed env var");
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "get_class_hash" => {
                let contract_address = inputs[0].to_contract_address();

                let mut blockifier_state = BlockifierState::from(self.extended_runtime.child.child.state);

                match blockifier_state.get_class_hash(contract_address) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);

                        buffer
                            .write(felt_class_hash)
                            .expect("Failed to insert contract class hash");
                        Ok(CheatcodeHadlingResult::Result(()))
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

                let mut blockifier_state = BlockifierState::from(self.extended_runtime.child.child.state);

                match blockifier_state
                    .l1_handler_execute(
                        self.extended_runtime.child.cheatnet_state,
                        contract_address,
                        &function_name,
                        &from_address,
                        &payload,
                    )
                    .result
                {
                    CallContractResult::Success { .. } => {
                        buffer.write(0);
                        Ok(CheatcodeHadlingResult::Result(()))
                    }
                    CallContractResult::Failure(CallContractFailure::Panic { panic_data }) => {
                        write_cheatcode_panic(&mut buffer, &panic_data);
                        Ok(CheatcodeHadlingResult::Result(()))
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
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "read_json" => {
                let file_path = inputs[0].clone();
                let parsed_content = file_operations::read_json(&file_path)?;
                buffer
                    .write_data(parsed_content.iter())
                    .expect("Failed to insert file content to memory");
                Ok(CheatcodeHadlingResult::Result(()))
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

                let id = self.extended_runtime.child.cheatnet_state.spy_events(spy_on);
                buffer
                    .write(Felt252::from(id))
                    .expect("Failed to insert spy id");
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "fetch_events" => {
                let id = &inputs[0];
                let (emitted_events_len, serialized_events) =
                    self.extended_runtime.child.cheatnet_state.fetch_events(id);

                buffer
                    .write(Felt252::from(emitted_events_len))
                    .expect("Failed to insert serialized events length");
                for felt in serialized_events {
                    buffer
                        .write(felt)
                        .expect("Failed to insert serialized events");
                }
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "event_name_hash" => {
                let name = inputs[0].clone();
                let hash = starknet_keccak(as_cairo_short_string(&name).unwrap().as_bytes());

                buffer
                    .write(Felt252::from(hash))
                    .expect("Failed to insert event name hash");
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "generate_ecdsa_keys" => {
                let key_pair = SigningKey::from_random();

                buffer
                    .write(key_pair.secret_scalar().to_felt252())
                    .expect("Failed to insert private key");
                buffer
                    .write(key_pair.verifying_key().scalar().to_felt252())
                    .expect("Failed to insert public key");
                Ok(CheatcodeHadlingResult::Result(()))
            }
            "get_public_key" => {
                let private_key = inputs[0].clone();
                let key_pair = SigningKey::from_secret_scalar(private_key.to_field_element());

                buffer
                    .write(key_pair.verifying_key().scalar().to_felt252())
                    .expect("Failed to insert public key");

                Ok(CheatcodeHadlingResult::Result(()))
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

                Ok(CheatcodeHadlingResult::Result(()))
            }
            _ => Ok(CheatcodeHadlingResult::Forward),
        }?;

        let result_end = buffer.ptr;
        insert_value_to_cellref!(vm, output_start, result_start)?;
        insert_value_to_cellref!(vm, output_end, result_end)?;
        Ok(res)
    }
}

pub struct TestExecutionState {
    pub environment_variables: HashMap<String, String>,
    pub contracts: HashMap<String, StarknetContractArtifacts>,
}

fn handle_deploy_result(
    deploy_result: Result<DeployCallPayload, CheatcodeError>,
    buffer: &mut MemBuffer,
) -> Result<CheatcodeHadlingResult, EnhancedHintError> {
    match deploy_result {
        Ok(deploy_payload) => {
            let felt_contract_address: Felt252 = deploy_payload.contract_address.to_felt252();

            buffer
                .write(Felt252::from(0))
                .expect("Failed to insert error code");
            buffer
                .write(felt_contract_address)
                .expect("Failed to insert deployed contract address");
            Ok(CheatcodeHadlingResult::Result(()))
        }
        Err(CheatcodeError::Recoverable(panic_data)) => {
            write_cheatcode_panic(buffer, &panic_data);
            Ok(CheatcodeHadlingResult::Result(()))
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
