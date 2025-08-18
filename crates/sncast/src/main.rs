use crate::starknet_commands::class_hash::ClassHash;
use crate::starknet_commands::deploy::DeployArguments;
use crate::starknet_commands::multicall;
use crate::starknet_commands::script::run_script_command;
use crate::starknet_commands::utils::{self, Utils};
use crate::starknet_commands::{
    account, account::Account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke,
    multicall::Multicall, script::Script, show_config::ShowConfig, tx_status::TxStatus,
};
use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser, Subcommand};
use configuration::load_config;
use data_transformer::transform;
use foundry_ui::components::warning::WarningMessage;
use foundry_ui::{Message, UI};
use shared::auto_completions::{Completions, generate_completions};
use sncast::helpers::config::{combine_cast_configs, get_global_config_path};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::DEFAULT_ACCOUNTS_FILE;
use sncast::helpers::output_format::output_format_from_json_flag;
use sncast::helpers::scarb_utils::{
    BuildConfig, assert_manifest_path_exists, build_and_load_artifacts, get_package_metadata,
    load_artifacts,
};
use sncast::response::cast_message::SncastMessage;
use sncast::response::command::CommandResponse;
use sncast::response::declare::DeclareResponse;
use sncast::response::errors::ResponseError;
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::explorer_link::{ExplorerLinksMessage, block_explorer_link_if_allowed};
use sncast::response::transformed_call::transform_response;
use sncast::{
    ValidatedWaitParams, WaitForTx, get_account, get_block_id, get_class_hash_by_address,
    get_contract_class,
};
use starknet::core::types::ContractClass;
use starknet::core::types::contract::AbiEntry;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::Provider;
use starknet_commands::verify::Verify;
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

#[derive(Subcommand)]
enum Commands {
    /// Declare a contract
    Declare(Declare),

    /// Deploy a contract
    Deploy(Deploy),

    /// Call a contract
    Call(Call),

    /// Get contract class hash
    ClassHash(ClassHash),

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

    /// Verify a contract
    Verify(Verify),

    /// Generate completions script
    // TODO(#3560): Remove the `completion` alias
    #[command(alias = "completion")]
    Completions(Completions),

    /// Utility commands
    Utils(Utils),
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
        contract_class: ContractClass,
        selector: &Felt,
    ) -> Result<Vec<Felt>> {
        if let Some(calldata) = self.calldata {
            calldata
                .iter()
                .map(|data| {
                    Felt::from_dec_str(data)
                        .or_else(|_| Felt::from_hex(data))
                        .context("Failed to parse to felt")
                })
                .collect()
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

fn main() -> Result<()> {
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
    };

    match cli.command {
        Commands::Declare(declare) => {
            let provider = declare.rpc.get_provider(&config, ui).await?;

            let rpc = declare.rpc.clone();

            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore.as_ref(),
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
                ui,
            )
            .expect("Failed to build contract");

            let result = starknet_commands::declare::declare(
                declare,
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
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &rpc, &config);
            process_command_result("declare", result, ui, block_explorer_link);

            Ok(())
        }

        Commands::Deploy(deploy) => {
            let Deploy {
                arguments,
                fee_args,
                rpc,
                ..
            } = deploy;

            let provider = rpc.get_provider(&config, ui).await?;

            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore.as_ref(),
            )
            .await?;

            // safe to unwrap because "constructor" is a standardized name
            let selector = get_selector_from_name("constructor").unwrap();

            let contract_class = get_contract_class(deploy.class_hash, &provider).await?;

            let arguments: Arguments = arguments.into();
            let calldata = arguments.try_into_calldata(contract_class, &selector)?;

            let result = starknet_commands::deploy::deploy(
                deploy.class_hash,
                &calldata,
                deploy.salt,
                deploy.unique,
                fee_args,
                deploy.nonce,
                &account,
                wait_config,
                ui,
            )
            .await
            .map_err(handle_starknet_command_error);

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &rpc, &config);
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

            let calldata = arguments.try_into_calldata(contract_class.clone(), &selector)?;

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

        Commands::ClassHash(class_hash_command) => {
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata =
                get_package_metadata(&manifest_path, &class_hash_command.package)?;
            let artifacts;
            let build_config = BuildConfig {
                scarb_toml_path: manifest_path,
                json: cli.json,
                profile: cli.profile.unwrap_or("release".to_string()),
            };

            if !class_hash_command.skip_compile {
                artifacts = build_and_load_artifacts(&package_metadata, &build_config, false, ui)
                    .expect("Failed to build contract");
            } else {
                artifacts =
                    load_artifacts(&package_metadata, &build_config, ui, &build_config.profile)
                        .expect("Failed to load artifacts");
            }

            let result =
                starknet_commands::class_hash::get_class_hash(class_hash_command, &artifacts)
                    .map_err(handle_starknet_command_error);

            process_command_result("class-hash", result, ui, None);

            Ok(())
        }

        Commands::Invoke(invoke) => {
            let Invoke {
                contract_address,
                function,
                arguments,
                fee_args,
                rpc,
                nonce,
                ..
            } = invoke;

            let provider = rpc.get_provider(&config, ui).await?;

            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore.as_ref(),
            )
            .await?;

            let selector = get_selector_from_name(&function)
                .context("Failed to convert entry point selector to FieldElement")?;

            let class_hash = get_class_hash_by_address(&provider, contract_address).await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;

            let calldata = arguments.try_into_calldata(contract_class, &selector)?;

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
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &rpc, &config);

            process_command_result("invoke", result, ui, block_explorer_link);

            Ok(())
        }

        Commands::Utils(utils) => utils::utils(utils, config, ui).await,

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
                ui,
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

            // TODO(#3560): Remove this warning when the `completion` alias is removed
            if std::env::args().nth(1).as_deref() == Some("completion") {
                let message = &WarningMessage::new(
                    "Command `sncast completion` is deprecated and will be removed in the future. Please use `sncast completions` instead.",
                );

                // `#` is required since `sncast completion` generates a script and the output is used directly
                ui.println(&format!("# {}", message.text()));
            }

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
    let global_config_path = get_global_config_path().unwrap_or_else(|err| {
        ui.eprintln(&format!("Error getting global config path: {err}"));
        Utf8PathBuf::new()
    });

    let global_config =
        load_config::<CastConfig>(Some(&global_config_path.clone()), cli.profile.as_deref())
            .unwrap_or_else(|_| {
                load_config::<CastConfig>(Some(&global_config_path), None).unwrap()
            });

    let local_config = load_config::<CastConfig>(None, cli.profile.as_deref())?;

    let mut combined_config = combine_cast_configs(&global_config, &local_config);

    config_with_cli(&mut combined_config, cli);
    Ok(combined_config)
}

fn process_command_result<T>(
    command: &str,
    result: Result<T>,
    ui: &UI,
    block_explorer_link: Option<ExplorerLinksMessage>,
) where
    T: CommandResponse,
    SncastMessage<T>: Message,
{
    let cast_msg = result.map(|command_response| SncastMessage {
        command: command.to_string(),
        command_response,
    });

    match cast_msg {
        Ok(response) => {
            ui.println(&response);
            if let Some(link) = block_explorer_link {
                ui.println(&link);
            }
        }
        Err(err) => {
            let err = ResponseError::new(command.to_string(), format!("{err:#}"));
            ui.eprintln(&err);
        }
    }
}
