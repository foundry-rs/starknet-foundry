use crate::starknet_commands::account::create::Create;
use crate::starknet_commands::account::delete::Delete;
use crate::starknet_commands::account::deploy::Deploy;
use crate::starknet_commands::account::import::Import;
use crate::starknet_commands::account::list::List;
use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use configuration::resolve_config_file;
use configuration::{load_config, search_config_upwards_relative_to};
use serde_json::json;
use sncast::helpers::rpc::RpcArgs;
use sncast::{
    AccountType, chain_id_to_network_name, decode_chain_id, helpers::configuration::CastConfig,
};
use starknet::signers::SigningKey;
use starknet_types_core::felt::Felt;
use std::{fs::OpenOptions, io::Write};
use toml::Value;

pub mod create;
pub mod delete;
pub mod deploy;
pub mod import;
pub mod list;

#[derive(Args)]
#[command(about = "Creates and deploys an account to the Starknet")]
pub struct Account {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Import(Import),
    Create(Create),
    Deploy(Deploy),
    Delete(Delete),
    List(List),
}

pub fn prepare_account_json(
    private_key: &SigningKey,
    address: Felt,
    deployed: bool,
    legacy: bool,
    account_type: AccountType,
    class_hash: Option<Felt>,
    salt: Option<Felt>,
) -> serde_json::Value {
    let mut account_json = json!({
        "private_key": format!("{:#x}", private_key.secret_scalar()),
        "public_key": format!("{:#x}", private_key.verifying_key().scalar()),
        "address": format!("{address:#x}"),
        "type": format!("{account_type}").to_lowercase().replace("openzeppelin", "open_zeppelin"),
        "deployed": deployed,
        "legacy": legacy,
    });

    if let Some(salt) = salt {
        account_json["salt"] = serde_json::Value::String(format!("{salt:#x}"));
    }
    if let Some(class_hash) = class_hash {
        account_json["class_hash"] = serde_json::Value::String(format!("{class_hash:#x}"));
    }

    account_json
}

pub fn write_account_to_accounts_file(
    account: &str,
    accounts_file: &Utf8PathBuf,
    chain_id: Felt,
    account_json: serde_json::Value,
) -> Result<()> {
    if !accounts_file.exists() {
        std::fs::create_dir_all(accounts_file.clone().parent().unwrap())?;
        std::fs::write(accounts_file.clone(), "{}")?;
    }

    let contents = std::fs::read_to_string(accounts_file.clone())?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at = {}", accounts_file))?;

    let network_name = chain_id_to_network_name(chain_id);

    if !items[&network_name][account].is_null() {
        bail!(
            "Account with name = {} already exists in network with chain_id = {}",
            account,
            decode_chain_id(chain_id)
        );
    }
    items[&network_name][account] = account_json;

    std::fs::write(
        accounts_file.clone(),
        serde_json::to_string_pretty(&items).unwrap(),
    )?;
    Ok(())
}

pub fn add_created_profile_to_configuration(
    profile: Option<&str>,
    cast_config: &CastConfig,
    path: &Utf8PathBuf,
) -> Result<()> {
    if !load_config::<CastConfig>(Some(path), profile)
        .unwrap_or_default()
        .account
        .is_empty()
    {
        bail!(
            "Failed to add profile = {} to the snfoundry.toml. Profile already exists",
            profile.unwrap_or("default")
        );
    }

    let toml_string = {
        let mut new_profile = toml::value::Table::new();

        new_profile.insert("url".to_string(), Value::String(cast_config.url.clone()));
        new_profile.insert(
            "account".to_string(),
            Value::String(cast_config.account.clone()),
        );
        if let Some(keystore) = cast_config.keystore.clone() {
            new_profile.insert("keystore".to_string(), Value::String(keystore.to_string()));
        } else {
            new_profile.insert(
                "accounts-file".to_string(),
                Value::String(cast_config.accounts_file.to_string()),
            );
        }
        let mut profile_config = toml::value::Table::new();
        profile_config.insert(
            profile.map_or_else(|| cast_config.account.clone(), ToString::to_string),
            Value::Table(new_profile),
        );

        let mut sncast_config = toml::value::Table::new();
        sncast_config.insert(String::from("sncast"), Value::Table(profile_config));

        toml::to_string(&Value::Table(sncast_config)).context("Failed to convert toml to string")?
    };

    let config_path = search_config_upwards_relative_to(path)?;

    let mut snfoundry_toml = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)
        .context("Failed to open snfoundry.toml")?;
    snfoundry_toml
        .write_all(format!("\n{toml_string}").as_bytes())
        .context("Failed to write to the snfoundry.toml")?;

    Ok(())
}

fn generate_add_profile_message(
    profile_name: Option<&String>,
    rpc: &RpcArgs,
    account_name: &str,
    accounts_file: &Utf8PathBuf,
    keystore: Option<Utf8PathBuf>,
) -> Result<Option<String>> {
    if let Some(profile_name) = profile_name {
        let url = rpc
            .url
            .clone()
            .expect("the argument '--network' should not be used with '--add-profile' argument");
        let config = CastConfig {
            url,
            account: account_name.into(),
            accounts_file: accounts_file.into(),
            keystore,
            ..Default::default()
        };
        let config_path = resolve_config_file();
        add_created_profile_to_configuration(Some(profile_name), &config, &config_path)?;
        Ok(Some(format!(
            "Profile {profile_name} successfully added to {config_path}",
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use configuration::copy_config_to_tempdir;
    use sncast::helpers::configuration::CastConfig;
    use sncast::helpers::constants::DEFAULT_ACCOUNTS_FILE;
    use std::fs;

    use crate::starknet_commands::account::add_created_profile_to_configuration;

    #[test]
    fn test_add_created_profile_to_configuration_happy_case() {
        let tempdir =
            copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
        let path = Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap();
        let config = CastConfig {
            url: String::from("http://some-url"),
            account: String::from("some-name"),
            accounts_file: "accounts".into(),
            ..Default::default()
        };
        let res = add_created_profile_to_configuration(
            Some(&String::from("some-name")),
            &config,
            &path.clone(),
        );
        assert!(res.is_ok());

        let contents =
            fs::read_to_string(path.join("snfoundry.toml")).expect("Failed to read snfoundry.toml");
        assert!(contents.contains("[sncast.some-name]"));
        assert!(contents.contains("account = \"some-name\""));
        assert!(contents.contains("url = \"http://some-url\""));
        assert!(contents.contains("accounts-file = \"accounts\""));
    }

    #[test]
    fn test_add_created_profile_to_configuration_profile_already_exists() {
        let tempdir =
            copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
        let config = CastConfig {
            url: String::from("http://127.0.0.1:5055/rpc"),
            account: String::from("user1"),
            accounts_file: DEFAULT_ACCOUNTS_FILE.into(),
            ..Default::default()
        };
        let res = add_created_profile_to_configuration(
            Some(&String::from("default")),
            &config,
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap(),
        );
        assert!(res.is_err());
    }
}
