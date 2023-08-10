use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::scarb::StarknetContractArtifacts;
use anyhow::{anyhow, Result};
use blockifier::state::cached_state::CachedState;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessorLogic;
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatable_starknet::rpc::{call_contract, CallContractOutput};
use cheatable_starknet::state::DictStateReader;
use cheatable_starknet::{
    cheatcodes::{ContractArtifacts, EnhancedHintError},
    CheatedState,
};
use serde::Deserialize;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;

use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{
    casm_run::{cell_ref_to_relocatable, extract_buffer, get_ptr},
    insert_value_to_cellref, CairoHintProcessor as OriginalCairoHintProcessor,
};
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};

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
    pub blockifier_state: CachedState<DictStateReader>,
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
    pub cheated_state: CheatedState,
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
            return execute_syscall(system, vm, &mut self.blockifier_state, &self.cheated_state);
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
                self.cheated_state.start_roll(contract_address, value)
            }
            "stop_roll" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.cheated_state.stop_roll(contract_address)
            }
            "start_warp" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);
                let value = inputs[1].clone();
                self.cheated_state.start_warp(contract_address, value)
            }
            "stop_warp" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.cheated_state.stop_warp(contract_address)
            }
            "start_prank" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                let caller_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[1].clone().to_be_bytes(),
                )?)?);

                self.cheated_state
                    .start_prank(contract_address, caller_address)
            }
            "stop_prank" => {
                let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
                    inputs[0].clone().to_be_bytes(),
                )?)?);

                self.cheated_state.stop_prank(contract_address)
            }
            "mock_call" => todo!(),
            "declare" => self.cheated_state.declare(
                &mut buffer,
                &mut self.blockifier_state,
                &inputs,
                // TODO(#41) Remove after we have a separate scarb package
                &contracts
                    .iter()
                    .map(|(k, v)| (k.clone(), ContractArtifacts::from(v)))
                    .collect(),
            ),
            "deploy" => self
                .cheated_state
                .deploy(&mut buffer, &mut self.blockifier_state, &inputs),
            "print" => {
                print(inputs);
                Ok(())
            }
            "precalculate_address" => self
                .cheated_state
                .precalculate_address(&mut buffer, &inputs),
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
    blockifier_state: &mut CachedState<DictStateReader>,
    cheated_state: &CheatedState,
) -> Result<(), HintError> {
    let (cell, offset) = extract_buffer(system);
    let system_ptr = get_ptr(vm, cell, &offset)?;

    let mut buffer = MemBuffer::new(vm, system_ptr);

    let selector = buffer.next_felt252().unwrap().to_bytes_be();
    let gas_counter = buffer.next_usize().unwrap();
    let contract_address = buffer.next_felt252().unwrap().into_owned();
    let entry_point_selector = buffer.next_felt252().unwrap().into_owned();

    let calldata = buffer.next_arr().unwrap();

    assert_eq!(std::str::from_utf8(&selector).unwrap(), "CallContract");

    let call_result = call_contract(
        &contract_address,
        &entry_point_selector,
        &calldata,
        blockifier_state,
        cheated_state,
    )
    .unwrap_or_else(|err| panic!("Transaction execution error: {err}"));

    let (result, exit_code) = match call_result {
        CallContractOutput::Success { ret_data } => (ret_data, 0),
        CallContractOutput::Panic { panic_data } => (panic_data, 1),
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
