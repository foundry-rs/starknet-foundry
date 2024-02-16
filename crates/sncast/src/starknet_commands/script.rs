use std::collections::HashMap;
use std::fs;

use crate::starknet_commands::{call, declare, deploy, invoke};
use crate::{get_account, get_nonce, WaitForTx};
use anyhow::{anyhow, Context, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::{CallEntryPoint, ExecutionResources};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use cairo_felt::Felt252;
use cairo_lang_casm::hints::Hint;
use cairo_lang_runner::{build_hints_dict, RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use clap::command;
use clap::Args;
use conversions::{FromConv, IntoConv};
use itertools::chain;
use runtime::starknet::context::{build_context, BlockInfo};
use runtime::starknet::state::DictStateReader;
use runtime::utils::BufferReader;
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic, StarknetRuntime,
    SyscallHandlingResult,
};
use scarb_api::{package_matches_version_requirement, StarknetContractArtifacts};
use scarb_metadata::{Metadata, PackageMetadata};
use semver::{Comparator, Op, Version, VersionReq};
use shared::utils::build_readable_text;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::SCRIPT_LIB_ARTIFACT_NAME;
use sncast::response::print::print_as_warning;
use sncast::response::structs::ScriptResponse;
use starknet::accounts::Account;
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

type ScriptStarknetContractArtifacts = StarknetContractArtifacts;

#[derive(Args)]
#[command(about = "Execute a deployment script")]
pub struct Script {
    /// Module name that contains the `main` function, which will be executed
    pub script_module_name: String,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,
}

pub struct CastScriptExtension<'a> {
    pub hints: &'a HashMap<String, Hint>,
    pub provider: &'a JsonRpcClient<HttpTransport>,
    pub tokio_runtime: Runtime,
    pub config: &'a CastConfig,
    pub artifacts: &'a HashMap<String, StarknetContractArtifacts>,
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
                    self.artifacts,
                    WaitForTx {
                        wait: true,
                        wait_params: self.config.wait_params,
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
                        wait_params: self.config.wait_params,
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
                        wait_params: self.config.wait_params,
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
    metadata: &Metadata,
    package_metadata: &PackageMetadata,
    artifacts: &mut HashMap<String, StarknetContractArtifacts>,
    provider: &JsonRpcClient<HttpTransport>,
    tokio_runtime: Runtime,
    config: &CastConfig,
) -> Result<ScriptResponse> {
    warn_if_sncast_std_not_compatible(metadata)?;
    let artifacts = inject_lib_artifact(metadata, package_metadata, artifacts)?;

    let artifact = artifacts
        .get(SCRIPT_LIB_ARTIFACT_NAME)
        .ok_or(anyhow!("Failed to find script artifact"))?;

    let sierra_program = serde_json::from_str::<VersionedProgram>(&artifact.sierra)
        .with_context(|| "Failed to deserialize Sierra program")?
        .into_v1()
        .with_context(|| "Failed to load Sierra program")?
        .program;

    let runner = SierraCasmRunner::new(
        sierra_program,
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
        false,
    )
    .with_context(|| "Failed to set up runner")?;

    let name_suffix = module_name.to_string() + "::main";
    let func = runner.find_function(name_suffix.as_str())?;

    let (entry_code, builtins) = runner.create_entry_code(func, &Vec::new(), usize::MAX)?;
    let footer = SierraCasmRunner::create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
    );

    // import from cairo-lang-runner
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());
    let assembled_program = runner
        .get_casm_program()
        .clone()
        .assemble_ex(&entry_code, &footer);

    // hint processor
    let mut context = build_context(BlockInfo::default());

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
        artifacts: &artifacts,
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
        assembled_program.bytecode.iter(),
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

fn inject_lib_artifact(
    metadata: &Metadata,
    package_metadata: &PackageMetadata,
    artifacts: &mut HashMap<String, StarknetContractArtifacts>,
) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let sierra_filename = format!("{}.sierra.json", package_metadata.name);

    let target_dir = &metadata
        .target_dir
        .clone()
        .unwrap_or_else(|| metadata.workspace.root.join("target"));
    let sierra_path = &target_dir
        .join(&metadata.current_profile)
        .join(sierra_filename);

    let lib_artifacts = ScriptStarknetContractArtifacts {
        sierra: fs::read_to_string(sierra_path)?,
        casm: String::new(),
    };

    artifacts.insert(SCRIPT_LIB_ARTIFACT_NAME.to_string(), lib_artifacts);
    Ok(artifacts.clone())
}
