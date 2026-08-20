use crate::starknet_commands::account::{
    PrivateKeyArgs, generate_add_profile_message, notify_if_migrated, prepare_account_record,
    save_account, validate_private_key,
};
use crate::starknet_commands::utils::felt_or_id::ClassHash;
use anyhow::{Context, Result, bail};
use bigdecimal::BigDecimal;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;
use console::style;
use conversions::IntoConv;
use sncast::accounts::{AccountDeploymentService, AccountRecord, AccountRepository};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH};
use sncast::helpers::ledger;
use sncast::helpers::ledger::LedgerKeyLocatorAccount;
use sncast::helpers::rpc::{RpcArgs, generate_network_flag};
use sncast::response::account::create::AccountCreateResponse;
use sncast::response::ui::UI;
use sncast::signers::{
    KeystoreFile, KeystoreSpec, LedgerSpec, PrivateKeySpec, SignerSpec, keystore_password,
    resolve_keystore_path,
};
use sncast::{
    AccountType, SignerSource, check_class_hash_exists, check_if_legacy_contract,
    extract_or_generate_salt,
};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::{LocalWallet, Signer, SigningKey};
use starknet_types_core::felt::Felt;
use std::str::FromStr;

#[derive(Args, Debug)]
#[command(about = "Create an account with all important secrets")]
pub struct Create {
    /// Type of the account
    #[arg(value_enum, short = 't', long = "type", value_parser = AccountType::from_str, default_value_t = AccountType::OpenZeppelin)]
    pub account_type: AccountType,

    /// Account name under which account information is going to be saved
    #[arg(short, long)]
    pub name: Option<String>,

    /// Salt for the address
    #[arg(short, long)]
    pub salt: Option<Felt>,

    /// If passed, a profile with provided name and corresponding data will be created in snfoundry.toml
    #[arg(long)]
    pub add_profile: Option<String>,

    /// Custom contract class hash of declared contract (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short, long, requires = "account_type")]
    pub class_hash: Option<ClassHash>,

    #[command(flatten)]
    pub private_key_args: PrivateKeyArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(flatten)]
    pub ledger_key_locator: LedgerKeyLocatorAccount,

    /// Store the key in an encrypted keystore referenced by the native account
    #[arg(long, conflicts_with = "ledger_key_locator_account")]
    pub keystore: Option<Utf8PathBuf>,

    /// Environment variable containing the native keystore password
    #[arg(long, requires = "keystore")]
    pub keystore_password_env: Option<String>,
}

