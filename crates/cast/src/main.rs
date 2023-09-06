use std::{env, fs};

use crate::starknet_commands::account::Account;
use crate::starknet_commands::{
    account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke, multicall::Multicall,
};
use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::{DEFAULT_ACCOUNTS_FILE, KEYSTORE_PASSWORD_ENV_VAR};
use cast::helpers::scarb_utils::{parse_scarb_config, CastConfig};
use cast::{
    account_file_exists, get_account_from_accounts_file, get_block_id, get_chain_id, get_provider,
    print_command_result,
};
use clap::{Parser, Subcommand};
use starknet::accounts::SingleOwnerAccount;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};

mod starknet_commands;

#[derive(Parser)]
#[command(version)]
#[command(about = "Cast - a Starknet Foundry CLI", long_about = None)]
struct Cli {
    /// Profile name in Scarb.toml config file
    #[clap(short, long)]
    profile: Option<String>,

    /// Path to Scarb.toml that is to be used; overrides default behaviour of searching for scarb.toml in current or parent directories
    #[clap(short = 's', long)]
    path_to_scarb_toml: Option<Utf8PathBuf>,

    /// RPC provider url address; overrides url from Scarb.toml
    #[clap(short = 'u', long = "url")]
    rpc_url: Option<String>,

    /// Account name to be used for contract declaration; overrides account from Scarb.toml; should be a path if used with --keystore
    #[clap(short = 'a', long)]
    account: Option<String>,

    /// Path to the file holding accounts info
    #[clap(short = 'f', long = "accounts-file")]
    accounts_file_path: Option<Utf8PathBuf>,

    /// Path to keystore file; if specified, --account should be a path to starkli JSON account file
    #[clap(short = 'k', long)]
    keystore: Option<Utf8PathBuf>,

    /// If passed, values will be displayed as integers, otherwise as hexes
    #[clap(short, long)]
    int_format: bool,

    /// If passed, output will be displayed in json format
    #[clap(short, long)]
    json: bool,

    /// If passed, command will wait until transaction is accepted or rejected
    #[clap(short, long)]
    wait: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Declare a contract
    Declare(Declare),

    /// Deploy a contract
    Deploy(Deploy),

    /// Call a contract
    Call(Call),

    /// Invoke a contract
    Invoke(Invoke),

    /// Execute multiple calls
    Multicall(Multicall),

    /// Create and deploy an account
    Account(Account),
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = parse_scarb_config(&cli.profile, &cli.path_to_scarb_toml)?;
    update_cast_config(&mut config, &cli);

    let provider = get_provider(&config.rpc_url)?;
    let mut account: Option<SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>> = None;

    match &cli.command {
        Commands::Call(_) | Commands::Account(_) => {}
        Commands::Multicall(value)
            if matches!(
                &value.command,
                starknet_commands::multicall::Commands::New(_)
            ) => {}
        _ => account = Some(get_account(&cli, &config, &provider).await?),
    }

