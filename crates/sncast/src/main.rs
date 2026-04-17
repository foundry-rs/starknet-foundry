use std::num::{NonZeroU16, NonZeroU8};
use std::str::FromStr;

use crate::starknet_commands::declare::declare;
use crate::starknet_commands::declare_from::{ContractSource, DeclareFrom};
use crate::starknet_commands::deploy::{DeployArguments, DeployCommonArgs};
use crate::starknet_commands::get::Get;
use crate::starknet_commands::get::balance::Balance;
use crate::starknet_commands::invoke::InvokeCommonArgs;
use crate::starknet_commands::script::run_script_command;
use crate::starknet_commands::utils::{self, Utils};
use crate::starknet_commands::{
    account, account::Account as AccountCommand, call::Call, declare::Declare, deploy::Deploy,
    get::tx_status::TxStatus, invoke::Invoke, multicall::Multicall, script::Script,
    show_config::ShowConfig,
};
use crate::starknet_commands::{get, multicall};
use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser, Subcommand};
use configuration::{Override, find_config_file};
use conversions::IntoConv;
use data_transformer::transform;
use foundry_ui::components::warning::WarningMessage;
use mimalloc::MiMalloc;
use shared::auto_completions::{Completions, generate_completions};
use sncast::helpers::command::process_command_result;
use sncast::helpers::config::get_or_create_global_config_path;
use sncast::helpers::configuration::{
    CastConfig, CliConfigOpts, ConfigScope, MaybeConfig, PartialCastConfig,
};
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
    PartialWaitParams, WaitForTx, get_account, get_block_id, get_class_hash_by_address,
    get_contract_class, with_account,
};
use starknet_commands::ledger::{self, Ledger};
use starknet_commands::verify::Verify;
use starknet_rust::core::types::ContractClass;
use starknet_rust::core::types::contract::{AbiEntry, SierraClass};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_rust::providers::Provider;
use starknet_types_core::felt::Felt;
use std::process::ExitCode;
use tokio::runtime::Runtime;

mod starknet_commands;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    wait_timeout: Option<NonZeroU16>,

    /// Adjusts the time between consecutive attempts to fetch transaction by --wait flag
    #[arg(long)]
    wait_retry_interval: Option<NonZeroU8>,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn command_name(&self) -> String {
        match self.command {
            Commands::Get(_) => "get",
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
            Commands::Ledger(_) => "ledger",
        }
        .to_string()
    }

    /// Prepares and validates [`PartialCastConfig`] from CLI args.
    pub fn to_partial_config(&self) -> Result<PartialCastConfig> {
        let config = PartialCastConfig {
            account: self.account.clone(),
            keystore: self.keystore.clone(),
            accounts_file: self.accounts_file_path.clone(),
            wait_params: Some(PartialWaitParams {
                timeout: self.wait_timeout,
                retry_interval: self.wait_retry_interval,
            }),
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Get various data from the network
    Get(Get),

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

    /// Interact with Ledger hardware wallet
    Ledger(Ledger),
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
        .map(|data| Felt::from_str(data).with_context(|| format!("Failed to parse {data} to felt")))
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
                .with_default_directive("coins_ledger=off".parse().expect("valid directive"))
                .with_env_var("SNCAST_LOG")
                .from_env_lossy(),
        );

    LogTracer::init().expect("could not initialize log tracer");

    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(fmt_layer))
        .expect("could not set up global logger");
}

