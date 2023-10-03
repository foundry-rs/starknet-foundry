use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::response_structs::AccountDeleteResponse;
use cast::helpers::scarb_utils::get_scarb_manifest;
use clap::Args;
use serde_json::Map;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::io::Write;
use std::str::Split;

#[derive(Args, Debug)]
#[command(about = "Delete an account information from the accounts file")]
pub struct Delete {
    /// Name of the account to be deployed
    #[clap(short, long)]
    pub name: Option<String>,

    /// If passed, a profile with corresponding data will be removed in Scarb.toml
    #[clap(long)]
    pub delete_profile: bool,

    /// Network where the account exists
    #[clap(long)]
    pub network: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn delete(
    name: &str,
    path: &Utf8PathBuf,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    delete_profile: bool,
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

    let mut items: Map<String, Value> =
        serde_json::from_str(&contents).expect("failed to read file { path }");

    // Let's ask confirmation
    println!("Do you want to remove account {name} from network {network_name}? (Y/n)");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let mut result = "Account successfully removed".to_string();
    if input.starts_with('Y') {
        // get to the nested object "nested"
        let nested = items
            .get_mut(network_name)
            .expect("can't find network")
            .as_object_mut()
            .expect("invalid network format");

        // now remove the child from there
        nested.remove(name);

        std::fs::write(path.clone(), serde_json::to_string_pretty(&items).unwrap())?;
        // Remove profile from Scarb.toml
        if delete_profile {
            let profile_name = format!("[tool.sncast.{name}]");
            let manifest_path = match path_to_scarb_toml.clone() {
                Some(path) => path,
                None => {
                    get_scarb_manifest().context("Failed to obtain manifest path from scarb")?
                }
            };
            println!("Removing profile {profile_name} from {manifest_path}.");

            // Open Scarb file.
            let mut file = File::open(manifest_path.clone())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let eol = "\n";
            let mut lines: Split<&str> = contents.split(eol);

            // Find and remove profile.
            let mut filtered_lines: Vec<&str> = lines.clone().collect();
            let index = lines.position(|line| line == profile_name).unwrap();
            filtered_lines.remove(index); // Remove profile name
            filtered_lines.remove(index); // Remove account
            filtered_lines.remove(index); // Remove accounts-file
            filtered_lines.remove(index); // Remove url
            if index < filtered_lines.len() && filtered_lines[index].is_empty() {
                //Remove empty line
                filtered_lines.remove(index);
            }
            // Update Scarb file.
            let new_contents = filtered_lines.join("\n");
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(manifest_path)?;
            file.write_all(new_contents.as_bytes())?;
            file.flush()?;
        }
    } else {
        result = "Delete cancelled".to_string();
    };

    Ok(AccountDeleteResponse {
        result: result.to_string(),
    })
}
