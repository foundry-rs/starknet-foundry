use crate::helpers::constants::OZ_CLASS_HASH;
use crate::helpers::response_structs::AccountCreateResponse;
use crate::helpers::scarb_utils::{get_property, get_scarb_manifest};
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cast::{extract_or_generate_salt, get_network, parse_number};
use clap::Args;
use serde_json::json;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use std::fs::OpenOptions;
use std::io::Write;
use toml::Value;

#[derive(Args, Debug)]
#[command(about = "Create an account with all important secrets")]
pub struct Create {
    /// Account name under which account information is going to be saved
    #[clap(short, long)]
    pub name: String,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(short, long)]
    pub add_profile: bool,
    // TODO (#253): think about supporting different account providers
    /// Custom open zeppelin contract class hash of declared contract
    #[clap(short, long)]
    pub class_hash: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    provider: &JsonRpcClient<HttpTransport>,
    url: String,
    accounts_file_path: Utf8PathBuf,
    path_to_scarb_toml: Option<Utf8PathBuf>,
    name: String,
    network: &str,
    maybe_salt: Option<FieldElement>,
    add_profile: bool,
    class_hash: Option<String>,
) -> Result<AccountCreateResponse> {
    let private_key = SigningKey::from_random();
    let public_key = private_key.verifying_key();
    let salt = extract_or_generate_salt(maybe_salt);
    let oz_class_hash: &str = if let Some(value) = &class_hash {
        value
    } else {
        OZ_CLASS_HASH
    };

    let address = get_contract_address(
        salt,
        parse_number(oz_class_hash)?,
        &[public_key.scalar()],
        FieldElement::ZERO,
    );

    let max_fee = {
        let signer = LocalWallet::from_signing_key(private_key.clone());
        let factory = OpenZeppelinAccountFactory::new(
            parse_number(oz_class_hash)?,
            get_network(network)?.get_chain_id(),
            signer,
            provider,
        )
        .await?;
        let deployment = factory.deploy(salt);

        deployment.estimate_fee().await?.overall_fee
    };

    if !accounts_file_path.exists() {
        std::fs::create_dir_all(accounts_file_path.clone().parent().unwrap())?;
        std::fs::write(accounts_file_path.clone(), "{}")?;
    }

    let contents = std::fs::read_to_string(accounts_file_path.clone())?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {accounts_file_path}"))?;

    let network_value = get_network(network)?.get_value();

    if !items[network_value][&name].is_null() {
        return Err(anyhow!(
            "Account with provided name already exists in this network"
        ));
    }

    items[network_value][&name] = json!({
        "private_key": format!("{:#x}", private_key.secret_scalar()),
        "public_key": format!("{:#x}", public_key.scalar()),
        "address": format!("{address:#x}"),
        "salt": format!("{salt:#x}"),
        "deployed": false,
    });

    if add_profile {
        match add_created_profile_to_configuration(
            &path_to_scarb_toml,
            name,
            network.to_string(),
            url,
        ) {
            Ok(()) => {}
            Err(err) => return Err(anyhow!(err)),
        };
    }

    std::fs::write(
        accounts_file_path.clone(),
        serde_json::to_string_pretty(&items).unwrap(),
    )?;

    println!("Account successfully created. Prefund generated address with at least {max_fee} tokens. It is good to send more in the case of higher demand, max_fee * 2 = {}", max_fee * 2);
    let mut output = vec![
        ("max_fee", format!("{max_fee:#x}")),
        ("address", format!("{address:#x}")),
    ];
    if add_profile {
        output.push((
            "add-profile",
            "Profile successfully added to Scarb.toml".to_string(),
        ));
    }

    Ok(AccountCreateResponse {
        address,
        max_fee: FieldElement::from(max_fee),
        add_profile: if add_profile {
            "Profile successfully added to Scarb.toml".to_string()
        } else {
            "--add-profile flag was not set. No profile added to Scarb.toml".to_string()
        },
    })
}

pub fn add_created_profile_to_configuration(
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    name: String,
    network: String,
    url: String,
) -> Result<()> {
    let manifest_path = path_to_scarb_toml.clone().unwrap_or_else(|| {
        get_scarb_manifest().expect("Failed to obtain manifest path from scarb")
    });
    let metadata = scarb_metadata::MetadataCommand::new()
        .inherit_stderr()
        .manifest_path(&manifest_path)
        .no_deps()
        .exec()
        .context(
            "Failed to read Scarb.toml manifest file, not found in current nor parent directories",
        )
        .unwrap();

    let package = &metadata.packages[0].manifest_metadata.tool;

    let property = get_property(package, &Some(name.clone()), "account");
    if property.is_ok() {
        return Err(anyhow!(
            "Failed to add {name} profile to the Scarb.toml. Profile already exists"
        ));
    }

    let toml_string = {
        let mut tool_sncast = toml::value::Table::new();
        let mut new_profile = toml::value::Table::new();

        new_profile.insert("network".to_string(), Value::String(network));
        new_profile.insert("url".to_string(), Value::String(url));
        new_profile.insert("account".to_string(), Value::String(name.clone()));

        tool_sncast.insert(name, Value::Table(new_profile));

        let mut tool = toml::value::Table::new();
        tool.insert("sncast".to_string(), Value::Table(tool_sncast));

        let mut config = toml::value::Table::new();
        config.insert("tool".to_string(), Value::Table(tool));

        toml::to_string(&Value::Table(config)).unwrap()
    };

    let mut scarb_toml = OpenOptions::new()
        .append(true)
        .open(manifest_path)
        .expect("Couldn't open Scarb.toml");
    scarb_toml
        .write_all(format!("\n{toml_string}").as_bytes())
        .expect("Couldn't write to the Scarb.toml");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::starknet_commands::account::create::add_created_profile_to_configuration;
    use sealed_test::prelude::rusty_fork_test;
    use sealed_test::prelude::sealed_test;
    use std::fs;

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_add_created_profile_to_configuration_happy_case() {
        let res = add_created_profile_to_configuration(
            &None,
            String::from("some-name"),
            String::from("some-net"),
            String::from("http://some-url"),
        );

        assert!(res.is_ok());

        let contents = fs::read_to_string("Scarb.toml").expect("Unable to read Scarb.toml");
        assert!(contents.contains("[tool.sncast.some-name]"));
        assert!(contents.contains("account = \"some-name\""));
        assert!(contents.contains("network = \"some-net\""));
        assert!(contents.contains("url = \"http://some-url\""));
    }

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_add_created_profile_to_configuration_profile_already_exists() {
        let res = add_created_profile_to_configuration(
            &None,
            String::from("myprofile"),
            String::from("some-net"),
            String::from("http://some-url"),
        );

        assert!(res.is_err());
    }
}
