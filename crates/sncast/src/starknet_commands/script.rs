use std::any::Any;
use std::collections::HashMap;
use std::fs;

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
use num_traits::ToPrimitive;
use runtime::EnhancedHintError;
use scarb_metadata::ScarbCommand;
use serde::Serialize;
use serde_json::{json, Value};
use sncast::helpers::response_structs::ScriptResponse;
use sncast::helpers::scarb_utils::{
    get_package_metadata, get_scarb_manifest, get_scarb_metadata_with_deps, CastConfig,
};
use sncast::{print_formatted, stringify_command_result_struct, ValueFormat};
use starknet::accounts::Account;
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Verbosity {
    Quiet,

    #[default]
    Normal,

    Verbose,
}

#[derive(Debug)]
pub struct UI {
    verbosity: Verbosity,
    value_format: ValueFormat,
    json: bool,
}

pub trait DebugArgVal {
    fn get_arg_val(&self) -> String;
}

impl DebugArgVal for String {
    fn get_arg_val(&self) -> String {
        format!("\"{}\"", self.to_owned())
    }
}

impl DebugArgVal for FieldElement {
    fn get_arg_val(&self) -> String {
        format!("{self:#064x}")
    }
}

impl<T: DebugArgVal> DebugArgVal for Vec<T> {
    fn get_arg_val(&self) -> String {
        let res: Vec<String> = self.iter().map(DebugArgVal::get_arg_val).collect();
        let res = res.join(",");
        format!("[{res}]")
    }
}

impl<T: DebugArgVal> DebugArgVal for Option<T> {
    fn get_arg_val(&self) -> String {
        match self {
            Some(v) => format!("Some({})", v.get_arg_val()),
            None => "None".to_string(),
        }
    }
}

impl DebugArgVal for bool {
    fn get_arg_val(&self) -> String {
        self.to_string()
    }
}

macro_rules! stringify_args {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                let stringified_arg = (stringify!($x).to_string(), $x.get_arg_val());
                temp_vec.push(stringified_arg);
            )*
            temp_vec
        }
    };
}

impl UI {
    pub fn new(verbosity: Verbosity, value_format: ValueFormat, json: bool) -> Self {
        Self {
            verbosity,
            value_format,
            json,
        }
    }

    pub fn is_quiet(&self) -> bool {
        self.verbosity < Verbosity::Normal
    }

    pub fn print(&self, msg: &str) {
        if self.verbosity >= Verbosity::Normal {
            println!("{msg}");
        }
    }

    fn format_args_as_json(args: Vec<(String, String)>) -> Result<Value> {
        let mut json_args_map: HashMap<String, Value> = HashMap::new();
        for (key, value) in args {
            let json_value = match serde_json::from_str(&value).ok() {
                Some(v) => v,
                None => serde_json::to_value(value)?,
            };
            json_args_map.insert(key, json_value);
        }
        let json_value_args: Value = serde_json::to_value(json_args_map)?;
        Ok(json_value_args)
    }

    fn format_args_as_string(args: &[(String, String)]) -> String {
        let formatted_args = args
            .iter()
            .map(|(k, v)| format!("\t{k}: {v},\n"))
            .collect::<Vec<String>>()
            .concat();
        format!("[\n{formatted_args}]")
    }

    pub fn print_cheatcode_args(&self, cheatcode: &str, args: Vec<(String, String)>) -> Result<()> {
        if self.verbosity >= Verbosity::Verbose {
            if self.json {
                let json_value_args = UI::format_args_as_json(args)?;
                let json_output = json!({
                    "cheatcode": cheatcode,
                    "args_passed": json_value_args
                });
                let json_output = serde_json::to_string_pretty(&json_output)?;
                println!("{json_output}");
            } else {
                let formatted_args = UI::format_args_as_string(&args);
                let header = ("cheatcode".to_string(), cheatcode.to_string());
                let body = ("args_passed".to_string(), formatted_args);
                let output = vec![header, body];
                print_formatted(output, self.json, false)?;
            }
            println!();
        }
        Ok(())
    }

