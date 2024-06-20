use crate::starknet_commands::account::Account;
use crate::starknet_commands::show_config::ShowConfig;
use crate::starknet_commands::{
    account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke, multicall::Multicall,
    script::Script, tx_status::TxStatus,
};
use anyhow::{Context, Result};
use configuration::load_global_config;
use sncast::response::print::{print_command_result, OutputFormat};

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use shared::verify_and_warn_if_incompatible_rpc_version;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{DEFAULT_ACCOUNTS_FILE, DEFAULT_MULTICALL_CONTENTS};
use sncast::helpers::scarb_utils::{
    assert_manifest_path_exists, build, build_and_load_artifacts, get_package_metadata,
    get_scarb_metadata_with_deps, BuildConfig,
};
use sncast::response::errors::handle_starknet_command_error;
use sncast::{
    chain_id_to_network_name, get_account, get_block_id, get_chain_id, get_default_state_file_name,
    get_nonce, get_provider, NumbersFormat, ValidatedWaitParams, WaitForTx,
};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use tokio::runtime::Runtime;

mod starknet_commands;

#[derive(Parser)]
#[command(
    version,
    help_template = "\
{name} {version}
{author-with-newline}{about-with-newline}
Use -h for short descriptions and --help for more details.

{before-help}{usage-heading} {usage}

{all-args}{after-help}
",
    after_help = "Read the docs: https://foundry-rs.github.io/starknet-foundry/",
    after_long_help = "\
Read the docs:
- Starknet Foundry Book: https://foundry-rs.github.io/starknet-foundry/
- Cairo Book: https://book.cairo-lang.org/
- Starknet Book: https://book.starknet.io/
- Starknet Documentation: https://docs.starknet.io/
- Scarb Documentation: https://docs.swmansion.com/scarb/docs.html

Join the community:
- Follow core developers on X: https://twitter.com/swmansionxyz
- Get support via Telegram: https://t.me/starknet_foundry_support
- Or discord: https://discord.gg/KZWaFtPZJf
- Or join our general chat (Telegram): https://t.me/starknet_foundry

Report bugs: https://github.com/foundry-rs/starknet-foundry/issues/new/choose\
"
)]
#[command(about = "sncast - All-in-one tool for interacting with Starknet smart contracts", long_about = None)]
#[clap(name = "sncast")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Profile name in snfoundry.toml config file
    #[clap(short, long)]
    profile: Option<String>,

    /// RPC provider url address; overrides url from snfoundry.toml
    #[clap(short = 'u', long = "url")]
    rpc_url: Option<String>,

    /// Account to be used for contract declaration;
    /// When using keystore (`--keystore`), this should be a path to account file    
    /// When using accounts file, this should be an account name
    #[clap(short = 'a', long)]
    account: Option<String>,

    /// Path to the file holding accounts info
    #[clap(short = 'f', long = "accounts-file")]
    accounts_file_path: Option<Utf8PathBuf>,

    /// Path to keystore file; if specified, --account should be a path to starkli JSON account file
    #[clap(short, long)]
    keystore: Option<Utf8PathBuf>,

    /// If passed, values will be displayed as integers
    #[clap(long, conflicts_with = "hex_format")]
    int_format: bool,

    /// If passed, values will be displayed as hex
    #[clap(long, conflicts_with = "int_format")]
    hex_format: bool,

    /// If passed, output will be displayed in json format
    #[clap(short, long)]
    json: bool,

    /// If passed, command will wait until transaction is accepted or rejected
    #[clap(short = 'w', long)]
    wait: bool,

    /// Adjusts the time after which --wait assumes transaction was not received or rejected
    #[clap(long)]
    wait_timeout: Option<u16>,

    /// Adjusts the time between consecutive attempts to fetch transaction by --wait flag
    #[clap(long)]
    wait_retry_interval: Option<u8>,

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

    /// Show current configuration being used
    ShowConfig(ShowConfig),

    /// Run or initialize a deployment script
    Script(Script),

    /// Get the status of a transaction
    TxStatus(TxStatus),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let numbers_format = NumbersFormat::from_flags(cli.hex_format, cli.int_format);
    let output_format = OutputFormat::from_flag(cli.json);

    let runtime = Runtime::new().expect("Failed to instantiate Runtime");

    if let Commands::Script(script) = &cli.command {
        run_script_command(&cli, runtime, script, numbers_format, &output_format)
    } else {
        let mut config = load_global_config::<CastConfig>(&None, &cli.profile)?;
        update_cast_config(&mut config, &cli);
        let provider = get_provider(&config.url)?;
        runtime.block_on(run_async_command(
            cli,
            config,
            provider,
            numbers_format,
            output_format,
        ))
    }
}

