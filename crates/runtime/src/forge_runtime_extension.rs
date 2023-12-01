use std::collections::HashMap;
use std::convert::Into;
use std::path::PathBuf;

use anyhow::{Context, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::execution_utils::{
    stark_felt_from_ptr, stark_felt_to_felt, ReadOnlySegment,
};
use blockifier::execution::syscalls::hint_processor::{read_felt_array, SyscallExecutionError};
use blockifier::execution::syscalls::{
    SyscallRequest, SyscallResponse, SyscallResponseWrapper, SyscallResult,
};
use cairo_felt::Felt252;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatnet::cheatcodes::deploy::{deploy, deploy_at, DeployCallPayload};
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::execution::cheatable_syscall_handler::{CheatableSyscallHandler, SyscallSelector};
use cheatnet::rpc::{call_contract, CallContractFailure, CallContractOutput, CallContractResult};
use cheatnet::state::{BlockifierState, CheatTarget, CheatnetState};
use conversions::{FromConv, IntoConv};
use num_traits::{One, ToPrimitive};
use scarb_artifacts::StarknetContractArtifacts;
use serde::Deserialize;

use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet_api::core::ContractAddress;
use starknet_api::hash::StarkFelt;

use crate::forge_runtime_extension::file_operations::string_into_felt;
use crate::{
    CheatcodeHandlingResult, ExtendedRuntime, ExtensionLogic, RuntimeExtension,
    SyscallHandlingResult,
};
use cairo_lang_starknet::contract::starknet_keccak;
use cairo_vm::vm::errors::hint_errors::HintError::CustomHint;
use cheatnet::cheatcodes::spy_events::SpyTarget;
use cheatnet::execution::cheated_syscalls::SingleSegmentResponse;
use cheatnet::execution::contract_execution_syscall_handler::{
    print, ContractExecutionSyscallHandler,
};
use starknet::signers::SigningKey;

mod file_operations;

pub type ForgeRuntime<'a> =
    ExtendedRuntime<RuntimeExtension<TestExecutionState<'a>, ContractExecutionSyscallHandler<'a>>>;

pub struct TestExecutionState<'a> {
    pub environment_variables: &'a HashMap<String, String>,
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
}

