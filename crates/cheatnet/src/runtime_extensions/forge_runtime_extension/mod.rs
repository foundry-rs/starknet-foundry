use std::collections::HashMap;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult, UsedResources,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::{deploy, deploy_at};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use crate::state::{CallTrace, CheatTarget};
use anyhow::{Context, Result};
use blockifier::execution::call_info::{CallExecution, CallInfo};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use conversions::felt252::FromShortString;
use conversions::{FromConv, IntoConv};
use num_traits::ToPrimitive;
use scarb_api::StarknetContractArtifacts;

use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet_api::core::ContractAddress;

use crate::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::get_class_hash::get_class_hash;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::l1_handler_execute::l1_handler_execute;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::SpyTarget;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::storage::{
    calculate_variable_address, load, store,
};
use crate::runtime_extensions::forge_runtime_extension::file_operations::string_into_felt;
use cairo_lang_starknet::contract::starknet_keccak;
use conversions::byte_array::ByteArray;
use runtime::utils::BufferReader;
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic,
    SyscallHandlingResult,
};
use starknet::signers::SigningKey;
use starknet_api::deprecated_contract_class::EntryPointType;

use super::call_to_blockifier_runtime_extension::{CallToBlockifierRuntime, RuntimeState};
use super::cheatable_starknet_runtime_extension::SyscallSelector;

pub mod cheatcodes;
mod file_operations;

pub type ForgeRuntime<'a> = ExtendedRuntime<ForgeExtension<'a>>;

pub struct ForgeExtension<'a> {
    pub environment_variables: &'a HashMap<String, String>,
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
}

trait BufferReaderExt {
    fn read_cheat_target(&mut self) -> CheatTarget;
}

