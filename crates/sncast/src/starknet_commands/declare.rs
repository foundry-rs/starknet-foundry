use anyhow::{Context, Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use foundry_ui::OutputFormat;
use shared::rpc::get_starknet_version;
use sncast::helpers::artifacts::resolve_contract_artifacts;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::dry_run::DryRunArgs;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::{RpcArgs, generate_network_flag};
use sncast::helpers::scarb_utils::{
    BuildConfig, assert_manifest_path_exists, build_and_load_artifacts, get_package_metadata,
};
use sncast::response::declare::{
    AlreadyDeclaredResponse, DeclareResponse, DeclareTransactionResponse, DeployCommandMessage,
};
use sncast::response::errors::{
    SNCastProviderError, SNCastStarknetError, StarknetCommandError, handle_starknet_command_error,
};
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::ui::UI;
use sncast::response::verify::VerifyResponse;
use sncast::{WaitForTx, apply_optional_fields, get_account, handle_wait_for_tx, with_account};
use starknet_rust::accounts::AccountError::Provider;
use starknet_rust::accounts::{ConnectedAccount, DeclarationV3};
use starknet_rust::core::types::{
    ContractExecutionError, DeclareTransactionResult, StarknetError, TransactionExecutionErrorData,
};
use starknet_rust::providers::{Provider as _, ProviderError};
use starknet_rust::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{CompiledClass, SierraClass},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::Signer,
};
use starknet_types_core::felt::Felt;
use std::process::ExitCode;
use std::sync::Arc;
use universal_sierra_compiler_api::compile_contract_sierra;

use crate::starknet_commands::verify::explorer::ContractIdentifier;
use crate::starknet_commands::verify::{
    VerifyCommonArgs, resolve_verification_network, verify_contract,
};
use shared::utils::contract_name_from_module_path;

/// Common args shared by declare command variants.
#[derive(Args)]
pub struct DeclareCommonArgs {
    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub dry_run_args: DryRunArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,
}

#[derive(Args)]
#[command(about = "Declare a contract to starknet", long_about = None)]
#[command(mut_arg("verifier", |arg| arg.required(false)))]
pub struct Declare {
    /// Contract name or module tree path
    #[arg(short = 'c', long)]
    pub contract_name: String,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,

    /// If passed, omits ABI from the declared Sierra class. This changes the resulting class hash
    #[arg(long)]
    pub no_abi: bool,

    #[command(flatten)]
    pub common: DeclareCommonArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(flatten)]
    pub verify: Option<VerifyCommonArgs>,
}

pub async fn declare(
    args: Declare,
    mut wait_config: WaitForTx,
    config: CastConfig,
    ui: &UI,
) -> Result<ExitCode> {
    if args.verify.is_some() {
        // Force waiting for deployment before verification
        wait_config.wait = true;
    }

    let provider = args.rpc.get_provider(&config, ui).await?;
    let account = get_account(&config, &provider, &args.rpc, ui).await?;
    let scarb_toml_path = assert_manifest_path_exists()?;
    let package = get_package_metadata(&scarb_toml_path, &args.package)?;
    let base_ui = ui.base_ui();

    let artifacts = build_and_load_artifacts(
        &package,
        &BuildConfig {
            scarb_toml_path,
            json: base_ui.output_format() == OutputFormat::Json,
            profile: config.scarb_profile.clone(),
        },
        base_ui,
    )
    .context("Failed to build contract")?;

    let contract_artifacts = resolve_contract_artifacts(&args.contract_name, &artifacts)?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let casm_contract_definition: CompiledClass =
        serde_json::from_str(&contract_artifacts.casm).context("Failed to parse casm artifact")?;

    let result = with_account!(&account, |account| declare_with_artifacts(
        contract_definition,
        casm_contract_definition,
        args.common,
        args.no_abi,
        account,
        wait_config,
        false,
        ui,
    )
    .await)
    .map_err(handle_starknet_command_error)?;

    let mut response = match result {
        DeclareResponse::Success(response) => response,
        DeclareResponse::DryRun(response) => {
            return Ok(process_command_result("declare", Ok(response), ui, None));
        }
        DeclareResponse::AlreadyDeclared(_) => {
            unreachable!("Argument `skip_on_already_declared` is false")
        }
    };

    if let Some(verifier) = args.verify {
        let workspace_dir = package
            .manifest_path
            .parent()
            .ok_or(anyhow!("Failed to obtain workspace dir"))?;

        let network =
            resolve_verification_network(None, config.network_params.network(), &provider).await?;

        let verification_result = verify_contract(
            verifier,
            ContractIdentifier::ClassHash {
                class_hash: response.class_hash.0.to_fixed_hex_string(),
            },
            contract_name_from_module_path(&args.contract_name).to_string(),
            args.package.clone(),
            &provider,
            network,
            workspace_dir.to_path_buf(),
            ui,
        )
        .await;

        match verification_result {
            Ok(VerifyResponse { message }) => response.verification_message = Some(message),
            Err(e) => ui.print_error("declare", format!("Failed to verify the contract: {e}")),
        }
    }

    let contract_artifacts = resolve_contract_artifacts(&args.contract_name, &artifacts)
        .context("Failed to get contract artifacts")?;
    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let network_flag = generate_network_flag(&args.rpc, &config);

    let deploy_message = DeployCommandMessage::new(
        &contract_definition.abi,
        args.no_abi,
        &response,
        &config.account,
        &config.accounts_file,
        config.keystore.as_ref(),
        network_flag,
    );

    let response = Ok(response);

    let block_explorer_link =
        block_explorer_link_if_allowed(&response, provider.chain_id().await?, &config).await;

    let res = process_command_result("declare", response, ui, block_explorer_link);

    ui.print_notification(deploy_message?);

    Ok(res)
}

