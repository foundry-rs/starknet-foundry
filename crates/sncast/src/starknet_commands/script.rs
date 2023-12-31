use std::any::Any;
use std::collections::HashMap;
use std::fs;

use crate::starknet_commands::declare::BuildConfig;
use crate::starknet_commands::{call, declare, deploy, invoke};
use crate::{get_account, get_nonce, WaitForTx};
use anyhow::{anyhow, ensure, Context, Result};
use cairo_felt::Felt252;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_casm::operand::{CellRef, ResOperand};
use cairo_lang_runner::casm_run::{cell_ref_to_relocatable, execute_core_hint_base};
use cairo_lang_runner::casm_run::{extract_relocatable, vm_get_range, MemBuffer};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{
    build_hints_dict, insert_value_to_cellref, RunResultValue, SierraCasmRunner,
};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::bigint::BigIntAsHex;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use camino::Utf8PathBuf;
use clap::command;
use clap::Args;
use conversions::{FromConv, IntoConv};
use itertools::chain;
use runtime::utils::BufferReader;
use runtime::EnhancedHintError;
use scarb_api::ScarbCommand;
use sncast::helpers::scarb_utils::{
    get_package_metadata, get_scarb_manifest, get_scarb_metadata_with_deps, CastConfig,
};
use sncast::response::structs::ScriptResponse;
use starknet::accounts::Account;
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

#[derive(Args)]
#[command(about = "Execute a deployment script")]
pub struct Script {
    /// Module name that contains the `main` function, which will be executed
    pub script_module_name: String,
}

pub struct CairoHintProcessor<'a> {
    pub hints: &'a HashMap<String, Hint>,
    pub provider: &'a JsonRpcClient<HttpTransport>,
    pub runtime: Runtime,
    pub run_resources: RunResources,
    pub config: &'a CastConfig,
}

// cairo/crates/cairo-lang-runner/src/casm_run/mod.rs:457 (ResourceTracker for CairoHintProcessor)
impl ResourceTracker for CairoHintProcessor<'_> {
    fn consumed(&self) -> bool {
        self.run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.run_resources.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.run_resources.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.run_resources.run_resources()
    }
}

