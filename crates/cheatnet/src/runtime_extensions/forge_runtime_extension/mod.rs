use std::collections::HashMap;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallContractFailure, CallContractResult, UsedResources,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::{
    deploy, deploy_at, DeployCallPayload,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use crate::state::{BlockifierState, CheatTarget};
use anyhow::{Context, Result};
use blockifier::execution::call_info::{CallExecution, CallInfo};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
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

use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::SpyTarget;
use crate::runtime_extensions::forge_runtime_extension::file_operations::string_into_felt;
use cairo_lang_starknet::contract::starknet_keccak;
use runtime::utils::BufferReader;
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic,
    SyscallHandlingResult,
};
use starknet::signers::SigningKey;

use super::call_to_blockifier_runtime_extension::CallToBlockifierRuntime;
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
        inputs: Vec<Felt252>,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        let mut reader = BufferReader::new(&inputs);

        let res = match selector {
            "start_roll" => {
                let target = reader.read_cheat_target();
                let block_number = reader.read_felt();
                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_roll(target, block_number);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_roll" => {
                let target = reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_roll(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_warp" => {
                let target = reader.read_cheat_target();
                let warp_timestamp = reader.read_felt();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_warp(target, warp_timestamp);

                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_warp" => {
                let target = reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_warp(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_elect" => {
                let target = reader.read_cheat_target();
                let sequencer_address = reader.read_felt().into_();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_elect(target, sequencer_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_elect" => {
                let target = reader.read_cheat_target();
                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_elect(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_prank" => {
                let target = reader.read_cheat_target();

                let caller_address = reader.read_felt().into_();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_prank(target, caller_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_prank" => {
                let target = reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_prank(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_mock_call" => {
                let contract_address = reader.read_felt().into_();
                let function_name = reader.read_felt();

                let ret_data = reader.read_vec();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_mock_call(contract_address, &function_name, &ret_data);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_mock_call" => {
                let contract_address = reader.read_felt().into_();
                let function_name = reader.read_felt();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_mock_call(contract_address, &function_name);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_spoof" => {
                let target = reader.read_cheat_target();

                let version = reader.read_option_felt();
                let account_contract_address = reader.read_option_felt();
                let max_fee = reader.read_option_felt();
                let signature = reader.read_option_vec();
                let transaction_hash = reader.read_option_felt();
                let chain_id = reader.read_option_felt();
                let nonce = reader.read_option_felt();
                let resource_bounds = reader.read_option_felt().map(|resource_bounds_len| {
                    reader.read_vec_body(
                        3 * resource_bounds_len.to_usize().unwrap(), // ResourceBounds struct has 3 fields
                    )
                });
                let tip = reader.read_option_felt();
                let paymaster_data = reader.read_option_vec();
                let nonce_data_availability_mode = reader.read_option_felt();
                let fee_data_availability_mode = reader.read_option_felt();
                let account_deployment_data = reader.read_option_vec();

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
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_spoof(target, tx_info_mock);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_spoof" => {
                let target = reader.read_cheat_target();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_spoof(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "declare" => {
                let contract_name = reader.read_felt();
                let contracts = self.contracts;
                let mut blockifier_state = BlockifierState::from(
                    extended_runtime
                        .extended_runtime
                        .extended_runtime
                        .extended_runtime
                        .hint_handler
                        .state,
                );
                match blockifier_state.declare(&contract_name, contracts) {
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
                let class_hash = reader.read_felt().into_();
                let calldata = reader.read_vec();
                let cheatable_starknet_runtime = &mut extended_runtime.extended_runtime;
                let mut blockifier_state = BlockifierState::from(
                    cheatable_starknet_runtime
                        .extended_runtime
                        .extended_runtime
                        .hint_handler
                        .state,
                );

                handle_deploy_result(deploy(
                    &mut blockifier_state,
                    cheatable_starknet_runtime
                        .extended_runtime
                        .extension
                        .cheatnet_state,
                    &class_hash,
                    &calldata,
                ))
            }
            "deploy_at" => {
                let class_hash = reader.read_felt().into_();
                let calldata = reader.read_vec();
                let contract_address = reader.read_felt().into_();
                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let mut blockifier_state = BlockifierState::from(
                    cheatnet_runtime
                        .extended_runtime
                        .extended_runtime
                        .hint_handler
                        .state,
                );

                handle_deploy_result(deploy_at(
                    &mut blockifier_state,
                    cheatnet_runtime.extended_runtime.extension.cheatnet_state,
                    &class_hash,
                    &calldata,
                    contract_address,
                ))
            }
            "precalculate_address" => {
                let class_hash = reader.read_felt().into_();
                let calldata = reader.read_vec();

                let contract_address = extended_runtime
                    .extended_runtime
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
                let name = reader
                    .read_short_string()
                    .unwrap_or_else(|| panic!("Failed to parse var argument as short string"));

                let env_var = self
                    .environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = string_into_felt(env_var)
                    .with_context(|| format!("Failed to parse value = {env_var} to felt"))?;

                Ok(CheatcodeHandlingResult::Handled(vec![parsed_env_var]))
            }
            "get_class_hash" => {
                let contract_address = reader.read_felt().into_();

                let mut blockifier_state = BlockifierState::from(
                    extended_runtime
                        .extended_runtime
                        .extended_runtime
                        .extended_runtime
                        .hint_handler
                        .state,
                );

                match blockifier_state.get_class_hash(contract_address) {
                    Ok(class_hash) => {
                        let felt_class_hash = stark_felt_to_felt(class_hash.0);

                        Ok(CheatcodeHandlingResult::Handled(vec![felt_class_hash]))
                    }
                    Err(CheatcodeError::Recoverable(_)) => unreachable!(),
                    Err(CheatcodeError::Unrecoverable(err)) => Err(err),
                }
            }
            "l1_handler_execute" => {
                let contract_address = reader.read_felt().into_();
                let function_name = reader.read_felt();
                let from_address = reader.read_felt();

                let payload = reader.read_vec();

                let cheatnet_runtime = &mut extended_runtime.extended_runtime;
                let mut blockifier_state = BlockifierState::from(
                    cheatnet_runtime
                        .extended_runtime
                        .extended_runtime
                        .hint_handler
                        .state,
                );

                match blockifier_state
                    .l1_handler_execute(
                        cheatnet_runtime.extended_runtime.extension.cheatnet_state,
                        contract_address,
                        &function_name,
                        &from_address,
                        &payload,
                    )
                    .result
                {
                    CallContractResult::Success { .. } => {
                        Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(0)]))
                    }
                    CallContractResult::Failure(CallContractFailure::Panic { panic_data }) => Ok(
                        CheatcodeHandlingResult::Handled(cheatcode_panic_result(panic_data)),
                    ),
                    CallContractResult::Failure(CallContractFailure::Error { msg }) => Err(
                        EnhancedHintError::from(HintError::CustomHint(Box::from(msg))),
                    ),
                }
            }
            "read_txt" => {
                let file_path = reader.read_felt();
                let parsed_content = file_operations::read_txt(&file_path)?;
                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "read_json" => {
                let file_path = reader.read_felt();
                let parsed_content = file_operations::read_json(&file_path)?;

                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "spy_events" => {
                let spy_target_variant = reader
                    .read_felt()
                    .to_u8()
                    .expect("Invalid spy_target length");
                let spy_on = match spy_target_variant {
                    0 => SpyTarget::All,
                    1 => SpyTarget::One(reader.read_felt().into_()),
                    _ => {
                        let addresses = reader
                            .read_vec()
                            .iter()
                            .map(|el| ContractAddress::from_(el.clone()))
                            .collect();

                        SpyTarget::Multiple(addresses)
                    }
                };

                let id = extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .spy_events(spy_on);
                Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(id)]))
            }
            "fetch_events" => {
                let id = &reader.read_felt();
                let (emitted_events_len, serialized_events) = extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .fetch_events(id);
                let mut result = vec![Felt252::from(emitted_events_len)];
                result.extend(serialized_events);
                Ok(CheatcodeHandlingResult::Handled(result))
            }
            "event_name_hash" => {
                let name = reader.read_felt();
                let hash = starknet_keccak(as_cairo_short_string(&name).unwrap().as_bytes());

                Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(hash)]))
            }
            "generate_ecdsa_keys" => {
                let key_pair = SigningKey::from_random();

                Ok(CheatcodeHandlingResult::Handled(vec![
                    key_pair.secret_scalar().into_(),
                    key_pair.verifying_key().scalar().into_(),
                ]))
            }
            "get_public_key" => {
                let private_key = reader.read_felt();
                let key_pair = SigningKey::from_secret_scalar(private_key.into_());

                Ok(CheatcodeHandlingResult::Handled(vec![key_pair
                    .verifying_key()
                    .scalar()
                    .into_()]))
            }
            "ecdsa_sign_message" => {
                let private_key = reader.read_felt();
                let message_hash = reader.read_felt();

                let key_pair = SigningKey::from_secret_scalar(private_key.into_());

                if let Ok(signature) = key_pair.sign(&message_hash.into_()) {
                    Ok(CheatcodeHandlingResult::Handled(vec![
                        Felt252::from(0),
                        Felt252::from_(signature.r),
                        Felt252::from_(signature.s),
                    ]))
                } else {
                    Ok(CheatcodeHandlingResult::Handled(vec![
                        Felt252::from(1),
                        Felt252::from_short_string("message_hash out of range").unwrap(),
                    ]))
                }
            }
            _ => Ok(CheatcodeHandlingResult::Forwarded),
        }?;

        Ok(res)
    }

    fn override_system_call(
        &mut self,
        selector: SyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        match selector {
            DeprecatedSyscallSelector::ReplaceClass => Err(HintError::CustomHint(Box::from(
                "Replace class can't be used in tests".to_string(),
            ))),
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }
}

fn handle_deploy_result(
    deploy_result: Result<DeployCallPayload, CheatcodeError>,
) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
    match deploy_result {
        Ok(deploy_payload) => {
            let felt_contract_address: Felt252 = deploy_payload.contract_address.into_();
            let result = vec![Felt252::from(0), felt_contract_address];
            Ok(CheatcodeHandlingResult::Handled(result))
        }
        Err(CheatcodeError::Recoverable(panic_data)) => Ok(CheatcodeHandlingResult::Handled(
            cheatcode_panic_result(panic_data),
        )),
        Err(CheatcodeError::Unrecoverable(err)) => Err(err),
    }
}

#[must_use]
pub fn cheatcode_panic_result(panic_data: Vec<Felt252>) -> Vec<Felt252> {
    let mut result = vec![Felt252::from(1), Felt252::from(panic_data.len())];
    result.extend(panic_data);
    result
}

#[must_use]
pub fn get_all_execution_resources(runtime: ForgeRuntime) -> UsedResources {
    let runtime_execution_resources = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .resources
        .clone();
    let runtime_l1_to_l2_messages = runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .l2_to_l1_messages;

    let runtime_call_info = CallInfo {
        execution: CallExecution {
            l2_to_l1_messages: runtime_l1_to_l2_messages,
            ..Default::default()
        },
        ..Default::default()
    };
    let runtime_l2_to_l1_payloads_length = runtime_call_info
        .get_sorted_l2_to_l1_payloads_length()
        .unwrap();

    let mut all_resources = UsedResources {
        execution_resources: runtime_execution_resources,
        l2_to_l1_payloads_length: runtime_l2_to_l1_payloads_length,
    };

    let cheatnet_used_resources = &runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .used_resources;
    all_resources.extend(cheatnet_used_resources);

    all_resources
}
