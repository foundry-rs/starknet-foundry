use crate::starknet_commands::account::{prepare_account_json, write_account_to_accounts_file};
use anyhow::{anyhow, bail, ensure, Result};
use camino::Utf8PathBuf;
use cast::helpers::response_structs::AccountAddResponse;
use cast::helpers::scarb_utils::CastConfig;
use cast::{get_keystore_password, parse_number};
use clap::Args;
use starknet::core::types::FieldElement;
use starknet::core::utils::get_contract_address;
use starknet::signers::SigningKey;
use std::fs;

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Add {
    /// Address of the account
    #[clap(short, long)]
    pub address: FieldElement,

    /// Class hash of the account
    #[clap(short, long)]
    pub class_hash: Option<FieldElement>,

    /// Account deployment status
    #[clap(short, long)]
    pub deployed: bool,

    /// Account private key
    #[clap(long)]
    pub private_key: FieldElement,

    /// Account public key
    #[clap(long)]
    pub public_key: Option<FieldElement>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(long)]
    pub add_profile: bool,

    /// If passed, import account from keystore and starkli account JSON file
    #[clap(short, long)]
    pub from_keystore: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn add(
    config: &CastConfig,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    chain_id: FieldElement,
    add: &Add,
    keystore_path: Option<Utf8PathBuf>,
    account_path: Option<Utf8PathBuf>,
) -> Result<AccountAddResponse> {
    let account_json = if add.from_keystore {
        let keystore_path_ = keystore_path
            .ok_or_else(|| anyhow!("--keystore must be passed when using --from-keystore"))?;
        let account_path_ = account_path
            .ok_or_else(|| anyhow!("--account must be passed when using --from-keystore"))?;
        import_from_keystore(keystore_path_, account_path_)?
    } else {
        let private_key = &SigningKey::from_secret_scalar(add.private_key);
        if let Some(public_key) = &add.public_key {
            ensure!(
                public_key == &private_key.verifying_key().scalar(),
                "public key mismatch"
            );
        }

        prepare_account_json(
            private_key,
            add.address,
            add.deployed,
            add.class_hash,
            add.salt,
        )
    };

    write_account_to_accounts_file(
        path_to_scarb_toml,
        &config.rpc_url,
        &config.account,
        &config.accounts_file,
        chain_id,
        account_json.clone(),
        add.add_profile,
    )?;

    Ok(AccountAddResponse {
        add_profile: if add.add_profile {
            "Profile successfully added to Scarb.toml".to_string()
        } else {
            "--add-profile flag was not set. No profile added to Scarb.toml".to_string()
        },
    })
}

fn import_from_keystore(
    keystore_path: Utf8PathBuf,
    account_path: Utf8PathBuf,
) -> Result<serde_json::Value> {
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

    let account_json =
        prepare_account_json(&private_key, address, deployed, Some(class_hash), salt);

    Ok(account_json)
}