    match cli.command {
        Commands::Declare(declare) => {
            let mut result = starknet_commands::declare::declare(
                &declare.contract,
                declare.max_fee,
                &mut account.unwrap(),
                &cli.path_to_scarb_toml,
                cli.wait,
            )
            .await;

            print_command_result("declare", &mut result, cli.int_format, cli.json)?;
            Ok(())
        }
        Commands::Deploy(deploy) => {
            let mut result = starknet_commands::deploy::deploy(
                deploy.class_hash,
                deploy.constructor_calldata,
                deploy.salt,
                deploy.unique,
                deploy.max_fee,
                &account.unwrap(),
                cli.wait,
            )
            .await;

            print_command_result("deploy", &mut result, cli.int_format, cli.json)?;
            Ok(())
        }
        Commands::Call(call) => {
            let block_id = get_block_id(&call.block_id)?;

            let mut result = starknet_commands::call::call(
                call.contract_address,
                call.function.as_ref(),
                call.calldata,
                &provider,
                block_id.as_ref(),
            )
            .await;

            print_command_result("call", &mut result, cli.int_format, cli.json)?;
            Ok(())
        }
        Commands::Invoke(invoke) => {
            let mut result = starknet_commands::invoke::invoke(
                invoke.contract_address,
                &invoke.function,
                invoke.calldata,
                invoke.max_fee,
                &mut account.unwrap(),
                cli.wait,
            )
            .await;

            print_command_result("invoke", &mut result, cli.int_format, cli.json)?;
            Ok(())
        }
        Commands::Multicall(multicall) => {
            match &multicall.command {
                starknet_commands::multicall::Commands::New(new) => {
                    let result = starknet_commands::multicall::new::new(
                        new.output_path.clone(),
                        new.overwrite,
                    )?;
                    println!("{result}");
                }
                starknet_commands::multicall::Commands::Run(run) => {
                    let mut result = starknet_commands::multicall::run::run(
                        &run.path,
                        &mut account.unwrap(),
                        run.max_fee,
                        cli.wait,
                    )
                    .await;

                    print_command_result("multicall run", &mut result, cli.int_format, cli.json)?;
                }
            }
            Ok(())
        }
        Commands::Account(account) => match account.command {
            account::Commands::Create(create) => {
                let chain_id = get_chain_id(&provider).await?;
                config.account = create.name;
                let mut result = starknet_commands::account::create::create(
                    &config,
                    &provider,
                    cli.path_to_scarb_toml,
                    chain_id,
                    create.salt,
                    create.add_profile,
                    create.class_hash,
                )
                .await;

                print_command_result("account create", &mut result, cli.int_format, cli.json)?;
                Ok(())
            }
            account::Commands::Deploy(deploy) => {
                account_file_exists(&config.accounts_file)?;
                let chain_id = get_chain_id(&provider).await?;
                let mut result = starknet_commands::account::deploy::deploy(
                    &provider,
                    config.accounts_file,
                    deploy.name,
                    chain_id,
                    deploy.max_fee,
                    cli.wait,
                    deploy.class_hash,
                )
                .await;

                print_command_result("account deploy", &mut result, cli.int_format, cli.json)?;
                Ok(())
            }
        },
    }
}

fn update_cast_config(config: &mut CastConfig, cli: &Cli) {
    macro_rules! clone_or_else {
        ($field:expr, $config_field:expr) => {
            $field.clone().unwrap_or_else(|| $config_field.clone())
        };
    }

    config.rpc_url = clone_or_else!(cli.rpc_url, config.rpc_url);
    config.account = clone_or_else!(cli.account, config.account);

    if config.accounts_file == Utf8PathBuf::default() {
        config.accounts_file = Utf8PathBuf::from(DEFAULT_ACCOUNTS_FILE);
    }
    let new_accounts_file = clone_or_else!(cli.accounts_file_path, config.accounts_file);

    config.accounts_file = Utf8PathBuf::from(shellexpand::tilde(&new_accounts_file).to_string());
}

async fn get_account<'a>(
    cli: &Cli,
    config: &'a CastConfig,
    provider: &'a JsonRpcClient<HttpTransport>,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    if let Some(keystore_path) = &cli.keystore {
        if !keystore_path.exists() {
            bail!("keystore file does not exist");
        }
        if cli.account.is_none() {
            bail!("--account is required when using keystore");
        }
        let path_to_account = Utf8PathBuf::from(cli.account.as_ref().unwrap());
        if !path_to_account.exists() {
            bail!("account file does not exist; when using --keystore, --account argument should be a path to the starkli JSON account file");
        }

        let password = match env::var(KEYSTORE_PASSWORD_ENV_VAR) {
            Ok(password) => {
                println!("Getting keystore password from {KEYSTORE_PASSWORD_ENV_VAR} evironment variable");
                password
            }
            _ => rpassword::prompt_password("Enter password: ")?,
        };
        let signer =
            LocalWallet::from(SigningKey::from_keystore(keystore_path, password.as_str())?);

        let account_info: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path_to_account)?)?;
        let address = FieldElement::from_hex_be(
            account_info
                .get("deployment")
                .and_then(|deployment| deployment.get("address"))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Failed to get address from account JSON file - make sure the account is deployed"))?
        )?;

        let chain_id = get_chain_id(provider).await?;
        return Ok(SingleOwnerAccount::new(provider, signer, address, chain_id));
    }

    account_file_exists(&config.accounts_file)?;
    let chain_id = get_chain_id(provider).await?;
    let account =
        get_account_from_accounts_file(&config.account, &config.accounts_file, provider, chain_id)?;
    Ok(account)
}
