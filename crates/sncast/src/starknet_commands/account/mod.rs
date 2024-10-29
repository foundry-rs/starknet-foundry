use crate::starknet_commands::account::create::Create;
use crate::starknet_commands::account::delete::Delete;
use crate::starknet_commands::account::deploy::Deploy;
use crate::starknet_commands::account::import::Import;
use crate::starknet_commands::account::list::List;
use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand, ValueEnum};
use configuration::{
    find_config_file, load_global_config, search_config_upwards_relative_to, CONFIG_FILENAME,
};
use serde_json::json;
use sncast::{chain_id_to_network_name, decode_chain_id, helpers::configuration::CastConfig};
use starknet::{core::types::Felt, signers::SigningKey};
use std::{fmt, fs::OpenOptions, io::Write};
use toml::Value;

pub mod create;
pub mod delete;
pub mod deploy;
pub mod import;
pub mod list;

#[derive(Args)]
#[command(about = "Creates and deploys an account to the Starknet")]
pub struct Account {
    #[clap(subcommand)]
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

#[allow(clippy::doc_markdown)]
#[derive(ValueEnum, Clone, Debug)]
pub enum AccountType {
    /// OpenZeppelin account implementation
    Oz,
    /// Argent account implementation
    Argent,
    /// Braavos account implementation
    Braavos,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AccountType::Oz => write!(f, "open_zeppelin"),
            AccountType::Argent => write!(f, "argent"),
            AccountType::Braavos => write!(f, "braavos"),
        }
    }
}

pub fn prepare_account_json(
    private_key: &SigningKey,
    address: Felt,
    deployed: bool,
    legacy: bool,
    account_type: &AccountType,
    class_hash: Option<Felt>,
    salt: Option<Felt>,
) -> serde_json::Value {
    let mut account_json = json!({
        "private_key": format!("{:#x}", private_key.secret_scalar()),
        "public_key": format!("{:#x}", private_key.verifying_key().scalar()),
        "address": format!("{address:#x}"),
        "type": format!("{account_type}"),
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

#[allow(clippy::too_many_arguments)]
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
    profile: &Option<String>,
    cast_config: &CastConfig,
    path: &Option<Utf8PathBuf>,
) -> Result<()> {
    if !load_global_config::<CastConfig>(path, profile)
        .unwrap_or_default()
        .account
        .is_empty()
    {
        bail!(
            "Failed to add profile = {} to the snfoundry.toml. Profile already exists",
            profile.as_ref().unwrap_or(&"default".to_string())
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
            profile
                .clone()
                .unwrap_or_else(|| cast_config.account.clone()),
            Value::Table(new_profile),
        );

        let mut sncast_config = toml::value::Table::new();
        sncast_config.insert(String::from("sncast"), Value::Table(profile_config));

        toml::to_string(&Value::Table(sncast_config)).context("Failed to convert toml to string")?
    };

    let config_path = match path.as_ref() {
        Some(p) => search_config_upwards_relative_to(p)?,
        None => find_config_file().unwrap_or(Utf8PathBuf::from(CONFIG_FILENAME)),
    };

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
            &Some(String::from("some-name")),
            &config,
            &Some(path.clone()),
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
            &Some(String::from("default")),
            &config,
            &Some(Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
        );
        assert!(res.is_err());
    }
}