impl BufferReaderExt for BufferReader<'_> {
    fn read_cheat_target(&mut self) -> CheatTarget {
        let cheat_target_variant = self.read_felt().to_u8();
        match cheat_target_variant {
            Some(0) => CheatTarget::All,
            Some(1) => CheatTarget::One(self.read_felt().into_()),
            Some(2) => {
                let contract_addresses: Vec<_> = self
                    .read_vec()
                    .iter()
                    .map(|el| ContractAddress::from_(el.clone()))
                    .collect();
                CheatTarget::Multiple(contract_addresses)
            }
            _ => unreachable!("Invalid CheatTarget variant"),
        }
    }
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
            "start_roll" => {
                let target = input_reader.read_cheat_target();
                let block_number = input_reader.read_felt();
                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_roll(target, block_number);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_roll" => {
                let target = input_reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_roll(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_warp" => {
                let target = input_reader.read_cheat_target();
                let warp_timestamp = input_reader.read_felt();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_warp(target, warp_timestamp);

                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_warp" => {
                let target = input_reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_warp(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_elect" => {
                let target = input_reader.read_cheat_target();
                let sequencer_address = input_reader.read_felt().into_();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_elect(target, sequencer_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_elect" => {
                let target = input_reader.read_cheat_target();
                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_elect(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_prank" => {
                let target = input_reader.read_cheat_target();

                let caller_address = input_reader.read_felt().into_();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_prank(target, caller_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_prank" => {
                let target = input_reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_prank(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_mock_call" => {
                let contract_address = input_reader.read_felt().into_();
                let function_selector = input_reader.read_felt();

                let ret_data = input_reader.read_vec();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_mock_call(contract_address, function_selector, &ret_data);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_mock_call" => {
                let contract_address = input_reader.read_felt().into_();
                let function_selector = input_reader.read_felt();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_mock_call(contract_address, function_selector);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_spoof" => {
                let target = input_reader.read_cheat_target();

                let version = input_reader.read_option_felt();
                let account_contract_address = input_reader.read_option_felt();
                let max_fee = input_reader.read_option_felt();
                let signature = input_reader.read_option_vec();
                let transaction_hash = input_reader.read_option_felt();
                let chain_id = input_reader.read_option_felt();
                let nonce = input_reader.read_option_felt();
                let resource_bounds = input_reader.read_option_felt().map(|resource_bounds_len| {
                    input_reader.read_vec_body(
                        3 * resource_bounds_len.to_usize().unwrap(), // ResourceBounds struct has 3 fields
                    )
                });
                let tip = input_reader.read_option_felt();
                let paymaster_data = input_reader.read_option_vec();
                let nonce_data_availability_mode = input_reader.read_option_felt();
                let fee_data_availability_mode = input_reader.read_option_felt();
                let account_deployment_data = input_reader.read_option_vec();

                let tx_info_mock = cheatcodes::spoof::TxInfoMock {
                    version,
                    account_contract_address,
                    max_fee,
                    signature,
                    transaction_hash,
                    chain_id,
                    nonce,
                    resource_bounds,
                    tip,
                    paymaster_data,
                    nonce_data_availability_mode,
                    fee_data_availability_mode,
                    account_deployment_data,
                };

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_spoof(target, tx_info_mock);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_spoof" => {
                let target = input_reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_spoof(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "declare" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;

                let contract_name = input_reader.read_string();
                let contracts = self.contracts;
                match declare(*state, &contract_name, contracts) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);
                        let result = vec![Felt252::from(0), felt_class_hash];
                        Ok(CheatcodeHandlingResult::Handled(result))
                    }
                    Err(CheatcodeError::Recoverable(_)) => {
                        panic!("Declare should not fail recoverably!")
                    }
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "deploy" => {
                let class_hash = input_reader.read_felt().into_();
                let calldata = input_reader.read_vec();
                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;

                syscall_handler.increment_syscall_count_by(&DeprecatedSyscallSelector::Deploy, 1);

                handle_deploy_result(deploy(
                    syscall_handler,
                    &mut RuntimeState {
                        cheatnet_state: cheatnet_runtime.extension.cheatnet_state,
                    },
                    &class_hash,
                    &calldata,
                ))
            }
            "deploy_at" => {
                let class_hash = input_reader.read_felt().into_();
                let calldata = input_reader.read_vec();
                let contract_address = input_reader.read_felt().into_();
                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;

                syscall_handler.increment_syscall_count_by(&DeprecatedSyscallSelector::Deploy, 1);

                handle_deploy_result(deploy_at(
                    syscall_handler,
                    &mut RuntimeState {
                        cheatnet_state: cheatnet_runtime.extension.cheatnet_state,
                    },
                    &class_hash,
                    &calldata,
                    contract_address,
                ))
            }
            "precalculate_address" => {
                let class_hash = input_reader.read_felt().into_();
                let calldata = input_reader.read_vec();

                let contract_address = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .precalculate_address(&class_hash, &calldata);

                let felt_contract_address: Felt252 = contract_address.into_();

                Ok(CheatcodeHandlingResult::Handled(vec![
                    felt_contract_address,
                ]))
            }
            "var" => {
                let name = input_reader.read_string();

                let env_var = self
                    .environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = string_into_felt(env_var)
                    .with_context(|| format!("Failed to parse value = {env_var} to felt"))?;

                Ok(CheatcodeHandlingResult::Handled(vec![parsed_env_var]))
            }
            "get_class_hash" => {
                let contract_address = input_reader.read_felt().into_();

                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;

                match get_class_hash(*state, contract_address) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);

                        Ok(CheatcodeHandlingResult::Handled(vec![felt_class_hash]))
                    }
                    Err(CheatcodeError::Recoverable(_)) => unreachable!(),
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "l1_handler_execute" => {
                let contract_address = input_reader.read_felt().into_();
                let function_selector = input_reader.read_felt();
                let from_address = input_reader.read_felt();

                let payload = input_reader.read_vec();

                let cheatnet_runtime = &mut extended_runtime.extended_runtime;

                let mut runtime_state = RuntimeState {
                    cheatnet_state: cheatnet_runtime.extension.cheatnet_state,
                };
                let syscall_handler = &mut cheatnet_runtime.extended_runtime.hint_handler;
                match l1_handler_execute(
                    syscall_handler,
                    &mut runtime_state,
                    contract_address,
                    &function_selector,
                    &from_address,
                    &payload,
                ) {
                    CallResult::Success { .. } => {
                        Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(0)]))
                    }
                    CallResult::Failure(CallFailure::Panic { panic_data }) => Ok(
                        CheatcodeHandlingResult::Handled(cheatcode_panic_result(panic_data)),
                    ),
                    CallResult::Failure(CallFailure::Error { msg }) => Err(
                        EnhancedHintError::from(HintError::CustomHint(Box::from(msg))),
                    ),
                }
            }
            "read_txt" => {
                let file_path = input_reader.read_string();
                let parsed_content = file_operations::read_txt(file_path)?;
                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "read_json" => {
                let file_path = input_reader.read_string();
                let parsed_content = file_operations::read_json(file_path)?;

                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "spy_events" => {
                let spy_target_variant = input_reader
                    .read_felt()
                    .to_u8()
                    .expect("Invalid spy_target length");
                let spy_on = match spy_target_variant {
                    0 => SpyTarget::All,
                    1 => SpyTarget::One(input_reader.read_felt().into_()),
                    _ => {
                        let addresses = input_reader
                            .read_vec()
                            .iter()
                            .map(|el| ContractAddress::from_(el.clone()))
                            .collect();

                        SpyTarget::Multiple(addresses)
                    }
                };

                let id = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .spy_events(spy_on);
                Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(id)]))
            }
            "fetch_events" => {
                let id = &input_reader.read_felt();
                let (emitted_events_len, serialized_events) = extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .fetch_events(id);
                let mut result = vec![Felt252::from(emitted_events_len)];
                result.extend(serialized_events);
                Ok(CheatcodeHandlingResult::Handled(result))
            }
            "event_name_hash" => {
                let name = input_reader.read_felt();
                let hash = starknet_keccak(as_cairo_short_string(&name).unwrap().as_bytes());

                Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(hash)]))
            }
            "generate_stark_keys" => {
                let key_pair = SigningKey::from_random();

                Ok(CheatcodeHandlingResult::Handled(vec![
                    key_pair.secret_scalar().into_(),
                    key_pair.verifying_key().scalar().into_(),
                ]))
            }
            "stark_sign_message" => {
                let private_key = input_reader.read_felt();
                let message_hash = input_reader.read_felt();

                if private_key == Felt252::from(0) {
                    return Ok(handle_cheatcode_error("invalid secret_key"));
                }

                let key_pair = SigningKey::from_secret_scalar(private_key.into_());

                if let Ok(signature) = key_pair.sign(&message_hash.into_()) {
                    Ok(CheatcodeHandlingResult::Handled(vec![
                        Felt252::from(0),
                        Felt252::from_(signature.r),
                        Felt252::from_(signature.s),
                    ]))
                } else {
                    Ok(handle_cheatcode_error("message_hash out of range"))
                }
            }
            "generate_ecdsa_keys" => {
                let curve = as_cairo_short_string(&input_reader.read_felt());

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

                Ok(CheatcodeHandlingResult::Handled(vec![
                    Felt252::from_bytes_be(&signing_key_bytes[16..32]), // 16 low  bytes of secret_key
                    Felt252::from_bytes_be(&signing_key_bytes[0..16]), // 16 high bytes of secret_key
                    Felt252::from_bytes_be(&verifying_key_bytes[17..33]), // 16 low  bytes of public_key's x-coordinate
                    Felt252::from_bytes_be(&verifying_key_bytes[1..17]), // 16 high bytes of public_key's x-coordinate
                    Felt252::from_bytes_be(&verifying_key_bytes[49..65]), // 16 low  bytes of public_key's y-coordinate
                    Felt252::from_bytes_be(&verifying_key_bytes[33..49]), // 16 high bytes of public_key's y-coordinate
                ]))
            }
            "ecdsa_sign_message" => {
                let curve = as_cairo_short_string(&input_reader.read_felt());
                let sk_low = input_reader.read_felt();
                let sk_high = input_reader.read_felt();
                let msg_hash_low = input_reader.read_felt();
                let msg_hash_high = input_reader.read_felt();

                let secret_key = concat_u128_bytes(&sk_low.to_be_bytes(), &sk_high.to_be_bytes());
                let msg_hash =
                    concat_u128_bytes(&msg_hash_low.to_be_bytes(), &msg_hash_high.to_be_bytes());

                let (r_bytes, s_bytes) = {
                    match curve.as_deref() {
                        Some("Secp256k1") => {
                            let Ok(signing_key) = k256::ecdsa::SigningKey::from_slice(&secret_key)
                            else {
                                return Ok(handle_cheatcode_error("invalid secret_key"));
                            };

                            let signature: k256::ecdsa::Signature =
                                k256::ecdsa::signature::hazmat::PrehashSigner::sign_prehash(
                                    &signing_key,
                                    &msg_hash,
                                )
                                .unwrap();

                            signature.split_bytes()
                        }
                        Some("Secp256r1") => {
                            let Ok(signing_key) = p256::ecdsa::SigningKey::from_slice(&secret_key)
                            else {
                                return Ok(handle_cheatcode_error("invalid secret_key"));
                            };

                            let signature: p256::ecdsa::Signature =
                                p256::ecdsa::signature::hazmat::PrehashSigner::sign_prehash(
                                    &signing_key,
                                    &msg_hash,
                                )
                                .unwrap();

                            signature.split_bytes()
                        }
                        _ => return Ok(CheatcodeHandlingResult::Forwarded),
                    }
                };

                Ok(CheatcodeHandlingResult::Handled(vec![
                    Felt252::from(0),
                    Felt252::from_bytes_be(&r_bytes[16..32]), // 16 low  bytes of r
                    Felt252::from_bytes_be(&r_bytes[0..16]),  // 16 high bytes of r
                    Felt252::from_bytes_be(&s_bytes[16..32]), // 16 low  bytes of s
                    Felt252::from_bytes_be(&s_bytes[0..16]),  // 16 high bytes of s
                ]))
            }
            "get_call_trace" => {
                let call_trace = &extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .trace_data
                    .current_call_stack
                    .borrow_full_trace();

                let mut output = vec![];
                serialize_call_trace(call_trace, &mut output);

                Ok(CheatcodeHandlingResult::Handled(output))
            }
            "store" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;
                let target = ContractAddress::from_(input_reader.read_felt());
                let storage_address = input_reader.read_felt();
                store(*state, target, &storage_address, input_reader.read_felt())
                    .expect("Failed to store");
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "load" => {
                let state = &mut extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler
                    .state;
                let target = ContractAddress::from_(input_reader.read_felt());
                let storage_address = &input_reader.read_felt();
                let loaded = load(*state, target, storage_address).expect("Failed to load");
                Ok(CheatcodeHandlingResult::Handled(vec![loaded]))
            }
            "map_entry_address" => {
                let map_selector = &input_reader.read_felt();
                let keys = &input_reader.read_vec();
                let map_entry_address = calculate_variable_address(map_selector, Some(keys));
                Ok(CheatcodeHandlingResult::Handled(vec![map_entry_address]))
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

fn handle_deploy_result(
    deploy_result: Result<ContractAddress, CheatcodeError>,
) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
    match deploy_result {
        Ok(contract_address) => {
            let felt_contract_address = contract_address.into_();
            let result = vec![Felt252::from(0), felt_contract_address];
            Ok(CheatcodeHandlingResult::Handled(result))
        }
        Err(CheatcodeError::Recoverable(panic_data)) => Ok(CheatcodeHandlingResult::Handled(
            cheatcode_panic_result(panic_data),
        )),
        Err(CheatcodeError::Unrecoverable(err)) => Err(err),
    }
}

