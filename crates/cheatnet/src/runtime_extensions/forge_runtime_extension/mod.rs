use self::contracts_data::ContractsData;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use crate::runtime_extensions::common::{get_syscalls_gas_consumed, sum_syscall_usage};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::replace_bytecode::ReplaceBytecodeError;
use crate::runtime_extensions::{
    call_to_blockifier_runtime_extension::{
        CallToBlockifierRuntime,
        rpc::{CallFailure, CallResult},
    },
    cheatable_starknet_runtime_extension::SyscallSelector,
    common::get_relocated_vm_trace,
    forge_runtime_extension::cheatcodes::{
        CheatcodeError,
        declare::declare,
        deploy::{deploy, deploy_at},
        generate_random_felt::generate_random_felt,
        get_class_hash::get_class_hash,
        l1_handler_execute::l1_handler_execute,
        storage::{calculate_variable_address, load, store},
    },
};
use crate::state::CallTraceNode;
use anyhow::{Context, Result, anyhow};
use blockifier::bouncer::builtins_to_sierra_gas;
use blockifier::context::TransactionContext;
use blockifier::execution::contract_class::TrackedResource;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::ExecutionResourcesTraits;
use blockifier::utils::u64_from_usize;
use blockifier::{
    execution::{
        call_info::CallInfo, deprecated_syscalls::DeprecatedSyscallSelector,
        syscalls::hint_processor::SyscallUsageMap,
    },
    versioned_constants::VersionedConstants,
};
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
use starknet::signers::SigningKey;
use starknet_api::execution_resources::GasAmount;
use starknet_api::{contract_class::EntryPointType::L1Handler, core::ClassHash};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
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

                handle_declare_deploy_result(declare(*state, &contract_name, self.contracts_data))
            }
            "deploy" => {
                let class_hash = input_reader.read()?;
                let calldata: Vec<_> = input_reader.read()?;
                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;

                syscall_handler.increment_syscall_count_by(&DeprecatedSyscallSelector::Deploy, 1);
                syscall_handler
                    .increment_linear_factor_by(&DeprecatedSyscallSelector::Deploy, calldata.len());

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
                syscall_handler
                    .increment_linear_factor_by(&DeprecatedSyscallSelector::Deploy, calldata.len());

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

                Ok(CheatcodeHandlingResult::from_serializable(contract_address))
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
            DeprecatedSyscallSelector::ReplaceClass => Err(HintError::CustomHint(Box::from(
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

fn calculate_total_syscall_usage(runtime: &mut ForgeRuntime) -> SyscallUsageMap {
    let top_call = runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top();
    let top_call = top_call.borrow_mut();
    let top_call_syscalls = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .syscalls_usage
        .clone();

    // Only sum 1-level since these include syscalls from inner calls
    let nested_calls_syscalls = top_call
        .nested_calls
        .iter()
        .filter_map(CallTraceNode::extract_entry_point_call)
        .fold(SyscallUsageMap::new(), |syscalls, trace| {
            sum_syscall_usage(syscalls, &trace.borrow().used_syscalls)
        });

    sum_syscall_usage(top_call_syscalls, &nested_calls_syscalls)
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

#[must_use]
pub fn get_all_used_resources(
    mut runtime: ForgeRuntime,
    transaction_context: &TransactionContext,
    tracked_resource: TrackedResource,
    call_info: &CallInfo,
) -> UsedResources {
    let total_syscall_usage = calculate_total_syscall_usage(&mut runtime);
    let versioned_constants = transaction_context.block_context.versioned_constants();
    let summary = call_info.summarize(versioned_constants);
    let l2_to_l1_payload_lengths = summary.l2_to_l1_payload_lengths;
    let l1_handler_payload_lengths = get_l1_handlers_payloads_lengths(&call_info.inner_calls);

    let mut gas_consumed = summary.charged_resources.gas_consumed.0;

    if tracked_resource == TrackedResource::SierraGas {
        gas_consumed += get_syscalls_gas_consumed(&total_syscall_usage, versioned_constants);
    }

    let events = call_info
        .iter() // This method iterates over inner calls as well
        .flat_map(|call_info| {
            call_info
                .execution
                .events
                .iter()
                .map(|evt| evt.event.clone())
        })
        .collect();

    UsedResources {
        events,
        syscall_usage: total_syscall_usage,
        execution_resources: summary.charged_resources.vm_resources,
        gas_consumed: GasAmount(gas_consumed),
        l1_handler_payload_lengths,
        l2_to_l1_payload_lengths,
    }
}

fn n_steps_to_sierra_gas(n_steps: usize, versioned_constants: &VersionedConstants) -> GasAmount {
    let n_steps_u64 = u64_from_usize(n_steps);
    let gas_per_step = versioned_constants
        .os_constants
        .gas_costs
        .base
        .step_gas_cost;
    let n_steps_gas_cost = n_steps_u64.checked_mul(gas_per_step).unwrap_or_else(|| {
        panic!(
            "Multiplication overflow while converting steps to gas. steps: {n_steps}, gas per step: {gas_per_step}."
        )
    });
    GasAmount(n_steps_gas_cost)
}

// Based on: https://github.com/starkware-libs/sequencer/blob/main-v0.13.4/crates/blockifier/src/bouncer.rs#L320
#[must_use]
pub fn vm_resources_to_sierra_gas(
    resources: &ExecutionResources,
    versioned_constants: &VersionedConstants,
) -> GasAmount {
    let builtins_gas_cost =
        builtins_to_sierra_gas(&resources.prover_builtins(), versioned_constants);
    let n_steps_gas_cost = n_steps_to_sierra_gas(resources.total_n_steps(), versioned_constants);
    n_steps_gas_cost.checked_add(builtins_gas_cost).unwrap_or_else(|| {
        panic!(
            "Addition overflow while converting vm resources to gas. steps gas: {n_steps_gas_cost}, \
            builtins gas: {builtins_gas_cost}."
        )
    })
}
