use crate::starknet_commands::balance::Balance;
use crate::starknet_commands::declare::declare;
use crate::starknet_commands::declare_from::{ContractSource, DeclareFrom};
use crate::starknet_commands::deploy::{DeployArguments, DeployCommonArgs};
use crate::starknet_commands::invoke::InvokeCommonArgs;
use crate::starknet_commands::multicall;
use crate::starknet_commands::script::run_script_command;
use crate::starknet_commands::utils::{self, Utils};
use crate::starknet_commands::{
    account, account::Account as AccountCommand, call::Call, declare::Declare, deploy::Deploy,
    invoke::Invoke, multicall::Multicall, script::Script, show_config::ShowConfig,
    tx_status::TxStatus,
};
use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser, Subcommand};
use configuration::load_config;
use conversions::IntoConv;
use data_transformer::transform;
use shared::auto_completions::{Completions, generate_completions};
use sncast::helpers::command::process_command_result;
use sncast::helpers::config::{combine_cast_configs, get_global_config_path};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::DEFAULT_ACCOUNTS_FILE;
use sncast::helpers::output_format::output_format_from_json_flag;
use sncast::helpers::rpc::generate_network_flag;
use sncast::helpers::scarb_utils::{
    BuildConfig, assert_manifest_path_exists, build_and_load_artifacts, get_package_metadata,
};
use sncast::response::declare::{
    AlreadyDeclaredResponse, DeclareResponse, DeclareTransactionResponse, DeployCommandMessage,
};
use sncast::response::deploy::{DeployResponse, DeployResponseWithDeclare};
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::transformed_call::transform_response;
use sncast::response::ui::UI;
use sncast::{
    ValidatedWaitParams, WaitForTx, get_account, get_block_id, get_class_hash_by_address,
    get_contract_class,
};
use starknet_commands::verify::Verify;
use starknet_rust::accounts::Account;
use starknet_rust::core::types::ContractClass;
use starknet_rust::core::types::contract::{AbiEntry, SierraClass};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_rust::providers::Provider;
use starknet_types_core::felt::Felt;
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
- Or discord: https://discord.gg/starknet-community
- Or join our general chat (Telegram): https://t.me/starknet_foundry

Report bugs: https://github.com/foundry-rs/starknet-foundry/issues/new/choose\
"
)]
#[command(about = "sncast - All-in-one tool for interacting with Starknet smart contracts", long_about = None)]
#[command(name = "sncast")]
struct Cli {
    /// Profile name in snfoundry.toml config file
    #[arg(short, long)]
    profile: Option<String>,

    /// Account to be used for contract declaration;
    /// When using keystore (`--keystore`), this should be a path to account file
    /// When using accounts file, this should be an account name
    #[arg(short = 'a', long)]
    account: Option<String>,

    /// Path to the file holding accounts info
    #[arg(short = 'f', long = "accounts-file")]
    accounts_file_path: Option<Utf8PathBuf>,

    /// Path to keystore file; if specified, --account should be a path to starkli JSON account file
    #[arg(short, long)]
    keystore: Option<Utf8PathBuf>,

    /// If passed, output will be displayed in json format
    #[arg(short, long)]
    json: bool,

    /// If passed, command will wait until transaction is accepted or rejected
    #[arg(short = 'w', long)]
    wait: bool,

    /// Adjusts the time after which --wait assumes transaction was not received or rejected
    #[arg(long)]
    wait_timeout: Option<u16>,

