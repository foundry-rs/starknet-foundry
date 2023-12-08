use std::collections::HashMap;
use std::path::PathBuf;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallContractFailure, CallContractResult,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::{
    deploy, deploy_at, DeployCallPayload,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use crate::state::{BlockifierState, CheatTarget};
use anyhow::{Context, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use conversions::felt252::FromShortString;
use conversions::{FromConv, IntoConv};
use num_traits::{One, ToPrimitive};
use scarb_artifacts::StarknetContractArtifacts;
use serde::Deserialize;

use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet_api::core::ContractAddress;

use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::SpyTarget;
use crate::runtime_extensions::forge_runtime_extension::file_operations::string_into_felt;
use cairo_lang_starknet::contract::starknet_keccak;
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
        let res = match selector {
            "start_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let block_number = inputs.last().unwrap().clone();
                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_roll(target, block_number);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_roll(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_warp" => {
                // The last element in `inputs` should be the timestamp in all cases
                let warp_timestamp = inputs.last().unwrap().clone();

                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_warp(target, warp_timestamp);

                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_warp" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_warp(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_elect" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let sequencer_address = inputs.last().unwrap().clone().into_();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_elect(target, sequencer_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_elect" => {
                let (target, _) = deserialize_cheat_target(&inputs);
                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_elect(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                // The last element in `inputs` should be the contract address in all cases
                let caller_address = inputs.last().unwrap().clone().into_();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_prank(target, caller_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_prank(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_mock_call" => {
                let contract_address = inputs[0].clone().into_();
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

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_mock_call(contract_address, &function_name, &ret_data);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_mock_call" => {
                let contract_address = inputs[0].clone().into_();
                let function_name = inputs[1].clone();

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_mock_call(contract_address, &function_name);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_spoof" => {
                let (target, inputs_start) = deserialize_cheat_target(&inputs);
                let mut idx = inputs_start;

                let version = read_option_felt(&inputs, &mut idx);
                let account_contract_address = read_option_felt(&inputs, &mut idx);
                let max_fee = read_option_felt(&inputs, &mut idx);
                let signature = read_option_vec(&inputs, &mut idx);
                let transaction_hash = read_option_felt(&inputs, &mut idx);
                let chain_id = read_option_felt(&inputs, &mut idx);
                let nonce = read_option_felt(&inputs, &mut idx);

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .start_spoof(
                        target,
                        version,
                        account_contract_address,
                        max_fee,
                        signature,
                        transaction_hash,
                        chain_id,
                        nonce,
                    );
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_spoof" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .extension
                    .cheatnet_state
                    .stop_spoof(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "declare" => {
                let contract_name = inputs[0].clone();
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
                let class_hash = inputs[0].clone().into_();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
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
                let class_hash = inputs[0].clone().into_();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
                let contract_address = inputs[2 + calldata_length].clone().into_();
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
                let class_hash = inputs[0].clone().into_();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

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
                let name = inputs[0].clone();
                let name = as_cairo_short_string(&name).unwrap_or_else(|| {
                    panic!("Failed to parse var argument = {name} as short string")
                });

                let env_var = self
                    .environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = string_into_felt(env_var)
                    .with_context(|| format!("Failed to parse value = {env_var} to felt"))?;

                Ok(CheatcodeHandlingResult::Handled(vec![parsed_env_var]))
            }
            "get_class_hash" => {
                let contract_address = inputs[0].clone().into_();

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
                let contract_address = inputs[0].clone().into_();
                let function_name = inputs[1].clone();
                let from_address = inputs[2].clone();

                let payload = Vec::from(&inputs[4..inputs.len()]);

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
                let file_path = inputs[0].clone();
                let parsed_content = file_operations::read_txt(&file_path)?;
                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "read_json" => {
                let file_path = inputs[0].clone();
                let parsed_content = file_operations::read_json(&file_path)?;

                Ok(CheatcodeHandlingResult::Handled(parsed_content))
            }
            "spy_events" => {
                let spy_on = match inputs.len() {
                    0 => unreachable!("Serialized enum should always be longer than 0"),
                    1 => SpyTarget::All,
                    2 => SpyTarget::One(inputs[1].clone().into_()),
                    _ => {
                        let addresses_length = inputs[1].to_usize().unwrap();
                        let addresses = Vec::from(&inputs[2..(2 + addresses_length)])
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
                let id = &inputs[0];
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
                let name = inputs[0].clone();
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
                let private_key = inputs[0].clone();
                let key_pair = SigningKey::from_secret_scalar(private_key.into_());

                Ok(CheatcodeHandlingResult::Handled(vec![key_pair
                    .verifying_key()
                    .scalar()
                    .into_()]))
            }
            "ecdsa_sign_message" => {
                let private_key = inputs[0].clone();
                let message_hash = inputs[1].clone();

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
// Returns the tuple (target, n read elements)
fn deserialize_cheat_target(inputs: &[Felt252]) -> (CheatTarget, usize) {
    // First element encodes the variant of CheatTarget
    match inputs[0].to_u8() {
        Some(0) => (CheatTarget::All, 1),
        Some(1) => (CheatTarget::One(inputs[1].clone().into_()), 2),
        Some(2) => {
            let n_targets = inputs[1].to_usize().unwrap();
            let contract_addresses: Vec<_> = inputs[2..2 + n_targets]
                .iter()
                .map(|el| ContractAddress::from_(el.clone()))
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

fn cheatcode_panic_result(panic_data: Vec<Felt252>) -> Vec<Felt252> {
    let mut result = vec![Felt252::from(1), Felt252::from(panic_data.len())];
    result.extend(panic_data);
    result
}

fn read_felt(buffer: &[Felt252], idx: &mut usize) -> Felt252 {
    *idx += 1;
    buffer[*idx - 1].clone()
}

fn read_vec(buffer: &[Felt252], idx: &mut usize, count: usize) -> Vec<Felt252> {
    *idx += count;
    buffer[*idx - count..*idx].to_vec()
}

fn read_option_felt(buffer: &[Felt252], idx: &mut usize) -> Option<Felt252> {
    *idx += 1;
    (!buffer[*idx - 1].is_one()).then(|| read_felt(buffer, idx))
}

fn read_option_vec(buffer: &[Felt252], idx: &mut usize) -> Option<Vec<Felt252>> {
    read_option_felt(buffer, idx).map(|count| read_vec(buffer, idx, count.to_usize().unwrap()))
}
