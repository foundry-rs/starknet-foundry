use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::response_structs::AccountDeleteResponse;
use cast::helpers::scarb_utils::get_scarb_manifest;
use clap::Args;
use promptly::prompt;
use serde_json::Map;
use std::fs::File;
use std::io::Read;
use std::io::Write;

#[derive(Args, Debug)]
#[command(about = "Delete account information from the accounts file")]
pub struct Delete {
    /// Name of the account to be deleted
    #[clap(short, long)]
    pub name: Option<String>,

    /// If passed with false, Scarb profile won't be removed
    #[clap(long, num_args = 1, default_value = "true")]
    pub delete_profile: Option<bool>,

    /// Network where the account exists; defaults to network of rpc node
    #[clap(long)]
    pub network: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn delete(
    name: &str,
    path: &Utf8PathBuf,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    delete_profile: Option<bool>,
    network_name: &str,
) -> Result<AccountDeleteResponse> {
    let contents = std::fs::read_to_string(path.clone()).context("Couldn't read accounts file")?;
    let items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {path}"))?;

    if items[&network_name].is_null() {
        bail!("No accounts defined for network {}", network_name);
    }
    if items[&network_name][&name].is_null() {
        bail!("Account with name {name} does not exist")
    }

    let mut items: Map<String, serde_json::Value> =
        serde_json::from_str(&contents).expect("failed to read file { path }");

    // Let's ask confirmation
    let prompt_text =
        format!("Do you want to remove the account {name} deployed to network {network_name} from local file {path}? (Y/n)");
    let input: String = prompt(prompt_text)?;

    if !input.starts_with('Y') {
        bail!("Delete aborted");
    }

    // get to the nested object "nested"
    let nested = items
        .get_mut(network_name)
        .expect("can't find network")
        .as_object_mut()
        .expect("invalid network format");

    // now remove the child from there
    nested.remove(name);

    std::fs::write(path.clone(), serde_json::to_string_pretty(&items).unwrap())?;

    let mut scarb_result = "Account not removed from Scarb.toml".to_string();
    // delete profile if delete_profile is true or not passed
    if delete_profile == Some(true) {
        let manifest_path = match path_to_scarb_toml.clone() {
            Some(path) => path,
            None => get_scarb_manifest().context("Failed to obtain manifest path from scarb")?,
        };
        let mut toml_content = String::new();
        let mut file = File::open(manifest_path.clone()).expect("Failed to open file");
        file.read_to_string(&mut toml_content)
            .expect("Failed to read file");

        // Parse the TOML content
        let mut parsed_toml: toml::Value =
            toml::de::from_str(&toml_content).expect("Failed to parse TOML");

        // Remove the nested section from the Value object.
        if let Some(table) = parsed_toml
            .get_mut("tool")
            .and_then(toml::Value::as_table_mut)
        {
            if let Some(nested_table) = table.get_mut("sncast").and_then(toml::Value::as_table_mut)
            {
                if nested_table.remove(name).is_some() {
                    scarb_result = "Account removed from Scarb.toml".to_string();
                }
            }
        }

        // Serialize the modified TOML data back to a string
        let modified_toml = toml::to_string(&parsed_toml).expect("Failed to serialize TOML");

        // Write the modified content back to the file
        let mut file = File::create(manifest_path).expect("Failed to create file");
        file.write_all(modified_toml.as_bytes())
            .expect("Failed to write to file");
    };

    let result = "Account successfully removed".to_string();
    Ok(AccountDeleteResponse {
        result,
        scarb_result,
    })
}
