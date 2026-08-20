use crate::starknet_commands::account::create::Create;
use crate::starknet_commands::account::delete::Delete;
use crate::starknet_commands::account::deploy::Deploy;
use crate::starknet_commands::account::import::Import;
use crate::starknet_commands::account::list::{AccountsListMessage, List};
use crate::starknet_commands::account::migrate::Migrate;
use crate::{process_command_result, starknet_commands};
use anyhow::{Context, Result, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Subcommand};
use configuration::resolve_config_file;
use configuration::{load_config, search_config_upwards_relative_to};
use conversions::string::{TryFromDecStr, TryFromHexStr};
use sncast::accounts::{AccountName, AccountRecord, AccountRepository, NetworkName};
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::configuration::{
    CastConfig, NetworkParams, PartialCastConfig, SncastProfileAppend,
};
use sncast::helpers::constants::BRAAVOS_BASE_ACCOUNT_CLASS_HASH;
use sncast::helpers::interactive::prompt_to_add_account_as_default;
use sncast::helpers::ledger;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::ui::UI;
use sncast::signers::SignerSpec;
use sncast::{AccountType, chain_id_to_network_name};
use sncast::{SignerSource, WaitForTx, get_chain_id};
use starknet_curve::curve_params::EC_ORDER;
use starknet_rust::accounts::{AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_rust::signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::Felt;
use std::collections::BTreeMap;
use std::io::{self, IsTerminal};
use std::process::ExitCode;
use std::{fs::OpenOptions, io::Write};

pub mod create;
pub mod delete;
pub mod deploy;
pub mod import;
pub mod list;
pub mod migrate;

#[derive(Args)]
#[command(about = "Creates and deploys an account to the Starknet")]
pub struct Account {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Import(Import),
    Create(Create),
    Deploy(Deploy),
    Delete(Delete),
    List(List),
    Migrate(Migrate),
}

#[derive(Args, Debug)]
pub struct PrivateKeyArgs {
    /// Account private key
    #[arg(
        long,
        group = "private_key_input",
        conflicts_with = "ledger_key_locator_account"
    )]
    pub private_key: Option<Felt>,

    /// Path to the file holding account private key
    #[arg(
        long = "private-key-file",
        group = "private_key_input",
        conflicts_with = "ledger_key_locator_account"
    )]
    pub private_key_file_path: Option<Utf8PathBuf>,
}

impl PrivateKeyArgs {
    fn resolve_optional(&self) -> Result<Option<Felt>> {
        match (&self.private_key, &self.private_key_file_path) {
            (Some(key), None) => Ok(Some(*key)),
            (None, Some(path)) => get_private_key_from_file(path)
                .with_context(|| format!("Failed to obtain private key from the file {path}"))
                .map(Some),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => {
                unreachable!("`--private-key` and `--private-key-file` are mutually exclusive")
            }
        }
    }

