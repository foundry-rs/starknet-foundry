use crate::starknet_commands::account::{
    generate_add_profile_message, profile_exists, save_account,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use camino::Utf8PathBuf;
use clap::Args;
use configuration::resolve_config_file;
use foundry_ui::UI;
use sncast::accounts::AccountRepository;
use sncast::compat::starkli;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::import::AccountImportResponse;
use sncast::signers::{KeystoreSpec, SignerSpec, keystore_password};
use sncast::{check_class_hash_exists, check_if_legacy_contract, get_chain_id, handle_rpc_error};
use starknet_rust::core::types::{BlockId, BlockTag, StarknetError};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::providers::{Provider, ProviderError};

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
    if let Some(profile_name) = import.add_profile.as_deref() {
        ensure!(
            !profile_exists(Some(profile_name), &resolve_config_file())?,
            "Failed to add profile = {profile_name} to the snfoundry.toml. Profile already exists"
        );
    }

    let credential_spec = KeystoreSpec::new(
        import.keystore.clone(),
        import.keystore_password_env.clone(),
    );

    let password = keystore_password(&credential_spec)?;

    let starkli_account = native_account_from_starkli(
        &import.account_file,
        &import.keystore,
        &password,
        import.keystore_password_env.clone(),
    )?;

    let address = starkli_account.account.address;
    let fetched_class_hash = match provider
        .get_class_hash_at(BlockId::Tag(BlockTag::PreConfirmed), address)
        .await
    {
        Ok(class_hash) => Some(class_hash),
        Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => None,
        Err(error) => return Err(handle_rpc_error(error)),
    };

    let class_hash = match (fetched_class_hash, starkli_account.account.class_hash) {
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

    let mut account = starkli_account.into_record(class_hash)?;

    let chain_id = get_chain_id(provider).await?;
    account.legacy = Some(check_if_legacy_contract(Some(class_hash), address, provider).await?);
    account.deployed = Some(fetched_class_hash.is_some());
    account.class_hash = Some(class_hash);

    let account_name = save_account(import.name.as_deref(), repository, chain_id, account, ui)?;

    let add_profile = generate_add_profile_message(
        import.add_profile.as_ref(),
        &import.rpc,
        account_name.as_str(),
        repository.path(),
        config,
    )?;

    Ok(AccountImportResponse {
        add_profile,
        account_name: account_name.into(),
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
) -> Result<starkli::StarkliAccount> {
    let mut starkli_account =
        starkli::load_account_with_password(account_file, keystore, password)?;
    starkli::verify_private_key(&starkli_account.account)?;
    let persisted_keystore =
        canonical_utf8_path(keystore).context("Failed to resolve the imported keystore path")?;
    starkli_account.account.signer =
        SignerSpec::Keystore(KeystoreSpec::new(persisted_keystore, password_env));
    Ok(starkli_account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use sncast::accounts::{AccountName, ChainId};
    use sncast::signers::KeystoreFile;
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

        let SignerSpec::Keystore(spec) = account.account.signer else {
            panic!("expected a native keystore signer");
        };
        assert!(spec.path().is_absolute());
        assert_eq!(spec.password_env(), Some("ALICE_KEYSTORE_PASSWORD"));
    }

    #[tokio::test]
    async fn duplicate_profile_is_rejected_before_credentials_or_mutation() {
        const CHILD_ENV: &str = "SNCAST_TEST_STARKLI_DUPLICATE_PROFILE_CHILD";
        const PASSWORD_ENV: &str = "SNCAST_TEST_STARKLI_UNSET_PASSWORD";
        if std::env::var_os(CHILD_ENV).is_none() {
            // Config resolution uses cwd. Isolate it in a child test process rather than
            // changing the working directory shared by concurrently running tests.
            let directory = tempfile::tempdir().unwrap();
            let test_name = concat!(
                module_path!(),
                "::duplicate_profile_is_rejected_before_credentials_or_mutation"
            );
            // The test harness omits the crate name included by module_path!().
            let (_, test_name) = test_name.split_once("::").unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", test_name, "--nocapture"])
                .env(CHILD_ENV, "1")
                .env_remove(PASSWORD_ENV)
                .env_remove("SNCAST_KEYSTORE_PASSWORD")
                .current_dir(directory.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "child test failed: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        }

        let root = Utf8PathBuf::try_from(std::env::current_dir().unwrap()).unwrap();
        let config_path = root.join("snfoundry.toml");
        let config_bytes = b"[sncast.existing]\naccount = \"alice\"\n";
        std::fs::write(&config_path, config_bytes).unwrap();
        let fixtures = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/keystore");
        let keystore = root.join("key.json");
        let keystore_bytes = std::fs::read(fixtures.join("my_key.json")).unwrap();
        std::fs::write(&keystore, &keystore_bytes).unwrap();
        let starkli_file = root.join("starkli.json");
        let source_bytes = std::fs::read(fixtures.join("my_account_argent.json")).unwrap();
        std::fs::write(&starkli_file, &source_bytes).unwrap();
        let import = ImportStarkli {
            name: None,
            account_file: starkli_file.clone(),
            keystore: keystore.clone(),
            // An unset explicit password variable must not be consulted before preflight.
            keystore_password_env: Some(PASSWORD_ENV.to_owned()),
            add_profile: Some("existing".to_owned()),
            rpc: RpcArgs::default(),
            silent: true,
        };
        let provider = sncast::get_provider(&"http://127.0.0.1:1".parse().unwrap()).unwrap();

        for existing_file in [true, false] {
            let accounts_file = root.join(if existing_file {
                "accounts.json"
            } else {
                "absent.json"
            });
            let accounts_bytes = b"{}\n";
            if existing_file {
                std::fs::write(&accounts_file, accounts_bytes).unwrap();
            }
            let repository = AccountRepository::try_new(&accounts_file).unwrap();

            let error = import_starkli(
                &repository,
                &provider,
                &import,
                &CastConfig::default(),
                &UI::default(),
            )
            .await
            .err()
            .expect("duplicate profile should be rejected");

            assert!(
                error.to_string().contains("Profile already exists"),
                "{error:#}"
            );
            if existing_file {
                assert_eq!(std::fs::read(&accounts_file).unwrap(), accounts_bytes);
            } else {
                assert!(!accounts_file.exists());
            }
            assert!(
                !repository
                    .backup_path(sncast::accounts::schema::SourceVersion::V1)
                    .exists()
            );
            assert_eq!(std::fs::read(&config_path).unwrap(), config_bytes);
            assert_eq!(std::fs::read(&keystore).unwrap(), keystore_bytes);
            assert_eq!(std::fs::read(&starkli_file).unwrap(), source_bytes);
        }
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

    /// Old `sncast account create --keystore` files omit `deployment.address` until deployment.
    #[tokio::test]
    #[expect(clippy::too_many_lines)]
    async fn import_starkli_derives_missing_address_from_account_metadata() {
        use sncast::helpers::braavos::BraavosAccountFactory;
        use sncast::helpers::constants::{
            BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH,
        };
        use starknet_rust::accounts::{
            AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory,
        };
        use starknet_rust::signers::LocalWallet;

        // Expected (address, class hash) on success, or an error.
        type Expected = Result<(Felt, Felt), &'static str>;

        let directory = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(directory.path().to_owned()).unwrap();
        let keystore = root.join("key.json");
        let password = std::env::var("PATH").unwrap();
        KeystoreFile::create(&keystore, Felt::ONE, &password).unwrap();
        let wallet = || LocalWallet::from_signing_key(SigningKey::from_secret_scalar(Felt::ONE));
        let public_key = format!(
            "{:#x}",
            SigningKey::from_secret_scalar(Felt::ONE)
                .verifying_key()
                .scalar()
        );
        let provider = sncast::get_provider(&"http://127.0.0.1:1".parse().unwrap()).unwrap();
        let salt = Felt::from_hex("0x5a17").unwrap();
        let guardian = Felt::from_hex("0x6a4d").unwrap();
        let other_base = Felt::from_hex("0xba5e").unwrap();

        // Expected addresses come from starknet-rust's account factories, the code sncast uses
        // to deploy, rather than from the import's own constructor encoding.
        let oz = OpenZeppelinAccountFactory::new(OZ_CLASS_HASH, Felt::ONE, wallet(), &provider)
            .await
            .unwrap()
            .deploy_v3(salt)
            .address();
        let ready = |guardian| {
            let provider = &provider;
            async move {
                ArgentAccountFactory::new(READY_CLASS_HASH, Felt::ONE, guardian, wallet(), provider)
                    .await
                    .unwrap()
                    .deploy_v3(salt)
                    .address()
            }
        };
        let braavos = |base| {
            let provider = &provider;
            async move {
                BraavosAccountFactory::new(BRAAVOS_CLASS_HASH, base, Felt::ONE, wallet(), provider)
                    .await
                    .unwrap()
                    .deploy_v3(salt)
                    .address()
            }
        };
        let ready_default = ready(None).await;
        let ready_guardian = ready(Some(guardian)).await;
        let braavos_default = braavos(BRAAVOS_BASE_ACCOUNT_CLASS_HASH).await;
        let braavos_other_base = braavos(other_base).await;
        // The metadata must change the address, or these cases could not tell honoring it from
        // sncast's defaults.
        assert_ne!(ready_guardian, ready_default);
        assert_ne!(braavos_other_base, braavos_default);

        let hex = |felt: Felt| format!("{felt:#x}");
        let deployment = |class_hash: Felt| json!({"status": "undeployed", "class_hash": hex(class_hash), "salt": hex(salt)});
        let oz_file = json!({
            "version": 1,
            "variant": {"type": "open_zeppelin", "version": 1, "public_key": public_key, "legacy": false},
            "deployment": deployment(OZ_CLASS_HASH),
        });
        let ready_file = |guardian: Felt| {
            json!({
                "version": 1,
                "variant": {"type": "ready", "version": 1, "owner": public_key, "guardian": hex(guardian)},
                "deployment": deployment(READY_CLASS_HASH),
            })
        };
        let braavos_file = |context: Value| {
            let mut deployment = deployment(BRAAVOS_CLASS_HASH);
            deployment["context"] = context;
            json!({
                "version": 1,
                "variant": {
                    "type": "braavos", "version": 1, "multisig": {"status": "off"},
                    "signers": [{"type": "stark", "public_key": public_key}]
                },
                "deployment": deployment,
            })
        };
        let default_context = json!({"variant": "braavos", "base_account_class_hash": hex(BRAAVOS_BASE_ACCOUNT_CLASS_HASH)});
        let edit = |mut file: Value, pointer: &str, value: Option<Value>| {
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            let parent = file.pointer_mut(parent).unwrap().as_object_mut().unwrap();
            match value {
                Some(value) => parent.insert(key.to_owned(), value),
                None => parent.remove(key),
            };
            file
        };

        let mut legacy_signer = ready_file(guardian);
        legacy_signer["variant"]["signer"] = legacy_signer["variant"]["owner"].take();
        legacy_signer["variant"]
            .as_object_mut()
            .unwrap()
            .remove("owner");

        let cases: Vec<(&str, Value, Expected)> = vec![
            ("oz", oz_file.clone(), Ok((oz, OZ_CLASS_HASH))),
            (
                "ready",
                ready_file(Felt::ZERO),
                Ok((ready_default, READY_CLASS_HASH)),
            ),
            (
                "argent_alias",
                edit(
                    ready_file(Felt::ZERO),
                    "/variant/type",
                    Some(json!("argent")),
                ),
                Ok((ready_default, READY_CLASS_HASH)),
            ),
            (
                "ready_guardian",
                ready_file(guardian),
                Ok((ready_guardian, READY_CLASS_HASH)),
            ),
            (
                "ready_signer_alias",
                legacy_signer,
                Ok((ready_guardian, READY_CLASS_HASH)),
            ),
            (
                "braavos",
                braavos_file(default_context.clone()),
                Ok((braavos_default, BRAAVOS_CLASS_HASH)),
            ),
            (
                "braavos_other_base",
                braavos_file(
                    json!({"variant": "braavos", "base_account_class_hash": hex(other_base)}),
                ),
                Ok((braavos_other_base, BRAAVOS_CLASS_HASH)),
            ),
            (
                "braavos_mock_implementation_alias",
                braavos_file(json!({"variant": "braavos", "mock_implementation": hex(other_base)})),
                Ok((braavos_other_base, BRAAVOS_CLASS_HASH)),
            ),
            (
                "supplied_address_matches",
                edit(
                    ready_file(guardian),
                    "/deployment/address",
                    Some(json!(hex(ready_guardian))),
                ),
                Ok((ready_guardian, READY_CLASS_HASH)),
            ),
            (
                // sncast's defaults would accept this address; the file's guardian does not.
                "supplied_address_mismatch",
                edit(
                    ready_file(guardian),
                    "/deployment/address",
                    Some(json!(hex(ready_default))),
                ),
                Err("does not match the provided address"),
            ),
            (
                "braavos_supplied_default_address_with_other_base",
                edit(
                    braavos_file(
                        json!({"variant": "braavos", "base_account_class_hash": hex(other_base)}),
                    ),
                    "/deployment/address",
                    Some(json!(hex(braavos_default))),
                ),
                Err("does not match the provided address"),
            ),
            (
                "missing_salt",
                edit(oz_file.clone(), "/deployment/salt", None),
                Err("Missing deployment.salt"),
            ),
            (
                "missing_class_hash",
                edit(oz_file.clone(), "/deployment/class_hash", None),
                Err("Missing deployment.class_hash"),
            ),
            (
                "malformed_salt",
                edit(oz_file.clone(), "/deployment/salt", Some(json!("salty"))),
                Err("Invalid felt at `/deployment/salt`"),
            ),
            (
                "malformed_class_hash",
                edit(
                    oz_file.clone(),
                    "/deployment/class_hash",
                    Some(json!("0xzz")),
                ),
                Err("Invalid felt at `/deployment/class_hash`"),
            ),
            (
                // Deployed account: the class hash is recovered from the network and the supplied
                // address is still checked against the salt.
                "supplied_address_recovers_class_hash",
                edit(
                    edit(oz_file.clone(), "/deployment/class_hash", None),
                    "/deployment/address",
                    Some(json!(hex(oz))),
                ),
                Ok((oz, OZ_CLASS_HASH)),
            ),
            (
                "malformed_address",
                edit(oz_file, "/deployment/address", Some(json!(7))),
                Err("Invalid felt at `/deployment/address`"),
            ),
            (
                "missing_guardian",
                edit(ready_file(guardian), "/variant/guardian", None),
                Err("Missing variant.guardian"),
            ),
            (
                "malformed_guardian",
                edit(
                    ready_file(guardian),
                    "/variant/guardian",
                    Some(json!("0xzz")),
                ),
                Err("Invalid felt at `/variant/guardian`"),
            ),
            (
                "ready_proxy",
                edit(
                    ready_file(Felt::ZERO),
                    "/variant/implementation",
                    Some(json!("0x1")),
                ),
                Err("legacy (Cairo 0) proxy accounts is not supported"),
            ),
            (
                "braavos_missing_context",
                braavos_file(Value::Null),
                Err("Missing or unsupported Braavos deployment context"),
            ),
            (
                "braavos_missing_base",
                braavos_file(json!({"variant": "braavos"})),
                Err("Missing Braavos base_account_class_hash"),
            ),
            (
                "braavos_duplicate_base",
                braavos_file(json!({
                    "variant": "braavos",
                    "base_account_class_hash": hex(other_base),
                    "mock_implementation": hex(other_base)
                })),
                Err("duplicate field `base_account_class_hash`"),
            ),
            (
                "braavos_malformed_base",
                braavos_file(json!({"variant": "braavos", "base_account_class_hash": "base"})),
                Err("Invalid felt at `/deployment/context/base_account_class_hash`"),
            ),
        ];

        let missing_address = json!({"error": {"code": 20, "message": "Contract not found"}});
        for (case, file, expected) in cases {
            let server = MockServer::start().await;
            let deployed = case == "supplied_address_recovers_class_hash";
            mock_rpc(
                &server,
                "starknet_getClassHashAt",
                if deployed {
                    json!({"result": hex(OZ_CLASS_HASH)})
                } else {
                    missing_address.clone()
                },
            )
            .await;
            mock_rpc(
                &server,
                "starknet_getClass",
                json!({"result": {
                    "sierra_program": [],
                    "contract_class_version": "0.1.0",
                    "entry_points_by_type": {"CONSTRUCTOR": [], "EXTERNAL": [], "L1_HANDLER": []},
                    "abi": "[]"
                }}),
            )
            .await;
            mock_rpc(&server, "starknet_chainId", json!({"result": "0x1"})).await;
            let provider = sncast::get_provider(&server.uri().parse().unwrap()).unwrap();
            let case_root = root.join(case);
            std::fs::create_dir(&case_root).unwrap();
            let accounts_file = case_root.join("accounts.json");
            let original = b"{}\n";
            std::fs::write(&accounts_file, original).unwrap();
            let repository = AccountRepository::try_new(&accounts_file).unwrap();
            let starkli_file = case_root.join("starkli.json");
            let source = serde_json::to_vec(&file).unwrap();
            std::fs::write(&starkli_file, &source).unwrap();
            let import = ImportStarkli {
                name: Some("imported".to_owned()),
                account_file: starkli_file.clone(),
                keystore: keystore.clone(),
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

            assert_eq!(std::fs::read(&starkli_file).unwrap(), source, "{case}");
            match expected {
                Ok((address, class_hash)) => {
                    result.unwrap_or_else(|error| panic!("{case}: {error:#}"));
                    let record = repository
                        .find(
                            &ChainId::from(Felt::ONE),
                            &AccountName::new("imported").unwrap(),
                        )
                        .unwrap();
                    assert_eq!(record.address, address, "{case}");
                    assert_eq!(record.class_hash, Some(class_hash), "{case}");
                    assert_eq!(record.salt, Some(salt), "{case}");
                    assert_eq!(record.deployed, Some(deployed), "{case}");
                }
                Err(expected_error) => {
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
                        "{case}"
                    );
                }
            }
        }
    }
}