    pub fn print_cheatcode_response<T: Serialize>(
        &self,
        cheatcode: &str,
        result: &T,
    ) -> Result<()> {
        if self.verbosity >= Verbosity::Normal {
            let mut output = vec![("cheatcode".to_string(), cheatcode.to_string())];
            output.extend(stringify_command_result_struct(result, self.value_format)?);

            print_formatted(output, self.json, false)?;
            println!();
        }
        Ok(())
    }
}

#[derive(Args)]
#[command(about = "Execute a deployment script")]
pub struct Script {
    /// Module name that contains the `main` function, which will be executed
    pub script_module_name: String,

    /// Logging verbosity
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

pub struct CairoHintProcessor<'a> {
    pub hints: &'a HashMap<String, Hint>,
    pub provider: &'a JsonRpcClient<HttpTransport>,
    pub runtime: Runtime,
    pub run_resources: RunResources,
    pub config: &'a CastConfig,
    pub script_ui: UI,
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

        match selector {
            "call" => {
                let contract_address: FieldElement = inputs[0].clone().into_();
                let function_name = as_cairo_short_string(&inputs[1])
                    .expect("Failed to convert function name to short string");
                let calldata_length = inputs[2]
                    .to_usize()
                    .expect("Failed to convert calldata length to usize");
                let calldata = Vec::from(&inputs[3..(3 + calldata_length)]);
                let calldata_felts: Vec<FieldElement> = calldata
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                self.script_ui.print_cheatcode_args(
                    selector,
                    stringify_args!(contract_address, function_name, calldata_felts),
                );

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
                    .write_data(call_response.response.iter().map(|el| Felt252::from_(*el)))
                    .expect("Failed to insert data");

                self.script_ui
                    .print_cheatcode_response(selector, &call_response)?;

                Ok(())
            }
            "declare" => {
                let contract_name = as_cairo_short_string(&inputs[0])
                    .expect("Failed to convert contract name to string");
                let mut offset = 1;
                let max_fee = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };
                offset += 1;
                let nonce = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };

