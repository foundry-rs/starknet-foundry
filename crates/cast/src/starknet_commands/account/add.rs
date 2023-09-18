use crate::starknet_commands::account::{prepare_account_json, write_account_to_accounts_file};
use anyhow::{anyhow, bail, ensure, Context, Result};
use camino::Utf8PathBuf;
use cast::get_chain_id;
use cast::helpers::response_structs::AccountAddResponse;
use cast::helpers::scarb_utils::CastConfig;
use cast::{get_keystore_password, parse_number};
use clap::Args;
use starknet::core::types::BlockTag::Latest;
use starknet::core::types::{BlockId, FieldElement};
use starknet::core::utils::get_contract_address;
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider,
};
use starknet::signers::SigningKey;
use std::fs;

#[derive(Args, Debug, Default)]
#[command(about = "Add an account to the accounts file")]
pub struct Add {
    /// Name of the account to be added
    #[clap(short, long)]
    pub name: String,

    /// Address of the account
    #[clap(
        short,
        long,
        default_value = "0",
        required_unless_present = "from_keystore"
    )]
    pub address: FieldElement,

    /// Class hash of the account
    #[clap(long, visible_alias = "ch")]
    pub class_hash: Option<FieldElement>,

    /// Account deployment status
    /// If not passed, sncast will check whether the account is deployed or not
    #[clap(short, long)]
    pub deployed: bool,

    /// Account private key
    #[clap(
        long,
        visible_alias = "priv",
        default_value = "0",
        required_unless_present = "from_keystore"
    )]
    pub private_key: FieldElement,

    /// Account public key
    #[clap(long, visible_alias = "pub")]
    pub public_key: Option<FieldElement>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(long)]
    pub add_profile: bool,

    /// If passed, import account from keystore and starkli account JSON file
    #[clap(short, long, conflicts_with_all = ["address", "class_hash", "deployed", "private_key", "public_key", "salt"])]
    pub from_keystore: bool,
}

pub async fn add(
    config: &CastConfig,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    provider: &JsonRpcClient<HttpTransport>,
    add: &Add,
    keystore_path: Option<Utf8PathBuf>,
    account_path: Option<Utf8PathBuf>,
) -> Result<AccountAddResponse> {
    let account_json = if add.from_keystore {
        let keystore_path_ = keystore_path
            .ok_or_else(|| anyhow!("--keystore must be passed when using --from-keystore"))?;
        let account_path_ = account_path
            .ok_or_else(|| anyhow!("--account must be passed when using --from-keystore"))?;
        import_from_keystore(&keystore_path_, &account_path_).context(format!(
            "Couldn't import account from keystore at path {} and account JSON file at path {}",
            keystore_path_, account_path_
        ))?
    } else {
        let private_key = &SigningKey::from_secret_scalar(add.private_key);
        if let Some(public_key) = &add.public_key {
            ensure!(
                public_key == &private_key.verifying_key().scalar(),
                "public key mismatch"
            );
        }

        let deployed = add.deployed
            || provider
                .get_class_hash_at(BlockId::Tag(Latest), add.address)
                .await
                .map_or(false, |_| {
                    println!("Contract detected as deployed on chain");
                    true
                });

        prepare_account_json(private_key, add.address, deployed, add.class_hash, add.salt)
    };

    let chain_id = get_chain_id(provider).await?;
    write_account_to_accounts_file(
        path_to_scarb_toml,
        &config.rpc_url,
        &add.name,
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
    keystore_path: &Utf8PathBuf,
    account_path: &Utf8PathBuf,
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

#[cfg(test)]
mod tests {
    use std::env;

    use crate::starknet_commands::account::add::import_from_keystore;
    use camino::Utf8PathBuf;
    use cast::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
    use serde_json::json;

    #[test]
    fn test_import_from_keystore() {
        let keystore_path = Utf8PathBuf::from("tests/data/keystore/my_key.json");
        let account_path = Utf8PathBuf::from("tests/data/keystore/my_account.json");

        env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
        let account_json = import_from_keystore(&keystore_path, &account_path).unwrap();

        assert_eq!(
            account_json,
            json!(
                {
                    "address": "0xcce3217e4aea0ab738b55446b1b378750edfca617db549fda1ede28435206c",
                    "class_hash": "0x646a72e2aab2fca75d713fbe4a58f2d12cbd64105621b89dc9ce7045b5bf02b",
                    "deployed": true,
                    "private_key": "0x55ae34c86281fbd19292c7e3bfdfceb4",
                    "public_key": "0xe2d3d7080bfc665e0060a06e8e95c3db3ff78a1fec4cc81ddc87e49a12e0a",
                }
            )
        );
    }

    #[test]
    fn test_import_from_keystore_undeployed() {
        let keystore_path = Utf8PathBuf::from("tests/data/keystore/my_key.json");
        let account_path = Utf8PathBuf::from("tests/data/keystore/my_account_undeployed.json");

        env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
        let account_json = import_from_keystore(&keystore_path, &account_path).unwrap();

        assert_eq!(
            account_json,
            json!(
                {
                    "address": "0xcce3217e4aea0ab738b55446b1b378750edfca617db549fda1ede28435206c",
                    "class_hash": "0x646a72e2aab2fca75d713fbe4a58f2d12cbd64105621b89dc9ce7045b5bf02b",
                    "deployed": false,
                    "private_key": "0x55ae34c86281fbd19292c7e3bfdfceb4",
                    "public_key": "0xe2d3d7080bfc665e0060a06e8e95c3db3ff78a1fec4cc81ddc87e49a12e0a",
                    "salt": "0x14df438ac6825165c7a0af29decd5892528b763a333f93a5f6b12980dbddd9f",
                }
            )
        );
    }
}