// append all to one output Vec instead of allocating new one for each nested call
fn serialize_call_trace(call_trace: &CallTrace, output: &mut Vec<Felt252>) {
    serialize_call_entry_point(&call_trace.entry_point, output);

    output.push(Felt252::from(call_trace.nested_calls.len()));

    for call_trace in &call_trace.nested_calls {
        serialize_call_trace(&call_trace.borrow(), output);
    }

    serialize_call_result(&call_trace.result, output);
}

fn serialize_call_entry_point(call_entry_point: &CallEntryPoint, output: &mut Vec<Felt252>) {
    let entry_point_type = match call_entry_point.entry_point_type {
        EntryPointType::Constructor => 0,
        EntryPointType::External => 1,
        EntryPointType::L1Handler => 2,
    };

    let calldata = call_entry_point
        .calldata
        .0
        .iter()
        .copied()
        .map(IntoConv::into_)
        .collect::<Vec<_>>();

    let call_type = match call_entry_point.call_type {
        CallType::Call => 0,
        CallType::Delegate => 1,
    };

    output.push(Felt252::from(entry_point_type));
    output.push(call_entry_point.entry_point_selector.0.into_());
    output.push(Felt252::from(calldata.len()));
    output.extend(calldata);
    output.push(call_entry_point.storage_address.into_());
    output.push(call_entry_point.caller_address.into_());
    output.push(Felt252::from(call_type));
}

