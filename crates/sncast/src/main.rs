use crate::starknet_commands::account::Account;
use crate::starknet_commands::show_config::ShowConfig;
use crate::starknet_commands::{
    account, call::Call, declare::Declare, deploy::Deploy, invoke::Invoke, multicall::Multicall,
    script::Script, tx_status::TxStatus,
};
use anyhow::{Context, Result};
use configuration::load_global_config;
use data_transformer::Calldata;
use sncast::response::explorer_link::print_block_explorer_link_if_allowed;
use sncast::response::print::{print_command_result, OutputFormat};

use crate::starknet_commands::deploy::DeployArguments;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::{DEFAULT_ACCOUNTS_FILE, DEFAULT_MULTICALL_CONTENTS};
use sncast::helpers::fee::PayableTransaction;
use sncast::helpers::scarb_utils::{
    assert_manifest_path_exists, build, build_and_load_artifacts, get_package_metadata,
    get_scarb_metadata_with_deps, BuildConfig,
};
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::structs::DeclareResponse;
use sncast::{
    chain_id_to_network_name, get_account, get_block_id, get_chain_id, get_class_hash_by_address,
    get_contract_class, get_default_state_file_name, NumbersFormat, ValidatedWaitParams, WaitForTx,
};
use starknet::accounts::ConnectedAccount;
use starknet::core::types::ContractClass;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::Provider;
use starknet_commands::account::list::print_account_list;
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

    /// Account to be used for contract declaration;
    /// When using keystore (`--keystore`), this should be a path to account file
    /// When using accounts file, this should be an account name
    #[clap(short = 'a', long)]
    account: Option<String>,

    /// Path to the file holding accounts info
    #[clap(long = "accounts-file")]
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

    /// Verify a contract
    Verify(Verify),
}

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct Arguments {
    /// Arguments of the called function serialized as a series of felts
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[clap(long)]
    pub arguments: Option<String>,
}

