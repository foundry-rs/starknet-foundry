use self::contracts_data::ContractsData;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::replace_bytecode::ReplaceBytecodeError;
use crate::runtime_extensions::{
    call_to_blockifier_runtime_extension::{
        rpc::{CallFailure, CallResult, UsedResources},
        CallToBlockifierRuntime,
    },
    cheatable_starknet_runtime_extension::SyscallSelector,
    common::{get_relocated_vm_trace, sum_syscall_counters},
    forge_runtime_extension::cheatcodes::{
        declare::declare,
        deploy::{deploy, deploy_at},
        get_class_hash::get_class_hash,
        l1_handler_execute::l1_handler_execute,
        storage::{calculate_variable_address, load, store},
        CheatcodeError,
    },
};
use crate::state::CallTraceNode;
use anyhow::{anyhow, Context, Result};
use blockifier::state::errors::StateError;
use blockifier::{
    context::TransactionContext,
    execution::{
        call_info::{CallExecution, CallInfo},
        deprecated_syscalls::DeprecatedSyscallSelector,
        syscalls::hint_processor::SyscallCounter,
    },
    versioned_constants::VersionedConstants,
};
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_vm::vm::{
    errors::hint_errors::HintError, runners::cairo_runner::ExecutionResources,
    vm_core::VirtualMachine,
};
use conversions::byte_array::ByteArray;
use conversions::serde::deserialize::{BufferReader, CairoDeserialize};
use conversions::serde::serialize::CairoSerialize;
use conversions::{felt252::TryInferFormat, IntoConv};
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic,
    SyscallHandlingResult,
};
use starknet::core::types::FieldElement;
use starknet::signers::SigningKey;
use starknet_api::{core::ClassHash, deprecated_contract_class::EntryPointType::L1Handler};
use std::collections::HashMap;

pub mod cheatcodes;
pub mod contracts_data;
mod file_operations;

pub type ForgeRuntime<'a> = ExtendedRuntime<ForgeExtension<'a>>;

pub struct ForgeExtension<'a> {
    pub environment_variables: &'a HashMap<String, String>,
    pub contracts_data: &'a ContractsData,
}

// This runtime extension provides an implementation logic for functions from snforge_std library.
impl<'a> ExtensionLogic for ForgeExtension<'a> {
    type Runtime = CallToBlockifierRuntime<'a>;

