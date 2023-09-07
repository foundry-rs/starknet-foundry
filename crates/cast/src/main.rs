use crate::starknet_commands::account::Account;
use crate::starknet_commands::{
    account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke, multicall::Multicall,
};
use anyhow::Result;
use camino::Utf8PathBuf;
use cast::helpers::constants::{DEFAULT_ACCOUNTS_FILE, DEFAULT_MULTICALL_CONTENTS};
use cast::helpers::scarb_utils::{parse_scarb_config, CastConfig};
use cast::{
    account_file_exists, get_account, get_block_id, get_chain_id, get_provider,
    print_command_result,
};
use clap::{Parser, Subcommand};

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

    /// Account name to be used for contract declaration; overrides account from Scarb.toml
    #[clap(short = 'a', long)]
    account: Option<String>,

    /// Path to the file holding accounts info
    #[clap(short = 'f', long = "accounts-file")]
    accounts_file_path: Option<Utf8PathBuf>,

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

    match cli.command {
        Commands::Declare(declare) => {
            account_file_exists(&config.accounts_file)?;
            let chain_id = get_chain_id(&provider).await?;
            let mut account =
                get_account(&config.account, &config.accounts_file, &provider, chain_id)?;

            let mut result = starknet_commands::declare::declare(
                &declare.contract,
                declare.max_fee,
                &mut account,
                &cli.path_to_scarb_toml,
                cli.wait,
            )
            .await;

            print_command_result("declare", &mut result, cli.int_format, cli.json)?;

            Ok(())
        }
        Commands::Deploy(deploy) => {
            account_file_exists(&config.accounts_file)?;
            let chain_id = get_chain_id(&provider).await?;
            let account = get_account(&config.account, &config.accounts_file, &provider, chain_id)?;

            let mut result = starknet_commands::deploy::deploy(
                deploy.class_hash,
                deploy.constructor_calldata,
                deploy.salt,
                deploy.unique,
                deploy.max_fee,
                &account,
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
            account_file_exists(&config.accounts_file)?;
            let chain_id = get_chain_id(&provider).await?;
            let mut account =
                get_account(&config.account, &config.accounts_file, &provider, chain_id)?;
            let mut result = starknet_commands::invoke::invoke(
                invoke.contract_address,
                &invoke.function,
                invoke.calldata,
                invoke.max_fee,
                &mut account,
                cli.wait,
            )
            .await;
            print_command_result("invoke", &mut result, cli.int_format, cli.json)?;

            Ok(())
        }
        Commands::Multicall(multicall) => {
            match &multicall.command {
                starknet_commands::multicall::Commands::New(new) => {
                    if let Some(output_path) = &new.output_path {
                        let mut result =
                            starknet_commands::multicall::new::new(output_path, new.overwrite);
                        print_command_result(
                            "multicall new",
                            &mut result,
                            cli.int_format,
                            cli.json,
                        )?;
                    } else {
                        println!("{DEFAULT_MULTICALL_CONTENTS}");
                    }
                }
                starknet_commands::multicall::Commands::Run(run) => {
                    account_file_exists(&config.accounts_file)?;
                    let chain_id = get_chain_id(&provider).await?;
                    let mut account =
                        get_account(&config.account, &config.accounts_file, &provider, chain_id)?;
                    let mut result = starknet_commands::multicall::run::run(
                        &run.path,
                        &mut account,
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