impl Arguments {
    fn try_into_calldata(
        self,
        contract_class: ContractClass,
        selector: &Felt,
    ) -> Result<Vec<Felt>> {
        if let Some(arguments) = self.arguments {
            Calldata::new(arguments).serialized(contract_class, selector)
        } else if let Some(calldata) = self.calldata {
            calldata
                .iter()
                .map(|data| {
                    Felt::from_dec_str(data)
                        .or_else(|_| Felt::from_hex(data))
                        .context("Failed to parse to felt")
                })
                .collect()
        } else {
            Ok(vec![])
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

    let numbers_format = NumbersFormat::from_flags(cli.hex_format, cli.int_format);
    let output_format = OutputFormat::from_flag(cli.json);

    let runtime = Runtime::new().expect("Failed to instantiate Runtime");

    if let Commands::Script(script) = &cli.command {
        run_script_command(&cli, runtime, script, numbers_format, output_format)
    } else {
        let mut config = load_global_config::<CastConfig>(&None, &cli.profile)?;
        update_cast_config(&mut config, &cli);

        runtime.block_on(run_async_command(
            cli,
            config,
            numbers_format,
            output_format,
        ))
    }
}

#[allow(clippy::too_many_lines)]
async fn run_async_command(
    cli: Cli,
    config: CastConfig,
    numbers_format: NumbersFormat,
    output_format: OutputFormat,
) -> Result<()> {
    let wait_config = WaitForTx {
        wait: cli.wait,
        wait_params: config.wait_params,
    };

    match cli.command {
        Commands::Declare(declare) => {
            let provider = declare.rpc.get_provider(&config).await?;

            declare.validate()?;

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
                    profile: cli.profile.unwrap_or("release".to_string()),
                },
                false,
            )
            .expect("Failed to build contract");
            let result = starknet_commands::declare::declare(
                declare,
                &account,
                &artifacts,
                wait_config,
                false,
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

            print_command_result("declare", &result, numbers_format, output_format)?;
            print_block_explorer_link_if_allowed(
                &result,
                output_format,
                provider.chain_id().await?,
                config.show_explorer_links,
                config.block_explorer,
            );
            Ok(())
        }

        Commands::Deploy(deploy) => {
            deploy.validate()?;

            let fee_token = deploy.token_from_version();

            let Deploy {
                arguments,
                fee_args,
                rpc,
                ..
            } = deploy;

            let provider = rpc.get_provider(&config).await?;

            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore,
            )
            .await?;

            let fee_settings = fee_args
                .clone()
                .fee_token(fee_token)
                .try_into_fee_settings(&provider, account.block_id())
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
                fee_settings,
                deploy.nonce,
                &account,
                wait_config,
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("deploy", &result, numbers_format, output_format)?;
            print_block_explorer_link_if_allowed(
                &result,
                output_format,
                provider.chain_id().await?,
                config.show_explorer_links,
                config.block_explorer,
            );
            Ok(())
        }

        Commands::Call(Call {
            contract_address,
            function,
            arguments,
            block_id,
            rpc,
        }) => {
            let provider = rpc.get_provider(&config).await?;

            let block_id = get_block_id(&block_id)?;
            let class_hash = get_class_hash_by_address(&provider, contract_address).await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;

            let selector = get_selector_from_name(&function)
                .context("Failed to convert entry point selector to FieldElement")?;

            let calldata = arguments.try_into_calldata(contract_class, &selector)?;

            let result = starknet_commands::call::call(
                contract_address,
                selector,
                calldata,
                &provider,
                block_id.as_ref(),
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("call", &result, numbers_format, output_format)?;
            Ok(())
        }

        Commands::Invoke(invoke) => {
            invoke.validate()?;

            let fee_token = invoke.token_from_version();

            let Invoke {
                contract_address,
                function,
                arguments,
                fee_args,
                rpc,
                nonce,
                ..
            } = invoke;

            let provider = rpc.get_provider(&config).await?;

            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore,
            )
            .await?;

            let fee_args = fee_args.fee_token(fee_token);

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
            )
            .await
            .map_err(handle_starknet_command_error);

            print_command_result("invoke", &result, numbers_format, output_format)?;
            print_block_explorer_link_if_allowed(
                &result,
                output_format,
                provider.chain_id().await?,
                config.show_explorer_links,
                config.block_explorer,
            );
            Ok(())
        }

        Commands::Multicall(multicall) => {
            match &multicall.command {
                starknet_commands::multicall::Commands::New(new) => {
                    if let Some(output_path) = &new.output_path {
                        let result = starknet_commands::multicall::new::write_empty_template(
                            output_path,
                            new.overwrite,
                        );

                        print_command_result(
                            "multicall new",
                            &result,
                            numbers_format,
                            output_format,
                        )?;
                    } else {
                        println!("{DEFAULT_MULTICALL_CONTENTS}");
                    }
                }
                starknet_commands::multicall::Commands::Run(run) => {
                    let provider = run.rpc.get_provider(&config).await?;

                    run.validate()?;

                    let account = get_account(
                        &config.account,
                        &config.accounts_file,
                        &provider,
                        config.keystore,
                    )
                    .await?;
                    let result =
                        starknet_commands::multicall::run::run(run.clone(), &account, wait_config)
                            .await;

                    print_command_result("multicall run", &result, numbers_format, output_format)?;
                    print_block_explorer_link_if_allowed(
                        &result,
                        output_format,
                        provider.chain_id().await?,
                        config.show_explorer_links,
                        config.block_explorer,
                    );
                }
            }
            Ok(())
        }

