use crate::helpers::scarb_utils::parse_scarb_config;
use crate::starknet_commands::account::Account;
use crate::starknet_commands::{
    account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke, multicall::Multicall,
};
use anyhow::Result;
use camino::Utf8PathBuf;
use cast::{account_file_exists, get_account, get_block_id, get_provider, print_command_result};
use clap::{Parser, Subcommand};

mod helpers;
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

    /// Network name, one of: testnet, testnet2, mainnet; overrides network from Scarb.toml
    #[clap(short = 'n', long)]
    network: Option<String>,

    /// Account name to be used for contract declaration; overrides account from Scarb.toml
    #[clap(short = 'a', long)]
    account: Option<String>,

    /// Path to the file holding accounts info
    #[clap(
        short = 'f',
        long = "accounts-file",
        default_value = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json"
    )]
    accounts_file_path: Utf8PathBuf,

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

    let accounts_file_path =
        Utf8PathBuf::from(shellexpand::tilde(&cli.accounts_file_path).to_string());

    let config = parse_scarb_config(&cli.profile, &cli.path_to_scarb_toml)?;

    let rpc_url = cli.rpc_url.unwrap_or(config.rpc_url);
    let network = cli.network.unwrap_or(config.network);
    let account = cli.account.unwrap_or(config.account);

    let provider = get_provider(&rpc_url, &network).await?;

    match cli.command {
        Commands::Declare(declare) => {
            account_file_exists(&accounts_file_path)?;
            let mut account = get_account(&account, &accounts_file_path, &provider, &network)?;

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
            account_file_exists(&accounts_file_path)?;
            let account = get_account(&account, &accounts_file_path, &provider, &network)?;

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
            account_file_exists(&accounts_file_path)?;
            let mut account = get_account(&account, &accounts_file_path, &provider, &network)?;
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
                    let result = starknet_commands::multicall::new::new(
                        new.output_path.clone(),
                        new.overwrite,
                    )?;
                    println!("{result}");
                }
                starknet_commands::multicall::Commands::Run(run) => {
                    account_file_exists(&accounts_file_path)?;
                    let mut account =
                        get_account(&account, &accounts_file_path, &provider, &network)?;
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
                let mut result = starknet_commands::account::create::create(
                    &provider,
                    rpc_url,
                    accounts_file_path,
                    cli.path_to_scarb_toml,
                    create.name,
                    &network,
                    create.salt,
                    create.add_profile,
                )
                .await;

                print_command_result("account create", &mut result, cli.int_format, cli.json)?;
                Ok(())
            }
            account::Commands::Deploy(deploy) => {
                account_file_exists(&accounts_file_path)?;
                let mut result = starknet_commands::account::deploy::deploy(
                    &provider,
                    accounts_file_path,
                    deploy.name,
                    &network,
                    deploy.max_fee,
                    cli.wait,
                )
                .await;

                print_command_result("account deploy", &mut result, cli.int_format, cli.json)?;
                Ok(())
            }
        },
    }
}
