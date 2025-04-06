use crate::starknet_commands::account::{
    add_created_profile_to_configuration, prepare_account_json, write_account_to_accounts_file,
};
use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::IntoConv;
use serde_json::json;
use sncast::helpers::braavos::{BraavosAccountFactory, assert_non_braavos_account_type};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{
    ARGENT_CLASS_HASH, BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH,
    CREATE_KEYSTORE_PASSWORD_ENV_VAR, OZ_CLASS_HASH,
};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::structs::AccountCreateResponse;
use sncast::{
    AccountType, Network, check_class_hash_exists, check_if_legacy_contract,
    extract_or_generate_salt, get_chain_id, get_keystore_password, handle_account_factory_error,
};
use starknet::accounts::{
    AccountDeploymentV3, AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory,
};
use starknet::core::types::FeeEstimate;
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::signers::{LocalWallet, SigningKey};
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
    #[arg(long, conflicts_with = "network")]
    pub add_profile: Option<String>,

    /// Custom contract class hash of declared contract
    #[arg(short, long, requires = "account_type")]
    pub class_hash: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[arg(long)]
    pub silent: bool,
}

pub async fn create(
    account: &str,
    accounts_file: &Utf8PathBuf,
    keystore: Option<Utf8PathBuf>,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    create: &Create,
) -> Result<AccountCreateResponse> {
    // TODO(#3118): Remove this check once braavos integration is restored
    assert_non_braavos_account_type(Some(create.account_type), create.class_hash)?;

    let add_profile = create.add_profile.clone();
    let salt = extract_or_generate_salt(create.salt);
    let class_hash = create.class_hash.unwrap_or(match create.account_type {
        AccountType::OpenZeppelin => OZ_CLASS_HASH,
        AccountType::Argent => ARGENT_CLASS_HASH,
        AccountType::Braavos => BRAAVOS_CLASS_HASH,
    });
    check_class_hash_exists(provider, class_hash).await?;

    let (account_json, max_fee) =
        generate_account(provider, salt, class_hash, create.account_type).await?;

    let address: Felt = account_json["address"]
        .as_str()
        .context("Invalid address")?
        .parse()?;

    let mut message = "Account successfully created. Prefund generated address with at least <max_fee> STRK tokens. It is good to send more in the case of higher demand.".to_string();

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
            create.account_type,
            &keystore,
            &account_path,
            legacy,
        )?;

        let deploy_command = generate_deploy_command_with_keystore(
            account,
            &keystore,
            create.rpc.url.as_deref(),
            create.rpc.network.as_ref(),
        );
        message.push_str(&deploy_command);
    } else {
        write_account_to_accounts_file(account, accounts_file, chain_id, account_json.clone())?;

        let deploy_command = generate_deploy_command(
            accounts_file,
            create.rpc.url.as_deref(),
            create.rpc.network.as_ref(),
            account,
        );
        message.push_str(&deploy_command);
    }

    if add_profile.is_some() {
        if let Some(url) = &create.rpc.url {
            let config = CastConfig {
                url: url.clone(),
                account: account.into(),
                accounts_file: accounts_file.into(),
                keystore,
                ..Default::default()
            };
            add_created_profile_to_configuration(create.add_profile.as_deref(), &config, None)?;
        } else {
            unreachable!("Conflicting arguments should be handled in clap");
        }
    }

    Ok(AccountCreateResponse {
        address: address.into_(),
        max_fee,
        add_profile: if add_profile.is_some() {
            format!(
                "Profile {} successfully added to snfoundry.toml",
                add_profile.clone().expect("Failed to get profile name")
            )
        } else {
            "--add-profile flag was not set. No profile added to snfoundry.toml".to_string()
        },
        message: if account_json["deployed"] == json!(false) {
            message
        } else {
            "Account already deployed".to_string()
        },
    })
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
) -> Result<(serde_json::Value, Felt)> {
    let chain_id = get_chain_id(provider).await?;
    let private_key = SigningKey::from_random();
    let signer = LocalWallet::from_signing_key(private_key.clone());

    let (address, fee_estimate) = match account_type {
        AccountType::OpenZeppelin => {
            let factory =
                OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
            get_address_and_deployment_fee(factory, salt).await?
        }
        AccountType::Argent => {
            let factory =
                ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;

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
    salt: Felt,
) -> Result<(Felt, FeeEstimate)>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v3(salt);
    Ok((deployment.address(), get_deployment_fee(&deployment).await?))
}

async fn get_deployment_fee<T>(
    account_deployment: &AccountDeploymentV3<'_, T>,
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

fn create_to_keystore(
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
        AccountType::OpenZeppelin => {
            json!({
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
            })
        }
        AccountType::Argent => {
            json!({
                "version": 1,
                "variant": {
                    "type": AccountType::Argent,
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
                    "type": AccountType::Braavos,
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

fn generate_network_flag(rpc_url: Option<&str>, network: Option<&Network>) -> String {
    if let Some(rpc_url) = rpc_url {
        format!("--url {rpc_url}")
    } else if let Some(network) = network {
        format!("--network {network}")
    } else {
        unreachable!("Either `--rpc_url` or `--network` must be provided.")
    }
}

fn generate_deploy_command(
    accounts_file: &Utf8PathBuf,
    rpc_url: Option<&str>,
    network: Option<&Network>,
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

    let network_flag = generate_network_flag(rpc_url, network);

    format!(
        "\n\nAfter prefunding the address, run:\n\
        sncast{accounts_flag} account deploy {network_flag} --name {account}"
    )
}

fn generate_deploy_command_with_keystore(
    account: &str,
    keystore: &Utf8PathBuf,
    rpc_url: Option<&str>,
    network: Option<&Network>,
) -> String {
    let network_flag = generate_network_flag(rpc_url, network);

    format!(
        "\n\nAfter prefunding the address, run:\n\
        sncast --account {account} --keystore {keystore} account deploy {network_flag}"
    )
}
