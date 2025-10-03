use self::contracts_data::ContractsData;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use crate::runtime_extensions::common::sum_syscall_usage;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::replace_bytecode::ReplaceBytecodeError;
use crate::runtime_extensions::{
    call_to_blockifier_runtime_extension::{
        CallToBlockifierRuntime,
        rpc::{CallFailure, CallResult},
    },
    common::get_relocated_vm_trace,
    forge_runtime_extension::cheatcodes::{
        CheatcodeError,
        declare::declare,
        generate_random_felt::generate_random_felt,
        get_class_hash::get_class_hash,
        l1_handler_execute::l1_handler_execute,
        storage::{calculate_variable_address, load, store},
    },
};
use crate::state::{CallTrace, CallTraceNode};
use anyhow::{Context, Result, anyhow};
use blockifier::bouncer::vm_resources_to_sierra_gas;
use blockifier::context::TransactionContext;
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::syscalls::vm_syscall_utils::{SyscallSelector, SyscallUsageMap};
use blockifier::state::errors::StateError;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::vm::{
    errors::hint_errors::HintError, runners::cairo_runner::ExecutionResources,
    vm_core::VirtualMachine,
};
use conversions::byte_array::ByteArray;
use conversions::felt::{ToShortString, TryInferFormat};
use conversions::serde::deserialize::BufferReader;
use conversions::serde::serialize::CairoSerialize;
use data_transformer::cairo_types::CairoU256;
use rand::prelude::StdRng;
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic,
    SyscallHandlingResult,
};
use scarb_oracle_hint_service::OracleHintService;
use starknet::signers::SigningKey;
use starknet_api::{contract_class::EntryPointType::L1Handler, core::ClassHash};
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub mod cheatcodes;
pub mod contracts_data;
mod file_operations;
mod fuzzer;

pub type ForgeRuntime<'a> = ExtendedRuntime<ForgeExtension<'a>>;

pub struct ForgeExtension<'a> {
    pub environment_variables: &'a HashMap<String, String>,
    pub contracts_data: &'a ContractsData,
    pub fuzzer_rng: Option<Arc<Mutex<StdRng>>>,
    /// Whether `--experimental-oracles` flag has been enabled.
    pub experimental_oracles_enabled: bool,
    pub oracle_hint_service: OracleHintService,
}

// This runtime extension provides an implementation logic for functions from snforge_std library.
impl<'a> ExtensionLogic for ForgeExtension<'a> {
    type Runtime = CallToBlockifierRuntime<'a>;