    /// Adjusts the time between consecutive attempts to fetch transaction by --wait flag
    #[arg(long)]
    wait_retry_interval: Option<u8>,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn command_name(&self) -> String {
        match self.command {
            Commands::Declare(_) => "declare",
            Commands::DeclareFrom(_) => "declare-from",
            Commands::Deploy(_) => "deploy",
            Commands::Call(_) => "call",
            Commands::Invoke(_) => "invoke",
            Commands::Multicall(_) => "multicall",
            Commands::Account(_) => "account",
            Commands::ShowConfig(_) => "show-config",
            Commands::Script(_) => "script",
            Commands::TxStatus(_) => "tx-status",
            Commands::Verify(_) => "verify",
            Commands::Completions(_) => "completions",
            Commands::Utils(_) => "utils",
            Commands::Balance(_) => "balance",
        }
        .to_string()
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Declare a contract
    Declare(Declare),

    /// Declare a contract by fetching it from a different Starknet instance
    DeclareFrom(DeclareFrom),

    /// Deploy a contract
    Deploy(Deploy),

    /// Call a contract
    Call(Call),

    /// Invoke a contract
    Invoke(Invoke),

    /// Execute multiple calls
    Multicall(Multicall),

    /// Create and deploy an account
    Account(AccountCommand),

    /// Show current configuration being used
    ShowConfig(ShowConfig),

    /// Run or initialize a deployment script
    Script(Script),

    /// Get the status of a transaction
    TxStatus(TxStatus),

    /// Verify a contract
    Verify(Verify),

    /// Generate completions script
    Completions(Completions),

    /// Utility commands
    Utils(Utils),

    /// Fetch balance of the account for specified token
    Balance(Balance),
}

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct Arguments {
    /// Arguments of the called function serialized as a series of felts
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    pub calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true)]
    pub arguments: Option<String>,
}

impl Arguments {
    fn try_into_calldata(
        self,
        contract_class: &ContractClass,
        selector: &Felt,
    ) -> Result<Vec<Felt>> {
        if let Some(calldata) = self.calldata {
            calldata_to_felts(&calldata)
        } else {
            let ContractClass::Sierra(sierra_class) = contract_class else {
                bail!("Transformation of arguments is not available for Cairo Zero contracts")
            };

            let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str())
                .context("Couldn't deserialize ABI received from network")?;

            transform(&self.arguments.unwrap_or_default(), &abi, selector)
        }
    }
}

pub fn calldata_to_felts(calldata: &[String]) -> Result<Vec<Felt>> {
    calldata
        .iter()
        .map(|data| {
            Felt::from_dec_str(data)
                .or_else(|_| Felt::from_hex(data))
                .context("Failed to parse to felt")
        })
        .collect()
}

impl From<DeployArguments> for Arguments {
    fn from(value: DeployArguments) -> Self {
        let DeployArguments {
            constructor_calldata,
            arguments,
        } = value;
        Self {
            calldata: constructor_calldata,
            arguments,
        }
    }
}

fn init_logging() {
    use std::io;
    use std::io::IsTerminal;
    use tracing_log::LogTracer;
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::fmt::time::Uptime;
    use tracing_subscriber::prelude::*;

    let fmt_layer = Layer::new()
        .with_writer(io::stderr)
        .with_ansi(io::stderr().is_terminal())
        .with_timer(Uptime::default())
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .with_env_var("SNCAST_LOG")
                .from_env_lossy(),
        );

    LogTracer::init().expect("could not initialize log tracer");

    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(fmt_layer))
        .expect("could not set up global logger");
}

fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();

    let output_format = output_format_from_json_flag(cli.json);

    let ui = UI::new(output_format);

    let runtime = Runtime::new().expect("Failed to instantiate Runtime");

    if let Commands::Script(script) = &cli.command {
        run_script_command(&cli, runtime, script, &ui)
    } else {
        let config = get_cast_config(&cli, &ui)?;
        runtime.block_on(run_async_command(cli, config, &ui))
    }
}