#[allow(clippy::too_many_lines)]
async fn run_async_command(
    cli: Cli,
    config: CastConfig,
    provider: JsonRpcClient<HttpTransport>,
    numbers_format: NumbersFormat,
    output_format: OutputFormat,
) -> Result<()> {
    verify_and_warn_if_incompatible_rpc_version(&provider, &config.url).await?;

    let wait_config = WaitForTx {
        wait: cli.wait,
        wait_params: config.wait_params,
    };

    match cli.command {
        Commands::Declare(declare) => {
            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore,
            )
            .await?;
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &declare.package)?;
            let artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path,
                    json: cli.json,
                    profile: cli.profile.unwrap_or("dev".to_string()),
                },
            )
            .expect("Failed to build contract");
            let mut result = starknet_commands::declare::declare(
                &declare.contract,
                declare.fee.max_fee,
                &account,
                declare.nonce,
                &artifacts,
                wait_config,
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("declare", &mut result, numbers_format, &output_format)?;
            Ok(())
        }
        Commands::Deploy(deploy) => {
            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore,
            )
            .await?;
            let mut result = starknet_commands::deploy::deploy(
                deploy.class_hash,
                deploy.constructor_calldata,
                deploy.salt,
                deploy.unique,
                deploy.fee.max_fee,
                &account,
                deploy.nonce,
                wait_config,
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("deploy", &mut result, numbers_format, &output_format)?;
            Ok(())
        }
        Commands::Call(call) => {
            let block_id = get_block_id(&call.block_id)?;

            let mut result = starknet_commands::call::call(
                call.contract_address,
                get_selector_from_name(&call.function)
                    .context("Failed to convert entry point selector to FieldElement")?,
                call.calldata,
                &provider,
                block_id.as_ref(),
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("call", &mut result, numbers_format, &output_format)?;
            Ok(())
        }
        Commands::Invoke(invoke) => {
            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore,
            )
            .await?;
            let mut result = starknet_commands::invoke::invoke(
                invoke.contract_address,
                get_selector_from_name(&invoke.function)
                    .context("Failed to convert entry point selector to FieldElement")?,
                invoke.calldata,
                invoke.fee.max_fee,
                &account,
                invoke.nonce,
                wait_config,
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("invoke", &mut result, numbers_format, &output_format)?;
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
                            numbers_format,
                            &output_format,
                        )?;
                    } else {
                        println!("{DEFAULT_MULTICALL_CONTENTS}");
                    }
                }
                starknet_commands::multicall::Commands::Run(run) => {
                    let account = get_account(
                        &config.account,
                        &config.accounts_file,
                        &provider,
                        config.keystore,
                    )
                    .await?;
                    let mut result = starknet_commands::multicall::run::run(
                        &run.path,
                        &account,
                        run.max_fee,
                        wait_config,
                    )
                    .await;

                    print_command_result(
                        "multicall run",
                        &mut result,
                        numbers_format,
                        &output_format,
                    )?;
                }
            }
            Ok(())
        }
        Commands::Account(account) => match account.command {
            account::Commands::Add(add) => {
                let mut result = starknet_commands::account::add::add(
                    &config.url,
                    &add.name.clone(),
                    &config.accounts_file,
                    &provider,
                    &add,
                )
                .await;

                print_command_result("account add", &mut result, numbers_format, &output_format)?;
                Ok(())
            }
            account::Commands::Create(create) => {
                let chain_id = get_chain_id(&provider).await?;
                let account = if config.keystore.is_none() {
                    create
                        .name
                        .context("Required argument `--name` not provided")?
                } else {
                    config.account
                };
                let mut result = starknet_commands::account::create::create(
                    &config.url,
                    &account,
                    &config.accounts_file,
                    config.keystore,
                    &provider,
                    chain_id,
                    create.account_type,
                    create.salt,
                    create.add_profile,
                    create.class_hash,
                )
                .await;

                print_command_result(
                    "account create",
                    &mut result,
                    numbers_format,
                    &output_format,
                )?;
                Ok(())
            }
            account::Commands::Deploy(deploy) => {
                deploy.validate()?;
                let chain_id = get_chain_id(&provider).await?;
                let keystore_path = config.keystore.clone();
                let mut result = starknet_commands::account::deploy::deploy(
                    &provider,
                    config.accounts_file,
                    deploy,
                    chain_id,
                    wait_config,
                    &config.account,
                    keystore_path,
                )
                .await;

                print_command_result(
                    "account deploy",
                    &mut result,
                    numbers_format,
                    &output_format,
                )?;
                Ok(())
            }
            account::Commands::Delete(delete) => {
                let network_name = match delete.network {
                    Some(network) => network,
                    None => chain_id_to_network_name(get_chain_id(&provider).await?),
                };

                let mut result = starknet_commands::account::delete::delete(
                    &delete.name,
                    &config.accounts_file,
                    &network_name,
                    delete.yes,
                );

                print_command_result(
                    "account delete",
                    &mut result,
                    numbers_format,
                    &output_format,
                )?;
                Ok(())
            }
        },
        Commands::ShowConfig(_) => {
            let mut result =
                starknet_commands::show_config::show_config(&provider, config, cli.profile).await;
            print_command_result("show-config", &mut result, numbers_format, &output_format)?;
            Ok(())
        }
        Commands::TxStatus(tx_status) => {
            let mut result =
                starknet_commands::tx_status::tx_status(&provider, tx_status.transaction_hash)
                    .await
                    .context("Failed to get transaction status");
            print_command_result("tx-status", &mut result, numbers_format, &output_format)?;
            Ok(())
        }
        Commands::Script(_) => unreachable!(),
    }
}

