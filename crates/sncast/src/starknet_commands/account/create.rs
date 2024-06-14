use crate::starknet_commands::account::{
    add_created_profile_to_configuration, prepare_account_json, write_account_to_accounts_file,
    AccountType,
};
use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use serde_json::json;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{
    ARGENT_CLASS_HASH, BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH,
    CREATE_KEYSTORE_PASSWORD_ENV_VAR, OZ_CLASS_HASH,
};
use sncast::response::structs::{AccountCreateResponse, Felt};
use sncast::{
    check_class_hash_exists, check_if_legacy_contract, extract_or_generate_salt, get_chain_id,
    get_keystore_password, handle_account_factory_error,
};
use starknet::accounts::{
    AccountDeployment, AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory,
};
use starknet::core::types::{FeeEstimate, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};

#[derive(Args, Debug)]
#[command(about = "Create an account with all important secrets")]
pub struct Create {
    /// Type of the account
    #[clap(value_enum, short = 't', long = "type", default_value_t = AccountType::Oz)]
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

    let (account_json, max_fee) =
        generate_account(provider, salt, class_hash, &account_type).await?;

    let address = account_json["address"]
        .as_str()
        .context("Invalid address")?
        .parse()?;

    if let Some(keystore) = keystore.clone() {
        let account_path = Utf8PathBuf::from(&account);
        if account_path == Utf8PathBuf::default() {
            bail!("Argument `--account` must be passed and be a path when using `--keystore`");
        }

        let private_key = account_json["private_key"]
            .as_str()
            .context("Invalid private_key")?
            .parse()?;
        let legacy = account_json["legacy"]
            .as_bool()
            .expect("Invalid legacy entry");

        create_to_keystore(
            private_key,
            salt,
            class_hash,
            &account_type,
            &keystore,
            &account_path,
            legacy,
        )?;
    } else {
        write_account_to_accounts_file(account, accounts_file, chain_id, account_json.clone())?;
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
        message: if account_json["deployed"] == json!(false) {
            "Account successfully created. Prefund generated address with at least <max_fee> tokens. It is good to send more in the case of higher demand.".to_string()
        } else {
            "Account already deployed".to_string()
        },
    })
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
    account_type: &AccountType,
) -> Result<(serde_json::Value, FieldElement)> {
    let chain_id = get_chain_id(provider).await?;
    let private_key = SigningKey::from_random();
    let signer = LocalWallet::from_signing_key(private_key.clone());

    let (address, fee_estimate) = match account_type {
        AccountType::Oz => {
            let factory =
                OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
            get_address_and_deployment_fee(factory, salt).await?
        }
        AccountType::Argent => {
            let factory = ArgentAccountFactory::new(
                class_hash,
                chain_id,
                FieldElement::ZERO,
                signer,
                provider,
            )
            .await?;
            get_address_and_deployment_fee(factory, salt).await?
        }
        AccountType::Braavos => {
            let factory = BraavosAccountFactory::new(
                class_hash,
                BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                chain_id,
                signer,
                provider,
            )
            .await?;
            get_address_and_deployment_fee(factory, salt).await?
        }
    };

    let legacy = check_if_legacy_contract(Some(class_hash), address, provider).await?;

    let account_json = prepare_account_json(
        &private_key,
        address,
        false,
        legacy,
        account_type,
        Some(class_hash),
        Some(salt),
    );

    Ok((account_json, fee_estimate.overall_fee))
}

async fn get_address_and_deployment_fee<T>(
    account_factory: T,
    salt: FieldElement,
) -> Result<(FieldElement, FeeEstimate)>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy(salt);
    Ok((deployment.address(), get_deployment_fee(&deployment).await?))
}

async fn get_deployment_fee<'a, T>(
    account_deployment: &AccountDeployment<'a, T>,
) -> Result<FeeEstimate>
where
    T: AccountFactory + Sync,
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

#[allow(clippy::too_many_arguments)]
fn create_to_keystore(
    private_key: FieldElement,
    salt: FieldElement,
    class_hash: FieldElement,
    account_type: &AccountType,
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
        AccountType::Oz => {
            json!({
                "version": 1,
                "variant": {
                    "type": format!("{account_type}"),
                    "version": 1,
                    "public_key": format!("{:#x}", private_key.verifying_key().scalar()),
                    "legacy": legacy,
                },
                "deployment": {
                    "status": "undeployed",
                    "class_hash": format!("{class_hash:#x}"),
                    "salt": format!("{salt:#x}"),
                }
            })
        }
        AccountType::Argent => {
            json!({
                "version": 1,
                "variant": {
                    "type": format!("{account_type}"),
                    "version": 1,
                    "owner": format!("{:#x}", private_key.verifying_key().scalar()),
                    "guardian": "0x0",
                },
                "deployment": {
                    "status": "undeployed",
                    "class_hash": format!("{class_hash:#x}"),
                    "salt": format!("{salt:#x}"),
                }
            })
        }
        AccountType::Braavos => {
            json!(
                {
                  "version": 1,
                  "variant": {
                    "type": format!("{account_type}"),
                    "version": 1,
                    "multisig": {
                      "status": "off"
                    },
                    "signers": [
                      {
                        "type": "stark",
                        "public_key": format!("{:#x}", private_key.verifying_key().scalar())
                      }
                    ]
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
                }
            )
        }
    };

    write_account_to_file(&account_json, account_path)
}

fn write_account_to_file(
    account_json: &serde_json::Value,
    account_file: &Utf8PathBuf,
) -> Result<()> {
    std::fs::create_dir_all(account_file.clone().parent().unwrap())?;
    std::fs::write(
        account_file.clone(),
        serde_json::to_string_pretty(&account_json).unwrap(),
    )?;
    Ok(())
}
