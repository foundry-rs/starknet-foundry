use std::collections::HashMap;
use std::fs;

use crate::starknet_commands::{call, declare, deploy, invoke};
use crate::{get_account, get_nonce, WaitForTx};
use anyhow::{anyhow, Context, Result};
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector;
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::{
    CachedState, GlobalContractCache, GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST,
};
use cairo_felt::Felt252;
use cairo_lang_casm::hints::Hint;
use cairo_lang_runner::{build_hints_dict, RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::vm_core::VirtualMachine;
use camino::Utf8PathBuf;
use clap::Args;
use conversions::felt252::SerializeAsFelt252Vec;
use conversions::{FromConv, IntoConv};
use itertools::chain;
use runtime::starknet::context::{build_context, SerializableBlockInfo};
use runtime::starknet::state::DictStateReader;
use runtime::utils::BufferReader;
use runtime::{
    CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic, StarknetRuntime,
    SyscallHandlingResult,
};
use scarb_api::{package_matches_version_requirement, StarknetContractArtifacts};
use scarb_metadata::{Metadata, PackageMetadata};
use semver::{Comparator, Op, Version, VersionReq};
use shared::print::print_as_warning;
use shared::utils::build_readable_text;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::SCRIPT_LIB_ARTIFACT_NAME;
use sncast::response::structs::ScriptRunResponse;
use sncast::state::hashing::{
    generate_declare_tx_id, generate_deploy_tx_id, generate_invoke_tx_id,
};
use sncast::state::state_file::{serialize_as_script_function_result, StateManager};
use starknet::accounts::{Account, SingleOwnerAccount};
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use tokio::runtime::Runtime;

type ScriptStarknetContractArtifacts = StarknetContractArtifacts;

#[derive(Args, Debug)]
#[command(about = "Execute a deployment script")]
pub struct Run {
    /// Module name that contains the `main` function, which will be executed
    pub script_name: String,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,

    /// Do not use the state file
    #[clap(long)]
    pub no_state_file: bool,
}

pub struct CastScriptExtension<'a> {
    pub hints: &'a HashMap<String, Hint>,
    pub provider: &'a JsonRpcClient<HttpTransport>,
    pub account: Option<&'a SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>>,
    pub tokio_runtime: Runtime,
    pub config: &'a CastConfig,
    pub artifacts: &'a HashMap<String, StarknetContractArtifacts>,
    pub state: StateManager,
}

impl<'a> CastScriptExtension<'a> {
    pub fn account(
        &self,
    ) -> Result<&SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>> {
        self.account.ok_or_else(|| anyhow!("Account not defined. Please ensure the correct account is passed to `script run` command"))
    }
}