                self.script_ui
                    .print_cheatcode_args(selector, stringify_args!(contract_name, max_fee, nonce));

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
                    &None,
                    nonce,
                    WaitForTx {
                        wait: true,
                        timeout: self.config.wait_timeout,
                        retry_interval: self.config.wait_retry_interval,
                    },
                ))?;

                buffer
                    .write(Felt252::from_(declare_response.class_hash))
                    .expect("Failed to insert class hash");

                buffer
                    .write(Felt252::from_(declare_response.transaction_hash))
                    .expect("Failed to insert transaction hash");

                self.script_ui
                    .print_cheatcode_response(selector, &declare_response)?;

                Ok(())
            }
            "deploy" => {
                let class_hash: FieldElement = inputs[0].clone().into_();
                let calldata_length = inputs[1]
                    .to_usize()
                    .expect("Failed to convert calldata length to usize");
                let constructor_calldata: Vec<FieldElement> = {
                    let calldata = Vec::from(&inputs[2..(2 + calldata_length)]);
                    calldata
                        .iter()
                        .map(|el| FieldElement::from_(el.clone()))
                        .collect()
                };
                let mut offset = 2 + calldata_length;
                let salt = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };
                offset += 1;
                let unique = { inputs[offset] == 1.into() };
                offset += 1;
                let max_fee = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };
                offset += 1;
                let nonce = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };

                self.script_ui.print_cheatcode_args(
                    selector,
                    stringify_args!(
                        class_hash,
                        constructor_calldata,
                        salt,
                        unique,
                        max_fee,
                        nonce
                    ),
                );

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
                    nonce,
                    WaitForTx {
                        wait: true,
                        timeout: self.config.wait_timeout,
                        retry_interval: self.config.wait_retry_interval,
                    },
                ))?;

                buffer
                    .write(Felt252::from_(deploy_response.contract_address))
                    .expect("Failed to insert contract address");

                buffer
                    .write(Felt252::from_(deploy_response.transaction_hash))
                    .expect("Failed to insert transaction hash");

                self.script_ui
                    .print_cheatcode_response(selector, &deploy_response)?;

                Ok(())
            }
            "invoke" => {
                let contract_address = FieldElement::from_(inputs[0].clone());
                let entry_point_name = as_cairo_short_string(&inputs[1])
                    .expect("Failed to convert entry point name to short string");
                let calldata_length = inputs[2]
                    .to_usize()
                    .expect("Failed to convert calldata length to usize");
                let calldata: Vec<FieldElement> = {
                    let calldata = Vec::from(&inputs[3..(3 + calldata_length)]);
                    calldata
                        .iter()
                        .map(|el| FieldElement::from_(el.clone()))
                        .collect()
                };
                let mut offset = 3 + calldata_length;
                let max_fee = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };
                offset += 1;
                let nonce = if inputs[offset] == 0.into() {
                    offset += 1;
                    Some(inputs[offset].clone().into_())
                } else {
                    None
                };

                self.script_ui.print_cheatcode_args(
                    selector,
                    stringify_args!(contract_address, entry_point_name, calldata, max_fee, nonce),
                );

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
                    nonce,
                    WaitForTx {
                        wait: true,
                        timeout: self.config.wait_timeout,
                        retry_interval: self.config.wait_retry_interval,
                    },
                ))?;

                buffer
                    .write(Felt252::from_(invoke_response.transaction_hash))
                    .expect("Failed to insert transaction hash");

                self.script_ui
                    .print_cheatcode_response(selector, &invoke_response)?;

                Ok(())
            }
            "get_nonce" => {
                let block_id = as_cairo_short_string(&inputs[0])
                    .expect("Failed to convert entry point name to short string");

                self.script_ui
                    .print_cheatcode_args(selector, stringify_args!(block_id));

                let account = self.runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    &self.config.keystore,
                ))?;

                let nonce = self.runtime.block_on(get_nonce(
                    self.provider,
                    &block_id,
                    account.address(),
                ))?;
                buffer
                    .write(Felt252::from_(nonce))
                    .expect("Failed to insert nonce");

                self.script_ui.print_cheatcode_response(selector, &nonce)?;

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
    script_ui: UI,
) -> Result<ScriptResponse> {
    let path = compile_script(path_to_scarb_toml.clone(), &script_ui)?;

    let sierra_program = serde_json::from_str::<VersionedProgram>(
        &fs::read_to_string(path.clone())
            .with_context(|| format!("failed to read Sierra file: {path}"))?,
    )
    .with_context(|| format!("failed to deserialize Sierra program: {path}"))?
    .into_v1()
    .with_context(|| format!("failed to load Sierra program: {path}"))?;

    let runner = SierraCasmRunner::new(
        sierra_program,
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
    )
    .with_context(|| "Failed setting up runner.")?;

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
        script_ui,
    };

    cairo_hint_processor.script_ui.print("\n");
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

fn compile_script(path_to_scarb_toml: Option<Utf8PathBuf>, script_ui: &UI) -> Result<Utf8PathBuf> {
    let scripts_manifest_path = match path_to_scarb_toml {
        Some(path) => path,
        None => get_scarb_manifest()
            .context("Failed to obtain manifest path from scarb")
            .unwrap(),
    };
    ensure!(
        scripts_manifest_path.exists(),
        "Path {scripts_manifest_path} does not exist"
    );

    let mut scarb_args = vec!["build"];
    if script_ui.is_quiet() {
        scarb_args.push("--quiet");
    }
    if script_ui.json {
        scarb_args.insert(0, "--json");
    }

    ScarbCommand::new()
        .args(scarb_args)
        .env("SCARB_MANIFEST_PATH", &scripts_manifest_path)
        .run()?;

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
        "package has not been compiled, file does not exist: {path}"
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