        Commands::Account(account) => match account.command {
            account::Commands::Import(import) => {
                let provider = import.rpc.get_provider(&config).await?;
                let result = starknet_commands::account::import::import(
                    import.name.clone(),
                    &config.accounts_file,
                    &provider,
                    &import,
                )
                .await;

                print_command_result("account import", &result, numbers_format, output_format)?;
                Ok(())
            }

            account::Commands::Create(create) => {
                let provider = create.rpc.get_provider(&config).await?;

                let chain_id = get_chain_id(&provider).await?;
                let account = if config.keystore.is_none() {
                    create
                        .name
                        .clone()
                        .context("Required argument `--name` not provided")?
                } else {
                    config.account
                };
                let result = starknet_commands::account::create::create(
                    &account,
                    &config.accounts_file,
                    config.keystore,
                    &provider,
                    chain_id,
                    &create,
                )
                .await;

                print_command_result("account create", &result, numbers_format, output_format)?;
                print_block_explorer_link_if_allowed(
                    &result,
                    output_format,
                    provider.chain_id().await?,
                    config.show_explorer_links,
                    config.block_explorer,
                );
                Ok(())
            }

            account::Commands::Deploy(deploy) => {
                deploy.validate()?;

                let provider = deploy.rpc.get_provider(&config).await?;

                let chain_id = get_chain_id(&provider).await?;
                let keystore_path = config.keystore.clone();
                let result = starknet_commands::account::deploy::deploy(
                    &provider,
                    config.accounts_file,
                    deploy,
                    chain_id,
                    wait_config,
                    &config.account,
                    keystore_path,
                )
                .await;

                print_command_result("account deploy", &result, numbers_format, output_format)?;
                print_block_explorer_link_if_allowed(
                    &result,
                    output_format,
                    provider.chain_id().await?,
                    config.show_explorer_links,
                    config.block_explorer,
                );
                Ok(())
            }

            account::Commands::Delete(delete) => {
                let network_name =
                    starknet_commands::account::delete::get_network_name(&delete, &config).await?;

                let result = starknet_commands::account::delete::delete(
                    &delete.name,
                    &config.accounts_file,
                    &network_name,
                    delete.yes,
                );

                print_command_result("account delete", &result, numbers_format, output_format)?;
                Ok(())
            }

            account::Commands::List(options) => print_account_list(
                &config.accounts_file,
                options.display_private_keys,
                numbers_format,
                output_format,
            ),
        },

        Commands::ShowConfig(show) => {
            let provider = show.rpc.get_provider(&config).await?;

            let result =
                starknet_commands::show_config::show_config(&show, &provider, config, cli.profile)
                    .await;

            print_command_result("show-config", &result, numbers_format, output_format)?;

            Ok(())
        }

        Commands::TxStatus(tx_status) => {
            let provider = tx_status.rpc.get_provider(&config).await?;

            let result =
                starknet_commands::tx_status::tx_status(&provider, tx_status.transaction_hash)
                    .await
                    .context("Failed to get transaction status");

            print_command_result("tx-status", &result, numbers_format, output_format)?;
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
            )
            .expect("Failed to build contract");
            let result = starknet_commands::verify::verify(
                verify.contract_address,
                verify.contract_name,
                verify.verifier,
                verify.network,
                verify.confirm_verification,
                &package_metadata.manifest_path,
                &artifacts,
            )
            .await;

            print_command_result("verify", &result, numbers_format, output_format)?;
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
    output_format: OutputFormat,
) -> Result<()> {
    match &script.command {
        starknet_commands::script::Commands::Init(init) => {
            let result = starknet_commands::script::init::init(init);
            print_command_result("script init", &result, numbers_format, output_format)?;
        }
        starknet_commands::script::Commands::Run(run) => {
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &run.package)?;

            let mut config = load_global_config::<CastConfig>(
                &Some(package_metadata.root.clone()),
                &cli.profile,
            )?;
            update_cast_config(&mut config, cli);
            let provider = runtime.block_on(run.rpc.get_provider(&config))?;

            let mut artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path.clone(),
                    json: cli.json,
                    profile: cli.profile.clone().unwrap_or("dev".to_string()),
                },
                true,
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
                "dev",
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

            let result = starknet_commands::script::run::run(
                &run.script_name,
                &metadata_with_deps,
                &package_metadata,
                &mut artifacts,
                &provider,
                runtime,
                &config,
                state_file_path,
            );

            print_command_result("script run", &result, numbers_format, output_format)?;
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
