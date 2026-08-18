use std::env;

use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use serde::de::DeserializeOwned;
use serde_json::Deserializer;
use serde_json::{Map, Value, json};
use starknet_rust::signers::{SigningKey, VerifyingKey};
use starknet_types_core::felt::Felt;

use crate::accounts::{AccountRecord, AccountType};
use crate::helpers::constants::{
    BRAAVOS_BASE_ACCOUNT_CLASS_HASH, CREATE_KEYSTORE_PASSWORD_ENV_VAR, KEYSTORE_PASSWORD_ENV_VAR,
};
use crate::signers::{PrivateKeySpec, SignerSpec};

pub fn get_keystore_password(env_var: &str) -> std::io::Result<String> {
    match env::var(env_var) {
        Ok(password) => Ok(password),
        _ => rpassword::prompt_password("Enter password: "),
    }
}

pub fn load_account(account: &str, keystore_path: &Utf8PathBuf) -> Result<AccountRecord> {
    let password = get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?;
    load_account_with_password(&Utf8PathBuf::from(account), keystore_path, &password)
}

pub fn load_account_with_password(
    account_path: &Utf8PathBuf,
    keystore_path: &Utf8PathBuf,
    password: &str,
) -> Result<AccountRecord> {
    check_files_exist(keystore_path, account_path)?;

    let private_key = SigningKey::from_keystore(keystore_path, password)?.secret_scalar();

    let account_info: Value = read_json_file(account_path)?;

    let parse_to_felt = |pointer: &str| -> Option<Felt> {
        string_value(&account_info, pointer).and_then(|value| value.parse().ok())
    };

    let address = parse_to_felt("/deployment/address");
    let class_hash = parse_to_felt("/deployment/class_hash");
    let salt = parse_to_felt("/deployment/salt");
    let deployed =
        string_value(&account_info, "/deployment/status").map(|status| status == "deployed");
    let legacy = account_info
        .pointer("/variant/legacy")
        .and_then(Value::as_bool);
    let account_type = string_value(&account_info, "/variant/type")
        .map(|account_type| match account_type.as_str() {
            "argent" => "ready".to_owned(),
            _ => account_type,
        })
        .and_then(|account_type| account_type.parse().ok());

    let public_key = match account_type.context("Failed to get type key")? {
        AccountType::Ready => parse_to_felt("/variant/owner"),
        AccountType::OpenZeppelin => parse_to_felt("/variant/public_key"),
        AccountType::Braavos => braavos_public_key(&account_info)?,
    }
    .context("Failed to get public key from account JSON file")?;

    Ok(AccountRecord {
        public_key,
        address,
        salt,
        deployed,
        class_hash,
        legacy,
        account_type,
        signer: SignerSpec::PrivateKey(PrivateKeySpec::new(private_key)),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_account_files(
    private_key: Felt,
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    keystore_path: &Utf8PathBuf,
    account_path: &Utf8PathBuf,
    legacy: bool,
) -> Result<()> {
    if keystore_path.exists() {
        bail!("Keystore file {keystore_path} already exists");
    }
    if account_path.exists() {
        bail!("Account file {account_path} already exists");
    }
    let password = get_keystore_password(CREATE_KEYSTORE_PASSWORD_ENV_VAR)?;
    let private_key = SigningKey::from_secret_scalar(private_key);
    private_key.save_as_keystore(keystore_path, &password)?;
    let account_json = match account_type {
        AccountType::OpenZeppelin => json!({
            "version": 1,
            "variant": {
                "type": AccountType::OpenZeppelin,
                "version": 1,
                "public_key": format!("{:#x}", private_key.verifying_key().scalar()),
                "legacy": legacy,
            },
            "deployment": {
                "status": "undeployed",
                "class_hash": format!("{class_hash:#x}"),
                "salt": format!("{salt:#x}"),
            }
        }),
        AccountType::Ready => json!({
            "version": 1,
            "variant": {
                "type": AccountType::Ready,
                "version": 1,
                "owner": format!("{:#x}", private_key.verifying_key().scalar()),
                "guardian": "0x0",
            },
            "deployment": {
                "status": "undeployed",
                "class_hash": format!("{class_hash:#x}"),
                "salt": format!("{salt:#x}"),
            }
        }),
        AccountType::Braavos => json!({
            "version": 1,
            "variant": {
                "type": AccountType::Braavos,
                "version": 1,
                "multisig": { "status": "off" },
                "signers": [{
                    "type": "stark",
                    "public_key": format!("{:#x}", private_key.verifying_key().scalar())
                }]
            },
            "deployment": {
                "status": "undeployed",
                "class_hash": format!("{class_hash:#x}"),
                "salt": format!("{salt:#x}"),
                "context": {
                    "variant": "braavos",
                    "base_account_class_hash": BRAAVOS_BASE_ACCOUNT_CLASS_HASH
                }
            }
        }),
    };

    if let Some(parent) = account_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(account_path, serde_json::to_string_pretty(&account_json)?)?;
    Ok(())
}

pub fn update_deployment(account: &str, address: Felt) -> Result<()> {
    let account_path = Utf8PathBuf::from(account);
    let contents = std::fs::read_to_string(&account_path).context("Failed to read account file")?;
    let mut items: Map<String, Value> = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse account file at {account_path}"))?;

    items["deployment"]["status"] = Value::from("deployed");
    items
        .get_mut("deployment")
        .and_then(Value::as_object_mut)
        .context("Failed to get deployment as an object")?
        .retain(|key, _| key != "salt" && key != "context");
    items["deployment"]["address"] = format!("{address:#x}").into();

    std::fs::write(&account_path, serde_json::to_string_pretty(&items)?)
        .context("Failed to write to account file")
}

pub fn verify_private_key(account: &AccountRecord) -> Result<VerifyingKey> {
    let private_key = account
        .signer
        .private_key()
        .context("Private key not found in starkli account")?;
    let signing_key = SigningKey::from_secret_scalar(private_key);
    let verifying_key = signing_key.verifying_key();
    if verifying_key.scalar() != account.public_key {
        bail!("Public key and private key from keystore do not match");
    }
    Ok(verifying_key)
}

fn braavos_public_key(account_info: &Value) -> Result<Option<Felt>> {
    string_value(account_info, "/variant/multisig/status")
        .filter(|status| status == "off")
        .context("Braavos accounts cannot be deployed with multisig on")?;

    account_info
        .pointer("/variant/signers")
        .and_then(Value::as_array)
        .filter(|signers| signers.len() == 1)
        .context("Braavos accounts can only be deployed with one seed signer")?;

    Ok(string_value(account_info, "/variant/signers/0/public_key")
        .and_then(|value| value.parse().ok()))
}

fn string_value(json: &Value, pointer: &str) -> Option<String> {
    json.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn read_json_file<T>(path: &Utf8PathBuf) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    let contents =
        std::fs::read_to_string(path).with_context(|| format!("Failed to read a file = {path}"))?;
    if contents.trim().is_empty() {
        return Ok(T::default());
    }

    let deserializer = &mut Deserializer::from_str(&contents);
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let field = error.path().to_string();
        anyhow!(
            "Failed to parse field `{field}` in file '{path}': {}",
            error.into_inner()
        )
    })
}

fn check_files_exist(keystore_path: &Utf8PathBuf, account_path: &Utf8PathBuf) -> Result<()> {
    if !keystore_path.exists() {
        bail!("Keystore file = {keystore_path} does not exist");
    }
    if !account_path.exists() {
        bail!("Account file = {account_path} does not exist");
    }
    Ok(())
}
