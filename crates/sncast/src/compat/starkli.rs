use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use serde::de::DeserializeOwned;
use serde_json::{Deserializer, Value};
use starknet_rust::signers::{SigningKey, VerifyingKey};
use starknet_types_core::felt::Felt;

use crate::accounts::{AccountRecord, AccountType};
use crate::signers::{KeystoreFile, PrivateKeySpec, SignerSpec};

pub fn load_account_with_password(
    account_path: &Utf8PathBuf,
    keystore_path: &Utf8PathBuf,
    password: &str,
) -> Result<AccountRecord> {
    check_files_exist(keystore_path, account_path)?;

    let private_key = KeystoreFile::decrypt(keystore_path, password)?.secret_scalar();
    let account_info: Value = read_json_file(account_path)?;

    let parse_to_felt = |pointer: &str| -> Option<Felt> {
        string_value(&account_info, pointer).and_then(|value| value.parse().ok())
    };

    let address = parse_to_felt("/deployment/address")
        .context("Failed to get account address from account JSON file")?;
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
    match string_value(account_info, "/variant/multisig/status").as_deref() {
        Some("off") => {}
        Some("on") => bail!("Braavos accounts cannot be imported with multisig on"),
        _ => {
            bail!("Braavos accounts can only be imported with an explicit multisig status of 'off'")
        }
    }

    account_info
        .pointer("/variant/signers")
        .and_then(Value::as_array)
        .filter(|signers| signers.len() == 1)
        .context("Braavos accounts can only be imported with one seed signer")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use test_case::test_case;

    #[test_case(Some(json!("off")), None; "off")]
    #[test_case(Some(json!("on")), Some("cannot be imported with multisig on"); "on")]
    #[test_case(None, Some("explicit multisig status of 'off'"); "missing")]
    #[test_case(Some(Value::Null), Some("explicit multisig status of 'off'"); "null")]
    #[test_case(Some(json!(false)), Some("explicit multisig status of 'off'"); "boolean")]
    #[test_case(Some(json!(0)), Some("explicit multisig status of 'off'"); "number")]
    #[test_case(Some(json!("unknown")), Some("explicit multisig status of 'off'"); "unknown")]
    #[test_case(Some(json!("OFF")), Some("explicit multisig status of 'off'"); "uppercase")]
    fn braavos_import_requires_explicit_multisig_off(status: Option<Value>, error: Option<&str>) {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        let mut account: Value = serde_json::from_str(
            &std::fs::read_to_string(fixtures.join("my_account_braavos_invalid_multisig.json"))
                .unwrap(),
        )
        .unwrap();
        if let Some(status) = status {
            account["variant"]["multisig"]["status"] = status;
        } else {
            account["variant"]["multisig"] = json!({});
        }
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("account.json")).unwrap();
        std::fs::write(&path, serde_json::to_vec(&account).unwrap()).unwrap();

        let result = load_account_with_password(&path, &fixtures.join("my_key.json"), "123");

        if let Some(error) = error {
            assert!(result.unwrap_err().to_string().contains(error));
        } else {
            let account = result.unwrap();
            assert_eq!(account.account_type, Some(AccountType::Braavos));
            verify_private_key(&account).unwrap();
        }
    }

    #[test]
    fn braavos_import_rejects_multiple_signers() {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        let error = load_account_with_password(
            &fixtures.join("my_account_braavos_multiple_signers.json"),
            &fixtures.join("my_key.json"),
            "123",
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Braavos accounts can only be imported with one seed signer"
        );
    }

    #[test]
    fn starkli_conversion_rejects_wrong_password_and_mismatched_key() {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        let error = load_account_with_password(
            &fixtures.join("my_account.json"),
            &fixtures.join("my_key.json"),
            "wrong password",
        )
        .unwrap_err();
        assert!(error.to_string().contains("decrypt"));

        let account = load_account_with_password(
            &fixtures.join("my_account.json"),
            &fixtures.join("predeployed_key.json"),
            "123",
        )
        .unwrap();
        assert_eq!(
            verify_private_key(&account).unwrap_err().to_string(),
            "Public key and private key from keystore do not match"
        );
    }

    #[test]
    fn starkli_conversion_rejects_nonregular_keystore() {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().to_owned()).unwrap();
        let error = load_account_with_password(&fixtures.join("my_account.json"), &path, "123")
            .unwrap_err();
        assert!(format!("{error:#}").contains("is a directory"));
    }
}
