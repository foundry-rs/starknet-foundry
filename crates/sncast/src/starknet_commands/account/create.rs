use crate::starknet_commands::account::{
    generate_add_profile_message, prepare_account_json, write_account_to_accounts_file,
};
use anyhow::{Context, Result, anyhow, bail};
use bigdecimal::BigDecimal;
use camino::Utf8PathBuf;
use clap::Args;
use console::style;
use conversions::IntoConv;
use foundry_ui::components::warning::WarningMessage;
use serde_json::json;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{
    BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH, CREATE_KEYSTORE_PASSWORD_ENV_VAR,
    OZ_CLASS_HASH, READY_CLASS_HASH,
};
use sncast::helpers::ledger;
use sncast::helpers::rpc::{RpcArgs, generate_network_flag};
use sncast::response::account::create::AccountCreateResponse;
use sncast::response::ui::UI;
use sncast::{
    AccountType, SignerSource, check_class_hash_exists, check_if_legacy_contract,
    extract_or_generate_salt, get_keystore_password, handle_account_factory_error,
};
use starknet_rust::accounts::{
    AccountDeploymentV3, AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory,
};
use starknet_rust::core::types::FeeEstimate;
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

    /// Custom contract class hash of declared contract
    #[arg(short, long, requires = "account_type")]
    pub class_hash: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub async fn create(
    account: &str,
    accounts_file: &Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    create: &Create,
    config: &CastConfig,
    signer_source: &SignerSource,
    ui: &UI,
) -> Result<AccountCreateResponse> {
    // TODO(#3556): Remove this warning once we drop Argent account type
    if create.account_type == AccountType::Argent {
        ui.print_warning(WarningMessage::new(
            "Argent has rebranded as Ready. The `argent` option for the `--type` flag in `account create` is deprecated, please use `ready` instead.",
        ));
        ui.print_blank_line();
    }

    let salt = extract_or_generate_salt(create.salt);
    let class_hash = create.class_hash.unwrap_or(match create.account_type {
        AccountType::OpenZeppelin => OZ_CLASS_HASH,
        AccountType::Argent | AccountType::Ready => READY_CLASS_HASH,
        AccountType::Braavos => BRAAVOS_CLASS_HASH,
    });
    check_class_hash_exists(provider, class_hash).await?;

    let (account_json, estimated_fee) = generate_account(
        provider,
        salt,
        class_hash,
        create.account_type,
        signer_source,
        chain_id,
        ui,
    )
    .await?;

    let address: Felt = account_json["address"]
        .as_str()
        .context("Invalid address")?
        .parse()?;

    let estimated_fee_strk = BigDecimal::new(estimated_fee.into(), 18.into());
    let mut message = format!(
        "Account successfully created but it needs to be deployed. The estimated deployment fee is {} STRK. Prefund the account to cover deployment transaction fee",
        style(estimated_fee_strk).magenta()
    );

    match signer_source {
        sncast::SignerSource::Keystore(keystore) => {
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
                keystore,
                &account_path,
                legacy,
            )?;

            let deploy_command =
                generate_deploy_command_with_keystore(account, keystore, &create.rpc, config);
            message.push_str(&deploy_command);
        }
        sncast::SignerSource::Ledger(ledger_path) => {
            write_account_to_accounts_file(account, accounts_file, chain_id, account_json.clone())?;
            let deploy_command = generate_deploy_command_with_ledger(
                accounts_file,
                &create.rpc,
                config,
                account,
                ledger_path,
            );
            message.push_str(&deploy_command);
        }
        sncast::SignerSource::AccountsFile => {
            write_account_to_accounts_file(account, accounts_file, chain_id, account_json.clone())?;
            let deploy_command =
                generate_deploy_command(accounts_file, &create.rpc, config, account);
            message.push_str(&deploy_command);
        }
    }

    let add_profile_message = generate_add_profile_message(
        create.add_profile.as_ref(),
        &create.rpc,
        account,
        accounts_file,
        match signer_source {
            sncast::SignerSource::Keystore(path) => Some(path.clone()),
            _ => None,
        },
        config,
    )?;

    Ok(AccountCreateResponse {
        address: address.into_(),
        estimated_fee,
        add_profile: add_profile_message,
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
    signer_source: &SignerSource,
    chain_id: Felt,
    ui: &UI,
) -> Result<(serde_json::Value, u128)> {
    if let SignerSource::Ledger(ledger_path) = signer_source {
        let public_key = ledger::get_ledger_public_key(ledger_path, true, ui).await?;
        let signer = ledger::create_ledger_signer(ledger_path, ui).await?;

        finalize_account_generation(
            provider,
            signer,
            public_key,
            None,
            Some(ledger_path.clone()),
            salt,
            class_hash,
            account_type,
            chain_id,
        )
        .await
    } else {
        let private_key = SigningKey::from_random();
        let public_key = private_key.verifying_key().scalar();
        let signer = LocalWallet::from_signing_key(private_key.clone());

        finalize_account_generation(
            provider,
            signer,
            public_key,
            Some(&private_key),
            None,
            salt,
            class_hash,
            account_type,
            chain_id,
        )
        .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn finalize_account_generation<S>(
    provider: &JsonRpcClient<HttpTransport>,
    signer: S,
    public_key: Felt,
    private_key: Option<&SigningKey>,
    ledger_path: Option<String>,
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    chain_id: Felt,
) -> Result<(serde_json::Value, u128)>
where
    S: Signer + Send + Sync,
    <S as Signer>::GetPublicKeyError: 'static,
{
    let (address, estimated_fee) =
        create_factory_and_estimate_fee(account_type, class_hash, chain_id, signer, provider, salt)
            .await?;

    let legacy = check_if_legacy_contract(Some(class_hash), address, provider).await?;

    let account_json = prepare_account_json(
        private_key,
        public_key,
        address,
        false,
        legacy,
        account_type,
        Some(class_hash),
        Some(salt),
        ledger_path,
    );

    Ok((account_json, estimated_fee.overall_fee))
}

async fn create_factory_and_estimate_fee<S>(
    account_type: AccountType,
    class_hash: Felt,
    chain_id: Felt,
    signer: S,
    provider: &JsonRpcClient<HttpTransport>,
    salt: Felt,
) -> Result<(Felt, FeeEstimate)>
where
    S: Signer + Send + Sync,
    <S as Signer>::GetPublicKeyError: 'static,
{
    // Each account type produces a different concrete factory type (OpenZeppelinAccountFactory,
    // ArgentAccountFactory, BraavosAccountFactory). Because `AccountFactory` is not object-safe
    // (it has `Sized` as a supertrait), we cannot unify them behind `Box<dyn AccountFactory>` or
    // store them in a single typed variable. Instead we call the downstream function inside each
    // match arm so that the factory's concrete type stays local to that arm.
    match account_type {
        AccountType::OpenZeppelin => {
            let factory =
                OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
            get_address_and_deployment_fee(factory, salt).await
        }
        AccountType::Argent | AccountType::Ready => {
            let factory =
                ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;
            get_address_and_deployment_fee(factory, salt).await
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
            get_address_and_deployment_fee(factory, salt).await
        }
    }
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
        AccountType::Argent | AccountType::Ready => {
            json!({
                "version": 1,
                "variant": {
                    // TODO(#3556): Remove hardcoded "argent" and use format! with `AccountType::Ready`
                    "type": "argent",
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

fn generate_accounts_file_flag(accounts_file: &Utf8PathBuf) -> String {
    if accounts_file
        .to_string()
        .contains("starknet_accounts/starknet_open_zeppelin_accounts.json")
    {
        String::new()
    } else {
        format!("--accounts-file {accounts_file}")
    }
}

fn generate_deploy_command(
    accounts_file: &Utf8PathBuf,
    rpc_args: &RpcArgs,
    config: &CastConfig,
    account: &str,
) -> String {
    let accounts_flag = generate_accounts_file_flag(accounts_file);
    let network_flag = generate_network_flag(rpc_args, config);

    format!(
        "\n\nAfter prefunding the account, run:\n\
        sncast {accounts_flag} account deploy {network_flag} --name {account}"
    )
}

fn generate_deploy_command_with_keystore(
    account: &str,
    keystore: &Utf8PathBuf,
    rpc_args: &RpcArgs,
    config: &CastConfig,
) -> String {
    let network_flag = generate_network_flag(rpc_args, config);

    format!(
        "\n\nAfter prefunding the account, run:\n\
        sncast --account {account} --keystore {keystore} account deploy {network_flag}"
    )
}

fn generate_deploy_command_with_ledger(
    accounts_file: &Utf8PathBuf,
    rpc_args: &RpcArgs,
    config: &CastConfig,
    account: &str,
    ledger_path: &String,
) -> String {
    let accounts_flag = generate_accounts_file_flag(accounts_file);
    let network_flag = generate_network_flag(rpc_args, config);

    format!(
        "\n\nAfter prefunding the account, run:\n\
        sncast {accounts_flag} --ledger-path \"{ledger_path}\" account deploy {network_flag} --name {account}"
    )
}