impl Create {
    pub fn resolved_class_hash(&self, config: &CastConfig) -> Result<Option<Felt>> {
        ClassHash::resolve_optional(self.class_hash.as_ref(), config)
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub async fn create(
    account: &str,
    repository: &AccountRepository,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    create: &Create,
    config: &CastConfig,
    signer_source: &SignerSource,
    ui: &UI,
) -> Result<AccountCreateResponse> {
    let salt = extract_or_generate_salt(create.salt);
    let class_hash = create
        .resolved_class_hash(config)?
        .unwrap_or(match create.account_type {
            AccountType::OpenZeppelin => OZ_CLASS_HASH,
            AccountType::Ready => READY_CLASS_HASH,
            AccountType::Braavos => BRAAVOS_CLASS_HASH,
        });

    let private_key = create
        .private_key_args
        .resolve_optional()?
        .map(validate_private_key)
        .transpose()?;
    check_class_hash_exists(provider, class_hash).await?;

    let generation_params = AccountGenerationParams {
        salt,
        class_hash,
        account_type: create.account_type,
        private_key,
        chain_id,
        keystore_password_env: create.keystore_password_env.clone(),
    };

    let (account_record, estimated_fee, generated_private_key) =
        generate_account(provider, signer_source, ui, generation_params).await?;

    let address = account_record.address.context("Invalid address")?;

    let estimated_fee_strk = BigDecimal::new(estimated_fee.into(), 18.into());
    let mut message = format!(
        "Account successfully created but it needs to be deployed. The estimated deployment fee is {} STRK. Prefund the account to cover deployment transaction fee",
        style(estimated_fee_strk).magenta()
    );

    match signer_source {
        SignerSource::Keystore(keystore) => {
            let private_key = generated_private_key.context("Generated private key missing")?;
            create_native_keystore(repository.path(), keystore, private_key, &account_record)?;
            let migrated = match save_account(account, repository, chain_id, account_record.clone())
            {
                Ok(migrated) => migrated,
                Err(error) => {
                    let _ =
                        KeystoreFile::remove(&resolve_keystore_path(repository.path(), keystore));
                    return Err(error);
                }
            };
            notify_if_migrated(migrated, ui);
            let deploy_command =
                generate_deploy_command(repository.path(), &create.rpc, config, account);
            message.push_str(&deploy_command);
        }
        SignerSource::Ledger(_) | SignerSource::AccountsFile => {
            let migrated = save_account(account, repository, chain_id, account_record.clone())?;
            notify_if_migrated(migrated, ui);
            let deploy_command =
                generate_deploy_command(repository.path(), &create.rpc, config, account);
            message.push_str(&deploy_command);
        }
    }

    let add_profile_message = generate_add_profile_message(
        create.add_profile.as_ref(),
        &create.rpc,
        account,
        repository.path(),
        config,
    )?;

    Ok(AccountCreateResponse {
        address: address.into_(),
        estimated_fee,
        add_profile: add_profile_message,
        message: if account_record.deployed == Some(false) {
            message
        } else {
            "Account already deployed".to_string()
        },
    })
}

struct AccountGenerationParams {
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    private_key: Option<Felt>,
    chain_id: Felt,
    keystore_password_env: Option<String>,
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    signer_source: &SignerSource,
    ui: &UI,
    params: AccountGenerationParams,
) -> Result<(AccountRecord, u128, Option<Felt>)> {
    if let SignerSource::Ledger(ledger_path) = signer_source {
        let signer = ledger::create_ledger_signer(ledger_path, ui, false).await?;
        let signer_spec = SignerSpec::Ledger(LedgerSpec::new(ledger_path.clone()));

        let (account, estimated_fee) = finalize_account_generation(
            provider,
            signer,
            signer_spec,
            params.salt,
            params.class_hash,
            params.account_type,
            params.chain_id,
        )
        .await?;
        Ok((account, estimated_fee, None))
    } else {
        let private_key = params
            .private_key
            .map_or_else(SigningKey::from_random, SigningKey::from_secret_scalar);
        let signer = LocalWallet::from_signing_key(private_key.clone());
        let secret = private_key.secret_scalar();
        let signer_spec = match signer_source {
            SignerSource::Keystore(path) => SignerSpec::Keystore(KeystoreSpec::new(
                path.clone(),
                params.keystore_password_env,
            )),
            _ => SignerSpec::PrivateKey(PrivateKeySpec::new(secret)),
        };

        let (account, estimated_fee) = finalize_account_generation(
            provider,
            signer,
            signer_spec,
            params.salt,
            params.class_hash,
            params.account_type,
            params.chain_id,
        )
        .await?;
        Ok((account, estimated_fee, Some(secret)))
    }
}

fn create_native_keystore(
    accounts_file: &Utf8Path,
    keystore_path: &Utf8PathBuf,
    private_key: Felt,
    account: &AccountRecord,
) -> Result<()> {
    let resolved_path = resolve_keystore_path(accounts_file, keystore_path);
    let SignerSpec::Keystore(spec) = &account.signer else {
        bail!("native keystore account has an invalid signer")
    };
    let password = keystore_password(spec)?;
    KeystoreFile::create(&resolved_path, private_key, &password).map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
async fn finalize_account_generation<S>(
    provider: &JsonRpcClient<HttpTransport>,
    signer: S,
    signer_spec: SignerSpec,
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    chain_id: Felt,
) -> Result<(AccountRecord, u128)>
where
    S: Signer + Send + Sync,
    <S as Signer>::GetPublicKeyError: 'static,
{
    let public_key = signer.get_public_key().await?.scalar();

    let (address, estimated_fee) = AccountDeploymentService::estimate_fee(
        provider,
        account_type,
        class_hash,
        signer,
        salt,
        chain_id,
    )
    .await?;

    let legacy = check_if_legacy_contract(Some(class_hash), address, provider).await?;

    let account = prepare_account_record(
        signer_spec,
        public_key,
        address,
        false,
        legacy,
        account_type,
        Some(class_hash),
        Some(salt),
    );

    Ok((account, estimated_fee.overall_fee))
}

fn generate_deploy_command(
    accounts_file: &Utf8Path,
    rpc_args: &RpcArgs,
    config: &CastConfig,
    account: &str,
) -> String {
    let accounts_flag = if accounts_file
        .to_string()
        .contains("starknet_accounts/starknet_open_zeppelin_accounts.json")
    {
        String::new()
    } else {
        format!(" --accounts-file {accounts_file}")
    };

    let network_flag = generate_network_flag(rpc_args, config);

    format!(
        "\n\nAfter prefunding the account, run:\n\
        sncast{accounts_flag} account deploy {network_flag} --name {account}"
    )
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn creates_native_keystore_relative_to_accounts_file() {
        const PASSWORD_ENV: &str = "SNCAST_TEST_NATIVE_KEYSTORE_PASSWORD";
        // SAFETY: This test uses a unique variable, writes one fixed value, and never removes it.
        unsafe { std::env::set_var(PASSWORD_ENV, "secret") };

        let directory = tempdir().unwrap();
        let directory = Utf8PathBuf::from_path_buf(directory.path().to_owned()).unwrap();
        let accounts_file = directory.join("config/accounts.json");
        let keystore_path = Utf8PathBuf::from("keys/alice.json");
        let account = AccountRecord {
            public_key: SigningKey::from_secret_scalar(Felt::ONE)
                .verifying_key()
                .scalar(),
            address: None,
            salt: None,
            deployed: None,
            class_hash: None,
            legacy: None,
            account_type: None,
            signer: SignerSpec::Keystore(KeystoreSpec::new(
                keystore_path.clone(),
                Some(PASSWORD_ENV.to_owned()),
            )),
        };

        create_native_keystore(&accounts_file, &keystore_path, Felt::ONE, &account).unwrap();

        let resolved = resolve_keystore_path(&accounts_file, &keystore_path);
        assert_eq!(
            SigningKey::from_keystore(resolved, "secret")
                .unwrap()
                .secret_scalar(),
            Felt::ONE
        );
    }
}
