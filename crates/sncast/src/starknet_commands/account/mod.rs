use crate::starknet_commands::account::create::Create;
use crate::starknet_commands::account::delete::Delete;
use crate::starknet_commands::account::deploy::Deploy;
use crate::starknet_commands::account::import::Import;
use crate::starknet_commands::account::list::{AccountsListMessage, List};
use crate::{process_command_result, starknet_commands};
use anyhow::{Context, Result, bail, ensure};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use configuration::resolve_config_file;
use configuration::{load_config, search_config_upwards_relative_to};
use serde_json::json;
use sncast::helpers::account::{generate_account_name, load_accounts};
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
use sncast::{AccountType, chain_id_to_network_name, decode_chain_id};
use sncast::{SignerSource, SignerType, WaitForTx, get_chain_id};
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
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_account_json(
    signer_type: &SignerType,
    public_key: Felt,
    address: Felt,
    deployed: bool,
    legacy: bool,
    account_type: AccountType,
    class_hash: Option<Felt>,
    salt: Option<Felt>,
) -> serde_json::Value {
    let mut account_json = json!({
        "public_key": format!("{public_key:#x}"),
        "address": format!("{address:#x}"),
        "type": format!("{account_type}").to_lowercase().replace("openzeppelin", "open_zeppelin"),
        "deployed": deployed,
        "legacy": legacy,
    });

    match signer_type {
        SignerType::Local { private_key } => {
            account_json["private_key"] = serde_json::Value::String(format!("{private_key:#x}"));
        }
        SignerType::Ledger { ledger_path } => {
            account_json["ledger_path"] =
                serde_json::Value::String(ledger_path.derivation_string());
        }
    }

    if let Some(salt) = salt {
        account_json["salt"] = serde_json::Value::String(format!("{salt:#x}"));
    }
    if let Some(class_hash) = class_hash {
        account_json["class_hash"] = serde_json::Value::String(format!("{class_hash:#x}"));
    }

    account_json
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

pub fn write_account_to_accounts_file(
    account: &str,
    accounts_file: &Utf8PathBuf,
    chain_id: Felt,
    account_json: serde_json::Value,
) -> Result<()> {
    if !accounts_file.exists() {
        std::fs::create_dir_all(accounts_file.clone().parent().unwrap())?;
        std::fs::write(accounts_file.clone(), "{}")?;
    }

    let mut items = load_accounts(accounts_file)?;

    let network_name = chain_id_to_network_name(chain_id);

    if !items[&network_name][account].is_null() {
        bail!(
            "Account with name = {} already exists in network with chain_id = {}",
            account,
            decode_chain_id(chain_id)
        );
    }
    items[&network_name][account] = account_json;

    std::fs::write(
        accounts_file.clone(),
        serde_json::to_string_pretty(&items).unwrap(),
    )?;
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
    accounts_file: &Utf8PathBuf,
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
    match account.command {
        Commands::Import(import) => {
            let provider = import.rpc.get_provider(&config, ui).await?;

            let result = starknet_commands::account::import::import(
                import.name.clone(),
                &config.accounts_file,
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
            let provider = create.rpc.get_provider(&config, ui).await?;

            let chain_id = get_chain_id(&provider).await?;

            let signer_type = create
                .ledger_key_locator
                .resolve(ui)
                .map(|ledger_path| SignerType::Ledger { ledger_path });

            let signer_source = SignerSource::new(config.keystore.clone(), signer_type.as_ref())?;

            let account = if config.keystore.is_none() {
                create
                    .name
                    .clone()
                    .unwrap_or_else(|| generate_account_name(&config.accounts_file).unwrap())
            } else {
                config.account.clone()
            };

            let result = starknet_commands::account::create::create(
                &account,
                &config.accounts_file,
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
                &config.accounts_file,
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
                &config.accounts_file,
                &network_name,
                delete.yes,
            );

            Ok(process_command_result("account delete", result, ui, None))
        }

        Commands::List(options) => {
            ui.print_message(
                "account delete",
                AccountsListMessage::new(config.accounts_file, options.display_private_keys)?,
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

pub async fn compute_account_address(
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
    chain_id: Felt,
    signer_type: &SignerType,
    provider: &JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<Felt> {
    let address = match signer_type {
        SignerType::Local { private_key } => {
            let signer =
                LocalWallet::from_signing_key(SigningKey::from_secret_scalar(*private_key));
            compute_address_with_signer(salt, class_hash, account_type, chain_id, signer, provider)
                .await?
        }
        SignerType::Ledger { ledger_path } => {
            let signer = ledger::create_ledger_signer(ledger_path, ui, false).await?;
            compute_address_with_signer(salt, class_hash, account_type, chain_id, signer, provider)
                .await?
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
