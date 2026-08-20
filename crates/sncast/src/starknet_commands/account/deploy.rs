use anyhow::{Result, anyhow};
use clap::Args;
use sncast::accounts::{AccountDeploymentService, AccountRecord, AccountRepository};
use sncast::helpers::dry_run::DryRunArgs;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::deploy::AccountDeployResponse;
use sncast::response::invoke::InvokeResponse;
use sncast::response::ui::UI;
use sncast::signers::{SignerProviderContext, SignerResolver};
use sncast::{
    AccountType, WaitForTx, chain_id_to_network_name, get_account_record_from_repository,
};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[arg(short, long)]
    pub name: Option<String>,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub dry_run_args: DryRunArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[arg(long)]
    pub silent: bool,
}

#[expect(clippy::too_many_arguments)]
pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    repository: &AccountRepository,
    deploy_args: &Deploy,
    chain_id: Felt,
    wait_config: WaitForTx,
    fee_args: FeeArgs,
    dry_run_args: DryRunArgs,
    ui: &UI,
) -> Result<AccountDeployResponse> {
    let account_name = deploy_args
        .name
        .clone()
        .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
    anyhow::ensure!(repository.exists()?);
    deploy_from_accounts_file(
        provider,
        repository,
        account_name,
        chain_id,
        fee_args,
        dry_run_args,
        wait_config,
        ui,
    )
    .await
    .map(Into::into)
}

#[expect(clippy::too_many_arguments)]
async fn deploy_from_accounts_file(
    provider: &JsonRpcClient<HttpTransport>,
    repository: &AccountRepository,
    name: String,
    chain_id: Felt,
    fee_args: FeeArgs,
    dry_run_args: DryRunArgs,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse> {
    let account_data = get_account_record_from_repository(&name, chain_id, repository)?;
    let (account_type, class_hash, salt) = extract_deployment_fields(&account_data)?;
    let context = SignerProviderContext { repository, ui };
    let signer = SignerResolver::default()
        .resolve_and_verify(&account_data.signer, account_data.public_key, &context)
        .await?;
    let (_, result) = AccountDeploymentService::deploy(
        provider,
        account_type,
        class_hash,
        signer,
        salt,
        chain_id,
        fee_args,
        dry_run_args,
        wait_config,
        ui,
    )
    .await?;

    if let InvokeResponse::Transaction(_) = &result {
        update_account_in_accounts_file(repository, &name, chain_id)?;
    }

    Ok(result)
}

fn extract_deployment_fields(account: &AccountRecord) -> Result<(AccountType, Felt, Felt)> {
    let deployable = account.as_deployable()?;
    Ok((
        deployable.account_type(),
        deployable.class_hash(),
        deployable.salt(),
    ))
}

fn update_account_in_accounts_file(
    repository: &AccountRepository,
    account_name: &str,
    chain_id: Felt,
) -> Result<()> {
    let network_name = chain_id_to_network_name(chain_id);

    repository
        .mutate(|registry| {
            let account = registry
                .networks_mut()
                .get_mut(network_name.as_str())
                .and_then(|accounts| accounts.get_mut(account_name))
                .ok_or_else(|| sncast::accounts::AccountsError::AccountNotFound {
                    network: network_name.clone(),
                    account: account_name.to_owned(),
                })?;
            account.deployed = Some(true);
            Ok(())
        })
        .map_err(|error| anyhow!(error))?;

    Ok(())
}
