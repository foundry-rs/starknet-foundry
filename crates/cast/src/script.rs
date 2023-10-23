use std::any::Any;
use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use blockifier::execution::entry_point::{
    CallEntryPoint, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use cairo_felt::Felt252;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::diagnostics::DiagnosticsReporter;
use cairo_lang_compiler::project::{check_compiler_path, setup_project};
use cairo_lang_runner::casm_run::cell_ref_to_relocatable;
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{build_hints_dict, insert_value_to_cellref, SierraCasmRunner};

use cairo_lang_diagnostics::ToOption;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::{DebugReplacer, SierraIdReplacer};
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use camino::Utf8PathBuf;
use cast::get_account;
use cast::helpers::scarb_utils::CastConfig;
use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::constants::{build_block_context, build_transaction_context};
use cheatnet::state::DictStateReader;
use clap::command;
use clap::Args;
use conversions::StarknetConversions;
use itertools::chain;
use num_traits::ToPrimitive;
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

use crate::starknet_commands::{call, declare, deploy, invoke};

#[derive(Args)]
#[command(about = "")]
pub struct Script {
    /// Path to the script
    pub script_path: Utf8PathBuf,
}

pub struct CairoHintProcessor<'a> {
    pub blockifier_syscall_handler: SyscallHintProcessor<'a>,
    pub hints: &'a HashMap<String, Hint>,
    pub provider: &'a JsonRpcClient<HttpTransport>,
    pub config: &'a CastConfig,
    pub path_to_scarb_toml: &'a Option<Utf8PathBuf>,
    pub runtime: Runtime,
}

// crates/blockifier/src/execution/syscalls/hint_processor.rs:472 (ResourceTracker for SyscallHintProcessor)
impl ResourceTracker for CairoHintProcessor<'_> {
    fn consumed(&self) -> bool {
        self.blockifier_syscall_handler
            .context
            .vm_run_resources
            .consumed()
    }

    fn consume_step(&mut self) {
        self.blockifier_syscall_handler
            .context
            .vm_run_resources
            .consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.blockifier_syscall_handler
            .context
            .vm_run_resources
            .get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.blockifier_syscall_handler
            .context
            .vm_run_resources
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
            );
        }
        self.blockifier_syscall_handler
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

        self.match_cheatcode_by_selector(vm, selector, inputs, output_start, output_end)
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
    ) -> Result<(), EnhancedHintError> {
        let mut buffer = MemBuffer::new_segment(vm);
        let result_start = buffer.ptr;

        match selector {
            "call" => {
                let contract_address = inputs[0].to_field_element();
                let function_name = as_cairo_short_string(&inputs[1])
                    .expect("Failed to convert function name to short string");
                let calldata_length = inputs[2]
                    .to_usize()
                    .expect("Failed to convert calldata length to usize");
                let calldata: Vec<FieldElement> = {
                    let calldata = Vec::from(&inputs[3..(3 + calldata_length)]);
                    calldata.iter().map(|x| x.to_field_element()).collect()
                };

                let call_response = self.runtime.block_on(call::call(
                    contract_address,
                    &function_name,
                    calldata,
                    self.provider,
                    &BlockId::Tag(Pending),
                ))?;

                buffer
                    .write(call_response.data.len())
                    .expect("Failed to insert data length");

                buffer
                    .write_data(call_response.data.iter().map(|x| x.to_felt252()))
                    .expect("Failed to insert data");

                Ok(())
            }
            "invoke" => {
                let contract_address = inputs[0].to_field_element();
                let entry_point_name = as_cairo_short_string(&inputs[1])
                    .expect("Failed to convert entry point name to short string");
                let calldata_length = inputs[2]
                    .to_usize()
                    .expect("Failed to convert calldata length to usize");
                let calldata: Vec<FieldElement> = {
                    let calldata = Vec::from(&inputs[3..(3 + calldata_length)]);
                    calldata.iter().map(|x| x.to_field_element()).collect()
                };
                let max_fee = if inputs[3 + calldata_length] == 0.into() {
                    Some(inputs[3 + calldata_length + 1].to_field_element())
                } else {
                    None
                };

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    &self.config.keystore,
                ))?;

                let invoke_response = self.runtime.block_on(invoke::invoke(
                    contract_address,
                    &entry_point_name,
                    calldata,
                    max_fee,
                    &account,
                    true,
                ))?;

                buffer
                    .write(invoke_response.transaction_hash.to_felt252())
                    .expect("Failed to insert transaction hash");

                Ok(())
            }
            "declare" => {
                let contract_name = as_cairo_short_string(&inputs[0])
                    .expect("Failed to convert contract name to short string");
                let max_fee = if inputs[1] == 0.into() {
                    Some(inputs[2].to_field_element())
                } else {
                    None
                };

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    &self.config.keystore,
                ))?;

                let declare_response = self.runtime.block_on(declare::declare(
                    &contract_name,
                    max_fee,
                    &account,
                    self.path_to_scarb_toml,
                    true,
                ))?;

                buffer
                    .write(declare_response.class_hash.to_felt252())
                    .expect("Failed to insert class hash");

                buffer
                    .write(declare_response.transaction_hash.to_felt252())
                    .expect("Failed to insert transaction hash");

                Ok(())
            }
            "deploy" => {
                let class_hash = inputs[0].to_field_element();
                let calldata_length = inputs[1]
                    .to_usize()
                    .expect("Failed to convert calldata length to usize");
                let constructor_calldata: Vec<FieldElement> = {
                    let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
                    calldata.iter().map(|x| x.to_field_element()).collect()
                };
                let mut offset = 2 + calldata_length;
                let salt = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset - 1].to_field_element())
                } else {
                    None
                };
                let unique = {
                    offset += 1;
                    inputs[offset - 1] == 1.into()
                };
                let max_fee = if inputs[offset] == 0.into() {
                    Some(inputs[offset + 1].to_field_element())
                } else {
                    None
                };

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    &self.config.keystore,
                ))?;

                let deploy_response = self.runtime.block_on(deploy::deploy(
                    class_hash,
                    constructor_calldata,
                    salt,
                    unique,
                    max_fee,
                    &account,
                    true,
                ))?;

                buffer
                    .write(deploy_response.contract_address.to_felt252())
                    .expect("Failed to insert contract address");

                buffer
                    .write(deploy_response.transaction_hash.to_felt252())
                    .expect("Failed to insert transaction hash");

                Ok(())
            }
            _ => Err(anyhow!("Unknown cheatcode selector: {selector}")),
        }?;

        let result_end = buffer.ptr;
        insert_value_to_cellref!(vm, output_start, result_start)?;
        insert_value_to_cellref!(vm, output_end, result_end)?;

        Ok(())
    }
}

