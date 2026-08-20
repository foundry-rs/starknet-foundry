use crate::starknet_commands::account::{
    generate_add_profile_message, notify_if_migrated, save_account,
};
use anyhow::{Context, Result, anyhow};
use camino::Utf8PathBuf;
use clap::Args;
use sncast::accounts::{AccountRecord, AccountRepository};
use sncast::compat::starkli;
use sncast::get_chain_id;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::import::AccountImportResponse;
use sncast::response::ui::UI;
use sncast::signers::{KeystoreSpec, SignerSpec, keystore_password};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};

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
    let account = native_account_from_starkli(
        &import.account_file,
        &import.keystore,
        &password,
        import.keystore_password_env.clone(),
    )?;

    let account_name = match &import.name {
        Some(name) => name.clone(),
        None => repository.generate_account_name()?,
    };
    let chain_id = get_chain_id(provider).await?;
    let migrated = save_account(&account_name, repository, chain_id, account)?;
    notify_if_migrated(migrated, ui);

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
}