fn run_script_command(
    cli: &Cli,
    runtime: Runtime,
    script: &Script,
    numbers_format: NumbersFormat,
    output_format: &OutputFormat,
) -> Result<()> {
    match &script.command {
        starknet_commands::script::Commands::Init(init) => {
            let mut result = starknet_commands::script::init::init(init);
            print_command_result("script init", &mut result, numbers_format, output_format)?;
        }
        starknet_commands::script::Commands::Run(run) => {
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &run.package)?;

            let mut config = load_global_config::<CastConfig>(
                &Some(package_metadata.root.clone()),
                &cli.profile,
            )?;
            update_cast_config(&mut config, cli);
            let provider = get_provider(&config.url)?;
            runtime.block_on(verify_and_warn_if_incompatible_rpc_version(
                &provider,
                &config.url,
            ))?;

            let mut artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path.clone(),
                    json: cli.json,
                    profile: cli.profile.clone().unwrap_or("dev".to_string()),
                },
            )
            .expect("Failed to build artifacts");
            // TODO(#2042): remove duplicated compilation
            build(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path.clone(),
                    json: cli.json,
                    profile: "dev".to_string(),
                },
            )
            .expect("Failed to build script");
            let metadata_with_deps = get_scarb_metadata_with_deps(&manifest_path)?;

            let chain_id = runtime.block_on(get_chain_id(&provider))?;
            let state_file_path = if run.no_state_file {
                None
            } else {
                Some(package_metadata.root.join(get_default_state_file_name(
                    &run.script_name,
                    &chain_id_to_network_name(chain_id),
                )))
            };

            let mut result = starknet_commands::script::run::run(
                &run.script_name,
                &metadata_with_deps,
                &package_metadata,
                &mut artifacts,
                &provider,
                runtime,
                &config,
                state_file_path,
            );

            print_command_result("script run", &mut result, numbers_format, output_format)?;
        }
    }

    Ok(())
}

fn update_cast_config(config: &mut CastConfig, cli: &Cli) {
    macro_rules! clone_or_else {
        ($field:expr, $config_field:expr) => {
            $field.clone().unwrap_or_else(|| $config_field.clone())
        };
    }

    config.url = clone_or_else!(cli.rpc_url, config.url);
    config.account = clone_or_else!(cli.account, config.account);
    config.keystore = cli.keystore.clone().or(config.keystore.clone());

    if config.accounts_file == Utf8PathBuf::default() {
        config.accounts_file = Utf8PathBuf::from(DEFAULT_ACCOUNTS_FILE);
    }
    let new_accounts_file = clone_or_else!(cli.accounts_file_path, config.accounts_file);

    config.accounts_file = Utf8PathBuf::from(shellexpand::tilde(&new_accounts_file).to_string());

    config.wait_params = ValidatedWaitParams::new(
        clone_or_else!(
            cli.wait_retry_interval,
            config.wait_params.get_retry_interval()
        ),
        clone_or_else!(cli.wait_timeout, config.wait_params.get_timeout()),
    );
}