#[expect(clippy::too_many_lines)]
async fn run_async_command(cli: Cli, config: CastConfig, ui: &UI) -> Result<()> {
    let wait_config = WaitForTx {
        wait: cli.wait,
        wait_params: config.wait_params,
        show_ui_outputs: true,
    };

    match cli.command {
        Commands::Declare(declare) => {
            let provider = declare.common.rpc.get_provider(&config, ui).await?;

            let rpc = declare.common.rpc.clone();

            let account = get_account(
                &config,
                &provider,
                &declare.common.rpc,
                config.keystore.as_ref(),
                ui,
            )
            .await?;
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &declare.package)?;
            let artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path,
                    json: cli.json,
                    profile: cli.profile.unwrap_or("release".to_string()),
                },
                false,
                // TODO(#3959) Remove `base_ui`
                ui.base_ui(),
            )
            .expect("Failed to build contract");

            let result = starknet_commands::declare::declare(
                declare.contract_name.clone(),
                declare.common.fee_args,
                declare.common.nonce,
                &account,
                &artifacts,
                wait_config,
                false,
                ui,
            )
            .await
            .map_err(handle_starknet_command_error)
            .map(|result| match result {
                DeclareResponse::Success(declare_transaction_response) => {
                    declare_transaction_response
                }
                DeclareResponse::AlreadyDeclared(_) => {
                    unreachable!("Argument `skip_on_already_declared` is false")
                }
            });

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;

            let deploy_command_message = if let Ok(response) = &result {
                // TODO(#3785)
                let contract_artifacts = artifacts
                    .get(&declare.contract_name)
                    .expect("Failed to get contract artifacts");
                let contract_definition: SierraClass =
                    serde_json::from_str(&contract_artifacts.sierra)
                        .context("Failed to parse sierra artifact")?;
                let network_flag = generate_network_flag(&rpc, &config);
                Some(DeployCommandMessage::new(
                    &contract_definition.abi,
                    response,
                    &config.account,
                    &config.accounts_file,
                    network_flag,
                ))
            } else {
                None
            };

            process_command_result("declare", result, ui, block_explorer_link);

            if let Some(deploy_command_message) = deploy_command_message {
                ui.print_notification(deploy_command_message?);
            }

            Ok(())
        }

        Commands::DeclareFrom(declare_from) => {
            let provider = declare_from.common.rpc.get_provider(&config, ui).await?;

            let contract_source = if let Some(sierra_file) = declare_from.sierra_file {
                ContractSource::LocalFile {
                    sierra_path: sierra_file,
                }
            } else {
                let source_provider = declare_from.source_rpc.get_provider(ui).await?;
                let block_id = get_block_id(&declare_from.block_id)?;
                let class_hash = declare_from.class_hash.expect("missing class_hash");

                ContractSource::Network {
                    source_provider,
                    class_hash,
                    block_id,
                }
            };

            let account = get_account(
                &config,
                &provider,
                &declare_from.common.rpc,
                config.keystore.as_ref(),
                ui,
            )
            .await?;

            let result = starknet_commands::declare_from::declare_from(
                contract_source,
                &declare_from.common,
                &account,
                wait_config,
                false,
                ui,
            )
            .await
            .map_err(handle_starknet_command_error)
            .map(|result| match result {
                DeclareResponse::Success(declare_transaction_response) => {
                    declare_transaction_response
                }
                DeclareResponse::AlreadyDeclared(_) => {
                    unreachable!("Argument `skip_on_already_declared` is false")
                }
            });

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            process_command_result("declare-from", result, ui, block_explorer_link);

            Ok(())
        }

        Commands::Deploy(deploy) => {
            let Deploy {
                common:
                    DeployCommonArgs {
                        contract_identifier: identifier,
                        arguments,
                        package,
                        salt,
                        unique,
                    },
                fee_args,
                rpc,
                mut nonce,
                ..
            } = deploy;

            let provider = rpc.get_provider(&config, ui).await?;

            let account =
                get_account(&config, &provider, &rpc, config.keystore.as_ref(), ui).await?;

            let (class_hash, declare_response) = if let Some(class_hash) = identifier.class_hash {
                (class_hash, None)
            } else if let Some(contract_name) = identifier.contract_name {
                let manifest_path = assert_manifest_path_exists()?;
                let package_metadata = get_package_metadata(&manifest_path, &package)?;
                let artifacts = build_and_load_artifacts(
                    &package_metadata,
                    &BuildConfig {
                        scarb_toml_path: manifest_path,
                        json: cli.json,
                        profile: cli.profile.unwrap_or("release".to_string()),
                    },
                    false,
                    // TODO(#3959) Remove `base_ui`
                    ui.base_ui(),
                )
                .expect("Failed to build contract");

                let declare_result = declare(
                    contract_name,
                    fee_args.clone(),
                    nonce,
                    &account,
                    &artifacts,
                    WaitForTx {
                        wait: true,
                        wait_params: wait_config.wait_params,
                        // Only show outputs if user explicitly provides `--wait` flag
                        show_ui_outputs: wait_config.wait,
                    },
                    true,
                    ui,
                )
                .await
                .map_err(handle_starknet_command_error);

                // Increment nonce after successful declare if it was explicitly provided
                nonce = nonce.map(|n| n + Felt::ONE);

                match declare_result {
                    Ok(DeclareResponse::AlreadyDeclared(AlreadyDeclaredResponse {
                        class_hash,
                    })) => (class_hash.into_(), None),
                    Ok(DeclareResponse::Success(declare_transaction_response)) => (
                        declare_transaction_response.class_hash.into_(),
                        Some(declare_transaction_response),
                    ),
                    Err(err) => {
                        // TODO(#3960) This will return json output saying that `deploy` command was run
                        //  even though the invoked command was declare.
                        process_command_result::<DeclareTransactionResponse>(
                            "deploy",
                            Err(err),
                            ui,
                            None,
                        );
                        return Ok(());
                    }
                }
            } else {
                unreachable!("Either `--class_hash` or `--contract_name` must be provided");
            };

            // safe to unwrap because "constructor" is a standardized name
            let selector = get_selector_from_name("constructor").unwrap();

            let contract_class = get_contract_class(class_hash, &provider).await?;

            let arguments: Arguments = arguments.into();
            let calldata = arguments.try_into_calldata(&contract_class, &selector)?;

            let result = starknet_commands::deploy::deploy(
                class_hash,
                &calldata,
                salt,
                unique,
                fee_args,
                nonce,
                &account,
                wait_config,
                ui,
            )
            .await
            .map_err(handle_starknet_command_error);

            let result = if let Some(declare_response) = declare_response {
                result.map(|r| {
                    DeployResponse::WithDeclare(DeployResponseWithDeclare::from_responses(
                        &r,
                        &declare_response,
                    ))
                })
            } else {
                result.map(DeployResponse::Standard)
            };

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            process_command_result("deploy", result, ui, block_explorer_link);

            Ok(())
        }

        Commands::Call(Call {
            contract_address,
            function,
            arguments,
            block_id,
            rpc,
        }) => {
            let provider = rpc.get_provider(&config, ui).await?;

            let block_id = get_block_id(&block_id)?;
            let class_hash = get_class_hash_by_address(&provider, contract_address).await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;

            let selector = get_selector_from_name(&function)
                .context("Failed to convert entry point selector to FieldElement")?;

            let calldata = arguments.try_into_calldata(&contract_class, &selector)?;

            let result = starknet_commands::call::call(
                contract_address,
                selector,
                calldata,
                &provider,
                block_id.as_ref(),
            )
            .await
            .map_err(handle_starknet_command_error);

            if let Some(transformed_result) =
                transform_response(&result, &contract_class, &selector)
            {
                process_command_result("call", Ok(transformed_result), ui, None);
            } else {
                process_command_result("call", result, ui, None);
            }

            Ok(())
        }

        Commands::Invoke(invoke) => {
            let Invoke {
                common:
                    InvokeCommonArgs {
                        contract_address,
                        function,
                        arguments,
                    },
                fee_args,
                rpc,
                nonce,
                ..
            } = invoke;

            let provider = rpc.get_provider(&config, ui).await?;

            let account =
                get_account(&config, &provider, &rpc, config.keystore.as_ref(), ui).await?;

            let selector = get_selector_from_name(&function)
                .context("Failed to convert entry point selector to FieldElement")?;

            let contract_address = contract_address.parse()?;
            let class_hash = get_class_hash_by_address(&provider, contract_address).await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;

            let calldata = arguments.try_into_calldata(&contract_class, &selector)?;

            let result = starknet_commands::invoke::invoke(
                contract_address,
                calldata,
                nonce,
                fee_args,
                selector,
                &account,
                wait_config,
                ui,
            )
            .await
            .map_err(handle_starknet_command_error);

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;

            process_command_result("invoke", result, ui, block_explorer_link);

            Ok(())
        }

        Commands::Utils(utils) => {
            utils::utils(
                utils,
                config,
                ui,
                cli.json,
                cli.profile.clone().unwrap_or("release".to_string()),
            )
            .await
        }

        Commands::Multicall(multicall) => {
            multicall::multicall(multicall, config, ui, wait_config).await
        }

        Commands::Account(account) => account::account(account, config, ui, wait_config).await,

        Commands::ShowConfig(show) => {
            let provider = show.rpc.get_provider(&config, ui).await.ok();

            let result = starknet_commands::show_config::show_config(
                &show,
                provider.as_ref(),
                config,
                cli.profile,
            )
            .await;

            process_command_result("show-config", result, ui, None);

            Ok(())
        }

        Commands::TxStatus(tx_status) => {
            let provider = tx_status.rpc.get_provider(&config, ui).await?;

            let result =
                starknet_commands::tx_status::tx_status(&provider, tx_status.transaction_hash)
                    .await
                    .context("Failed to get transaction status");

            process_command_result("tx-status", result, ui, None);
            Ok(())
        }

        Commands::Verify(verify) => {
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &verify.package)?;
            let artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path.clone(),
                    json: cli.json,
                    profile: cli.profile.unwrap_or("release".to_string()),
                },
                false,
                // TODO(#3959) Remove `base_ui`
                ui.base_ui(),
            )
            .expect("Failed to build contract");
            let result = starknet_commands::verify::verify(
                verify,
                &package_metadata.manifest_path,
                &artifacts,
                &config,
                ui,
            )
            .await;

            process_command_result("verify", result, ui, None);
            Ok(())
        }

        Commands::Completions(completions) => {
            generate_completions(completions.shell, &mut Cli::command())?;
            Ok(())
        }

        Commands::Balance(balance) => {
            let provider = balance.rpc.get_provider(&config, ui).await?;
            let account = get_account(
                &config,
                &provider,
                &balance.rpc,
                config.keystore.as_ref(),
                ui,
            )
            .await?;

            let result =
                starknet_commands::balance::balance(account.address(), &provider, &balance).await?;

            process_command_result("balance", Ok(result), ui, None);

            Ok(())
        }

        Commands::Script(_) => unreachable!(),
    }
}

fn config_with_cli(config: &mut CastConfig, cli: &Cli) {
    macro_rules! clone_or_else {
        ($field:expr, $config_field:expr) => {
            $field.clone().unwrap_or_else(|| $config_field.clone())
        };
    }

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

fn get_cast_config(cli: &Cli, ui: &UI) -> Result<CastConfig> {
    let command = cli.command_name();
    let global_config_path = get_global_config_path().unwrap_or_else(|err| {
        ui.print_error(&command, format!("Error getting global config path: {err}"));
        Utf8PathBuf::new()
    });

    let global_config =
        load_config::<CastConfig>(Some(&global_config_path.clone()), cli.profile.as_deref())
            .or_else(|_| load_config::<CastConfig>(Some(&global_config_path), None))
            .map_err(|err| anyhow::anyhow!(format!("Failed to load config: {err}")))?;

    let local_config = load_config::<CastConfig>(None, cli.profile.as_deref())
        .map_err(|err| anyhow::anyhow!(format!("Failed to load config: {err}")))?;

    let mut combined_config = combine_cast_configs(&global_config, &local_config);

    config_with_cli(&mut combined_config, cli);
    Ok(combined_config)
}
