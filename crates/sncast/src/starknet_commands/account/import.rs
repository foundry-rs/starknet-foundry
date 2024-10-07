use crate::starknet_commands::account::{
    add_created_profile_to_configuration, prepare_account_json, write_account_to_accounts_file,
    AccountType,
};
use anyhow::{bail, ensure, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::TryFromHexStr;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::structs::AccountImportResponse;
use sncast::{check_class_hash_exists, get_chain_id, AccountType as SNCastAccountType};
use sncast::{check_if_legacy_contract, get_class_hash_by_address};
use starknet::core::types::Felt;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::signers::SigningKey;

use super::deploy::compute_account_address;

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Import {
    /// Name of the account to be imported
    #[clap(short, long)]
    pub name: String,

    /// Address of the account
    #[clap(short, long)]
    pub address: Felt,

    /// Type of the account
    #[clap(short = 't', long = "type")]
    pub account_type: AccountType,

    /// Class hash of the account
    #[clap(short, long)]
    pub class_hash: Option<Felt>,

    /// Account private key
    #[clap(long, group = "private_key_input")]
    pub private_key: Option<Felt>,

    /// Path to the file holding account private key
    #[clap(long = "private-key-file", group = "private_key_input")]
    pub private_key_file_path: Option<Utf8PathBuf>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<Felt>,

    /// If passed, a profile with the provided name and corresponding data will be created in snfoundry.toml
    #[allow(clippy::struct_field_names)]
    #[clap(long)]
    pub add_profile: Option<String>,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

pub async fn import(
    account: &str,
    accounts_file: &Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    import: &Import,
) -> Result<AccountImportResponse> {
    let private_key = match (&import.private_key, &import.private_key_file_path) {
        (Some(_), Some(_)) => {
            bail!(
                "Both private key and private key file path were provided. Please provide only one"
            )
        }
        (Some(passed_private_key), None) => passed_private_key,
        (None, Some(passed_private_key_file_path)) => &{
            get_private_key_from_file(passed_private_key_file_path).with_context(|| {
                format!("Failed to obtain private key from the file {passed_private_key_file_path}")
            })?
        },
        (None, None) => &get_private_key_from_input(),
    };
    let private_key = &SigningKey::from_secret_scalar(*private_key);

    let fetched_class_hash = get_class_hash_by_address(provider, import.address).await?;
    let deployed = fetched_class_hash.is_some();
    let class_hash = match (fetched_class_hash, import.class_hash) {
        (Some(from_provider), Some(from_user)) => {
            ensure!(
                from_provider == from_user,
                "Incorrect class hash {:#x} for account address {:#x}",
                from_user,
                import.address
            );
            fetched_class_hash
        }
        (None, Some(from_user)) => {
            check_class_hash_exists(provider, from_user).await?;
            Some(from_user)
        }
        _ => fetched_class_hash,
    };

    let chain_id = get_chain_id(provider).await?;
    if import.salt.is_some() && class_hash.is_some() {
        let sncast_account_type = match import.account_type {
            AccountType::Argent => SNCastAccountType::Argent,
            AccountType::Braavos => SNCastAccountType::Braavos,
            AccountType::Oz => SNCastAccountType::OpenZeppelin,
        };
        let computed_address = compute_account_address(
            import.salt.unwrap(),
            private_key,
            class_hash.unwrap(),
            sncast_account_type,
            chain_id,
        );
        if computed_address != import.address {
            ensure!(
                computed_address == import.address,
                "Computed address {:#x} does not match the provided address {:#x}",
                computed_address,
                import.address
            );
        }
    }

    let legacy = check_if_legacy_contract(class_hash, import.address, provider).await?;

    let account_json = prepare_account_json(
        private_key,
        import.address,
        deployed,
        legacy,
        &import.account_type,
        class_hash,
        import.salt,
    );

    write_account_to_accounts_file(account, accounts_file, chain_id, account_json.clone())?;

    if import.add_profile.is_some() {
        let config = CastConfig {
            url: import.rpc.url.clone().unwrap_or_default(),
            account: account.into(),
            accounts_file: accounts_file.into(),
            ..Default::default()
        };
        add_created_profile_to_configuration(&import.add_profile, &config, &None)?;
    }

    Ok(AccountImportResponse {
        add_profile: if import.add_profile.is_some() {
            format!(
                "Profile {} successfully added to snfoundry.toml",
                import
                    .add_profile
                    .clone()
                    .expect("Failed to get profile name")
            )
        } else {
            "--add-profile flag was not set. No profile added to snfoundry.toml".to_string()
        },
    })
}

fn get_private_key_from_file(file_path: &Utf8PathBuf) -> Result<Felt> {
    let private_key_string = std::fs::read_to_string(file_path.clone())?;
    Ok(private_key_string.parse()?)
}

fn get_private_key_from_input() -> Felt {
    let private_key =
        rpassword::prompt_password("Enter private key: ").expect("Failed to read private key");
    Felt::try_from_hex_str(&private_key).expect("Failed to parse private key into Felt")
}
