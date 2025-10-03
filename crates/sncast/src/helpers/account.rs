use crate::{
    NestedMap, build_account, check_account_file_exists, helpers::devnet_provider::DevnetProvider,
};
use anyhow::{Result, ensure};
use camino::Utf8PathBuf;
use starknet::{
    accounts::SingleOwnerAccount,
    providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport},
    signers::LocalWallet,
};
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::{AccountData, read_and_parse_json_file};
use anyhow::Context;
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

pub fn check_account_exists(
    account_name: &str,
    network_name: &str,
    accounts_file: &Utf8PathBuf,
) -> Result<bool> {
    check_account_file_exists(accounts_file)?;

    let accounts: HashMap<String, HashMap<String, AccountData>> =
        read_and_parse_json_file(accounts_file)?;

    accounts
        .get(network_name)
        .map(|network_accounts| network_accounts.contains_key(account_name))
        .ok_or_else(|| {
            anyhow::anyhow!("Network with name {network_name} does not exist in accounts file")
        })
}

#[must_use]
pub fn is_devnet_account(account: &str) -> bool {
    account.starts_with("devnet-")
}

pub async fn get_account_from_devnet<'a>(
    account: &str,
    provider: &'a JsonRpcClient<HttpTransport>,
    url: &str,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    let account_number: u8 = account
        .strip_prefix("devnet-")
        .map(|s| s.parse::<u8>().expect("Invalid devnet account number"))
        .context("Failed to parse devnet account number")?;

    let devnet_provider = DevnetProvider::new(url);
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
    build_account(account_data, chain_id, provider).await
}