fn main() -> Result<ExitCode> {
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
async fn run_async_command(cli: Cli, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let wait_config = WaitForTx {
        wait: cli.wait,
        wait_params: config.wait_params,
        show_ui_outputs: true,
    };

    match cli.command {
        Commands::Declare(declare) => {
            let provider = declare.common.rpc.get_provider(&config, ui).await?;

            let rpc = declare.common.rpc.clone();

            let account = get_account(&config, &provider, &declare.common.rpc, ui).await?;
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

            let result = with_account!(&account, |account| {
                starknet_commands::declare::declare(
                    declare.contract_name.clone(),
                    declare.common.fee_args,
                    declare.common.dry_run_args,
                    declare.common.nonce,
                    account,
                    &artifacts,
                    wait_config,
                    false,
                    ui,
                )
                .await
            });

            let result = match result {
                Ok(DeclareResponse::DryRun(response)) => {
                    return Ok(process_command_result("declare", Ok(response), ui, None));
                }
                Ok(DeclareResponse::Success(declare_transaction_response)) => {
                    Ok(declare_transaction_response)
                }
                Ok(DeclareResponse::AlreadyDeclared(_)) => {
                    unreachable!("Argument `skip_on_already_declared` is false")
                }
                Err(err) => Err(handle_starknet_command_error(err)),
            };

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

            let exit_code = process_command_result("declare", result, ui, block_explorer_link);

            if let Some(deploy_command_message) = deploy_command_message {
                ui.print_notification(deploy_command_message?);
            }

            Ok(exit_code)
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

            let account = get_account(&config, &provider, &declare_from.common.rpc, ui).await?;

            let result = with_account!(&account, |account| {
                starknet_commands::declare_from::declare_from(
                    contract_source,
                    &declare_from.common,
                    account,
                    wait_config,
                    false,
                    ui,
                )
                .await
            });

            let result = match result {
                Ok(DeclareResponse::DryRun(response)) => {
                    return Ok(process_command_result("declare", Ok(response), ui, None));
                }
                Ok(DeclareResponse::Success(declare_transaction_response)) => {
                    Ok(declare_transaction_response)
                }
                Ok(DeclareResponse::AlreadyDeclared(_)) => {
                    unreachable!("Argument `skip_on_already_declared` is false")
                }
                Err(err) => Err(handle_starknet_command_error(err)),
            };

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            Ok(process_command_result(
                "declare-from",
                result,
                ui,
                block_explorer_link,
            ))
        }

        Commands::Deploy(deploy) => {
            let Deploy {
                common:
                    DeployCommonArgs {
                        contract_identifier: identifier,
                        arguments,
                        package,
                        ..
                    },
                fee_args,
                dry_run_args,
                rpc,
                mut nonce,
                ..
            } = deploy;

            let provider = rpc.get_provider(&config, ui).await?;

            let account = get_account(&config, &provider, &rpc, ui).await?;

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

                let declare_result = with_account!(&account, |account| {
                    declare(
                        contract_name,
                        fee_args,
                        dry_run_args,
                        nonce,
                        account,
                        &artifacts,
                        WaitForTx {
                            wait: true,
                            wait_params: wait_config.wait_params,
                            show_ui_outputs: wait_config.wait,
                        },
                        true,
                        ui,
                    )
                    .await
                })
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
                    Ok(DeclareResponse::DryRun(_)) => {
                        unreachable!(
                            "Declaration run by deploy command should not return dry run response"
                        )
                    }
                    Err(err) => {
                        // TODO(#3960) This will return json output saying that `deploy` command was run
                        //  even though the invoked command was declare.
                        return Ok(process_command_result::<DeclareTransactionResponse>(
                            "deploy",
                            Err(err),
                            ui,
                            None,
                        ));
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

            let result = with_account!(&account, |account| {
                starknet_commands::deploy::deploy(
                    class_hash,
                    &calldata,
                    deploy.common.salt,
                    deploy.common.unique,
                    fee_args,
                    dry_run_args,
                    nonce,
                    account,
                    wait_config,
                    ui,
                )
                .await
            })
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
            Ok(process_command_result(
                "deploy",
                result,
                ui,
                block_explorer_link,
            ))
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
                Ok(process_command_result(
                    "call",
                    Ok(transformed_result),
                    ui,
                    None,
                ))
            } else {
                Ok(process_command_result("call", result, ui, None))
            }
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
                dry_run_args,
                proof_args,
                rpc,
                nonce,
                ..
            } = invoke;

            let provider = rpc.get_provider(&config, ui).await?;

            let account = get_account(&config, &provider, &rpc, ui).await?;

            let selector = get_selector_from_name(&function)
                .context("Failed to convert entry point selector to FieldElement")?;

            let contract_address = contract_address.try_into_felt()?;
            let class_hash = get_class_hash_by_address(&provider, contract_address).await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;

            let calldata = arguments.try_into_calldata(&contract_class, &selector)?;

            let result = with_account!(&account, |account| {
                starknet_commands::invoke::invoke(
                    contract_address,
                    calldata,
                    nonce,
                    fee_args,
                    dry_run_args,
                    proof_args,
                    selector,
                    account,
                    wait_config,
                    ui,
                )
                .await
            })
            .map_err(handle_starknet_command_error);

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;

            Ok(process_command_result(
                "invoke",
                result,
                ui,
                block_explorer_link,
            ))
        }

        Commands::Get(get) => get::get(get, config, ui).await,

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
            let provider = match show.rpc.get_provider(&config, ui).await {
                Ok(p) => Some(p),
                Err(err) => {
                    ui.print_warning(format!("Could not reach RPC provider: {err:#}"));
                    None
                }
            };

            let result = starknet_commands::show_config::show_config(
                &show,
                provider.as_ref(),
                config,
                cli.profile,
            )
            .await;

            Ok(process_command_result("show-config", result, ui, None))
        }

        // TODO(#4214): Remove moved sncast commands
        Commands::TxStatus(tx_status) => {
            print_cmd_move_warning("tx-status", "get tx-status", ui);
            get::tx_status::tx_status(tx_status, config, ui).await
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

            Ok(process_command_result("verify", result, ui, None))
        }

        Commands::Completions(completions) => {
            generate_completions(completions.shell, &mut Cli::command())?;
            Ok(ExitCode::SUCCESS)
        }

        // TODO(#4214): Remove moved sncast commands
        Commands::Balance(balance) => {
            print_cmd_move_warning("balance", "get balance", ui);
            get::balance::balance(balance, config, ui).await
        }

        Commands::Ledger(ledger) => {
            let result = ledger::ledger(&ledger, ui).await;
            Ok(process_command_result("ledger", result, ui, None))
        }

        Commands::Script(_) => unreachable!(),
    }
}