#[expect(clippy::result_large_err)]
pub fn compile_sierra_to_casm(
    sierra_class: &SierraClass,
) -> Result<CompiledClass, StarknetCommandError> {
    let casm_json: String = serde_json::to_string(
        &compile_contract_sierra(
            &serde_json::to_value(sierra_class)
                .with_context(|| "Failed to convert sierra to JSON value".to_string())?,
        )
        .with_context(|| "Failed to compile sierra to casm".to_string())?,
    )
    .expect("serialization should succeed");

    let casm: CompiledClass = serde_json::from_str(&casm_json)
        .with_context(|| "Failed to deserialize casm JSON into CompiledClass".to_string())?;
    Ok(casm)
}

#[expect(clippy::too_many_lines)]
#[expect(clippy::too_many_arguments)]
#[expect(clippy::result_large_err)]
pub async fn declare_with_artifacts<S>(
    mut sierra_class: SierraClass,
    compiled_casm: CompiledClass,
    common: DeclareCommonArgs,
    no_abi: bool,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, S>,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
    ui: &UI,
) -> Result<DeclareResponse, StarknetCommandError>
where
    S: Signer + Sync + Send,
{
    let starknet_version = get_starknet_version(account.provider()).await?;
    let hash_function = CompiledClass::hash_function_from_starknet_version(&starknet_version)
        .ok_or(anyhow!("Unsupported Starknet version: {starknet_version}"))?;
    let casm_class_hash = compiled_casm
        .class_hash_with_hash_function(hash_function)
        .map_err(anyhow::Error::from)?;

    if no_abi {
        sierra_class.abi.clear();
    }

    let class_hash = sierra_class.class_hash().map_err(anyhow::Error::from)?;

    let declaration = account.declare_v3(
        Arc::new(sierra_class.flatten().map_err(anyhow::Error::from)?),
        casm_class_hash,
    );

    if common.dry_run_args.dry_run {
        return common
            .dry_run_args
            .estimate(|| declaration.estimate_fee())
            .await
            .map(DeclareResponse::DryRun)
            .map_err(|e| {
                StarknetCommandError::from(anyhow!("Failed to estimate fee for dry run: {e}"))
            });
    }

    let fee_settings = if common.fee_args.max_fee.is_some() {
        let fee_estimate = declaration
            .estimate_fee()
            .await
            .map_err(|error| anyhow!("Failed to estimate fee: {error}"))?;
        common.fee_args.try_into_fee_settings(Some(&fee_estimate))
    } else {
        common.fee_args.try_into_fee_settings(None)
    };

    let FeeSettings {
        l1_gas,
        l1_gas_price,
        l2_gas,
        l2_gas_price,
        l1_data_gas,
        l1_data_gas_price,
        tip,
    } = fee_settings?;

    let declaration = apply_optional_fields!(
        declaration,
        l1_gas => DeclarationV3::l1_gas,
        l1_gas_price => DeclarationV3::l1_gas_price,
        l2_gas => DeclarationV3::l2_gas,
        l2_gas_price => DeclarationV3::l2_gas_price,
        l1_data_gas => DeclarationV3::l1_data_gas,
        l1_data_gas_price => DeclarationV3::l1_data_gas_price,
        tip => DeclarationV3::tip,
        common.nonce => DeclarationV3::nonce
    );

    let declared = declaration.send().await;

    match declared {
        Ok(DeclareTransactionResult {
            transaction_hash,
            class_hash,
        }) => handle_wait_for_tx(
            account.provider(),
            transaction_hash,
            DeclareResponse::Success(DeclareTransactionResponse {
                class_hash: class_hash.into_(),
                transaction_hash: transaction_hash.into_(),
                verification_message: None,
            }),
            wait_config,
            ui,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(ProviderError::StarknetError(StarknetError::ClassAlreadyDeclared)))
            if skip_on_already_declared =>
        {
            Ok(DeclareResponse::AlreadyDeclared(AlreadyDeclaredResponse {
                class_hash: class_hash.into_(),
            }))
        }
        Err(Provider(ProviderError::StarknetError(StarknetError::ClassAlreadyDeclared))) => Err(
            StarknetCommandError::ProviderError(SNCastProviderError::StarknetError(
                SNCastStarknetError::ClassAlreadyDeclared(class_hash.into_()),
            )),
        ),
        Err(Provider(ProviderError::StarknetError(StarknetError::TransactionExecutionError(
            TransactionExecutionErrorData {
                execution_error: ContractExecutionError::Message(message),
                ..
            },
        )))) if message.contains("is already declared") => {
            if skip_on_already_declared {
                Ok(DeclareResponse::AlreadyDeclared(AlreadyDeclaredResponse {
                    class_hash: class_hash.into_(),
                }))
            } else {
                Err(StarknetCommandError::ProviderError(
                    SNCastProviderError::StarknetError(SNCastStarknetError::ClassAlreadyDeclared(
                        class_hash.into_(),
                    )),
                ))
            }
        }
        Err(Provider(ProviderError::StarknetError(error))) => Err(
            StarknetCommandError::ProviderError(SNCastProviderError::StarknetError(
                SNCastStarknetError::from_starknet_error_with_account(
                    error,
                    account.address().into_(),
                ),
            )),
        ),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
