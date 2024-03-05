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
use clap::Args;
use conversions::byte_array::ByteArray;
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
use sncast::response::errors::{SNCastProviderError, SNCastStarknetError, StarknetCommandError};
use sncast::response::structs::ScriptRunResponse;
use sncast::{TransactionError, WaitForTransactionError};
use starknet::accounts::Account;
use starknet::core::types::{BlockId, BlockTag::Pending, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

type ScriptStarknetContractArtifacts = StarknetContractArtifacts;

pub trait SerializeAsFelt252 {
    fn serialize_as_felt252(&self) -> Vec<Felt252>;
}

impl SerializeAsFelt252 for StarknetCommandError {
    fn serialize_as_felt252(&self) -> Vec<Felt252> {
        match self {
            StarknetCommandError::UnknownError(err) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(ByteArray::from(err.to_string().as_str()).serialize_no_magic());
                res
            }
            StarknetCommandError::ContractArtifactsNotFound(err) => {
                let mut res = vec![Felt252::from(1)];
                res.extend(ByteArray::from(err.data.as_str()).serialize_no_magic());
                res
            }
            StarknetCommandError::WaitForTransactionError(err) => {
                let mut res = vec![Felt252::from(2)];
                res.extend(err.serialize_as_felt252());
                res
            }
            StarknetCommandError::ProviderError(err) => {
                let mut res = vec![Felt252::from(3)];
                res.extend(err.serialize_as_felt252());
                res
            }
        }
    }
}

impl SerializeAsFelt252 for SNCastProviderError {
    fn serialize_as_felt252(&self) -> Vec<Felt252> {
        match self {
            SNCastProviderError::StarknetError(err) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(err.serialize_as_felt252());
                res
            }
            SNCastProviderError::RateLimited => {
                vec![Felt252::from(1)]
            }
            SNCastProviderError::UnknownError(err) => {
                let mut res = vec![Felt252::from(2)];
                res.extend(ByteArray::from(err.to_string().as_str()).serialize_no_magic());
                res
            }
        }
    }
}

impl SerializeAsFelt252 for SNCastStarknetError {
    fn serialize_as_felt252(&self) -> Vec<Felt252> {
        match self {
            SNCastStarknetError::FailedToReceiveTransaction => vec![Felt252::from(0)],
            SNCastStarknetError::ContractNotFound => vec![Felt252::from(1)],
            SNCastStarknetError::BlockNotFound => vec![Felt252::from(2)],
            SNCastStarknetError::InvalidTransactionIndex => vec![Felt252::from(3)],
            SNCastStarknetError::ClassHashNotFound => vec![Felt252::from(4)],
            SNCastStarknetError::TransactionHashNotFound => vec![Felt252::from(5)],
            SNCastStarknetError::ContractError(err) => {
                let mut res = vec![Felt252::from(6)];
                res.extend(ByteArray::from(err.revert_error.as_str()).serialize_no_magic());
                res
            }
            SNCastStarknetError::TransactionExecutionError(err) => {
                let mut res = vec![Felt252::from(7), Felt252::from(err.transaction_index)];
                res.extend(ByteArray::from(err.execution_error.as_str()).serialize_no_magic());
                res
            }
            SNCastStarknetError::ClassAlreadyDeclared => vec![Felt252::from(8)],
            SNCastStarknetError::InvalidTransactionNonce => vec![Felt252::from(9)],
            SNCastStarknetError::InsufficientMaxFee => vec![Felt252::from(10)],
            SNCastStarknetError::InsufficientAccountBalance => vec![Felt252::from(11)],
            SNCastStarknetError::ValidationFailure(err) => {
                let mut res = vec![Felt252::from(12)];
                res.extend(ByteArray::from(err.as_str()).serialize_no_magic());
                res
            }
            SNCastStarknetError::CompilationFailed => vec![Felt252::from(13)],
            SNCastStarknetError::ContractClassSizeIsTooLarge => vec![Felt252::from(14)],
            SNCastStarknetError::NonAccount => vec![Felt252::from(15)],
            SNCastStarknetError::DuplicateTx => vec![Felt252::from(16)],
            SNCastStarknetError::CompiledClassHashMismatch => vec![Felt252::from(17)],
            SNCastStarknetError::UnsupportedTxVersion => vec![Felt252::from(18)],
            SNCastStarknetError::UnsupportedContractClassVersion => vec![Felt252::from(19)],
            SNCastStarknetError::UnexpectedError(err) => {
                let mut res = vec![Felt252::from(20)];
                res.extend(ByteArray::from(err.to_string().as_str()).serialize_no_magic());
                res
            }
        }
    }
}

