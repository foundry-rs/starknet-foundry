use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::OZ_CLASS_HASH;
use cast::helpers::response_structs::AccountCreateResponse;
use cast::helpers::scarb_utils::{
    get_package_tool_sncast, get_scarb_manifest, get_scarb_metadata, CastConfig,
};
use cast::{chain_id_to_network_name, decode_chain_id, extract_or_generate_salt, parse_number};
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
    config: &CastConfig,
    provider: &JsonRpcClient<HttpTransport>,
    path_to_scarb_toml: Option<Utf8PathBuf>,
    chain_id: FieldElement,
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
            chain_id,
            signer,
            provider,
        )
        .await?;
        let deployment = factory.deploy(salt);

        let fee_estimate = deployment.estimate_fee().await;

        if let Err(err) = &fee_estimate {
            if err
                .to_string()
                .contains("StarknetErrorCode.UNDECLARED_CLASS")
            {
                bail!("The class {oz_class_hash} is undeclared, try using --class-hash with a class hash that is already declared");
            }
        }

        fee_estimate?.overall_fee
    };

    if !config.accounts_file.exists() {
        std::fs::create_dir_all(config.accounts_file.clone().parent().unwrap())?;
        std::fs::write(config.accounts_file.clone(), "{}")?;
    }

    let contents = std::fs::read_to_string(config.accounts_file.clone())?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {}", config.accounts_file))?;

    let network_name = chain_id_to_network_name(chain_id);

    if !items[&network_name][&config.account].is_null() {
        return Err(anyhow!(
            "Account with name {} already exists in network with chain_id {}",
            &config.account,
            decode_chain_id(chain_id)
        ));
    }

    items[&network_name][&config.account] = json!({
        "private_key": format!("{:#x}", private_key.secret_scalar()),
        "public_key": format!("{:#x}", public_key.scalar()),
        "address": format!("{address:#x}"),
        "salt": format!("{salt:#x}"),
        "deployed": false,
    });

    if let Some(class_hash_) = class_hash {
        items[&network_name][&config.account]["class_hash"] =
            serde_json::Value::String(format!("{:#x}", parse_number(&class_hash_)?));
    }

    if add_profile {
        match add_created_profile_to_configuration(&path_to_scarb_toml, config) {
            Ok(()) => {}
            Err(err) => return Err(anyhow!(err)),
        };
    }

    std::fs::write(
        config.accounts_file.clone(),
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
    config: &CastConfig,
) -> Result<()> {
    let manifest_path = match path_to_scarb_toml.clone() {
        Some(path) => path,
        None => get_scarb_manifest().context("Failed to obtain manifest path from scarb")?,
    };
    let metadata = get_scarb_metadata(&manifest_path)?;

    if let Ok(tool_sncast) = get_package_tool_sncast(&metadata) {
        let property = tool_sncast
            .get(&config.account)
            .and_then(|profile_| profile_.get("account"));
        if property.is_some() {
            bail!(
                "Failed to add {} profile to the Scarb.toml. Profile already exists",
                config.account
            );
        }
    }

    let toml_string = {
        let mut tool_sncast = toml::value::Table::new();
        let mut new_profile = toml::value::Table::new();

        new_profile.insert("url".to_string(), Value::String(config.rpc_url.clone()));
        new_profile.insert("account".to_string(), Value::String(config.account.clone()));
        new_profile.insert(
            "accounts-file".to_string(),
            Value::String(config.accounts_file.to_string()),
        );

        tool_sncast.insert(config.account.clone(), Value::Table(new_profile));

        let mut tool = toml::value::Table::new();
        tool.insert("sncast".to_string(), Value::Table(tool_sncast));

        let mut config = toml::value::Table::new();
        config.insert("tool".to_string(), Value::Table(tool));

        toml::to_string(&Value::Table(config)).context("Couldn't convert toml to string")?
    };

    let mut scarb_toml = OpenOptions::new()
        .append(true)
        .open(manifest_path)
        .context("Couldn't open Scarb.toml")?;
    scarb_toml
        .write_all(format!("\n{toml_string}").as_bytes())
        .context("Couldn't write to the Scarb.toml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::starknet_commands::account::create::add_created_profile_to_configuration;
    use cast::helpers::constants::DEFAULT_ACCOUNTS_FILE;
    use cast::helpers::scarb_utils::CastConfig;
    use sealed_test::prelude::rusty_fork_test;
    use sealed_test::prelude::sealed_test;
    use std::fs;

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_add_created_profile_to_configuration_happy_case() {
        let config = CastConfig {
            rpc_url: String::from("http://some-url"),
            account: String::from("some-name"),
            accounts_file: "accounts".into(),
        };
        let res = add_created_profile_to_configuration(&None, &config);

        assert!(res.is_ok());

        let contents = fs::read_to_string("Scarb.toml").expect("Unable to read Scarb.toml");
        assert!(contents.contains("[tool.sncast.some-name]"));
        assert!(contents.contains("account = \"some-name\""));
        assert!(contents.contains("url = \"http://some-url\""));
        assert!(contents.contains("accounts-file = \"accounts\""));
    }

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_add_created_profile_to_configuration_profile_already_exists() {
        let config = CastConfig {
            rpc_url: String::from("http://some-url"),
            account: String::from("myprofile"),
            accounts_file: DEFAULT_ACCOUNTS_FILE.into(),
        };
        let res = add_created_profile_to_configuration(&None, &config);

        assert!(res.is_err());
    }
}
