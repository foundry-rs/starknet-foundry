use std::fs::OpenOptions;
use std::io::Write;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cast::{get_network, parse_number, print_formatted};
use clap::{Args, Subcommand};
use rand::rngs::OsRng;
use rand::RngCore;
use scarb::ops::find_manifest_path;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use toml::map::Map;
use toml::Value;
use crate::helpers::scarb_utils::get_property;

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

        // If passed, a profile with corresponding data will be created in Scarb.toml
        #[clap(short, long)]
        add_profile: bool,

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
        max_fee: Option<FieldElement>,
    },
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    provider: &JsonRpcClient<HttpTransport>,
    url: String,
    maybe_output_path: Option<Utf8PathBuf>,
    maybe_name: Option<String>,
    network: &str,
    maybe_salt: Option<FieldElement>,
    add_profile: bool,
) -> Result<Vec<(&'static str, String)>> {
    let private_key = SigningKey::from_random();
    let public_key = private_key.verifying_key();
    let salt = match maybe_salt {
        Some(salt) => salt,
        None => FieldElement::from(OsRng.next_u64()),
    };

    let address = get_contract_address(
        salt,
        FieldElement::from_hex_be(OZ_CLASS_HASH)?,
        &[public_key.scalar()],
        FieldElement::ZERO,
    );

    let max_fee = {
        let signer = LocalWallet::from_signing_key(private_key.clone());
        let factory = OpenZeppelinAccountFactory::new(
            parse_number(OZ_CLASS_HASH)?,
            get_network(network)?.get_chain_id(),
            signer,
            provider,
        )
        .await?;
        let deployment = factory.deploy(salt);

        deployment.estimate_fee().await?.overall_fee
    };

    let output: Vec<(&str, String)> = vec![
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

        return match maybe_name {
            Some(name) => {
                let contents = std::fs::read_to_string(output_path.clone())?;
                let mut items: serde_json::Value =
                    serde_json::from_str(&contents).expect("failed to parse json file");

                let network_value = get_network(network)?.get_value();

                if !items[network_value][&name].is_null() {
                    return Err(anyhow!(
                        "Account with provided name already exists in this network"
                    ));
                }

                let mut json_output: serde_json::Value = output
                    .into_iter()
                    .map(|(key, value)| (key, serde_json::Value::String(value)))
                    .collect();
                json_output["deployed"] = serde_json::Value::from(false);
                items[network_value][&name] = json_output;

                std::fs::write(
                    output_path.clone(),
                    serde_json::to_string_pretty(&items).unwrap(),
                )?;


                add_created_profile_to_configuration(name, network.to_string(), url);

                Ok(vec![(
                    "message",
                    format!("Account successfully created. Prefund generated address with at least {max_fee} tokens. \
                     It is good to send more in the case of higher demand, max_fee * 2 = {}", max_fee * 2),
                )])
            }
            None => Err(anyhow!(
                "Argument `name` has to be passed when `output-path` provided"
            )),
        };
    }

    Ok(output)
}

pub fn print_account_create_result(
    account_create_result: Result<Vec<(&'static str, String)>>,
    int_format: bool,
    json: bool,
) -> Result<()> {
    match account_create_result {
        Ok(mut values) => {
            values.insert(0, ("command", "Create account".to_string()));
            print_formatted(values, int_format, json, false)?;
        }
        Err(error) => {
            print_formatted(vec![("error", error.to_string())], int_format, json, true)?;
        }
    };

    Ok(())
}

pub fn add_created_profile_to_configuration(name: String, network: String, url: String) -> String {
    let manifest_path = find_manifest_path(None).expect("Failed to obtain Scarb.toml file path");

    let metadata = scarb_metadata::MetadataCommand::new()
        .inherit_stderr()
        .manifest_path(&manifest_path)
        .no_deps()
        .exec()
        .context(
            "Failed to read Scarb.toml manifest file, not found in current nor parent directories",
        ).unwrap();

    let package = &metadata.packages[0].manifest_metadata.tool;

    println!("dupa");
    dbg!(package);

    let property = get_property(package, &Some(name.clone()), "account");
    if property.is_ok() {
        return format!("Failed to add {name} profile to the Scarb.toml. Profile already exists")
    }

    let mut tool_sncast = toml::value::Table::new();
    let mut myprofile = toml::value::Table::new();

    myprofile.insert("network".to_string(), Value::String(network));
    myprofile.insert("url".to_string(), Value::String(url));
    myprofile.insert("account".to_string(), Value::String(name.clone()));

    tool_sncast.insert(name, Value::Table(myprofile));

    let mut tool = toml::value::Table::new();
    tool.insert("sncast".to_string(), Value::Table(tool_sncast));

    let mut config = toml::value::Table::new();
    config.insert("tool".to_string(), Value::Table(tool));

    let toml_string = toml::to_string(&Value::Table(config)).unwrap();

    let mut scarb_toml = OpenOptions::new().append(true).open(manifest_path).unwrap();
    scarb_toml.write_all(format!("\n{toml_string}").as_bytes()).unwrap();

    "abc".to_string()
}