    fn resolve_or_prompt(&self) -> Result<Felt> {
        self.resolve_optional()?
            .map_or_else(get_private_key_from_input, Ok)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_account_record(
    signer: SignerSpec,
    public_key: Felt,
    address: Felt,
    deployed: bool,
    legacy: bool,
    account_type: AccountType,
    class_hash: Option<Felt>,
    salt: Option<Felt>,
) -> AccountRecord {
    AccountRecord {
        public_key,
        address: Some(address),
        salt,
        deployed: Some(deployed),
        class_hash,
        legacy: Some(legacy),
        account_type: Some(account_type),
        signer,
    }
}

fn get_private_key_from_file(file_path: &Utf8PathBuf) -> Result<Felt> {
    let private_key_string = std::fs::read_to_string(file_path.clone())?;
    Ok(private_key_string.parse()?)
}

/// Validates that `private_key` is a valid secret scalar of the STARK curve,
/// i.e. it is non-zero and strictly smaller than the curve order.
fn validate_private_key(private_key: Felt) -> Result<Felt> {
    ensure!(
        private_key != Felt::ZERO,
        "Invalid private key: the private key cannot be 0"
    );
    ensure!(
        private_key < EC_ORDER,
        "Invalid private key: the private key must be smaller than the STARK curve order ({EC_ORDER:#x})"
    );
    Ok(private_key)
}

fn parse_input_to_felt(input: &str) -> Result<Felt> {
    Felt::try_from_hex_str(input)
        .or_else(|_| Felt::try_from_dec_str(input))
        .with_context(|| format!("Failed to parse the value {input} as a felt"))
}

fn get_private_key_from_input() -> Result<Felt> {
    let input = rpassword::prompt_password("Type in your private key and press enter: ")
        .expect("Failed to read private key from input");
    parse_input_to_felt(&input)
}

pub fn save_account(
    account: &str,
    repository: &AccountRepository,
    chain_id: Felt,
    account_record: AccountRecord,
) -> Result<()> {
    let network_name = chain_id_to_network_name(chain_id);
    repository
        .insert(
            NetworkName::new(network_name)?,
            AccountName::new(account)?,
            account_record,
        )
        .map_err(|error| anyhow::anyhow!(error))?;
    Ok(())
}

pub fn add_created_profile_to_configuration(
    profile: Option<&str>,
    cast_config: &CastConfig,
    path: &Utf8PathBuf,
) -> Result<()> {
    let config_path = search_config_upwards_relative_to(path)?;
    let existing = load_config::<PartialCastConfig>(&config_path, profile)?;
    if existing
        .as_ref()
        .and_then(|c| c.account.as_ref())
        .is_some_and(|a| !a.is_empty())
    {
        bail!(
            "Failed to add profile = {} to the snfoundry.toml. Profile already exists",
            profile.unwrap_or("default")
        );
    }

    let profile_config = PartialCastConfig {
        network_params: cast_config.network_params.clone(),
        account: Some(cast_config.account.clone()),
        keystore: cast_config.keystore.clone(),
        accounts_file: cast_config
            .keystore
            .is_none()
            .then(|| cast_config.accounts_file.clone()),
        ..Default::default()
    };

    let profile_key = profile.map_or_else(|| cast_config.account.clone(), ToString::to_string);
    let append = SncastProfileAppend {
        sncast: BTreeMap::from([(profile_key, profile_config)]),
    };
    let toml_string = toml::to_string(&append).context("Failed to convert toml to string")?;

    let mut snfoundry_toml = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)
        .context("Failed to open snfoundry.toml")?;
    snfoundry_toml
        .write_all(format!("\n{toml_string}").as_bytes())
        .context("Failed to write to the snfoundry.toml")?;

    Ok(())
}

fn generate_add_profile_message(
    profile_name: Option<&String>,
    rpc_args: &RpcArgs,
    account_name: &str,
    accounts_file: &Utf8Path,
    keystore: Option<Utf8PathBuf>,
    config: &CastConfig,
) -> Result<Option<String>> {
    if let Some(profile_name) = profile_name {
        let network_params = if rpc_args.url.is_some() || rpc_args.network.is_some() {
            NetworkParams::new(rpc_args.url.clone(), rpc_args.network)?
        } else {
            config.network_params.clone()
        };
        let config = CastConfig {
            network_params,
            account: account_name.into(),
            accounts_file: accounts_file.into(),
            keystore,
            ..Default::default()
        };
        let config_path = resolve_config_file();
        add_created_profile_to_configuration(Some(profile_name), &config, &config_path)?;
        Ok(Some(format!(
            "Profile {profile_name} successfully added to {config_path}",
        )))
    } else {
        Ok(None)
    }
}

#[allow(clippy::too_many_lines)]
pub async fn account(
    account: Account,
    config: CastConfig,
    ui: &UI,
    wait_config: WaitForTx,
) -> Result<ExitCode> {
    let repository = AccountRepository::new(config.accounts_file.clone());
    match account.command {
        Commands::Import(import) => {
            let provider = import.rpc.get_provider(&config, ui).await?;

            let result = starknet_commands::account::import::import(
                import.name.clone(),
                &repository,
                &provider,
                &import,
                &config,
                ui,
            )
            .await;

            let run_interactive_prompt =
                !import.silent && result.is_ok() && io::stdout().is_terminal();

            if run_interactive_prompt
                && let Some(account_name) = result.as_ref().ok().map(|r| r.account_name.clone())
                && let Err(err) = prompt_to_add_account_as_default(account_name.as_str(), ui)
            {
                // TODO(#3436)
                ui.print_error(
                    "account import",
                    format!("Error: Failed to launch interactive prompt: {err}"),
                );
            }

            Ok(process_command_result("account import", result, ui, None))
        }
        Commands::Create(create) => {
            let signer_type = create.ledger_key_locator.resolve(ui);

            let signer_source = SignerSource::new(config.keystore.clone(), signer_type)?;

            let account = if config.keystore.is_none() {
                create
                    .name
                    .clone()
                    .unwrap_or_else(|| repository.generate_account_name().unwrap())
            } else {
                config.account.clone()
            };

            let provider = create.rpc.get_provider(&config, ui).await?;

            let chain_id = get_chain_id(&provider).await?;

            let result = starknet_commands::account::create::create(
                &account,
                &repository,
                &provider,
                chain_id,
                &create,
                &config,
                &signer_source,
                ui,
            )
            .await;

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;

            Ok(process_command_result(
                "account create",
                result,
                ui,
                block_explorer_link,
            ))
        }

        Commands::Deploy(deploy) => {
            let provider = deploy.rpc.get_provider(&config, ui).await?;

            let chain_id = get_chain_id(&provider).await?;
            let result = starknet_commands::account::deploy::deploy(
                &provider,
                &repository,
                &deploy,
                chain_id,
                wait_config,
                &config.account,
                config.keystore.clone(),
                deploy.fee_args,
                deploy.dry_run_args,
                ui,
            )
            .await;

            let run_interactive_prompt =
                !deploy.silent && result.is_ok() && io::stdout().is_terminal();

            if config.keystore.is_none()
                && run_interactive_prompt
                && let Err(err) = prompt_to_add_account_as_default(
                    deploy
                        .name
                        .as_ref()
                        .expect("Must be provided when using accounts file"),
                    ui,
                )
            {
                // TODO(#3436)
                ui.print_error(
                    "account deploy",
                    format!("Error: Failed to launch interactive prompt: {err}"),
                );
            }

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            Ok(process_command_result(
                "account deploy",
                result,
                ui,
                block_explorer_link,
            ))
        }

        Commands::Delete(delete) => {
            let network_name =
                starknet_commands::account::delete::get_network_name(&delete, &config, ui).await?;

            let result = starknet_commands::account::delete::delete(
                &delete.name,
                &repository,
                &network_name,
                delete.yes,
            );

            Ok(process_command_result("account delete", result, ui, None))
        }

        Commands::List(options) => {
            ui.print_message(
                "account delete",
                AccountsListMessage::new(&repository, options.display_private_keys)?,
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::Migrate(_) => {
            let result = starknet_commands::account::migrate::migrate(&repository);
            Ok(process_command_result("account migrate", result, ui, None))
        }
    }
}

pub async fn compute_account_address(
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    chain_id: Felt,
    signer: &SignerSpec,
    provider: &JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<Felt> {
    let address = match signer {
        SignerSpec::PrivateKey(spec) => {
            let signer =
                LocalWallet::from_signing_key(SigningKey::from_secret_scalar(spec.private_key()));
            compute_address_with_signer(salt, class_hash, account_type, chain_id, signer, provider)
                .await?
        }
        SignerSpec::Ledger(spec) => {
            let signer = ledger::create_ledger_signer(spec.derivation_path(), ui, false).await?;
            compute_address_with_signer(salt, class_hash, account_type, chain_id, signer, provider)
                .await?
        }
        SignerSpec::Keystore(_) => {
            bail!("keystore signer must be resolved before computing an account address")
        }
    };
    Ok(address)
}

async fn compute_address_with_signer<S>(
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    chain_id: Felt,
    signer: S,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<Felt>
where
    S: starknet_rust::signers::Signer + Send + Sync,
    <S as starknet_rust::signers::Signer>::GetPublicKeyError: 'static,
{
    let address = match account_type {
        AccountType::OpenZeppelin => {
            let factory =
                OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
            factory.deploy_v3(salt).address()
        }
        AccountType::Ready => {
            let factory =
                ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;
            factory.deploy_v3(salt).address()
        }
        AccountType::Braavos => {
            let factory = BraavosAccountFactory::new(
                class_hash,
                BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                chain_id,
                signer,
                provider,
            )
            .await?;
            factory.deploy_v3(salt).address()
        }
    };
    Ok(address)
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use configuration::test_utils::copy_config_to_tempdir;
    use sncast::helpers::{
        configuration::{CastConfig, NetworkParams},
        constants::DEFAULT_ACCOUNTS_FILE,
    };
    use std::fs;
    use url::Url;

    use crate::starknet_commands::account::add_created_profile_to_configuration;

    #[test]
    fn test_add_created_profile_to_configuration_happy_case() {
        let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
        let path = Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap();
        let config = CastConfig {
            network_params: NetworkParams::new(
                Some(Url::parse("http://some-url.com/").unwrap()),
                None,
            )
            .unwrap(),
            account: String::from("some-name"),
            accounts_file: "accounts".into(),
            ..Default::default()
        };
        let res = add_created_profile_to_configuration(
            Some(&String::from("some-name")),
            &config,
            &path.clone(),
        );
        assert!(res.is_ok());

        let contents =
            fs::read_to_string(path.join("snfoundry.toml")).expect("Failed to read snfoundry.toml");

        assert!(contents.contains("[sncast.some-name]"));
        assert!(contents.contains("account = \"some-name\""));
        assert!(contents.contains("url = \"http://some-url.com/\""));
        assert!(contents.contains("accounts-file = \"accounts\""));
    }

    #[test]
    fn test_add_created_profile_to_configuration_profile_already_exists() {
        let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
        let config = CastConfig {
            network_params: NetworkParams::new(
                Some(Url::parse("http://some-url.com/").unwrap()),
                None,
            )
            .unwrap(),
            account: String::from("user1"),
            accounts_file: DEFAULT_ACCOUNTS_FILE.into(),
            ..Default::default()
        };
        let res = add_created_profile_to_configuration(
            Some(&String::from("default")),
            &config,
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap(),
        );
        assert!(res.is_err());
    }
}