impl HintProcessorLogic for CairoHintProcessor<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        _constants: &HashMap<String, Felt252>,
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

        let hint = maybe_extended_hint.ok_or(HintError::WrongHintData)?;
        match hint {
            Hint::Core(hint) => execute_core_hint_base(vm, exec_scopes, hint),
            Hint::Starknet(_) => Err(HintError::CustomHint(Box::from(
                "Starknet syscalls are not supported",
            ))),
        }
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

        let mut reader = BufferReader::new(inputs);

        match selector {
            "call" => {
                let contract_address = reader.read_felt().into_();
                let function_name = reader
                    .read_short_string()
                    .expect("Failed to convert function name to short string");
                let calldata = reader.read_vec();
                let calldata_felts: Vec<FieldElement> = calldata
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                let call_response = self.runtime.block_on(call::call(
                    contract_address,
                    &function_name,
                    calldata_felts,
                    self.provider,
                    &BlockId::Tag(Pending),
                ))?;

                buffer
                    .write(call_response.response.len())
                    .expect("Failed to insert data length");

                buffer
                    .write_data(call_response.response.iter().map(|el| Felt252::from_(el.0)))
                    .expect("Failed to insert data");

                Ok(())
            }
            "declare" => {
                let contract_name = reader
                    .read_short_string()
                    .expect("Failed to convert contract name to string");
                let max_fee = reader.read_option_felt().map(conversions::IntoConv::into_);
                let nonce = reader.read_option_felt().map(conversions::IntoConv::into_);

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let declare_response = self.runtime.block_on(declare::declare(
                    &contract_name,
                    max_fee,
                    &account,
                    nonce,
                    BuildConfig {
                        scarb_toml_path: None,
                        json: false,
                    },
                    WaitForTx {
                        wait: true,
                        timeout: self.config.wait_timeout,
                        retry_interval: self.config.wait_retry_interval,
                    },
                ))?;

                buffer
                    .write(Felt252::from_(declare_response.class_hash.0))
                    .expect("Failed to insert class hash");

                buffer
                    .write(Felt252::from_(declare_response.transaction_hash.0))
                    .expect("Failed to insert transaction hash");

                Ok(())
            }
            "deploy" => {
                let class_hash = reader.read_felt().into_();
                let constructor_calldata: Vec<FieldElement> = reader
                    .read_vec()
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                let salt = reader.read_option_felt().map(conversions::IntoConv::into_);
                let unique = reader.read_bool();
                let max_fee = reader.read_option_felt().map(conversions::IntoConv::into_);
                let nonce = reader.read_option_felt().map(conversions::IntoConv::into_);

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let deploy_response = self.runtime.block_on(deploy::deploy(
                    class_hash,
                    constructor_calldata,
                    salt,
                    unique,
                    max_fee,
                    &account,
                    nonce,
                    WaitForTx {
                        wait: true,
                        timeout: self.config.wait_timeout,
                        retry_interval: self.config.wait_retry_interval,
                    },
                ))?;

                buffer
                    .write(Felt252::from_(deploy_response.contract_address.0))
                    .expect("Failed to insert contract address");

                buffer
                    .write(Felt252::from_(deploy_response.transaction_hash.0))
                    .expect("Failed to insert transaction hash");

                Ok(())
            }
            "invoke" => {
                let contract_address = reader.read_felt().into_();
                let entry_point_name = reader
                    .read_short_string()
                    .expect("Failed to convert entry point name to short string");
                let calldata: Vec<FieldElement> = reader
                    .read_vec()
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();
                let max_fee = reader.read_option_felt().map(conversions::IntoConv::into_);
                let nonce = reader.read_option_felt().map(conversions::IntoConv::into_);

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let invoke_response = self.runtime.block_on(invoke::invoke(
                    contract_address,
                    &entry_point_name,
                    calldata,
                    max_fee,
                    &account,
                    nonce,
                    WaitForTx {
                        wait: true,
                        timeout: self.config.wait_timeout,
                        retry_interval: self.config.wait_retry_interval,
                    },
                ))?;

                buffer
                    .write(Felt252::from_(invoke_response.transaction_hash.0))
                    .expect("Failed to insert transaction hash");

                Ok(())
            }
            "get_nonce" => {
                let block_id = reader
                    .read_short_string()
                    .expect("Failed to convert entry point name to short string");
                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let nonce = self.runtime.block_on(get_nonce(
                    self.provider,
                    &block_id,
                    account.address(),
                ))?;
                buffer
                    .write(Felt252::from_(nonce))
                    .expect("Failed to insert nonce");

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
    module_name: &str,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    provider: &JsonRpcClient<HttpTransport>,
    runtime: Runtime,
    config: &CastConfig,
) -> Result<ScriptResponse> {
    let path = compile_script(path_to_scarb_toml.clone())?;

    let sierra_program = serde_json::from_str::<VersionedProgram>(
        &fs::read_to_string(path.clone())
            .with_context(|| format!("Failed to read Sierra file at path = {path}"))?,
    )
    .with_context(|| format!("Failed to deserialize Sierra program at path = {path}"))?
    .into_v1()
    .with_context(|| format!("Failed to load Sierra program at path = {path}"))?
    .program;

    let runner = SierraCasmRunner::new(
        sierra_program,
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
    )
    .with_context(|| "Failed to set up runner")?;

    let name_suffix = module_name.to_string() + "::main";
    let func = runner.find_function(name_suffix.as_str())?;

    let (entry_code, builtins) = runner.create_entry_code(func, &Vec::new(), usize::MAX)?;
    let footer = runner.create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
        footer.iter()
    );

    // import from cairo-lang-runner
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());

    let mut cairo_hint_processor = CairoHintProcessor {
        hints: &string_to_hint,
        provider,
        runtime,
        run_resources: RunResources::default(),
        config,
    };

    match runner.run_function(
        func,
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

fn compile_script(path_to_scarb_toml: Option<Utf8PathBuf>) -> Result<Utf8PathBuf> {
    let scripts_manifest_path = path_to_scarb_toml.unwrap_or_else(|| {
        get_scarb_manifest()
            .context("Failed to retrieve manifest path from scarb")
            .unwrap()
    });
    ensure!(
        scripts_manifest_path.exists(),
        "The path = {scripts_manifest_path} does not exist"
    );

    ScarbCommand::new_with_stdio()
        .arg("build")
        .manifest_path(&scripts_manifest_path)
        .run()
        .context("failed to compile script with scarb")?;

    let metadata = get_scarb_metadata_with_deps(&scripts_manifest_path)?;
    let package_metadata = get_package_metadata(&metadata, &scripts_manifest_path)?;

    let filename = format!("{}.sierra.json", package_metadata.name);
    let path = metadata
        .target_dir
        .unwrap_or(metadata.workspace.root.join("target"))
        .join(metadata.current_profile)
        .join(filename.clone());

    ensure!(
        path.exists(),
        "The package has not been compiled, the file at path = {path} does not exist"
    );

    Ok(path)
}

// taken from starknet-foundry/crates/forge/src/test_case_summary.rs
/// Helper function to build `readable_text` from a run data.
// TODO #1127
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
