use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::starknet_commands::call;
use anyhow::{anyhow, Context, Result};
use blockifier::execution::common_hints::ExecutionMode;
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
use cairo_lang_diagnostics::ToOption;
use cairo_lang_filesystem::db::init_dev_corelib;
use cairo_lang_runner::casm_run::cell_ref_to_relocatable;
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{
    build_hints_dict, insert_value_to_cellref, RunResultValue, SierraCasmRunner,
};
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
use cast::helpers::response_structs::ScriptResponse;
use cast::helpers::scarb_utils::parse_scarb_metadata;
use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::constants::{build_block_context, build_transaction_context};
use cheatnet::state::{CheatnetBlockInfo, DictStateReader};
use clap::command;
use clap::Args;
use conversions::StarknetConversions;
use itertools::chain;
use num_traits::ToPrimitive;
use scarb_artifacts::corelib_for_package;
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

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
                "Failed to parse the cheatcode selector".to_string(),
            ))
        })?;

        // Extract the inputs.
        let input_start = extract_relocatable(vm, input_start)?;
        let input_end = extract_relocatable(vm, input_end)?;
        let inputs = vm_get_range(vm, input_start, input_end).map_err(|_| {
            HintError::CustomHint(Box::from("Failed to read input data".to_string()))
        })?;

        self.match_cheatcode_by_selector(vm, selector, &inputs, output_start, output_end)
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
        inputs: &[Felt252],
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
                let calldata = Vec::from(&inputs[3..(3 + calldata_length)]);
                let calldata_felts: Vec<FieldElement> = calldata
                    .iter()
                    .map(StarknetConversions::to_field_element)
                    .collect();

                let call_response = self.runtime.block_on(call::call(
                    contract_address,
                    &function_name,
                    calldata_felts,
                    self.provider,
                    &BlockId::Tag(Pending),
                ))?;

                buffer
                    .write(call_response.data.len())
                    .expect("Failed to insert data length");

                buffer
                    .write_data(
                        call_response
                            .data
                            .iter()
                            .map(StarknetConversions::to_felt252),
                    )
                    .expect("Failed to insert data");

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
    script_path: &Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    runtime: Runtime,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
) -> Result<ScriptResponse> {
    check_compiler_path(true, Path::new(&script_path))?;

    let metadata = parse_scarb_metadata(path_to_scarb_toml)?;

    let corelib = corelib_for_package(&metadata, &metadata.workspace.members[0])?;
    let db = &mut RootDatabase::builder().build()?;

    init_dev_corelib(db, PathBuf::from(corelib));

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
    let block_context = build_block_context(CheatnetBlockInfo::default());
    let account_context = build_transaction_context();
    let mut context = EntryPointExecutionContext::new(
        &block_context.clone(),
        &account_context,
        ExecutionMode::Execute,
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
        runtime,
    };

    match runner.run_function(
        runner.find_function("::main")?,
        &mut cairo_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => match result.value {
            RunResultValue::Success(data) => Ok(ScriptResponse {
                status: "success".to_string(),
                msg: build_readable_text(&data),
            }),
            RunResultValue::Panic(panic_data) => Ok(ScriptResponse {
                status: "script panicked".to_string(),
                msg: build_readable_text(&panic_data),
            }),
        },
        Err(err) => Err(err.into()),
    }
}

// taken from starknet-foundry/crates/forge/src/test_case_summary.rs
/// Helper function to build `readable_text` from a run data.
fn build_readable_text(data: &Vec<Felt252>) -> Option<String> {
    let mut readable_text = String::new();

    for felt in data {
        readable_text.push_str(&format!("\n    original value: [{felt}]"));
        if let Some(short_string) = as_cairo_short_string(felt) {
            readable_text.push_str(&format!(", converted to a string: [{short_string}]"));
        }
    }

    if readable_text.is_empty() {
        None
    } else {
        readable_text.push('\n');
        Some(readable_text)
    }
}
