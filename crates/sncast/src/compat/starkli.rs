use anyhow::{Context, Result, anyhow, bail, ensure};
use camino::Utf8PathBuf;
use serde::de::DeserializeOwned;
use serde_json::{Deserializer, Value};
use starknet_rust::core::utils::get_contract_address;
use starknet_rust::signers::{SigningKey, VerifyingKey};
use starknet_types_core::felt::Felt;

use crate::accounts::{AccountRecord, AccountType};
use crate::signers::{KeystoreFile, PrivateKeySpec, SignerSpec};

/// A starkli account converted to a native record, retaining the constructor metadata needed to
/// verify its address once the import command has resolved the class hash via RPC.
///
/// The native record cannot store a Ready guardian or a Braavos base account class, so this
/// metadata is only used during import.
#[derive(Debug)]
pub struct StarkliAccount {
    pub account: AccountRecord,
    address_class_hash: Option<Felt>,
    constructor_calldata: Vec<Felt>,
}

impl StarkliAccount {
    /// Checks the address against the account's salt and constructor metadata, if it has a salt,
    /// and returns the [`AccountRecord`].
    /// `class_hash` is the implementation class hash resolved for the selected network.
    pub fn into_record(self, class_hash: Felt) -> Result<AccountRecord> {
        if let Some(salt) = self.account.salt {
            let computed_address = get_contract_address(
                salt,
                self.address_class_hash.unwrap_or(class_hash),
                &self.constructor_calldata,
                Felt::ZERO,
            );

            ensure!(
                computed_address == self.account.address,
                "Computed address {computed_address:#x} does not match the provided address {:#x}. Please ensure that the provided salt, class hash, and account type are correct.",
                self.account.address
            );
        }

        Ok(self.account)
    }
}

pub fn load_account_with_password(
    account_path: &Utf8PathBuf,
    keystore_path: &Utf8PathBuf,
    password: &str,
) -> Result<StarkliAccount> {
    check_files_exist(keystore_path, account_path)?;

    let private_key = KeystoreFile::decrypt(keystore_path, password)?.secret_scalar();
    let account_info: Value = read_json_file(account_path)?;

    let parse_to_felt = |pointer: &str| -> Option<Felt> {
        string_value(&account_info, pointer).and_then(|value| value.parse().ok())
    };

    let address = optional_felt(&account_info, "/deployment/address")?;
    let class_hash = optional_felt(&account_info, "/deployment/class_hash")?;
    let salt = optional_felt(&account_info, "/deployment/salt")?;
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

    let account_type = account_type.context("Failed to get type key")?;
    let public_key = match account_type {
        AccountType::Ready => {
            // Starkli renamed `signer` to `owner`, retaining it as a serde alias.
            // Like serde, reject both fields even if their values are identical.
            match (
                account_info.pointer("/variant/owner"),
                account_info.pointer("/variant/signer"),
            ) {
                (Some(_), Some(_)) => bail!("duplicate field `owner` in starkli account variant"),
                (Some(_), None) => parse_to_felt("/variant/owner"),
                (None, Some(_)) => parse_to_felt("/variant/signer"),
                (None, None) => None,
            }
        }
        AccountType::OpenZeppelin => parse_to_felt("/variant/public_key"),
        AccountType::Braavos => braavos_public_key(&account_info)?,
    }
    .context("Failed to get public key from account JSON file")?;

    // Old accounts linked to legacy starkli keystores don't have pre-computed addresses
    // that need to be derived from the account metadata.
    //
    // Some types of accounts (only Braavos at this moment) have a facade contract that the real account is injected to.
    // In such cases, we need the class hash of that facade to compute the address.
    let (facade_class_hash, constructor_calldata) = if salt.is_some() || address.is_none() {
        constructor_class_hash_and_calldata(&account_info, account_type, public_key)?
    } else {
        (None, Vec::new())
    };

    let address = match address {
        Some(address) => address,
        None => get_contract_address(
            salt.context("Missing deployment.salt: cannot derive account address")?,
            facade_class_hash
                .or(class_hash)
                .context("Missing deployment.class_hash: cannot derive account address")?,
            &constructor_calldata,
            Felt::ZERO,
        ),
    };

    let account = AccountRecord {
        public_key,
        address,
        salt,
        deployed,
        class_hash,
        legacy,
        account_type: Some(account_type),
        signer: SignerSpec::PrivateKey(PrivateKeySpec::new(private_key)),
    };

    Ok(StarkliAccount {
        account,
        address_class_hash: facade_class_hash,
        constructor_calldata,
    })
}

