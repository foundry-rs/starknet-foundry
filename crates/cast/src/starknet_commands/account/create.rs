use crate::starknet_commands::account::{
    add_created_profile_to_configuration, get_account_deployment_fee, prepare_account_json,
    write_account_to_accounts_file,
};
use anyhow::{anyhow, bail, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::OZ_CLASS_HASH;
use cast::helpers::response_structs::AccountCreateResponse;
use cast::helpers::scarb_utils::CastConfig;
use cast::{extract_or_generate_salt, get_test_keystore_password, parse_number};
use clap::Args;
use serde_json::json;
use starknet::core::types::FieldElement;
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::SigningKey;

#[derive(Args, Debug)]
#[command(about = "Create an account with all important secrets")]
pub struct Create {
    /// Account name under which account information is going to be saved
    #[clap(
        short,
        long,
        default_value = "",
        required_unless_present = "keystore",
        conflicts_with = "keystore"
    )]
    pub name: String,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(long)]
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
    salt: Option<FieldElement>,
    add_profile: bool,
    class_hash: Option<String>,
    keystore_path: Option<Utf8PathBuf>,
    account_path: Option<Utf8PathBuf>,
) -> Result<AccountCreateResponse> {
    let salt = extract_or_generate_salt(salt);
    let class_hash = {
        let ch = match &class_hash {
            Some(class_hash) => class_hash,
            None => OZ_CLASS_HASH,
        };
        parse_number(ch)?
    };
    let (account_json, max_fee) = generate_account(provider, salt, class_hash).await?;

    let address = parse_number(
        account_json["address"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid address"))?,
    )?;

    if let Some(keystore_path_) = &keystore_path {
        let account_path_ = account_path
            .ok_or_else(|| anyhow!("--account must be passed when using --keystore"))?;

        let private_key = parse_number(
            account_json["private_key"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid private_key"))?,
        )?;
        create_to_keystore(
            private_key,
            salt,
            class_hash,
            keystore_path_,
            &account_path_,
            add_profile,
            config,
            &path_to_scarb_toml,
        )?;
    } else {
        write_account_to_accounts_file(
            &path_to_scarb_toml,
            &config.rpc_url,
            &config.account,
            &config.accounts_file,
            &config.keystore,
            chain_id,
            account_json.clone(),
            add_profile,
        )?;
    }

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

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
) -> Result<(serde_json::Value, u64)> {
    let private_key = SigningKey::from_random();
    let address: FieldElement = get_contract_address(
        salt,
        class_hash,
        &[private_key.verifying_key().scalar()],
        FieldElement::ZERO,
    );

    let account_json =
        prepare_account_json(&private_key, address, false, Some(class_hash), Some(salt));

    let max_fee = get_account_deployment_fee(&private_key, class_hash, salt, provider)
        .await?
        .overall_fee;

    Ok((account_json, max_fee))
}

#[allow(clippy::too_many_arguments)]
fn create_to_keystore(
    private_key: FieldElement,
    salt: FieldElement,
    class_hash: FieldElement,
    keystore_path: &Utf8PathBuf,
    account_path: &Utf8PathBuf,
    add_profile: bool,
    config: &CastConfig,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
) -> Result<()> {
    let password = get_test_keystore_password()?;
    let private_key = SigningKey::from_secret_scalar(private_key);
    private_key.save_as_keystore(keystore_path, &password)?;

    let oz_account_json = json!({
        "version": 1,
        "variant": {
            "type": "open_zeppelin",
            "version": 1,
            "public_key": format!("{:#x}", private_key.verifying_key().scalar()),
        },
        "deployment": {
            "status": "undeployed",
            "class_hash": format!("{class_hash:#x}"),
            "salt": format!("{salt:#x}"),
        }
    });

    if add_profile {
        add_created_profile_to_configuration(path_to_scarb_toml, config)?;
    }
    write_account_to_file(&oz_account_json, account_path)
}

fn write_account_to_file(
    account_json: &serde_json::Value,
    account_file: &Utf8PathBuf,
) -> Result<()> {
    if account_file.exists() {
        bail!("Account file {} already exists", account_file);
    }

    std::fs::create_dir_all(account_file.clone().parent().unwrap())?;
    std::fs::write(
        account_file.clone(),
        serde_json::to_string_pretty(&account_json).unwrap(),
    )?;
    Ok(())
}
