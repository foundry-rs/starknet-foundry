use crate::helpers::scarb_utils::parse_scarb_config;
use crate::starknet_commands::account::Account;
use crate::starknet_commands::{
    account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke, multicall::Multicall,
};
use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use cast::{get_account, get_block_id, get_provider, print_formatted};
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

    /// execute multiple calls
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
    if !&accounts_file_path.exists() {
        bail! {"Accounts file {} does not exist! Make sure to supply correct path to accounts file.", cli.accounts_file_path}
    }

    let config = parse_scarb_config(&cli.profile, cli.path_to_scarb_toml)?;

    let rpc_url = cli.rpc_url.unwrap_or(config.rpc_url);
    let network = cli.network.unwrap_or(config.network);
    let account = cli.account.unwrap_or(config.account);

    match cli.command {
        Commands::Declare(declare) => {
            let provider = get_provider(&rpc_url, &network).await?;
            let mut account = get_account(&account, &accounts_file_path, &provider, &network)?;

            let result = starknet_commands::declare::declare(
                &declare.contract,
                declare.max_fee,
                &mut account,
            )
            .await;

            match result {
                Ok(declared_contract) => print_formatted(
                    vec![
                        ("command", "Declare".to_string()),
                        ("class_hash", format!("{}", declared_contract.class_hash)),
                        (
                            "transaction_hash",
                            format!("{}", declared_contract.transaction_hash),
                        ),
                    ],
                    cli.int_format,
                    cli.json,
                    false,
                )?,
                Err(error) => {
                    print_formatted(
                        vec![("error", error.to_string())],
                        cli.int_format,
                        cli.json,
                        true,
                    )?;
                }
            }

            Ok(())
        }
        Commands::Deploy(deploy) => {
            let provider = get_provider(&rpc_url, &network).await?;
            let account = get_account(&account, &accounts_file_path, &provider, &network)?;

            let result = starknet_commands::deploy::deploy(
                deploy.class_hash,
                deploy.constructor_calldata,
                deploy.salt,
                deploy.unique,
                deploy.max_fee,
                &account,
            )
            .await;
            starknet_commands::deploy::print_deploy_result(result, cli.int_format, cli.json)?;

            Ok(())
        }
        Commands::Call(call) => {
            let provider = get_provider(&rpc_url, &network).await?;
            let block_id = get_block_id(&call.block_id)?;

            let result = starknet_commands::call::call(
                call.contract_address,
                call.function.as_ref(),
                call.calldata,
                &provider,
                block_id.as_ref(),
            )
            .await;

            match result {
                Ok(response) => print_formatted(
                    vec![
                        ("command", "Call".to_string()),
                        ("response", format!("{response:?}")),
                    ],
                    cli.int_format,
                    cli.json,
                    false,
                )?,
                Err(error) => {
                    print_formatted(
                        vec![("error", error.to_string())],
                        cli.int_format,
                        cli.json,
                        true,
                    )?;
                }
            }

            Ok(())
        }
        Commands::Invoke(invoke) => {
            let provider = get_provider(&rpc_url, &network).await?;
            let mut account = get_account(&account, &accounts_file_path, &provider, &network)?;
            let result = starknet_commands::invoke::invoke(
                invoke.contract_address,
                &invoke.function,
                invoke.calldata,
                invoke.max_fee,
                &mut account,
            )
            .await;
            starknet_commands::invoke::print_invoke_result(result, cli.int_format, cli.json)?;

            Ok(())
        }
        Commands::Multicall(multicall) => {
            let provider = get_provider(&rpc_url, &network).await?;
            let mut account = get_account(&account, &accounts_file_path, &provider, &network)?;
            starknet_commands::multicall::multicall(
                &multicall.path,
                &mut account,
                cli.int_format,
                cli.json,
            )
            .await?;
            Ok(())
        }
        Commands::Account(account) => match account.command {
            account::Commands::Create {
                output_path,
                name,
                salt,
                constructor_calldata,
                as_profile,
            } => starknet_commands::account::create(
                output_path,
                name,
                if network.is_empty() {
                    None
                } else {
                    Some(network)
                },
                salt,
                constructor_calldata,
                as_profile,
                cli.int_format,
                cli.json,
            ),
            account::Commands::Deploy {
                path,
                name,
                max_fee,
            } => {
                let provider = get_provider(&rpc_url, &network).await?;
                starknet_commands::account::deploy(&provider, network, path, name, max_fee).await
            }
        },
    }
}