impl<'a> ExtensionLogic for CastScriptExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    #[allow(clippy::too_many_lines)]
    fn handle_cheatcode(
        &mut self,
        selector: &str,
        mut input_reader: BufferReader,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        let res = match selector {
            "call" => {
                let contract_address = input_reader.read_felt()?.into_();
                let function_selector = input_reader.read_felt()?.into_();
                let calldata = input_reader.read_vec()?;
                let calldata_felts: Vec<FieldElement> = calldata
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                let call_result = self.tokio_runtime.block_on(call::call(
                    contract_address,
                    function_selector,
                    calldata_felts,
                    self.provider,
                    &BlockId::Tag(Pending),
                ));
                Ok(CheatcodeHandlingResult::Handled(
                    call_result.serialize_as_felt252_vec(),
                ))
            }
            "declare" => {
                let contract_name = input_reader.read_string()?;
                let max_fee = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);
                let nonce = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);

                let declare_tx_id = generate_declare_tx_id(contract_name.as_str());

                if let Some(success_output) =
                    self.state.get_output_if_success(declare_tx_id.as_str())
                {
                    return Ok(CheatcodeHandlingResult::Handled(
                        serialize_as_script_function_result(success_output),
                    ));
                }

                let declare_result = self.tokio_runtime.block_on(declare::declare(
                    &contract_name,
                    max_fee,
                    self.account()?,
                    nonce,
                    self.artifacts,
                    WaitForTx {
                        wait: true,
                        wait_params: self.config.wait_params,
                    },
                ));

                self.state.maybe_insert_tx_entry(
                    declare_tx_id.as_str(),
                    selector,
                    &declare_result,
                )?;
                Ok(CheatcodeHandlingResult::Handled(
                    declare_result.serialize_as_felt252_vec(),
                ))
            }
            "deploy" => {
                let class_hash = input_reader.read_felt()?.into_();
                let constructor_calldata: Vec<FieldElement> = input_reader
                    .read_vec()?
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                let salt = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);
                let unique = input_reader.read_bool()?;
                let max_fee = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);
                let nonce = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);

                let deploy_tx_id =
                    generate_deploy_tx_id(class_hash, &constructor_calldata, salt, unique);

                if let Some(success_output) =
                    self.state.get_output_if_success(deploy_tx_id.as_str())
                {
                    return Ok(CheatcodeHandlingResult::Handled(
                        serialize_as_script_function_result(success_output),
                    ));
                }

                let deploy_result = self.tokio_runtime.block_on(deploy::deploy(
                    class_hash,
                    constructor_calldata,
                    salt,
                    unique,
                    max_fee,
                    self.account()?,
                    nonce,
                    WaitForTx {
                        wait: true,
                        wait_params: self.config.wait_params,
                    },
                ));

                self.state.maybe_insert_tx_entry(
                    deploy_tx_id.as_str(),
                    selector,
                    &deploy_result,
                )?;

                Ok(CheatcodeHandlingResult::Handled(
                    deploy_result.serialize_as_felt252_vec(),
                ))
            }
            "invoke" => {
                let contract_address = input_reader.read_felt()?.into_();
                let function_selector = input_reader.read_felt()?.into_();
                let calldata: Vec<FieldElement> = input_reader
                    .read_vec()?
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();
                let max_fee = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);
                let nonce = input_reader
                    .read_option_felt()?
                    .map(conversions::IntoConv::into_);

                let invoke_tx_id =
                    generate_invoke_tx_id(contract_address, function_selector, &calldata);

                if let Some(success_output) =
                    self.state.get_output_if_success(invoke_tx_id.as_str())
                {
                    return Ok(CheatcodeHandlingResult::Handled(
                        serialize_as_script_function_result(success_output),
                    ));
                }

                let invoke_result = self.tokio_runtime.block_on(invoke::invoke(
                    contract_address,
                    function_selector,
                    calldata,
                    max_fee,
                    self.account()?,
                    nonce,
                    WaitForTx {
                        wait: true,
                        wait_params: self.config.wait_params,
                    },
                ));

                self.state.maybe_insert_tx_entry(
                    invoke_tx_id.as_str(),
                    selector,
                    &invoke_result,
                )?;

                Ok(CheatcodeHandlingResult::Handled(
                    invoke_result.serialize_as_felt252_vec(),
                ))
            }
            "get_nonce" => {
                let block_id = input_reader
                    .read_short_string()?
                    .expect("Failed to convert entry point name to short string");

                let nonce = self.tokio_runtime.block_on(get_nonce(
                    self.provider,
                    &block_id,
                    self.account()?.address(),
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

#[allow(clippy::too_many_arguments)]
pub fn run(
    module_name: &str,
    metadata: &Metadata,
    package_metadata: &PackageMetadata,
    artifacts: &mut HashMap<String, StarknetContractArtifacts>,
    provider: &JsonRpcClient<HttpTransport>,
    tokio_runtime: Runtime,
    config: &CastConfig,
    state_file_path: Option<Utf8PathBuf>,
) -> Result<ScriptRunResponse> {
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
        None,
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
    let mut context = build_context(&SerializableBlockInfo::default().into());

    let mut blockifier_state = CachedState::new(
        DictStateReader::default(),
        GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
    );
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

    let account = if config.account.is_empty() {
        None
    } else {
        Some(tokio_runtime.block_on(get_account(
            &config.account,
            &config.accounts_file,
            provider,
            config.keystore.clone(),
        ))?)
    };
    let state = StateManager::from(state_file_path)?;

    let cast_extension = CastScriptExtension {
        hints: &string_to_hint,
        provider,
        tokio_runtime,
        config,
        artifacts: &artifacts,
        account: account.as_ref(),
        state,
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
            RunResultValue::Success(data) => Ok(ScriptRunResponse {
                status: "success".to_string(),
                message: build_readable_text(&data),
            }),
            RunResultValue::Panic(panic_data) => Ok(ScriptRunResponse {
                status: "script panicked".to_string(),
                message: build_readable_text(&panic_data),
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
