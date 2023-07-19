use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use cast::{get_network, print_formatted, Network};
use clap::{Args, Subcommand};
use serde_json::json;
use starknet::core::types::FieldElement;
use starknet::signers::SigningKey;
use std::collections::HashMap;

#[derive(Args)]
#[command(about = "Creates and deploys an account to the Starknet")]
pub struct Account {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Create {
        /// Output path to the file where the account secrets are going to be saved
        #[clap(short, long)]
        output_path: Option<Utf8PathBuf>,

        /// Account name under which secrets are going to be saved
        #[clap(short, long)]
        name: Option<String>,

        /// If passed, a profile with corresponding data will be created in Scarb.toml
        #[clap(short, long)]
        save_as_profile: bool,
        // TODO: think about supporting different account providers
    },
    Deploy {
        /// Path to the file where the account secrets are stored
        #[clap(short, long)]
        path: Utf8PathBuf,

        /// Name of the account to be deployed
        #[clap(short, long)]
        name: String,

        #[clap(short, long)]
        max_fee: FieldElement,
    },
}

pub fn create(
    maybe_output_path: Option<Utf8PathBuf>,
    maybe_name: Option<String>,
    maybe_network: Option<String>,
    save_as_profile: bool,
    int_format: bool,
    json: bool,
) -> Result<()> {
    let private_key = SigningKey::from_random();
    let public_key = private_key.verifying_key();

    let mut output: Vec<(&str, String)> = vec![
        ("private_key", private_key.secret_scalar().to_string()),
        ("public_key", public_key.scalar().to_string()),
    ];

    if let Some(output_path) = maybe_output_path {
        if !output_path.exists() {
            std::fs::write(output_path.clone(), "")?;
        }

        match (maybe_name, maybe_network) {
            (Some(name), Some(network)) => {
                let contents = std::fs::read_to_string(output_path.clone())?;
                let mut items: serde_json::Value =
                    serde_json::from_str(&contents).expect("failed to parse json file");

                let network = get_network(&network)?.get_value();

                let mut json_output: serde_json::Value = output
                    .into_iter()
                    .map(|(key, value)| (key, serde_json::Value::String(value)))
                    .collect();
                json_output["deployed"] = serde_json::Value::from(false);

                items[network][name] = json_output;

                std::fs::write(
                    output_path.clone(),
                    serde_json::to_string_pretty(&items).unwrap(),
                )
                .expect("xxx");
            }
            (_, None) => bail!("Argument `network` has to be passed when `output-path` provided"),
            (None, _) => bail!("Argument `name` has to be passed when `output-path` provided"),
        }
    } else {
        output.push(("command", "Create account".to_string()));
        print_formatted(output, int_format, json, false).expect("Couldn't print account secrets");
    }

    Ok(())
}