/// Returns a base class hash (if the account type has one) and the calldata
/// that were used to deploy the account.
fn constructor_class_hash_and_calldata(
    account_info: &Value,
    account_type: AccountType,
    public_key: Felt,
) -> Result<(Option<Felt>, Vec<Felt>)> {
    // Starkli marks Cairo 0 proxy accounts with `implementation`. sncast cannot deploy them, so
    // do not guess their address or validate one against a constructor it does not use.
    if account_type != AccountType::OpenZeppelin {
        ensure!(
            optional_felt(account_info, "/variant/implementation")?.is_none(),
            "Address derivation for legacy (Cairo 0) proxy accounts is not supported"
        );
    }

    match account_type {
        AccountType::OpenZeppelin => Ok((None, vec![public_key])),

        AccountType::Ready => {
            let guardian = optional_felt(account_info, "/variant/guardian")?
                .context("Missing variant.guardian: cannot derive Ready account address")?;
            // Cairo 1 Ready/Argent v0.4+: Starknet owner, then Option<Starknet guardian>.
            let mut calldata = vec![Felt::ZERO, public_key];
            if guardian == Felt::ZERO {
                calldata.push(Felt::ONE);
            } else {
                calldata.extend([Felt::ZERO, Felt::ZERO, guardian]);
            }
            Ok((None, calldata))
        }

        AccountType::Braavos => {
            ensure!(
                string_value(account_info, "/deployment/context/variant").as_deref()
                    == Some("braavos"),
                "Missing or unsupported Braavos deployment context"
            );

            let base = "/deployment/context/base_account_class_hash";
            let alias = "/deployment/context/mock_implementation";

            ensure!(
                !(account_info.pointer(base).is_some() && account_info.pointer(alias).is_some()),
                "duplicate field `base_account_class_hash` in Braavos deployment context"
            );

            let base_class_hash = optional_felt(account_info, base)?
                .or(optional_felt(account_info, alias)?)
                .context(
                    "Missing Braavos base_account_class_hash: cannot derive account address",
                )?;

            Ok((Some(base_class_hash), vec![public_key]))
        }
    }
}

fn optional_felt(json: &Value, pointer: &str) -> Result<Option<Felt>> {
    match json.pointer(pointer) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .and_then(|value| value.parse().ok())
            .map(Some)
            .with_context(|| format!("Invalid felt at `{pointer}` in account JSON file")),
    }
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

    ensure!(
        string_value(account_info, "/variant/signers/0/type").as_deref() == Some("stark"),
        "Braavos accounts can only be imported with a Stark seed signer"
    );

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

    #[test_case("owner", "my_key.json", true; "owner")]
    #[test_case("signer", "my_key.json", true; "legacy_signer")]
    #[test_case("owner", "predeployed_key.json", false; "owner_wrong_key")]
    #[test_case("signer", "predeployed_key.json", false; "legacy_signer_wrong_key")]
    fn argent_import_accepts_owner_alias_and_verifies_key(
        field: &str,
        keystore: &str,
        matching_key: bool,
    ) {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        let mut account: Value = serde_json::from_str(
            &std::fs::read_to_string(fixtures.join("my_account_argent.json")).unwrap(),
        )
        .unwrap();
        let owner = account["variant"]
            .as_object_mut()
            .unwrap()
            .remove("owner")
            .unwrap();
        account["variant"][field] = owner.clone();
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("account.json")).unwrap();
        std::fs::write(&path, serde_json::to_vec(&account).unwrap()).unwrap();

        let account = load_account_with_password(&path, &fixtures.join(keystore), "123")
            .unwrap()
            .account;

        assert_eq!(account.account_type, Some(AccountType::Ready));
        assert_eq!(
            account.public_key,
            owner.as_str().unwrap().parse::<Felt>().unwrap()
        );
        if matching_key {
            verify_private_key(&account).unwrap();
        } else {
            assert_eq!(
                verify_private_key(&account).unwrap_err().to_string(),
                "Public key and private key from keystore do not match"
            );
        }
    }

    #[test_case(json!({"owner": "0x1", "signer": "0x1"}), "duplicate field `owner`"; "identical_aliases")]
    #[test_case(json!({"owner": "0x1", "signer": "0x2"}), "duplicate field `owner`"; "conflicting_aliases")]
    #[test_case(json!({"owner": "invalid", "signer": "0x1"}), "duplicate field `owner`"; "malformed_owner_with_alias")]
    #[test_case(json!({"signer": "invalid"}), "Failed to get public key"; "malformed_alias")]
    fn argent_import_rejects_ambiguous_or_malformed_owner(mut variant: Value, error: &str) {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        variant["type"] = json!("argent");
        let account = json!({
            "variant": variant,
            "deployment": {"address": "0x123", "status": "deployed"}
        });
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("account.json")).unwrap();
        std::fs::write(&path, serde_json::to_vec(&account).unwrap()).unwrap();

        let result = load_account_with_password(&path, &fixtures.join("my_key.json"), "123");

        assert!(result.unwrap_err().to_string().contains(error));
    }

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
            let account = result.unwrap().account;
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
        .unwrap()
        .account;
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
