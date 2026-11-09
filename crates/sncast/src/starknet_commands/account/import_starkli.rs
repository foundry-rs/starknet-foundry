use crate::starknet_commands::account::{generate_add_profile_message, save_account};
use anyhow::{Context, Result, anyhow, bail, ensure};
use camino::Utf8PathBuf;
use clap::Args;
use sncast::accounts::{AccountDeploymentService, AccountRecord, AccountRepository};
use sncast::compat::starkli;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::import::AccountImportResponse;
use sncast::response::ui::UI;
use sncast::signers::{KeystoreFile, KeystoreSpec, SignerSpec, keystore_password};
use sncast::{check_class_hash_exists, check_if_legacy_contract, get_chain_id, handle_rpc_error};
use starknet_rust::core::types::{BlockId, BlockTag, StarknetError};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::providers::{Provider, ProviderError};
use starknet_rust::signers::LocalWallet;

/// Convert a starkli account/keystore pair into a native sncast account.
#[derive(Args, Debug)]
pub struct ImportStarkli {
    /// Name of the native account to create
    #[arg(short, long)]
    pub name: Option<String>,

    /// Path to the starkli JSON account file
    #[arg(long)]
    pub account_file: Utf8PathBuf,

    /// Path to the starkli encrypted keystore
    #[arg(long)]
    pub keystore: Utf8PathBuf,

    /// Environment variable containing the keystore password
    #[arg(long)]
    pub keystore_password_env: Option<String>,

    /// Add a profile with this name to snfoundry.toml
    #[arg(long)]
    pub add_profile: Option<String>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Do not prompt to make the imported account the default
    #[arg(long)]
    pub silent: bool,
}

pub async fn import_starkli(
    repository: &AccountRepository,
    provider: &JsonRpcClient<HttpTransport>,
    import: &ImportStarkli,
    config: &CastConfig,
    ui: &UI,
) -> Result<AccountImportResponse> {
    let credential_spec = KeystoreSpec::new(
        import.keystore.clone(),
        import.keystore_password_env.clone(),
    );
    let password = keystore_password(&credential_spec)?;
    let mut account = native_account_from_starkli(
        &import.account_file,
        &import.keystore,
        &password,
        import.keystore_password_env.clone(),
    )?;

    let address = account.address;
    let fetched_class_hash = match provider
        .get_class_hash_at(BlockId::Tag(BlockTag::PreConfirmed), address)
        .await
    {
        Ok(class_hash) => Some(class_hash),
        Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => None,
        Err(error) => return Err(handle_rpc_error(error)),
    };
    let class_hash = match (fetched_class_hash, account.class_hash) {
        (Some(from_provider), Some(from_file)) => {
            ensure!(
                from_provider == from_file,
                "Incorrect class hash {from_file:#x} for account address {address:#x} was provided"
            );
            from_provider
        }
        (None, Some(from_file)) => {
            check_class_hash_exists(provider, from_file).await?;
            from_file
        }
        (Some(from_provider), None) => from_provider,
        (None, None) => bail!(
            "Class hash for the account address {address:#x} could not be found. Please provide the class hash"
        ),
    };

    let chain_id = get_chain_id(provider).await?;
    if let Some(salt) = account.salt {
        let signing_key = KeystoreFile::decrypt(&import.keystore, &password)?;
        let computed_address = AccountDeploymentService::compute_address(
            provider,
            account.account_type.context("Failed to get account type")?,
            class_hash,
            LocalWallet::from_signing_key(signing_key),
            salt,
            chain_id,
        )
        .await?;
        ensure!(
            computed_address == address,
            "Computed address {computed_address:#x} does not match the provided address {address:#x}. Please ensure that the provided salt, class hash, and account type are correct."
        );
    }
    account.legacy = Some(check_if_legacy_contract(Some(class_hash), address, provider).await?);
    account.deployed = Some(fetched_class_hash.is_some());
    account.class_hash = Some(class_hash);

    let account_name = match &import.name {
        Some(name) => name.clone(),
        None => repository.generate_account_name()?,
    };
    save_account(&account_name, repository, chain_id, account, ui)?;

    let add_profile = generate_add_profile_message(
        import.add_profile.as_ref(),
        &import.rpc,
        &account_name,
        repository.path(),
        config,
    )?;

    Ok(AccountImportResponse {
        add_profile,
        account_name,
    })
}

fn canonical_utf8_path(path: &Utf8PathBuf) -> Result<Utf8PathBuf> {
    let canonical = std::fs::canonicalize(path)
        .with_context(|| format!("Failed to canonicalize keystore `{path}`"))?;
    Utf8PathBuf::from_path_buf(canonical)
        .map_err(|path| anyhow!("Keystore path is not valid UTF-8: {}", path.display()))
}

