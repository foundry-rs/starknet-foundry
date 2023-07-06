use crate::starknet_commands::{call::Call, declare::Declare, deploy::Deploy, invoke::Invoke};
use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use cast::{get_account, get_block_id, get_network, get_provider, print_formatted};
use clap::{Parser, Subcommand};
use console::style;

mod starknet_commands;

#[derive(Parser)]
#[command(version)]
#[command(about = "cast - a starknet-foundry CLI", long_about = None)]
struct Cli {
    /// RPC provider url address
    #[clap(short = 'u', long = "url")]
    rpc_url: String,

    /// Network name, one of: testnet, testnet2, mainnet
    #[clap(short = 'n', long = "network")]
    network: Option<String>,

    /// Account name to be used for contract declaration, defaults to __default__
    #[clap(short = 'a', long = "account", default_value = "__default__")]
    account: String,

    /// Path to the file holding accounts info, defaults to ~/.starknet_accounts/starknet_open_zeppelin_accounts.json
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

    let network_name = cli.network.unwrap_or_else(|| {
        eprintln!("{}", style("No --network flag passed!").red());
        std::process::exit(1);
    });
    let network = get_network(&network_name)?;
    let provider = get_provider(&cli.rpc_url)?;

    match cli.command {
        Commands::Declare(declare) => {
            let mut account = get_account(&cli.account, &accounts_file_path, &provider, &network)?;

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
            let account = get_account(&cli.account, &accounts_file_path, &provider, &network)?;

            let result = starknet_commands::deploy::deploy(
                &deploy.class_hash,
                deploy
                    .constructor_calldata
                    .iter()
                    .map(AsRef::as_ref)
                    .collect(),
                deploy.salt.as_deref(),
                deploy.unique,
                deploy.max_fee,
                &account,
            )
            .await;

            match result {
                Ok((transaction_hash, contract_address)) => print_formatted(
                    vec![
                        ("command", "Deploy".to_string()),
                        ("contract_address", format!("{contract_address}")),
                        ("transaction_hash", format!("{transaction_hash}")),
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
        Commands::Call(call) => {
            let block_id = get_block_id(&call.block_id)?;

            let result = starknet_commands::call::call(
                call.contract_address.as_ref(),
                call.function_name.as_ref(),
                call.calldata.as_ref(),
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
            let mut account = get_account(&cli.account, &accounts_file_path, &provider, &network)?;
            let result = starknet_commands::invoke::invoke(
                &invoke.contract_address,
                &invoke.entry_point_name,
                invoke.calldata.iter().map(AsRef::as_ref).collect(),
                invoke.max_fee,
                &mut account,
            )
            .await;

            match result {
                Ok(transaction_hash) => print_formatted(
                    vec![
                        ("command", "Invoke".to_string()),
                        ("transaction_hash", format!("{transaction_hash}")),
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
    }
}