    #[expect(clippy::too_many_lines)]
    fn handle_cheatcode(
        &mut self,
        selector: &str,
        mut input_reader: BufferReader<'_>,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        if let Some(oracle_selector) = self
            .oracle_hint_service
            .accept_cheatcode(selector.as_bytes())
        {
            if !self.experimental_oracles_enabled {
                return Err(anyhow!(
                    "Oracles are an experimental feature. \
                    To enable them, pass `--experimental-oracles` CLI flag."
                )
                .into());
            }

            let output = self
                .oracle_hint_service
                .execute_cheatcode(oracle_selector, input_reader.into_remaining());
            return Ok(CheatcodeHandlingResult::Handled(output));
        }

        match selector {
            "is_config_mode" => Ok(CheatcodeHandlingResult::from_serializable(false)),
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
                    .base
                    .state
                    .get_compiled_class(class)
                {
                    Err(StateError::UndeclaredClassHash(_)) => true,
                    Err(err) => return Err(err.into()),
                    _ => false,
                };

                let res = if extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .base
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
                    .base
                    .state;

                let contract_name: String = input_reader.read::<ByteArray>()?.to_string();

                handle_declare_result(declare(*state, &contract_name, self.contracts_data))
            }
            // Internal cheatcode used to pass a contract address when calling `deploy_at`.
            "set_deploy_at_address" => {
                let contract_address = input_reader.read()?;

                let state = &mut *extended_runtime.extended_runtime.extension.cheatnet_state;
                state.set_next_deploy_at_address(contract_address);

                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "precalculate_address" => {
                let class_hash = input_reader.read()?;
                let calldata: Vec<_> = input_reader.read()?;

                let contract_address = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .precalculate_address(&class_hash, &calldata);

                Ok(CheatcodeHandlingResult::from_serializable(contract_address))
            }
            // Internal cheatcode to guarantee unique salts for each deployment
            // when deploying via a method of the `ContractClass` struct.
            "get_salt" => {
                let state = &mut *extended_runtime.extended_runtime.extension.cheatnet_state;

                let salt = state.get_salt();
                state.increment_deploy_salt_base();

                Ok(CheatcodeHandlingResult::from_serializable(salt))
            }
            "var" => {
                let name: String = input_reader.read::<ByteArray>()?.to_string();

                let env_var = self
                    .environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = Felt::infer_format_and_parse(env_var)
                    .map_err(|_| anyhow!("Failed to parse value = {env_var} to felt"))?;

                Ok(CheatcodeHandlingResult::Handled(parsed_env_var))
            }
            "get_class_hash" => {
                let contract_address = input_reader.read()?;

                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .base
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
                    function_selector,
                    from_address,
                    &payload,
                ) {
                    CallResult::Success { .. } => {
                        Ok(CheatcodeHandlingResult::from_serializable(0_u8))
                    }
                    CallResult::Failure(CallFailure::Panic { panic_data }) => Ok(
                        CheatcodeHandlingResult::from_serializable(Err::<(), _>(panic_data)),
                    ),
                    CallResult::Failure(CallFailure::Error { msg }) => Err(
                        EnhancedHintError::from(HintError::CustomHint(Box::from(msg.to_string()))),
                    ),
                }
            }
            "read_txt" => {
                let file_path: String = input_reader.read::<ByteArray>()?.to_string();
                let parsed_content = file_operations::read_txt(file_path)?;

                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "read_json" => {
                let file_path: String = input_reader.read::<ByteArray>()?.to_string();
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
            "spy_messages_to_l1" => {
                let messages_offset = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .detected_messages_to_l1
                    .len();

                Ok(CheatcodeHandlingResult::from_serializable(messages_offset))
            }
            "get_messages_to_l1" => {
                let messages_offset = input_reader.read()?;

                let messages = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .get_messages_to_l1(messages_offset);

                Ok(CheatcodeHandlingResult::from_serializable(messages))
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

                if private_key == Felt::from(0_u8) {
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
                let curve: Felt = input_reader.read()?;
                let curve = curve.to_short_string().ok();

                let (signing_key_bytes, x_coordinate_bytes, y_coordinate_bytes) = {
                    let extract_coordinates_from_verifying_key = |verifying_key: Box<[u8]>| {
                        let verifying_key = verifying_key.iter().as_slice();
                        (
                            verifying_key[1..33].try_into().unwrap(),
                            verifying_key[33..65].try_into().unwrap(),
                        )
                    };

                    match curve.as_deref() {
                        Some("Secp256k1") => {
                            let signing_key = k256::ecdsa::SigningKey::random(
                                &mut k256::elliptic_curve::rand_core::OsRng,
                            );
                            let verifying_key = signing_key
                                .verifying_key()
                                .to_encoded_point(false)
                                .to_bytes();
                            let (x_coordinate, y_coordinate) =
                                extract_coordinates_from_verifying_key(verifying_key);
                            (
                                signing_key.to_bytes().as_slice()[0..32].try_into().unwrap(),
                                x_coordinate,
                                y_coordinate,
                            )
                        }
                        Some("Secp256r1") => {
                            let signing_key = p256::ecdsa::SigningKey::random(
                                &mut p256::elliptic_curve::rand_core::OsRng,
                            );
                            let verifying_key = signing_key
                                .verifying_key()
                                .to_encoded_point(false)
                                .to_bytes();
                            let (x_coordinate, y_coordinate) =
                                extract_coordinates_from_verifying_key(verifying_key);
                            (
                                signing_key.to_bytes().as_slice()[0..32].try_into().unwrap(),
                                x_coordinate,
                                y_coordinate,
                            )
                        }
                        _ => return Ok(CheatcodeHandlingResult::Forwarded),
                    }
                };

                Ok(CheatcodeHandlingResult::from_serializable((
                    CairoU256::from_bytes(&signing_key_bytes),
                    CairoU256::from_bytes(&x_coordinate_bytes), // bytes of public_key's x-coordinate
                    CairoU256::from_bytes(&y_coordinate_bytes), // bytes of public_key's y-coordinate
                )))
            }
            "ecdsa_sign_message" => {
                let curve: Felt = input_reader.read()?;
                let curve = curve.to_short_string().ok();
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
                    let r_bytes: [u8; 32] = r_bytes.as_slice()[0..32].try_into().unwrap();
                    let s_bytes: [u8; 32] = s_bytes.as_slice()[0..32].try_into().unwrap();
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
                    .base
                    .state;
                let target = input_reader.read()?;
                let storage_address = input_reader.read()?;
                store(*state, target, storage_address, input_reader.read()?)
                    .context("Failed to store")?;

                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "load" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .base
                    .state;
                let target = input_reader.read()?;
                let storage_address = input_reader.read()?;
                let loaded = load(*state, target, storage_address).context("Failed to load")?;

                Ok(CheatcodeHandlingResult::from_serializable(loaded))
            }
            "map_entry_address" => {
                let map_selector = input_reader.read()?;
                let keys: Vec<_> = input_reader.read()?;
                let map_entry_address = calculate_variable_address(map_selector, Some(&keys));

                Ok(CheatcodeHandlingResult::from_serializable(
                    map_entry_address,
                ))
            }
            "generate_random_felt" => Ok(CheatcodeHandlingResult::from_serializable(
                generate_random_felt(),
            )),
            "generate_arg" => {
                let min_value = input_reader.read()?;
                let max_value = input_reader.read()?;

                Ok(CheatcodeHandlingResult::from_serializable(
                    fuzzer::generate_arg(self.fuzzer_rng.clone(), min_value, max_value)?,
                ))
            }
            "save_fuzzer_arg" => {
                let arg = input_reader.read::<ByteArray>()?.to_string();
                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    // Skip first character, which is a snapshot symbol '@'
                    .update_fuzzer_args(arg[1..].to_string());

                Ok(CheatcodeHandlingResult::from_serializable(()))
            }
            "set_block_hash" => {
                let block_number = input_reader.read()?;
                let operation = input_reader.read()?;

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .cheat_block_hash(block_number, operation);
                Ok(CheatcodeHandlingResult::from_serializable(()))
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
            SyscallSelector::ReplaceClass => Err(HintError::CustomHint(Box::from(
                "Replace class can't be used in tests",
            ))),
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }
}

#[derive(CairoSerialize)]
enum SignError {
    InvalidSecretKey,
    HashOutOfRange,
}

fn handle_declare_result<T: CairoSerialize>(
    declare_result: Result<T, CheatcodeError>,
) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
    let result = match declare_result {
        Ok(data) => Ok(data),
        Err(CheatcodeError::Recoverable(panic_data)) => Err(panic_data),
        Err(CheatcodeError::Unrecoverable(err)) => return Err(err),
    };

    Ok(CheatcodeHandlingResult::from_serializable(result))
}

pub fn add_resources_to_top_call(
    runtime: &mut ForgeRuntime,
    resources: &ExecutionResources,
    tracked_resource: &TrackedResource,
) {
    let versioned_constants = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .base
        .context
        .tx_context
        .block_context
        .versioned_constants();
    let top_call = runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top();
    let mut top_call = top_call.borrow_mut();

    match tracked_resource {
        TrackedResource::CairoSteps => top_call.used_execution_resources += resources,
        TrackedResource::SierraGas => {
            top_call.gas_consumed +=
                vm_resources_to_sierra_gas(&resources.clone(), versioned_constants).0;
        }
    }
}

pub fn update_top_call_resources(
    runtime: &mut ForgeRuntime,
    top_call_tracked_resource: TrackedResource,
) {
    // call representing the test code
    let top_call = runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top();

    let all_execution_resources = add_execution_resources(top_call.clone());
    let all_sierra_gas_consumed = add_sierra_gas_resources(&top_call);

    // Below syscall usages are cumulative, meaning they include syscalls from their inner calls.
    let nested_calls_syscalls_vm_resources = get_nested_calls_syscalls_vm_resources(&top_call);
    let nested_calls_syscalls_sierra_gas = get_nested_calls_syscalls_sierra_gas(&top_call);

    let mut top_call = top_call.borrow_mut();
    top_call.used_execution_resources = all_execution_resources;
    top_call.gas_consumed = all_sierra_gas_consumed;

    // Syscall usage here is flat, meaning it only includes syscalls from current call (in this case the top-level call)
    let top_call_syscalls = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .base
        .syscalls_usage
        .clone();

    let mut total_syscalls_vm_resources = nested_calls_syscalls_vm_resources.clone();
    let mut total_syscalls_sierra_gas = nested_calls_syscalls_sierra_gas.clone();

    // Based on the tracked resource of top call, we add the syscall usage to respective totals.
    match top_call_tracked_resource {
        TrackedResource::CairoSteps => {
            total_syscalls_vm_resources =
                sum_syscall_usage(total_syscalls_vm_resources, &top_call_syscalls);
        }
        TrackedResource::SierraGas => {
            total_syscalls_sierra_gas =
                sum_syscall_usage(total_syscalls_sierra_gas, &top_call_syscalls);
        }
    }

    top_call.used_syscalls_vm_resources = total_syscalls_vm_resources;
    top_call.used_syscalls_sierra_gas = total_syscalls_sierra_gas;
}

/// Calculates the total syscall usage from nested calls where the tracked resource is Cairo steps.
pub fn get_nested_calls_syscalls_vm_resources(trace: &Rc<RefCell<CallTrace>>) -> SyscallUsageMap {
    // Only sum 1-level since these include syscalls from inner calls
    trace
        .borrow()
        .nested_calls
        .iter()
        .filter_map(CallTraceNode::extract_entry_point_call)
        .fold(SyscallUsageMap::new(), |syscalls, trace| {
            sum_syscall_usage(syscalls, &trace.borrow().used_syscalls_vm_resources)
        })
}

/// Calculates the total syscall usage from nested calls where the tracked resource is Sierra gas.
pub fn get_nested_calls_syscalls_sierra_gas(trace: &Rc<RefCell<CallTrace>>) -> SyscallUsageMap {
    // Only sum 1-level since these include syscalls from inner calls
    trace
        .borrow()
        .nested_calls
        .iter()
        .filter_map(CallTraceNode::extract_entry_point_call)
        .fold(SyscallUsageMap::new(), |syscalls, trace| {
            sum_syscall_usage(syscalls, &trace.borrow().used_syscalls_sierra_gas)
        })
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
        .base
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

pub fn update_top_call_vm_trace(runtime: &mut ForgeRuntime, cairo_runner: &mut CairoRunner) {
    let trace_data = &mut runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data;

    if trace_data.is_vm_trace_needed {
        trace_data.current_call_stack.top().borrow_mut().vm_trace =
            Some(get_relocated_vm_trace(cairo_runner));
    }
}

fn add_sierra_gas_resources(top_call: &Rc<RefCell<CallTrace>>) -> u64 {
    let mut gas_consumed = top_call.borrow().gas_consumed;
    for nested_call in &top_call.borrow().nested_calls {
        if let CallTraceNode::EntryPointCall(nested_call) = nested_call {
            gas_consumed += &nested_call.borrow().gas_consumed;
        }
    }
    gas_consumed
}

#[expect(clippy::needless_pass_by_value)]
fn add_execution_resources(top_call: Rc<RefCell<CallTrace>>) -> ExecutionResources {
    let mut execution_resources = top_call.borrow().used_execution_resources.clone();
    for nested_call in &top_call.borrow().nested_calls {
        match nested_call {
            CallTraceNode::EntryPointCall(nested_call) => {
                execution_resources += &nested_call.borrow().used_execution_resources;
            }
            CallTraceNode::DeployWithoutConstructor => {}
        }
    }
    execution_resources
}

#[must_use]
pub fn get_all_used_resources(
    call_info: &CallInfo,
    trace: &Rc<RefCell<CallTrace>>,
    transaction_context: &TransactionContext,
) -> UsedResources {
    let versioned_constants = transaction_context.block_context.versioned_constants();

    let summary = call_info.summarize(versioned_constants);

    let l1_handler_payload_lengths = get_l1_handlers_payloads_lengths(&call_info.inner_calls);

    // Syscalls are used only for `--detailed-resources` output.
    let top_call_syscalls = trace.borrow().get_total_used_syscalls();

    UsedResources {
        syscall_usage: top_call_syscalls,
        execution_summary: summary,
        l1_handler_payload_lengths,
    }
}