fn native_account_from_starkli(
    account_file: &Utf8PathBuf,
    keystore: &Utf8PathBuf,
    password: &str,
    password_env: Option<String>,
) -> Result<AccountRecord> {
    let mut account = starkli::load_account_with_password(account_file, keystore, password)?;
    starkli::verify_private_key(&account)?;
    let persisted_keystore =
        canonical_utf8_path(keystore).context("Failed to resolve the imported keystore path")?;
    account.signer = SignerSpec::Keystore(KeystoreSpec::new(persisted_keystore, password_env));
    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use sncast::accounts::{AccountName, ChainId};
    use starknet_rust::signers::SigningKey;
    use starknet_types_core::felt::Felt;
    use wiremock::matchers::{body_partial_json, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn canonicalizes_relative_keystore_path_for_native_storage() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("key.json");
        std::fs::write(&path, "{}").unwrap();
        let path = Utf8PathBuf::from_path_buf(path).unwrap();

        let canonical = canonical_utf8_path(&path).unwrap();

        assert!(canonical.is_absolute());
        assert!(canonical.ends_with("key.json"));
    }

    #[test]
    fn converts_starkli_pair_to_native_keystore_signer() {
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");

        let account = native_account_from_starkli(
            &fixtures.join("my_account.json"),
            &fixtures.join("my_key.json"),
            "123",
            Some("ALICE_KEYSTORE_PASSWORD".to_owned()),
        )
        .unwrap();

        let SignerSpec::Keystore(spec) = account.signer else {
            panic!("expected a native keystore signer");
        };
        assert!(spec.path().is_absolute());
        assert_eq!(spec.password_env(), Some("ALICE_KEYSTORE_PASSWORD"));
    }

    async fn mock_rpc(server: &MockServer, rpc_method: &str, response: Value) {
        let mut body = json!({"jsonrpc": "2.0", "id": 1});
        body.as_object_mut()
            .unwrap()
            .extend(response.as_object().unwrap().clone());
        Mock::given(method("POST"))
            .and(body_partial_json(json!({"method": rpc_method})))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(server)
            .await;
    }

    #[tokio::test]
    #[expect(clippy::too_many_lines)]
    async fn import_starkli_validates_metadata_before_saving() {
        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().to_owned()).unwrap();
        let keystore = root.join("key.json");
        // Reuse an existing environment value as a test password without mutating the process
        // environment (other tests run concurrently).
        let password = std::env::var("PATH").unwrap();
        KeystoreFile::create(&keystore, Felt::ONE, &password).unwrap();
        let public_key = SigningKey::from_secret_scalar(Felt::ONE)
            .verifying_key()
            .scalar();
        let sierra_class = json!({"result": {
            "sierra_program": [],
            "contract_class_version": "0.1.0",
            "entry_points_by_type": {"CONSTRUCTOR": [], "EXTERNAL": [], "L1_HANDLER": []},
            "abi": "[]"
        }});
        let missing_address = json!({"error": {"code": 20, "message": "Contract not found"}});
        let rpc_error = json!({"error": {"code": -32603, "message": "Internal error"}});
        let missing_class = json!({"error": {"code": 28, "message": "Class hash not found"}});

        for (case, class_hash, address_response, class_response, deployed, expected_error) in [
            (
                "legacy",
                Some("0x456"),
                json!({"result": "0x456"}),
                json!({"result": {
                    "program": "",
                    "entry_points_by_type": {"CONSTRUCTOR": [], "EXTERNAL": [], "L1_HANDLER": []},
                    "abi": []
                }}),
                Some(true),
                None,
            ),
            (
                "deployed",
                Some("0x456"),
                json!({"result": "0x456"}),
                sierra_class.clone(),
                Some(true),
                None,
            ),
            (
                "recover_hash",
                None,
                json!({"result": "0x456"}),
                sierra_class.clone(),
                Some(true),
                None,
            ),
            (
                "undeployed",
                Some("0x456"),
                missing_address.clone(),
                sierra_class.clone(),
                Some(false),
                None,
            ),
            (
                "mismatched_hash",
                Some("0x789"),
                json!({"result": "0x456"}),
                sierra_class.clone(),
                None,
                Some("Incorrect class hash"),
            ),
            (
                "missing_hash",
                None,
                missing_address.clone(),
                sierra_class.clone(),
                None,
                Some("could not be found"),
            ),
            (
                "undeclared_hash",
                Some("0x456"),
                missing_address,
                missing_class,
                None,
                Some("not declared"),
            ),
            (
                "address_rpc_error",
                Some("0x456"),
                rpc_error.clone(),
                sierra_class.clone(),
                None,
                Some("Unknown RPC error"),
            ),
            (
                "declaration_rpc_error",
                Some("0x456"),
                json!({"error": {"code": 20, "message": "Contract not found"}}),
                rpc_error.clone(),
                None,
                Some("Unknown RPC error"),
            ),
            (
                "chain_rpc_error",
                Some("0x456"),
                json!({"result": "0x456"}),
                sierra_class.clone(),
                None,
                Some("Failed to fetch chain_id"),
            ),
            (
                "legacy_rpc_error",
                Some("0x456"),
                json!({"result": "0x456"}),
                rpc_error.clone(),
                None,
                Some("Unknown RPC error"),
            ),
            (
                "wrong_salt",
                Some("0x456"),
                json!({"result": "0x456"}),
                sierra_class.clone(),
                None,
                Some("does not match the provided address"),
            ),
            (
                "wrong_key",
                Some("0x456"),
                json!({"result": "0x456"}),
                sierra_class.clone(),
                None,
                Some("Public key and private key from keystore do not match"),
            ),
            (
                "wrong_password",
                Some("0x456"),
                json!({"result": "0x456"}),
                sierra_class,
                None,
                Some("decrypt"),
            ),
        ] {
            let server = MockServer::start().await;
            mock_rpc(&server, "starknet_getClassHashAt", address_response).await;
            mock_rpc(&server, "starknet_getClass", class_response).await;
            mock_rpc(
                &server,
                "starknet_chainId",
                if case == "chain_rpc_error" {
                    rpc_error.clone()
                } else {
                    json!({"result": "0x1"})
                },
            )
            .await;
            let provider = sncast::get_provider(&server.uri().parse().unwrap()).unwrap();
            let case_root = root.join(case);
            std::fs::create_dir(&case_root).unwrap();
            let accounts_file = case_root.join("accounts.json");
            let original = b"{}\n";
            std::fs::write(&accounts_file, original).unwrap();
            let repository = AccountRepository::try_new(&accounts_file).unwrap();
            let starkli_file = case_root.join("starkli.json");
            let mut account = json!({
                "variant": {
                    "type": "open_zeppelin", "public_key": format!("{public_key:#x}"),
                    "legacy": case != "legacy"
                },
                "deployment": {
                    "address": "0x123", "class_hash": class_hash,
                    "status": if deployed == Some(false) { "deployed" } else { "undeployed" }
                }
            });
            if case == "wrong_salt" {
                account["deployment"]["salt"] = json!("0x1");
            }
            if case == "wrong_key" {
                account["variant"]["public_key"] = json!("0x2");
            }
            std::fs::write(&starkli_file, serde_json::to_vec(&account).unwrap()).unwrap();
            let import = ImportStarkli {
                name: Some("imported".to_owned()),
                account_file: starkli_file,
                keystore: if case == "wrong_password" {
                    assert_ne!(password, "123");
                    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("tests/data/keystore/my_key.json")
                } else {
                    keystore.clone()
                },
                keystore_password_env: Some("PATH".to_owned()),
                add_profile: None,
                rpc: RpcArgs::default(),
                silent: true,
            };

            let result = import_starkli(
                &repository,
                &provider,
                &import,
                &CastConfig::default(),
                &UI::default(),
            )
            .await;

            if let Some(expected_error) = expected_error {
                let error = result
                    .err()
                    .unwrap_or_else(|| panic!("{case} unexpectedly succeeded"));
                assert!(
                    format!("{error:#}").contains(expected_error),
                    "{case}: {error:#}"
                );
                assert_eq!(std::fs::read(&accounts_file).unwrap(), original, "{case}");
                assert!(
                    !repository
                        .backup_path(sncast::accounts::schema::SourceVersion::V1)
                        .exists(),
                    "{case} created a migration backup"
                );

                let absent_file = case_root.join("absent.json");
                let absent_repository = AccountRepository::try_new(&absent_file).unwrap();
                let result = import_starkli(
                    &absent_repository,
                    &provider,
                    &import,
                    &CastConfig::default(),
                    &UI::default(),
                )
                .await;
                assert!(
                    result.is_err(),
                    "{case} unexpectedly succeeded for a missing repository"
                );
                assert!(!absent_file.exists(), "{case} created an accounts file");
            } else {
                result.unwrap_or_else(|error| panic!("{case}: {error:#}"));
                let record = repository
                    .find(
                        &ChainId::from(Felt::ONE),
                        &AccountName::new("imported").unwrap(),
                    )
                    .unwrap();
                assert_eq!(record.deployed, deployed, "{case}");
                assert_eq!(record.legacy, Some(case == "legacy"), "{case}");
                assert_eq!(
                    record.class_hash,
                    Some(Felt::from_hex("0x456").unwrap()),
                    "{case}"
                );
                let SignerSpec::Keystore(spec) = record.signer else {
                    panic!("{case} did not persist the native keystore signer");
                };
                assert!(spec.path().is_absolute());
                assert_eq!(spec.password_env(), Some("PATH"));
            }
        }
    }
}