fn get_cast_config(cli: &Cli, ui: &UI) -> Result<CastConfig> {
    let opts = CliConfigOpts {
        command_name: cli.command_name(),
        profile: cli.profile.clone(),
    };

    let local_path = find_config_file().ok();
    let global_path = match get_or_create_global_config_path() {
        Ok(p) => Some(p),
        Err(err) => {
            ui.print_warning(WarningMessage::new(format!(
                "Could not get or create global config file: {err:?}. Proceeding without global config."
            )));
            None
        }
    };
    let profile = opts.profile.as_deref();

    let global_default =
        PartialCastConfig::load_maybe(global_path.as_ref(), None, ConfigScope::Global)?;
    let global_profile =
        PartialCastConfig::load_maybe(global_path.as_ref(), profile, ConfigScope::Global)?;
    let local_default =
        PartialCastConfig::load_maybe(local_path.as_ref(), None, ConfigScope::Local)?;
    let local_profile =
        PartialCastConfig::load_maybe(local_path.as_ref(), profile, ConfigScope::Local)?;

    match (profile, &local_profile, &global_profile) {
        // If local config file exists, profile must be in it.
        (Some(profile), MaybeConfig::NoProfile, _) => {
            bail!(
                "Profile [{profile}] not found in local config at {}",
                local_path.unwrap_or_default()
            );
        }
        // No local config file; profile must be in global config.
        (Some(profile), MaybeConfig::NoFile, MaybeConfig::NoProfile) => {
            // TODO: (#pending) Streamline approach wrt. `--profile` being re-used for foundry and `scarb`.
            ui.print_warning(WarningMessage::new(format!(
                "Profile [{profile}] not found in global config at {}, and no local config found.",
                global_path.clone().unwrap_or_default()
            )));
        }
        // Note: this is potentially unreachable: `get_or_create_global_config_path` should always return dir with existing config file.
        // TODO: (#3436) remove this if missing global config becomes an error
        (Some(profile), MaybeConfig::NoFile, MaybeConfig::NoFile) => {
            bail!("Profile [{profile}] not found: no config files present");
        }
        _ => {}
    }

    let cli_config = cli.to_partial_config()?;
    let partial_config = PartialCastConfig::default()
        .override_with(global_default.unwrap_or_default())
        .override_with(global_profile.unwrap_or_default())
        .override_with(local_default.unwrap_or_default())
        .override_with(local_profile.unwrap_or_default())
        .override_with(cli_config);

    CastConfig::try_from(partial_config).with_context(|| {
        indoc::formatdoc! {"
            Unable to combine configs. Fix conflicts between config sources and try again.
            Sources:
            - CLI flags
            - Local config: {local}
            - Global config: {global}
        ",
            global = global_path.as_ref().map_or("missing", |p| p.as_str()),
            local = local_path.as_ref().map_or("missing", |p| p.as_str()),
        }
    })
}

fn print_cmd_move_warning(command_name: &str, new_command_name: &str, ui: &UI) {
    ui.print_warning(WarningMessage::new(format!(
        "`sncast {command_name}` has moved to `sncast {new_command_name}`. `sncast {command_name}` will be removed in the next version."
    )));
    ui.print_blank_line();
}
