use crate::NestedMap;
use anyhow::Result;
use camino::Utf8PathBuf;
use std::collections::HashSet;
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

pub fn account_exists_in_accounts_file(
    account_name: &str,
    network_name: &str,
    accounts_file: &Utf8PathBuf,
) -> Result<bool> {
    let items = load_accounts(accounts_file)?;

    if items[network_name].is_null() {
        bail!("Network with name {network_name} does not exist in accounts file");
    }

    Ok(!items[network_name][account_name].is_null())
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
    let devnet_config = devnet_provider.get_config().await?;

    if account_number >= devnet_config.total_accounts {
        bail!(
            "Devnet account number must be between 1 and {}",
            devnet_config.total_accounts
        );
    }

    let devnet_accounts = devnet_provider.get_predeployed_accounts().await?;
    let predeployed_account = devnet_accounts
        .get((account_number - 1) as usize)
        .expect("Failed to get devnet account")
        .to_owned();

    let account_data = AccountData::from(predeployed_account);

    let chain_id = provider.chain_id().await?;

    build_account(account_data, chain_id, provider).await
}
