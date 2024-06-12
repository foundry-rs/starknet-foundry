use crate::starknet_commands::account::{
    add_created_profile_to_configuration, write_account_to_accounts_file,
};
use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use sncast::helpers::accounts_format;
use sncast::helpers::accounts_format::{AccountData, AccountKeystore, AccountType};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{
    ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, CREATE_KEYSTORE_PASSWORD_ENV_VAR, OZ_CLASS_HASH,
};
use sncast::helpers::factory::{create_account_factory, AccountFactory};
use sncast::response::structs::{AccountCreateResponse, Felt};
use sncast::{
    check_class_hash_exists, check_if_legacy_contract, extract_or_generate_salt, get_chain_id,
    get_keystore_password, handle_account_factory_error,
};
use starknet::accounts::AccountDeployment;
use starknet::core::types::{FeeEstimate, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};

#[derive(Args, Debug)]
#[command(about = "Create an account with all important secrets")]
pub struct Create {
    /// Type of the account
    #[clap(value_enum, short = 't', long = "type", default_value_t = accounts_format::AccountType::Oz)]
    pub account_type: AccountType,

    /// Account name under which account information is going to be saved
    #[clap(short, long)]
    pub name: Option<String>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with provided name and corresponding data will be created in snfoundry.toml
    #[clap(long)]
    pub add_profile: Option<String>,

    /// Custom contract class hash of declared contract
    #[clap(short, long, requires = "account_type")]
    pub class_hash: Option<FieldElement>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    rpc_url: &str,
    account: &str,
    accounts_file: &Utf8PathBuf,
    keystore: Option<Utf8PathBuf>,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: FieldElement,
    account_type: AccountType,
    salt: Option<FieldElement>,
    add_profile: Option<String>,
    class_hash: Option<FieldElement>,
) -> Result<AccountCreateResponse> {
    let salt = extract_or_generate_salt(salt);
    let class_hash = class_hash.unwrap_or(match account_type {
        AccountType::Oz => OZ_CLASS_HASH,
        AccountType::Argent => ARGENT_CLASS_HASH,
        AccountType::Braavos => BRAAVOS_CLASS_HASH,
    });
    check_class_hash_exists(provider, class_hash).await?;

    let (account_data, max_fee) =
        generate_account(provider, salt, class_hash, &account_type).await?;

    let address = account_data.address.context("Failed to get address")?;

    if let Some(keystore) = keystore.clone() {
        let account_path = Utf8PathBuf::from(&account);
        if account_path == Utf8PathBuf::default() {
            bail!("Argument `--account` must be passed and be a path when using `--keystore`");
        }

        write_to_keystore(&account_data, &keystore, &account_path)?;
    } else {
        write_account_to_accounts_file(account, accounts_file, chain_id, &account_data)?;
    }

    if add_profile.is_some() {
        let config = CastConfig {
            url: rpc_url.into(),
            account: account.into(),
            accounts_file: accounts_file.into(),
            keystore,
            ..Default::default()
        };
        add_created_profile_to_configuration(&add_profile, &config, &None)?;
    }

    Ok(AccountCreateResponse {
        address: Felt(address),
        max_fee: Felt(max_fee),
        add_profile: if add_profile.is_some() {
            format!(
                "Profile {} successfully added to snfoundry.toml",
                add_profile.clone().expect("Failed to get profile name")
            )
        } else {
            "--add-profile flag was not set. No profile added to snfoundry.toml".to_string()
        },
        message: if account_data.clone().deployed.unwrap_or(false) {
            "Account already deployed".to_string()
        } else {
            "Account successfully created. Prefund generated address with at least <max_fee> tokens. It is good to send more in the case of higher demand.".to_string()
        },
    })
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
    account_type: &AccountType,
) -> Result<(AccountData, FieldElement)> {
    let chain_id = get_chain_id(provider).await?;
    let private_key = SigningKey::from_random();
    let signer = LocalWallet::from_signing_key(private_key.clone());

    let factory =
        create_account_factory(account_type.clone(), class_hash, chain_id, signer, provider)
            .await?;

    let (address, fee_estimate) = match factory {
        AccountFactory::Oz(factory) => get_address_and_deployment_fee(factory, salt).await,
        AccountFactory::Argent(factory) => get_address_and_deployment_fee(factory, salt).await,
        AccountFactory::Braavos(factory) => get_address_and_deployment_fee(factory, salt).await,
    }?;

    let legacy = check_if_legacy_contract(Some(class_hash), address, provider).await?;

    let account = AccountData {
        private_key: private_key.secret_scalar(),
        public_key: private_key.verifying_key().scalar(),
        address: Some(address),
        deployed: Some(false),
        legacy: Some(legacy),
        account_type: Some(account_type.clone()),
        class_hash: Some(class_hash),
        salt: Some(salt),
    };

    Ok((account, fee_estimate.overall_fee))
}

async fn get_address_and_deployment_fee<T>(
    account_factory: T,
    salt: FieldElement,
) -> Result<(FieldElement, FeeEstimate)>
where
    T: starknet::accounts::AccountFactory + Sync,
{
    let deployment = account_factory.deploy(salt);
    Ok((deployment.address(), get_deployment_fee(&deployment).await?))
}

async fn get_deployment_fee<'a, T>(
    account_deployment: &AccountDeployment<'a, T>,
) -> Result<FeeEstimate>
where
    T: starknet::accounts::AccountFactory + Sync,
{
    let fee_estimate = account_deployment.estimate_fee().await;

    match fee_estimate {
        Ok(fee_estimate) => Ok(fee_estimate),
        Err(err) => Err(anyhow!(
            "Failed to estimate account deployment fee. Reason: {}",
            handle_account_factory_error::<T>(err)
        )),
    }
}

fn write_to_keystore(
    account_data: &AccountData,
    keystore_path: &Utf8PathBuf,
    account_path: &Utf8PathBuf,
) -> Result<()> {
    if keystore_path.exists() {
        bail!("Keystore file {keystore_path} already exists");
    }
    if account_path.exists() {
        bail!("Account file {account_path} already exists");
    }
    let password = get_keystore_password(CREATE_KEYSTORE_PASSWORD_ENV_VAR)?;
    let private_key = SigningKey::from_secret_scalar(account_data.private_key);
    private_key.save_as_keystore(keystore_path, &password)?;

    let account_json = serde_json::to_value(AccountKeystore::try_from(account_data)?)?;
    write_account_to_file(&account_json, account_path)
}

fn write_account_to_file(
    account_json: &serde_json::Value,
    account_file: &Utf8PathBuf,
) -> Result<()> {
    std::fs::create_dir_all(account_file.parent().unwrap())?;
    std::fs::write(
        account_file,
        serde_json::to_string_pretty(&account_json).unwrap(),
    )?;
    Ok(())
}