fn serialize_call_result(call_result: &CallResult, output: &mut Vec<Felt252>) {
    match call_result {
        CallResult::Success { ret_data } => {
            output.push(Felt252::from(0));
            output.push(Felt252::from(ret_data.len()));
            output.extend(ret_data.iter().cloned());
        }
        CallResult::Failure(call_failure) => {
            match call_failure {
                CallFailure::Panic { panic_data } => {
                    serialize_failure_data(0, panic_data.iter().cloned(), panic_data.len(), output);
                }
                CallFailure::Error { msg } => {
                    let data = ByteArray::from(msg.as_str()).serialize_with_magic();
                    let len = data.len();
                    serialize_failure_data(1, data, len, output);
                }
            };
        }
    };
}

#[inline]
fn serialize_failure_data(
    call_failure_variant: u8,
    failure_data: impl IntoIterator<Item = Felt252>,
    failure_data_len: usize,
    output: &mut Vec<Felt252>,
) {
    output.push(Felt252::from(1));
    output.push(Felt252::from(call_failure_variant));
    output.push(Felt252::from(failure_data_len));
    output.extend(failure_data);
}

#[must_use]
pub fn cheatcode_panic_result(panic_data: Vec<Felt252>) -> Vec<Felt252> {
    let mut result = Vec::with_capacity(panic_data.len() + 2);

    result.push(Felt252::from(1));
    result.push(Felt252::from(panic_data.len()));

    result.extend(panic_data);
    result
}

