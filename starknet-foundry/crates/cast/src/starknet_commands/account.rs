use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use cast::{get_network, print_formatted, Network};
use clap::{Args, Subcommand};
use rand::rngs::OsRng;
use rand::RngCore;
use serde_json::json;
use starknet::accounts::{AccountDeployment, OpenZeppelinAccountFactory};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use std::collections::HashMap;

pub const OZ_CLASS_HASH: &str =
    "0x058d97f7d76e78f44905cc30cb65b91ea49a4b908a76703c54197bca90f81773";

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

        #[clap(short, long)]
        salt: Option<FieldElement>,

        #[clap(short, long, value_delimiter = ' ')]
        constructor_calldata: Vec<FieldElement>,

        /// If passed, a profile with corresponding data will be created in Scarb.toml
        #[clap(short, long)]
        as_profile: bool,
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

#[allow(clippy::too_many_arguments)]
pub fn create(
    maybe_output_path: Option<Utf8PathBuf>,
    maybe_name: Option<String>,
    maybe_network: Option<String>,
    maybe_salt: Option<FieldElement>,
    constructor_calldata: Vec<FieldElement>,
    save_as_profile: bool,
    int_format: bool,
    json: bool,
) -> Result<()> {
    let private_key = SigningKey::from_random();
    let public_key = private_key.verifying_key();
    let salt = match maybe_salt {
        Some(salt) => salt,
        None => FieldElement::from(OsRng.next_u64()),
    };

    let address = get_contract_address(
        salt,
        FieldElement::from_hex_be(OZ_CLASS_HASH).unwrap(),
        &constructor_calldata,
        FieldElement::ZERO,
    );

    let mut output: Vec<(&str, String)> = vec![
        ("private_key", format!("{:#x}", private_key.secret_scalar())),
        ("public_key", format!("{:#x}", public_key.scalar())),
        ("address", format!("{address:#x}")),
        ("salt", format!("{salt:#x}")),
    ];

    if let Some(output_path) = maybe_output_path {
        if !output_path.exists() {
            std::fs::create_dir_all(output_path.clone().parent().unwrap())?;
            std::fs::write(output_path.clone(), "{}")?;
        }

        match (maybe_name, maybe_network.clone()) {
            (Some(name), Some(network)) => {
                let contents = std::fs::read_to_string(output_path.clone())?;
                let mut items: serde_json::Value =
                    serde_json::from_str(&contents).expect("failed to parse json file");

                let network = get_network(&network)?.get_value();

                if !items[network][&name].is_null() {
                    bail!("Account with provided name already exists in this network")
                }

                let mut json_output: serde_json::Value = output
                    .into_iter()
                    .map(|(key, value)| (key, serde_json::Value::String(value)))
                    .collect();
                json_output["deployed"] = serde_json::Value::from(false);
                json_output["constructor_calldata"] = serde_json::Value::from(
                    constructor_calldata
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>(),
                );
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
