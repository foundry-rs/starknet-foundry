use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use promptly::prompt;
use serde_json::Map;
use sncast::response::structs::AccountDeleteResponse;

#[derive(Args, Debug)]
#[command(about = "Delete account information from the accounts file")]
pub struct Delete {
    /// Name of the account to be deleted
    #[clap(short, long)]
    pub name: String,

    /// Network where the account exists; defaults to network of rpc node
    #[clap(long)]
    pub network: Option<String>,

    /// Assume "yes" as answer to confirmation prompt and run non-interactively
    #[clap(long, default_value = "false")]
    pub yes: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn delete(
    name: &str,
    path: &Utf8PathBuf,
    network_name: &str,
    yes: bool,
) -> Result<AccountDeleteResponse> {
    let contents = std::fs::read_to_string(path.clone()).context("Failed to read accounts file")?;
    let items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {path}"))?;

    if items[&network_name].is_null() {
        bail!("No accounts defined for network = {network_name}");
    }
    if items[&network_name][&name].is_null() {
        bail!("Account with name {name} does not exist")
    }

    let mut items: Map<String, serde_json::Value> = serde_json::from_str(&contents)
        .unwrap_or_else(|_| panic!("Failed to read file at path = {path}"));

    // Let's ask confirmation
    if !yes {
        let prompt_text =
            format!("Do you want to remove the account {name} deployed to network {network_name} from local file {path}? (Y/n)");
        let input: String = prompt(prompt_text)?;

        if !input.starts_with('Y') {
            bail!("Delete aborted");
        }
    }

    // get to the nested object "nested"
    let nested = items
        .get_mut(network_name)
        .expect("Failed to find network")
        .as_object_mut()
        .expect("Failed to convert network");

    // now remove the child from there
    nested.remove(name);

    std::fs::write(path.clone(), serde_json::to_string_pretty(&items).unwrap())?;
    let result = "Account successfully removed".to_string();
    Ok(AccountDeleteResponse { result })
}