pub fn run(
    script_path: Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    config: &CastConfig,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    runtime: Runtime,
) -> Result<()> {
    check_compiler_path(true, Path::new(&script_path))?;

    let db = &mut RootDatabase::builder().detect_corelib().build()?;

    let main_crate_ids = setup_project(db, Path::new(&script_path))?;
    if DiagnosticsReporter::stderr().check(db) {
        anyhow::bail!("failed to compile: {}", script_path);
    }

    let sierra_program = db
        .get_sierra_program(main_crate_ids.clone())
        .to_option()
        .with_context(|| "Compilation failed without any diagnostics.")?;
    let replacer = DebugReplacer { db };

    let runner = SierraCasmRunner::new(
        replacer.apply(&sierra_program),
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
    )
    .with_context(|| "Failed setting up runner.")?;

    let func = runner.find_function("::main")?;
    let (entry_code, builtins) = runner.create_entry_code(func, &Vec::new(), usize::MAX)?;
    let footer = runner.create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
        footer.iter()
    );

    // import from cairo-lang-runner
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());

    // hint processor
    let block_context = build_block_context();
    let account_context = build_transaction_context();
    let mut context = EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps.try_into().unwrap(),
    );

    let mut blockifier_state =
        CachedState::new(DictStateReader::default(), GlobalContractCache::default());
    let mut execution_resources = ExecutionResources::default();

    let syscall_handler = SyscallHintProcessor::new(
        &mut blockifier_state,
        &mut execution_resources,
        &mut context,
        // This segment is created by SierraCasmRunner
        Relocatable {
            segment_index: 10,
            offset: 0,
        },
        CallEntryPoint::default(),
        &string_to_hint,
        ReadOnlySegments::default(),
    );

    let mut cairo_hint_processor = CairoHintProcessor {
        blockifier_syscall_handler: syscall_handler,
        hints: &string_to_hint,
        provider,
        config,
        path_to_scarb_toml,
        runtime,
    };

    match runner.run_function(
        runner.find_function("::main")?,
        &mut cairo_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(_result) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