fn handle_cheatcode_error(error_short_string: &str) -> CheatcodeHandlingResult {
    CheatcodeHandlingResult::Handled(vec![
        Felt252::from(1),
        Felt252::from_short_string(error_short_string)
            .expect("Should convert shortstring to Felt252"),
    ])
}

fn concat_u128_bytes(low: &[u8; 32], high: &[u8; 32]) -> [u8; 32] {
    let mut result = [0; 32];
    // the values of u128 are contained in the last 16 bytes
    result[..16].copy_from_slice(&high[16..32]);
    result[16..].copy_from_slice(&low[16..32]);
    result
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
    top_call.borrow_mut().used_execution_resources = all_execution_resources;
}

pub fn update_top_call_l1_resources(runtime: &mut ForgeRuntime) {
    let l2_l1_message_sizes = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .l2_to_l1_messages
        .iter()
        .map(|ordered_message| ordered_message.message.payload.0.len())
        .collect();

    let all_storage_writes = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .state
        .to_state_diff()
        .storage_updates
        .iter()
        .map(|(_, entry)| entry.len())
        .sum();

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

    top_call.used_l1_resources.l2_l1_message_sizes = l2_l1_message_sizes;
    top_call.used_l1_resources.storage_writes = all_storage_writes;
}

#[must_use]
pub fn get_all_used_resources(runtime: ForgeRuntime) -> UsedResources {
    let starknet_runtime = runtime.extended_runtime.extended_runtime.extended_runtime;
    let top_call_l2_to_l1_messages = starknet_runtime.hint_handler.l2_to_l1_messages;

    // used just to obtain payloads of L2 -> L1 messages
    let runtime_call_info = CallInfo {
        execution: CallExecution {
            l2_to_l1_messages: top_call_l2_to_l1_messages,
            ..Default::default()
        },
        inner_calls: starknet_runtime.hint_handler.inner_calls,
        ..Default::default()
    };
    let l2_to_l1_payloads_length = runtime_call_info
        .get_sorted_l2_to_l1_payloads_length()
        .unwrap();

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

    UsedResources {
        execution_resources,
        l2_to_l1_payloads_length,
    }
}
