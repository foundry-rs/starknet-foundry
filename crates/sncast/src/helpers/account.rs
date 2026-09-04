use crate::helpers::devnet::provider::DevnetProvider;
use anyhow::{Result, ensure};
use camino::Utf8PathBuf;
use starknet_rust::providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport};
use std::fs;
use std::num::NonZeroU8;
use url::Url;

use crate::accounts::{AccountRecord, AccountRepository, AccountService};
use crate::signers::{RuntimeSigner, spec::PrivateKeySource};
use crate::{RuntimeAccount, check_account_file_exists};
use anyhow::Context;
use serde_json::{Value, json};

pub fn load_accounts(accounts_file: &Utf8PathBuf) -> Result<Value> {
    let contents = fs::read_to_string(accounts_file).context("Failed to read accounts file")?;

    if contents.trim().is_empty() {
        return Ok(json!({}));
    }

    let accounts = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse accounts file at = {accounts_file}"))?;

    Ok(accounts)
}

/// When `accounts_file_should_exist` is `true`, a missing accounts file or network entry is an error.
/// It is set to `false` only for devnet accounts, which don't require the default accounts file to exist.
pub fn check_account_exists(
    account_name: &str,
    network_name: &str,
    repository: &AccountRepository,
    accounts_file_should_exist: bool,
) -> Result<bool> {
    let accounts_file_exists = repository.exists();
    if !accounts_file_exists {
        if accounts_file_should_exist {
            check_account_file_exists(repository)?;
        }
        return Ok(false);
    }

    let registry = repository.load()?.registry;
    match registry.networks().get(network_name) {
        Some(network_accounts) => Ok(network_accounts.contains_key(account_name)),
        None if accounts_file_should_exist => Err(anyhow::anyhow!(
            "Network with name {network_name} does not exist in accounts file"
        )),
        None => Ok(false),
    }
}

#[must_use]
pub fn is_devnet_account(account: &str) -> bool {
    account.starts_with("devnet-")
}

pub(crate) async fn get_account_from_devnet<'a>(
    account_number: NonZeroU8,
    provider: &'a JsonRpcClient<HttpTransport>,
    url: &Url,
) -> Result<RuntimeAccount<'a>> {
    let devnet_provider = DevnetProvider::new(url.as_ref());
    devnet_provider.ensure_alive().await?;

    let devnet_config = devnet_provider.get_config().await;
    let devnet_config = match devnet_config {
        Ok(config) => config,
        Err(err) => {
            return Err(err);
        }
    };

    ensure!(
        account_number.get() <= devnet_config.total_accounts,
        "Devnet account number must be between 1 and {}",
        devnet_config.total_accounts
    );

    let devnet_accounts = devnet_provider.get_predeployed_accounts().await?;
    let predeployed_account = devnet_accounts
        .get((account_number.get() - 1) as usize)
        .expect("Failed to get devnet account");

    let account_data = AccountRecord::from(predeployed_account);
    let chain_id = provider.chain_id().await?;
    let private_key = account_data
        .signer
        .private_key()
        .context("Private key not found for devnet account")?;
    let signer = RuntimeSigner::from_private_key(private_key, PrivateKeySource::PrivateKey);
    AccountService::build_runtime_account(account_data, chain_id, provider, signer).await
}
