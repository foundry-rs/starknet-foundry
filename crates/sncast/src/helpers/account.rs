use crate::response::ui::UI;
use crate::{
    AccountVariant, NestedMap, build_account_variant, build_signer, check_account_file_exists,
    helpers::devnet::provider::DevnetProvider,
};
use anyhow::{Context, Result, ensure};
use camino::Utf8PathBuf;
use starknet_rust::providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport};
use std::collections::{HashMap, HashSet};
use std::fs;
use url::Url;

use crate::{AccountData, read_and_parse_json_file};
use serde_json::{Value, json};

pub fn generate_account_name(accounts_file: &Utf8PathBuf) -> Result<String> {
    let mut id = 1;

    if !accounts_file.exists() {
        return Ok(format!("account-{id}"));
    }

    let networks: NestedMap<AccountData> = read_and_parse_json_file(accounts_file)?;
    let mut result = HashSet::new();

    for (_, accounts) in networks {
        for (name, _) in accounts {
            if let Some(id) = name
                .strip_prefix("account-")
                .and_then(|id| id.parse::<u32>().ok())
            {
                result.insert(id);
            }
        }
    }

    while result.contains(&id) {
        id += 1;
    }

    Ok(format!("account-{id}"))
}

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
    accounts_file: &Utf8PathBuf,
    accounts_file_should_exist: bool,
) -> Result<bool> {
    if !accounts_file.exists() {
        if accounts_file_should_exist {
            check_account_file_exists(accounts_file)?;
        }
        return Ok(false);
    }

    let accounts: HashMap<String, HashMap<String, AccountData>> =
        read_and_parse_json_file(accounts_file)?;

    match accounts.get(network_name) {
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

pub async fn get_account_from_devnet<'a>(
    account: &str,
    provider: &'a JsonRpcClient<HttpTransport>,
    url: &Url,
    ui: &UI,
) -> Result<AccountVariant<'a>> {
    let account_number: u8 = account
        .strip_prefix("devnet-")
        .map(|s| s.parse::<u8>().expect("Invalid devnet account number"))
        .context("Failed to parse devnet account number")?;

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
        account_number <= devnet_config.total_accounts && account_number != 0,
        "Devnet account number must be between 1 and {}",
        devnet_config.total_accounts
    );

    let devnet_accounts = devnet_provider.get_predeployed_accounts().await?;
    let predeployed_account = devnet_accounts
        .get((account_number - 1) as usize)
        .expect("Failed to get devnet account")
        .to_owned();

    let account_data = AccountData::from(predeployed_account);
    let chain_id = provider.chain_id().await?;
    let signer = build_signer(&account_data.signer_type, ui, false).await?;
    build_account_variant(signer, account_data, chain_id, provider).await
}
