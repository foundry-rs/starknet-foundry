use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::response_structs::AccountDeleteResponse;
use cast::parse_number;
use clap::Args;
use serde_json::Map;
use serde_json::Value;

#[derive(Args, Debug)]
#[command(about = "Delete an account with all important secrets")]
pub struct Delete {
    /// Name of the account to be deployed
    #[clap(short, long)]
    pub name: Option<String>,

    /// Custom open zeppelin contract class hash of declared contract
    #[clap(long)]
    pub network: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn delete(name: &str, path: &Utf8PathBuf, network_name: &str) -> Result<AccountDeleteResponse> {
    let contents = std::fs::read_to_string(path.clone()).context("Couldn't read accounts file")?;
    let items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {path}"))?;

    if items[&network_name].is_null() {
        bail!("No accounts defined for network {}", network_name);
    }
    if items[&network_name][&name].is_null() {
        bail!("Account with name {name} does not exist")
    }

    let address = parse_number(
        items[&network_name][&name]["address"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid address"))?,
    )?;

    let mut items: Map<String, Value> =
        serde_json::from_str(&contents).expect("failed to read file");

    // get to the nested object "nested"
    let nested = items
        .get_mut(network_name)
        .expect("should exist")
        .as_object_mut()
        .expect("should be an object");

    // now remove the child from there
    nested.remove(name);

    std::fs::write(path.clone(), serde_json::to_string_pretty(&items).unwrap())?;

    println!("Account successfully removed");
    Ok(AccountDeleteResponse { address })
}
