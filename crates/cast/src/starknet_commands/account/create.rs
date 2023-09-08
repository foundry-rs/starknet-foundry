use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::OZ_CLASS_HASH;
use cast::helpers::response_structs::AccountCreateResponse;
use cast::helpers::scarb_utils::{
    get_package_tool_sncast, get_scarb_manifest, get_scarb_metadata, CastConfig,
};
use cast::{
    chain_id_to_network_name, decode_chain_id, extract_or_generate_salt, get_chain_id,
    get_keystore_password, parse_number,
};
use clap::Args;
use serde_json::json;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::{FeeEstimate, FieldElement};
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey, VerifyingKey};
use std::fs::{self, OpenOptions};
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

    /// If passed, create account from keystore and starkli account JSON file
    #[clap(short, long)]
    pub from_keystore: bool,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    config: &CastConfig,
    provider: &JsonRpcClient<HttpTransport>,
    path_to_scarb_toml: Option<Utf8PathBuf>,
    chain_id: FieldElement,
    salt: Option<FieldElement>,
    add_profile: bool,
    class_hash: Option<String>,
    from_keystore: bool,
    keystore_path: Option<Utf8PathBuf>,
    account_path: Option<Utf8PathBuf>,
) -> Result<AccountCreateResponse> {
    let (account_json, max_fee) = if from_keystore {
        let keystore_path_ = keystore_path
            .ok_or_else(|| anyhow!("--keystore must be passed when using --from-keystore"))?;
        let account_path_ = account_path
            .ok_or_else(|| anyhow!("--account must be passed when using --from-keystore"))?;
        import_from_keystore(provider, keystore_path_, account_path_).await?
    } else {
        let salt = extract_or_generate_salt(salt);
        let class_hash = {
            let ch = match &class_hash {
                Some(class_hash) => class_hash,
                None => OZ_CLASS_HASH,
            };
            parse_number(ch)?
        };
        generate_account(provider, salt, class_hash).await?
    };

    let address = parse_number(
        account_json["address"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid address"))?,
    )?;

    write_account_to_accounts_file(
        &path_to_scarb_toml,
        &config.rpc_url,
        &config.account,
        &config.accounts_file,
        chain_id,
        account_json.clone(),
        add_profile,
    )?;

    let mut output = vec![("address", format!("{address:#x}"))];
    if account_json["deployed"] == json!(false) {
        println!("Account successfully created. Prefund generated address with at least {max_fee} tokens. It is good to send more in the case of higher demand, max_fee * 2 = {}", max_fee * 2);
        output.push(("max_fee", format!("{max_fee:#x}")));
    }

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

pub async fn import_from_keystore(
    provider: &JsonRpcClient<HttpTransport>,
    keystore_path: Utf8PathBuf,
    account_path: Utf8PathBuf,
) -> Result<(serde_json::Value, u64)> {
    let account_info: serde_json::Value = serde_json::from_str(&fs::read_to_string(account_path)?)?;
    let deployment = account_info
        .get("deployment")
        .ok_or_else(|| anyhow!("No deployment field in account JSON file"))?;

    let deployed = match deployment.get("status").and_then(serde_json::Value::as_str) {
        Some("deployed") => true,
        Some("undeployed") => false,
        _ => bail!("Unknown deployment status value"),
    };

    let salt: Option<FieldElement> = {
        let salt = deployment.get("salt").and_then(serde_json::Value::as_str);
        match (salt, deployed) {
            (Some(salt), _) => Some(parse_number(salt)?),
            (None, false) => bail!("No salt field in account JSON file"),
            (None, true) => None,
        }
    };

    let class_hash: FieldElement = {
        let ch = deployment
            .get("class_hash")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("No class_hash field in account JSON file"))?;
        parse_number(ch)?
    };

    let private_key = SigningKey::from_keystore(keystore_path, get_keystore_password()?.as_str())?;
    let public_key: FieldElement = {
        let pk = account_info
            .get("variant")
            .and_then(|v| v.get("public_key"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("No public_key in account JSON file"))?;
        parse_number(pk)?
    };
    if public_key != private_key.verifying_key().scalar() {
        bail!("Public key mismatch");
    }

    let address: FieldElement = if let Some(salt) = salt {
        get_contract_address(salt, class_hash, &[public_key], FieldElement::ZERO)
    } else {
        let address = deployment
            .get("address")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("No address in account JSON file"))?;
        parse_number(address)?
    };

    let max_fee = match salt {
        Some(salt) => {
            get_account_deployment_fee(&private_key, class_hash, salt, provider)
                .await?
                .overall_fee
        }
        None => 0,
    };

    let account_json = prepare_account_json(
        &private_key,
        &private_key.verifying_key(),
        address,
        deployed,
        Some(class_hash),
        salt,
    );

    Ok((account_json, max_fee))
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
) -> Result<(serde_json::Value, u64)> {
    let private_key = SigningKey::from_random();
    let public_key = private_key.verifying_key();

    let address: FieldElement =
        get_contract_address(salt, class_hash, &[public_key.scalar()], FieldElement::ZERO);

    let account_json = prepare_account_json(
        &private_key,
        &public_key,
        address,
        false,
        Some(class_hash),
        Some(salt),
    );

    let max_fee = get_account_deployment_fee(&private_key, class_hash, salt, provider)
        .await?
        .overall_fee;

    Ok((account_json, max_fee))
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

async fn get_account_deployment_fee(
    private_key: &SigningKey,
    class_hash: FieldElement,
    salt: FieldElement,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<FeeEstimate> {
    let signer = LocalWallet::from_signing_key(private_key.clone());
    let chain_id = get_chain_id(provider).await?;
    let factory = OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
    let deployment = factory.deploy(salt);

    let fee_estimate = deployment.estimate_fee().await;

    if let Err(err) = &fee_estimate {
        if err
            .to_string()
            .contains("StarknetErrorCode.UNDECLARED_CLASS")
        {
            bail!("The class {class_hash} is undeclared, try using --class-hash with a class hash that is already declared");
        }
    }

    Ok(fee_estimate?)
}

fn write_account_to_accounts_file(
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    rpc_url: &str,
    account: &str,
    accounts_file: &Utf8PathBuf,
    chain_id: FieldElement,
    account_json: serde_json::Value,
    add_profile: bool,
) -> Result<()> {
    if !accounts_file.exists() {
        std::fs::create_dir_all(accounts_file.clone().parent().unwrap())?;
        std::fs::write(accounts_file.clone(), "{}")?;
    }

    let contents = std::fs::read_to_string(accounts_file.clone())?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {}", accounts_file))?;

    let network_name = chain_id_to_network_name(chain_id);

    if !items[&network_name][account].is_null() {
        bail!(
            "Account with name {} already exists in network with chain_id {}",
            account,
            decode_chain_id(chain_id)
        );
    }
    items[&network_name][account] = account_json;

    if add_profile {
        let config = CastConfig {
            rpc_url: rpc_url.into(),
            account: account.into(),
            accounts_file: accounts_file.into(),
        };
        add_created_profile_to_configuration(path_to_scarb_toml, &config)?;
    }

    std::fs::write(
        accounts_file.clone(),
        serde_json::to_string_pretty(&items).unwrap(),
    )?;
    Ok(())
}

fn prepare_account_json(
    private_key: &SigningKey,
    public_key: &VerifyingKey,
    address: FieldElement,
    deployed: bool,
    class_hash: Option<FieldElement>,
    salt: Option<FieldElement>,
) -> serde_json::Value {
    let mut account_json = json!({
        "private_key": format!("{:#x}", private_key.secret_scalar()),
        "public_key": format!("{:#x}", public_key.scalar()),
        "address": format!("{address:#x}"),
        "deployed": deployed,
    });

    if let Some(salt) = salt {
        account_json["salt"] = serde_json::Value::String(format!("{salt:#x}"));
    }
    if let Some(class_hash) = class_hash {
        account_json["class_hash"] = serde_json::Value::String(format!("{class_hash:#x}"));
    }

    account_json
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
