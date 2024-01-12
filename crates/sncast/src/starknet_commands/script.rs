use std::collections::HashMap;
use std::fs;

use crate::starknet_commands::declare::BuildConfig;
use crate::starknet_commands::{call, declare, deploy, invoke};
use crate::{get_account, get_nonce, WaitForTx};
use anyhow::{anyhow, ensure, Context, Result};
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::{
    CallEntryPoint, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use cairo_felt::Felt252;
use cairo_lang_casm::hints::Hint;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{build_hints_dict, RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use camino::Utf8PathBuf;
use clap::command;
use clap::Args;
use conversions::{FromConv, IntoConv};
use itertools::chain;
use runtime::starknet::context::{build_default_block_context, build_transaction_context};
use runtime::starknet::state::DictStateReader;
use runtime::utils::BufferReader;
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic, StarknetRuntime,
    SyscallHandlingResult,
};
use scarb_api::{package_matches_version_requirement, ScarbCommand};
use scarb_metadata::Metadata;
use semver::{Comparator, Op, Version, VersionReq};
use sncast::helpers::scarb_utils::{
    get_package_metadata, get_scarb_manifest, get_scarb_metadata_with_deps, CastConfig,
};
use sncast::response::print::print_as_warning;
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

pub struct CastScriptExtension<'a> {
    pub hints: &'a HashMap<String, Hint>,
    pub provider: &'a JsonRpcClient<HttpTransport>,
    pub tokio_runtime: Runtime,
    pub config: &'a CastConfig,
}

impl<'a> ExtensionLogic for CastScriptExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    #[allow(clippy::too_many_lines)]
    fn handle_cheatcode(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        let mut reader = BufferReader::new(&inputs);

        let res = match selector {
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

                let call_response = self.tokio_runtime.block_on(call::call(
                    contract_address,
                    &function_name,
                    calldata_felts,
                    self.provider,
                    &BlockId::Tag(Pending),
                ))?;

                let mut res: Vec<Felt252> = vec![Felt252::from(call_response.response.len())];
                res.extend(call_response.response.iter().map(|el| Felt252::from_(el.0)));
                Ok(CheatcodeHandlingResult::Handled(res))
            }
            "declare" => {
                let contract_name = reader
                    .read_short_string()
                    .expect("Failed to convert contract name to string");
                let max_fee = reader.read_option_felt().map(conversions::IntoConv::into_);
                let nonce = reader.read_option_felt().map(conversions::IntoConv::into_);

                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let declare_response = self.tokio_runtime.block_on(declare::declare(
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

                let res: Vec<Felt252> = vec![
                    Felt252::from_(declare_response.class_hash.0),
                    Felt252::from_(declare_response.transaction_hash.0),
                ];
                Ok(CheatcodeHandlingResult::Handled(res))
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

                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let deploy_response = self.tokio_runtime.block_on(deploy::deploy(
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

                let res: Vec<Felt252> = vec![
                    Felt252::from_(deploy_response.contract_address.0),
                    Felt252::from_(deploy_response.transaction_hash.0),
                ];
                Ok(CheatcodeHandlingResult::Handled(res))
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

                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let invoke_response = self.tokio_runtime.block_on(invoke::invoke(
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

                let res: Vec<Felt252> = vec![Felt252::from_(invoke_response.transaction_hash.0)];
                Ok(CheatcodeHandlingResult::Handled(res))
            }
            "get_nonce" => {
                let block_id = reader
                    .read_short_string()
                    .expect("Failed to convert entry point name to short string");
                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                let nonce = self.tokio_runtime.block_on(get_nonce(
                    self.provider,
                    &block_id,
                    account.address(),
                ))?;

                let res: Vec<Felt252> = vec![Felt252::from_(nonce)];
                Ok(CheatcodeHandlingResult::Handled(res))
            }
            _ => Ok(CheatcodeHandlingResult::Forwarded),
        };

        res
    }

    fn override_system_call(
        &mut self,
        _selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        Err(HintError::CustomHint(Box::from(
            "Starknet syscalls are not supported",
        )))
    }
}

pub fn run(
    module_name: &str,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    provider: &JsonRpcClient<HttpTransport>,
    tokio_runtime: Runtime,
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

    // hint processor
    let block_context = build_default_block_context();
    let account_context = build_transaction_context();
    let mut context = EntryPointExecutionContext::new(
        &block_context.clone(),
        &account_context,
        ExecutionMode::Execute,
        false,
    )
    .unwrap();

    let mut blockifier_state = CachedState::from(DictStateReader::default());
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

    let cast_extension = CastScriptExtension {
        hints: &string_to_hint,
        provider,
        tokio_runtime,
        config,
    };

    let mut cast_runtime = ExtendedRuntime {
        extension: cast_extension,
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
        },
    };

    let mut vm = VirtualMachine::new(true);
    match runner.run_function_with_vm(
        func,
        &mut vm,
        &mut cast_runtime,
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

fn sncast_std_version_requirement() -> VersionReq {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let comparator = Comparator {
        op: Op::Exact,
        major: version.major,
        minor: Some(version.minor),
        patch: Some(version.patch),
        pre: version.pre,
    };
    VersionReq {
        comparators: vec![comparator],
    }
}

fn warn_if_sncast_std_not_compatible(scarb_metadata: &Metadata) -> Result<()> {
    let sncast_std_version_requirement = sncast_std_version_requirement();
    if !package_matches_version_requirement(
        scarb_metadata,
        "sncast_std",
        &sncast_std_version_requirement,
    )? {
        print_as_warning(&anyhow!("Package sncast_std version does not meet the recommended version requirement {sncast_std_version_requirement}, it might result in unexpected behaviour"));
    }
    Ok(())
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

    warn_if_sncast_std_not_compatible(&metadata)?;

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