// This runtime extension provides an implementation logic for functions from snforge_std library.
impl<'a> ExtensionLogic
    for RuntimeExtension<TestExecutionState<'a>, ContractExecutionSyscallHandler<'a>>
{
    type Runtime = ContractExecutionSyscallHandler<'a>;

    fn get_extended_runtime_mut(&mut self) -> &mut ContractExecutionSyscallHandler<'a> {
        &mut self.extended_runtime
    }

    fn get_extended_runtime(&self) -> &ContractExecutionSyscallHandler<'a> {
        &self.extended_runtime
    }

    #[allow(clippy::too_many_lines)]
    fn handle_cheatcode(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        let res = match selector {
            "start_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let block_number = inputs.last().unwrap().clone();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_roll(target, block_number);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_roll" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime.child.cheatnet_state.stop_roll(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_warp" => {
                // The last element in `inputs` should be the timestamp in all cases
                let warp_timestamp = inputs.last().unwrap().clone();

                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_warp(target, warp_timestamp);

                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_warp" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime.child.cheatnet_state.stop_warp(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_elect" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);
                let sequencer_address = inputs.last().unwrap().clone().into_();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_elect(target, sequencer_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_elect" => {
                let (target, _) = deserialize_cheat_target(&inputs);
                self.extended_runtime
                    .child
                    .cheatnet_state
                    .stop_elect(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "start_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs[..inputs.len() - 1]);

                // The last element in `inputs` should be the contract address in all cases
                let caller_address = inputs.last().unwrap().clone().into_();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .start_prank(target, caller_address);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_prank" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime
                    .child
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

                self.extended_runtime.child.cheatnet_state.start_mock_call(
                    contract_address,
                    &function_name,
                    &ret_data,
                );
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_mock_call" => {
                let contract_address = inputs[0].clone().into_();
                let function_name = inputs[1].clone();

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .stop_mock_call(contract_address, &function_name);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
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
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "stop_spoof" => {
                let (target, _) = deserialize_cheat_target(&inputs);

                self.extended_runtime
                    .child
                    .cheatnet_state
                    .stop_spoof(target);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "declare" => {
                let contract_name = inputs[0].clone();
                let mut blockifier_state =
                    BlockifierState::from(self.extended_runtime.child.child.state);

                match blockifier_state.declare(&contract_name, self.extension_state.contracts) {
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
                let mut blockifier_state =
                    BlockifierState::from(self.extended_runtime.child.child.state);

                handle_deploy_result(deploy(
                    &mut blockifier_state,
                    self.extended_runtime.child.cheatnet_state,
                    &class_hash,
                    &calldata,
                ))
            }
            "deploy_at" => {
                let class_hash = inputs[0].clone().into_();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
                let contract_address = inputs[2 + calldata_length].clone().into_();

                let mut blockifier_state =
                    BlockifierState::from(self.extended_runtime.child.child.state);

                handle_deploy_result(deploy_at(
                    &mut blockifier_state,
                    self.extended_runtime.child.cheatnet_state,
                    &class_hash,
                    &calldata,
                    contract_address,
                ))
            }
            "print" => {
                print(inputs);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            "precalculate_address" => {
                let class_hash = inputs[0].clone().into_();
                let calldata_length = inputs[1].to_usize().unwrap();
                let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);

                let contract_address = self
                    .extended_runtime
                    .child
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
                    .extension_state
                    .environment_variables
                    .get(&name)
                    .with_context(|| format!("Failed to read from env var = {name}"))?;

                let parsed_env_var = string_into_felt(env_var)
                    .with_context(|| format!("Failed to parse value = {env_var} to felt"))?;

                Ok(CheatcodeHandlingResult::Handled(vec![parsed_env_var]))
            }
            "get_class_hash" => {
                let contract_address = inputs[0].clone().into_();

                let mut blockifier_state =
                    BlockifierState::from(self.extended_runtime.child.child.state);

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

                let mut blockifier_state =
                    BlockifierState::from(self.extended_runtime.child.child.state);

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

                let id = self
                    .extended_runtime
                    .child
                    .cheatnet_state
                    .spy_events(spy_on);
                Ok(CheatcodeHandlingResult::Handled(vec![Felt252::from(id)]))
            }
            "fetch_events" => {
                let id = &inputs[0];
                let (emitted_events_len, serialized_events) =
                    self.extended_runtime.child.cheatnet_state.fetch_events(id);
                let mut result = vec![Felt252::from(emitted_events_len)];
                result.extend(serialized_events);
                Ok(CheatcodeHandlingResult::Handled(result))
            }
            "event_name_hash" => {
                let name = inputs[0].clone();
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
                        Felt252::from_("message_hash out of range".to_string()),
                    ]))
                }
            }
            "get_stark_public_key" => {
                let private_key = inputs[0].clone();
                let key_pair = SigningKey::from_secret_scalar(private_key.into_());

                Ok(CheatcodeHandlingResult::Handled(vec![key_pair
                    .verifying_key()
                    .scalar()
                    .into_()]))
            }
            "generate_ecdsa_keys" => {
                let curve = as_cairo_short_string(&inputs[0]);

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
                    Felt252::from_bytes_be(&signing_key_bytes[16..32]),
                    Felt252::from_bytes_be(&signing_key_bytes[0..16]),
                    Felt252::from_bytes_be(&verifying_key_bytes[17..33]),
                    Felt252::from_bytes_be(&verifying_key_bytes[1..17]),
                    Felt252::from_bytes_be(&verifying_key_bytes[49..65]),
                    Felt252::from_bytes_be(&verifying_key_bytes[33..49]),
                ]))
            }
            "ecdsa_sign_message" => {
                let curve = as_cairo_short_string(&inputs[0]);
                let secret_key_low = inputs[1].clone();
                let secret_key_high = inputs[2].clone();
                let msg_hash_low = inputs[3].clone();
                let msg_hash_high = inputs[4].clone();

                let secret_key = concat_felts(&secret_key_low, &secret_key_high);
                let msg_hash = concat_felts(&msg_hash_low, &msg_hash_high);

                let (r_bytes, s_bytes) = {
                    match curve.as_deref() {
                        Some("Secp256k1") => {
                            let signing_key =
                                k256::ecdsa::SigningKey::from_slice(&secret_key).unwrap();

                            let signature: k256::ecdsa::Signature =
                                k256::ecdsa::signature::hazmat::PrehashSigner::sign_prehash(
                                    &signing_key,
                                    &msg_hash,
                                )
                                .unwrap();

                            signature.split_bytes()
                        }
                        Some("Secp256r1") => {
                            let signing_key =
                                p256::ecdsa::SigningKey::from_slice(&secret_key).unwrap();

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
                    Felt252::from_bytes_be(&r_bytes[16..32]),
                    Felt252::from_bytes_be(&r_bytes[0..16]),
                    Felt252::from_bytes_be(&s_bytes[16..32]),
                    Felt252::from_bytes_be(&s_bytes[0..16]),
                ]))
            }
            "get_ecdsa_public_key" => {
                let curve = as_cairo_short_string(&inputs[0]);
                let secret_key_low = inputs[1].clone();
                let secret_key_high = inputs[2].clone();

                let secret_key = concat_felts(&secret_key_low, &secret_key_high);

                let verifying_key_bytes = {
                    match curve.as_deref() {
                        Some("Secp256k1") => {
                            let signing_key =
                                k256::ecdsa::SigningKey::from_slice(&secret_key).unwrap();

                            signing_key
                                .verifying_key()
                                .to_encoded_point(false)
                                .to_bytes()
                        }
                        Some("Secp256r1") => {
                            let signing_key =
                                p256::ecdsa::SigningKey::from_slice(&secret_key).unwrap();

                            signing_key
                                .verifying_key()
                                .to_encoded_point(false)
                                .to_bytes()
                        }
                        _ => return Ok(CheatcodeHandlingResult::Forwarded),
                    }
                };

                Ok(CheatcodeHandlingResult::Handled(vec![
                    Felt252::from_bytes_be(&verifying_key_bytes[17..33]),
                    Felt252::from_bytes_be(&verifying_key_bytes[1..17]),
                    Felt252::from_bytes_be(&verifying_key_bytes[49..65]),
                    Felt252::from_bytes_be(&verifying_key_bytes[33..49]),
                ]))
            }
            _ => Ok(CheatcodeHandlingResult::Forwarded),
        }?;

        Ok(res)
    }

    fn override_system_call(
        &mut self,
        selector: SyscallSelector,
        vm: &mut VirtualMachine,
    ) -> Result<SyscallHandlingResult, HintError> {
        match selector {
            DeprecatedSyscallSelector::CallContract => {
                let call_args = CallContractArgs::read(
                    vm,
                    &mut self.get_extended_runtime_mut().child.child.syscall_ptr,
                )?;
                let cheatable_syscall_handler = &mut self.get_extended_runtime_mut().child;
                let mut blockifier_state =
                    BlockifierState::from(cheatable_syscall_handler.child.state);
                let cheatnet_state: &mut _ = cheatable_syscall_handler.cheatnet_state;

                let call_result =
                    execute_call_contract(&mut blockifier_state, cheatnet_state, &call_args);
                write_call_contract_response(
                    &mut self.get_extended_runtime_mut().child,
                    vm,
                    &call_args,
                    call_result,
                )?;
                Ok(SyscallHandlingResult::Handled(()))
            }
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

struct CallContractArgs {
    _selector: Felt252,
    gas_counter: u64,
    contract_address: ContractAddress,
    entry_point_selector: Felt252,
    calldata: Vec<Felt252>,
}

impl SyscallRequest for CallContractArgs {
    fn read(vm: &VirtualMachine, ptr: &mut Relocatable) -> SyscallResult<CallContractArgs> {
        let selector = stark_felt_from_ptr(vm, ptr)?.into_();
        let gas_counter = Felt252::from_(stark_felt_from_ptr(vm, ptr)?)
            .to_u64()
            .unwrap();

        let contract_address = stark_felt_from_ptr(vm, ptr)?.into_();
        let entry_point_selector = stark_felt_from_ptr(vm, ptr)?.into_();

        let calldata = read_felt_array::<SyscallExecutionError>(vm, ptr)?
            .iter()
            .map(|el| (*el).into_())
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

            // add execution resources used by call to all used resources
            cheatable_syscall_handler
                .cheatnet_state
                .used_resources
                .vm_resources += &call_output.used_resources.vm_resources;
            cheatable_syscall_handler
                .cheatnet_state
                .used_resources
                .syscall_counter
                .extend(call_output.used_resources.syscall_counter);

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
                    .map(|el| StarkFelt::from_(el.clone()))
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

fn cheatcode_panic_result(panic_data: Vec<Felt252>) -> Vec<Felt252> {
    let mut result = vec![Felt252::from(1), Felt252::from(panic_data.len())];
    result.extend(panic_data);
    result
}

fn concat_felts(low: &Felt252, high: &Felt252) -> [u8; 32] {
    let mut result = [0; 32];
    result[..16].copy_from_slice(&high.to_be_bytes()[16..32]);
    result[16..].copy_from_slice(&low.to_be_bytes()[16..32]);
    result
}