    #[allow(clippy::too_many_lines)]
    fn handle_cheatcode(
        &mut self,
        selector: &str,
        mut input_reader: BufferReader<'_>,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        match selector {
            "cheat_execution_info" => {
                let execution_info = input_reader.read()?;

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .cheat_execution_info(execution_info);

                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "mock_call" => {
                let contract_address = input_reader.read()?;
                let function_selector = input_reader.read()?;
                let span = input_reader.read()?;

                let ret_data: Vec<_> = input_reader.read()?;

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .mock_call(contract_address, function_selector, &ret_data, span);
                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "stop_mock_call" => {
                let contract_address = input_reader.read()?;
                let function_selector = input_reader.read()?;

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_mock_call(contract_address, function_selector);
                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "replace_bytecode" => {
                let contract = input_reader.read()?;
                let class = input_reader.read()?;

                let is_undeclared = match extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state
                    .get_compiled_contract_class(class)
                {
                    Err(StateError::UndeclaredClassHash(_)) => true,
                    Err(err) => return Err(err.into()),
                    _ => false,
                };

                let res = if extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state
                    .get_class_hash_at(contract)?
                    == ClassHash::default()
                {
                    Err(ReplaceBytecodeError::ContractNotDeployed)
                } else if is_undeclared {
                    Err(ReplaceBytecodeError::UndeclaredClassHash)
                } else {
                    extended_runtime
                        .extended_runtime
                        .extension
                        .cheatnet_state
                        .replace_class_for_contract(contract, class);
                    Ok(())
                };

                Ok(CheatcodeHandlingResult::from_serializable(res))
            }
            "declare" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;

                let contract_name: String = input_reader.read::<ByteArray>()?.into();

                handle_declare_deploy_result(declare(*state, &contract_name, self.contracts_data))
            }
            "deploy" => {
                let class_hash = input_reader.read()?;
                let calldata: Vec<_> = input_reader.read()?;
                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;

                syscall_handler.increment_syscall_count_by(&DeprecatedSyscallSelector::Deploy, 1);

                handle_declare_deploy_result(deploy(
                    syscall_handler,
                    cheatnet_runtime.extension.cheatnet_state,
                    &class_hash,
                    &calldata,
                ))
            }
            "deploy_at" => {
                let class_hash = input_reader.read()?;
                let calldata: Vec<_> = input_reader.read()?;
                let contract_address = input_reader.read()?;
                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;

                syscall_handler.increment_syscall_count_by(&DeprecatedSyscallSelector::Deploy, 1);

                handle_declare_deploy_result(deploy_at(
                    syscall_handler,
                    cheatnet_runtime.extension.cheatnet_state,
                    &class_hash,
                    &calldata,
                    contract_address,
                ))
            }
            "precalculate_address" => {
                let class_hash = input_reader.read()?;
                let calldata: Vec<_> = input_reader.read()?;

                let contract_address = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .precalculate_address(&class_hash, &calldata);

                let felt_contract_address: Felt252 = contract_address.into_();

                Ok(CheatcodeHandlingResult::from_serializable(
                    felt_contract_address,
                ))
            }
            "var" => {
                let name: String = input_reader.read::<ByteArray>()?.into();

                let env_var = self
                    .environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = Felt252::infer_format_and_parse(env_var)
                    .map_err(|_| anyhow!("Failed to parse value = {env_var} to felt"))?;

                Ok(CheatcodeHandlingResult::Handled(parsed_env_var))
            }
            "get_class_hash" => {
                let contract_address = input_reader.read()?;

                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;

                match get_class_hash(*state, contract_address) {
                    Ok(class_hash) => Ok(CheatcodeHandlingResult::from_serializable(class_hash)),
                    Err(CheatcodeError::Recoverable(_)) => unreachable!(),
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "l1_handler_execute" => {
                let contract_address = input_reader.read()?;
                let function_selector = input_reader.read()?;
                let from_address = input_reader.read()?;

                let payload: Vec<_> = input_reader.read()?;

                let cheatnet_runtime = &mut extended_runtime.extended_runtime;

                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;
                match l1_handler_execute(
                    syscall_handler,
                    cheatnet_runtime.extension.cheatnet_state,
                    contract_address,
                    &function_selector,
                    &from_address,
                    &payload,
                ) {
                    CallResult::Success { .. } => {
                        Ok(CheatcodeHandlingResult::from_serializable(0_u8))
                    }
                    CallResult::Failure(CallFailure::Panic { panic_data }) => Ok(
                        CheatcodeHandlingResult::from_serializable(Err::<(), _>(panic_data)),
                    ),
                    CallResult::Failure(CallFailure::Error { msg }) => Err(
                        EnhancedHintError::from(HintError::CustomHint(Box::from(msg))),
                    ),
                }
            }
            "read_txt" => {
                let file_path: String = input_reader.read::<ByteArray>()?.into();
                let parsed_content = file_operations::read_txt(file_path)?;

                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "read_json" => {
                let file_path: String = input_reader.read::<ByteArray>()?.into();
                let parsed_content = file_operations::read_json(file_path)?;

                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "spy_events" => {
                let events_offset = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .detected_events
                    .len();

                Ok(CheatcodeHandlingResult::from_serializable(events_offset))
            }
            "get_events" => {
                let events_offset = input_reader.read()?;

                let events = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .get_events(events_offset);

                Ok(CheatcodeHandlingResult::from_serializable(events))
            }
            "generate_stark_keys" => {
                let key_pair = SigningKey::from_random();

                Ok(CheatcodeHandlingResult::from_serializable((
                    key_pair.secret_scalar(),
                    key_pair.verifying_key().scalar(),
                )))
            }
            "stark_sign_message" => {
                let private_key = input_reader.read()?;
                let message_hash = input_reader.read()?;

                if private_key == FieldElement::from(0_u8) {
                    return Ok(CheatcodeHandlingResult::from_serializable(Err::<(), _>(
                        SignError::InvalidSecretKey,
                    )));
                }

                let key_pair = SigningKey::from_secret_scalar(private_key);

                let result = if let Ok(signature) = key_pair.sign(&message_hash) {
                    Ok((signature.r, signature.s))
                } else {
                    Err(SignError::HashOutOfRange)
                };

                Ok(CheatcodeHandlingResult::from_serializable(result))
            }
            "generate_ecdsa_keys" => {
                let curve = as_cairo_short_string(&input_reader.read()?);

                let (signing_key_bytes, verifying_key_bytes) = {
                    match curve.as_deref() {
                        Some("Secp256k1") => {
                            let signing_key = k256::ecdsa::SigningKey::random(
                                &mut k256::elliptic_curve::rand_core::OsRng,
                            );
                            let verifying_key = signing_key.verifying_key();
                            (
                                signing_key.to_bytes(),
                                verifying_key.to_encoded_point(false).to_bytes(),
                            )
                        }
                        Some("Secp256r1") => {
                            let signing_key = p256::ecdsa::SigningKey::random(
                                &mut p256::elliptic_curve::rand_core::OsRng,
                            );
                            let verifying_key = signing_key.verifying_key();
                            (
                                signing_key.to_bytes(),
                                verifying_key.to_encoded_point(false).to_bytes(),
                            )
                        }
                        _ => return Ok(CheatcodeHandlingResult::Forwarded),
                    }
                };

                Ok(CheatcodeHandlingResult::from_serializable((
                    CairoU256::from_bytes(&signing_key_bytes),
                    CairoU256::from_bytes(&verifying_key_bytes[1..]), // bytes of public_key's x-coordinate
                    CairoU256::from_bytes(&verifying_key_bytes[33..]), // bytes of public_key's y-coordinate
                )))
            }
            "ecdsa_sign_message" => {
                let curve = as_cairo_short_string(&input_reader.read()?);
                let secret_key: CairoU256 = input_reader.read()?;
                let msg_hash: CairoU256 = input_reader.read()?;

                let result = {
                    match curve.as_deref() {
                        Some("Secp256k1") => {
                            if let Ok(signing_key) =
                                k256::ecdsa::SigningKey::from_slice(&secret_key.to_be_bytes())
                            {
                                let signature: k256::ecdsa::Signature =
                                    k256::ecdsa::signature::hazmat::PrehashSigner::sign_prehash(
                                        &signing_key,
                                        &msg_hash.to_be_bytes(),
                                    )
                                    .unwrap();

                                Ok(signature.split_bytes())
                            } else {
                                Err(SignError::InvalidSecretKey)
                            }
                        }
                        Some("Secp256r1") => {
                            if let Ok(signing_key) =
                                p256::ecdsa::SigningKey::from_slice(&secret_key.to_be_bytes())
                            {
                                let signature: p256::ecdsa::Signature =
                                    p256::ecdsa::signature::hazmat::PrehashSigner::sign_prehash(
                                        &signing_key,
                                        &msg_hash.to_be_bytes(),
                                    )
                                    .unwrap();

                                Ok(signature.split_bytes())
                            } else {
                                Err(SignError::InvalidSecretKey)
                            }
                        }
                        _ => return Ok(CheatcodeHandlingResult::Forwarded),
                    }
                };

                let result = result.map(|(r_bytes, s_bytes)| {
                    (
                        CairoU256::from_bytes(&r_bytes),
                        CairoU256::from_bytes(&s_bytes),
                    )
                });

                Ok(CheatcodeHandlingResult::from_serializable(result))
            }
            "get_call_trace" => {
                let call_trace = &extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .trace_data
                    .current_call_stack
                    .borrow_full_trace();

                Ok(CheatcodeHandlingResult::from_serializable(call_trace))
            }
            "store" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;
                let target = input_reader.read()?;
                let storage_address = input_reader.read()?;
                store(*state, target, &storage_address, input_reader.read()?)
                    .context("Failed to store")?;

                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "load" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;
                let target = input_reader.read()?;
                let storage_address = &input_reader.read()?;
                let loaded = load(*state, target, storage_address).context("Failed to load")?;

                Ok(CheatcodeHandlingResult::from_serializable(loaded))
            }
            "map_entry_address" => {
                let map_selector = &input_reader.read()?;
                let keys: Vec<_> = input_reader.read()?;
                let map_entry_address = calculate_variable_address(map_selector, Some(&keys));

                Ok(CheatcodeHandlingResult::from_serializable(
                    map_entry_address,
                ))
            }
            _ => Ok(CheatcodeHandlingResult::Forwarded),
        }
    }

    fn override_system_call(
        &mut self,
        selector: SyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        match selector {
            DeprecatedSyscallSelector::ReplaceClass => Err(HintError::CustomHint(Box::from(
                "Replace class can't be used in tests",
            ))),
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }
}

#[derive(CairoDeserialize, CairoSerialize)]
struct CairoU256 {
    low: u128,
    high: u128,
}

impl CairoU256 {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            low: u128::from_be_bytes(bytes[16..32].try_into().unwrap()),
            high: u128::from_be_bytes(bytes[0..16].try_into().unwrap()),
        }
    }

    fn to_be_bytes(&self) -> [u8; 32] {
        let mut result = [0; 32];

        result[16..].copy_from_slice(&self.low.to_be_bytes());
        result[..16].copy_from_slice(&self.high.to_be_bytes());

        result
    }
}

#[derive(CairoSerialize)]
enum SignError {
    InvalidSecretKey,
    HashOutOfRange,
}

fn handle_declare_deploy_result<T: CairoSerialize>(
    declare_result: Result<T, CheatcodeError>,
) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
    let result = match declare_result {
        Ok(data) => Ok(data),
        Err(CheatcodeError::Recoverable(panic_data)) => Err(panic_data),
        Err(CheatcodeError::Unrecoverable(err)) => return Err(err),
    };

    Ok(CheatcodeHandlingResult::from_serializable(result))
}

pub fn update_top_call_execution_resources(runtime: &mut ForgeRuntime) {
    let all_execution_resources = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .resources
        .clone();

    // call representing the test code
    let top_call = runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top();
    let mut top_call = top_call.borrow_mut();

    top_call.used_execution_resources = all_execution_resources;

    let top_call_syscalls = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .syscall_counter
        .clone();

    // Only sum 1-level since these include syscalls from inner calls
    let nested_calls_syscalls = top_call
        .nested_calls
        .iter()
        .filter_map(CallTraceNode::extract_entry_point_call)
        .fold(SyscallCounter::new(), |syscalls, trace| {
            sum_syscall_counters(syscalls, &trace.borrow().used_syscalls)
        });

    top_call.used_syscalls = sum_syscall_counters(top_call_syscalls, &nested_calls_syscalls);
}

// Only top-level is considered relevant since we can't have l1 handlers deeper than 1 level of nesting
fn get_l1_handlers_payloads_lengths(inner_calls: &[CallInfo]) -> Vec<usize> {
    inner_calls
        .iter()
        .filter_map(|call_info| {
            if call_info.call.entry_point_type == L1Handler {
                return Some(call_info.call.calldata.0.len());
            }
            None
        })
        .collect()
}

pub fn update_top_call_l1_resources(runtime: &mut ForgeRuntime) {
    let all_l2_l1_message_sizes = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .l2_to_l1_messages
        .iter()
        .map(|ordered_message| ordered_message.message.payload.0.len())
        .collect();

    // call representing the test code
    let top_call = runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top();
    top_call.borrow_mut().used_l1_resources.l2_l1_message_sizes = all_l2_l1_message_sizes;
}

pub fn update_top_call_vm_trace(runtime: &mut ForgeRuntime, vm: &VirtualMachine) {
    let trace_data = &mut runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data;

    if trace_data.is_vm_trace_needed {
        trace_data.current_call_stack.top().borrow_mut().vm_trace =
            Some(get_relocated_vm_trace(vm));
    }
}
fn add_syscall_resources(
    versioned_constants: &VersionedConstants,
    execution_resources: &ExecutionResources,
    syscall_counter: &SyscallCounter,
) -> ExecutionResources {
    let mut total_vm_usage = execution_resources.filter_unused_builtins();
    total_vm_usage += &versioned_constants
        .get_additional_os_syscall_resources(syscall_counter)
        .expect("Could not get additional costs");
    total_vm_usage
}

#[must_use]
pub fn get_all_used_resources(
    runtime: ForgeRuntime,
    transaction_context: &TransactionContext,
) -> UsedResources {
    let starknet_runtime = runtime.extended_runtime.extended_runtime.extended_runtime;
    let top_call_l2_to_l1_messages = starknet_runtime.hint_handler.l2_to_l1_messages;
    let top_call_events = starknet_runtime.hint_handler.events;

    // used just to obtain payloads of L2 -> L1 messages
    let runtime_call_info = CallInfo {
        execution: CallExecution {
            l2_to_l1_messages: top_call_l2_to_l1_messages,
            events: top_call_events,
            ..Default::default()
        },
        inner_calls: starknet_runtime.hint_handler.inner_calls,
        ..Default::default()
    };
    let l2_to_l1_payload_lengths = runtime_call_info.get_l2_to_l1_payload_lengths();

    let l1_handler_payload_lengths =
        get_l1_handlers_payloads_lengths(&runtime_call_info.inner_calls);

    // call representing the test code
    let top_call = runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top();

    let execution_resources = top_call.borrow().used_execution_resources.clone();
    let top_call_syscalls = top_call.borrow().used_syscalls.clone();
    let events = runtime_call_info
        .iter() // This method iterates over inner calls as well
        .flat_map(|call_info| {
            call_info
                .execution
                .events
                .iter()
                .map(|evt| evt.event.clone())
        })
        .collect();

    let versioned_constants = transaction_context.block_context.versioned_constants();
    let execution_resources = add_syscall_resources(
        versioned_constants,
        &execution_resources,
        &top_call_syscalls,
    );

    UsedResources {
        events,
        syscall_counter: top_call_syscalls,
        execution_resources,
        l1_handler_payload_lengths,
        l2_to_l1_payload_lengths,
    }
}