impl SerializeAsFelt252 for WaitForTransactionError {
    fn serialize_as_felt252(&self) -> Vec<Felt252> {
        match self {
            WaitForTransactionError::TransactionError(err) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(err.serialize_as_felt252());
                res
            }
            WaitForTransactionError::TimedOut => vec![Felt252::from(1)],
            WaitForTransactionError::ProviderError(err) => {
                let mut res = vec![Felt252::from(2)];
                res.extend(err.serialize_as_felt252());
                res
            }
        }
    }
}

impl SerializeAsFelt252 for TransactionError {
    fn serialize_as_felt252(&self) -> Vec<Felt252> {
        match self {
            TransactionError::Rejected => vec![Felt252::from(0)],
            TransactionError::Reverted(err) => {
                let mut res = vec![Felt252::from(1)];
                res.extend(ByteArray::from(err.data.as_str()).serialize_no_magic());
                res
            }
        }
    }
}

fn handle_starknet_command_error_in_script(err: &StarknetCommandError) -> Vec<Felt252> {
    let error_msg_serialized = err.serialize_as_felt252();
    let mut res: Vec<Felt252> = vec![Felt252::from(1)];
    res.extend(error_msg_serialized);
    res
}

#[derive(Args, Debug)]
#[command(about = "Execute a deployment script")]
pub struct Run {
    /// Module name that contains the `main` function, which will be executed
    pub script_name: String,

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
        mut input_reader: BufferReader,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        let res = match selector {
            "call" => {
                let contract_address = input_reader.read_felt().into_();
                let function_selector = input_reader.read_felt().into_();
                let calldata = input_reader.read_vec();
                let calldata_felts: Vec<FieldElement> = calldata
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                match self.tokio_runtime.block_on(call::call(
                    contract_address,
                    function_selector,
                    calldata_felts,
                    self.provider,
                    &BlockId::Tag(Pending),
                )) {
                    Ok(call_response) => {
                        let mut res: Vec<Felt252> = vec![
                            Felt252::from(0),
                            Felt252::from(call_response.response.len()),
                        ];
                        res.extend(call_response.response.iter().map(|el| Felt252::from_(el.0)));
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                    Err(err) => {
                        let res = handle_starknet_command_error_in_script(&err);
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                }
            }
            "declare" => {
                let contract_name = input_reader.read_string();
                let max_fee = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);
                let nonce = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);

                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                match self.tokio_runtime.block_on(declare::declare(
                    &contract_name,
                    max_fee,
                    &account,
                    nonce,
                    self.artifacts,
                    WaitForTx {
                        wait: true,
                        wait_params: self.config.wait_params,
                    },
                )) {
                    Ok(declare_response) => {
                        let res: Vec<Felt252> = vec![
                            Felt252::from(0),
                            Felt252::from_(declare_response.class_hash.0),
                            Felt252::from_(declare_response.transaction_hash.0),
                        ];
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                    Err(err) => {
                        let res = handle_starknet_command_error_in_script(&err);
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                }
            }
            "deploy" => {
                let class_hash = input_reader.read_felt().into_();
                let constructor_calldata: Vec<FieldElement> = input_reader
                    .read_vec()
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();

                let salt = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);
                let unique = input_reader.read_bool();
                let max_fee = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);
                let nonce = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);

                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                match self.tokio_runtime.block_on(deploy::deploy(
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
                )) {
                    Ok(deploy_response) => {
                        let res: Vec<Felt252> = vec![
                            Felt252::from(0),
                            Felt252::from_(deploy_response.contract_address.0),
                            Felt252::from_(deploy_response.transaction_hash.0),
                        ];
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                    Err(err) => {
                        let res = handle_starknet_command_error_in_script(&err);
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                }
            }
            "invoke" => {
                let contract_address = input_reader.read_felt().into_();
                let function_selector = input_reader.read_felt().into_();
                let calldata: Vec<FieldElement> = input_reader
                    .read_vec()
                    .iter()
                    .map(|el| FieldElement::from_(el.clone()))
                    .collect();
                let max_fee = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);
                let nonce = input_reader
                    .read_option_felt()
                    .map(conversions::IntoConv::into_);

                let account = self.tokio_runtime.block_on(get_account(
                    &self.config.account,
                    &self.config.accounts_file,
                    self.provider,
                    self.config.keystore.clone(),
                ))?;

                match self.tokio_runtime.block_on(invoke::invoke(
                    contract_address,
                    function_selector,
                    calldata,
                    max_fee,
                    &account,
                    nonce,
                    WaitForTx {
                        wait: true,
                        wait_params: self.config.wait_params,
                    },
                )) {
                    Ok(invoke_response) => {
                        let res: Vec<Felt252> = vec![
                            Felt252::from(0),
                            Felt252::from_(invoke_response.transaction_hash.0),
                        ];
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                    Err(err) => {
                        let res = handle_starknet_command_error_in_script(&err);
                        Ok(CheatcodeHandlingResult::Handled(res))
                    }
                }
            }
            "get_nonce" => {
                let block_id = input_reader
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
