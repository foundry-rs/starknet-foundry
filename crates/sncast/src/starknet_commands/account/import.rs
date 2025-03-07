use super::deploy::compute_account_address;
use crate::starknet_commands::account::{
    AccountType, add_created_profile_to_configuration, prepare_account_json,
    write_account_to_accounts_file,
};
use anyhow::{Context, Result, bail, ensure};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::{TryFromDecStr, TryFromHexStr};
use regex::Regex;
use sncast::check_if_legacy_contract;
use sncast::helpers::account::generate_account_name;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::structs::AccountImportResponse;
use sncast::{
    AccountType as SNCastAccountType, check_class_hash_exists, get_chain_id, handle_rpc_error,
};
use starknet::core::types::{BlockId, BlockTag, StarknetError};
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::providers::{Provider, ProviderError};
use starknet::signers::SigningKey;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Import {
    /// Name of the account to be imported
    #[clap(short, long)]
    pub name: Option<String>,

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
    #[clap(long, conflicts_with = "network")]
    pub add_profile: Option<String>,

    #[clap(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[clap(long)]
    pub silent: bool,
}

pub async fn import(
    account: Option<String>,
    accounts_file: &Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    import: &Import,
) -> Result<AccountImportResponse> {
    let private_key = if let Some(passed_private_key) = &import.private_key {
        passed_private_key
    } else if let Some(passed_private_key_file_path) = &import.private_key_file_path {
        &get_private_key_from_file(passed_private_key_file_path).with_context(|| {
            format!("Failed to obtain private key from the file {passed_private_key_file_path}")
        })?
    } else if import.private_key.is_none() && import.private_key_file_path.is_none() {
        &get_private_key_from_input()?
    } else {
        unreachable!("Checked on clap level")
    };
    let private_key = &SigningKey::from_secret_scalar(*private_key);

    let account_name = account
        .clone()
        .unwrap_or_else(|| generate_account_name(accounts_file).unwrap());

    let fetched_class_hash = match provider
        .get_class_hash_at(BlockId::Tag(BlockTag::Pending), import.address)
        .await
    {
        Ok(class_hash) => Ok(Some(class_hash)),
        Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => Ok(None),
        Err(err) => Err(handle_rpc_error(err)),
    }?;

    let deployed: bool = fetched_class_hash.is_some();
    let class_hash = if let (Some(from_provider), Some(from_user)) =
        (fetched_class_hash, import.class_hash)
    {
        ensure!(
            from_provider == from_user,
            "Incorrect class hash {:#x} for account address {:#x} was provided",
            from_user,
            import.address
        );
        from_provider
    } else if let Some(from_user) = import.class_hash {
        check_class_hash_exists(provider, from_user).await?;
        from_user
    } else if let Some(from_provider) = fetched_class_hash {
        from_provider
    } else {
        bail!(
            "Class hash for the account address {:#x} could not be found. Please provide the class hash",
            import.address
        );
    };

    let chain_id = get_chain_id(provider).await?;
    if let Some(salt) = import.salt {
        // TODO(#2571)
        let sncast_account_type = match import.account_type {
            AccountType::Argent => SNCastAccountType::Argent,
            AccountType::Braavos => SNCastAccountType::Braavos,
            AccountType::Oz => SNCastAccountType::OpenZeppelin,
        };
        let computed_address =
            compute_account_address(salt, private_key, class_hash, sncast_account_type, chain_id);
        ensure!(
            computed_address == import.address,
            "Computed address {:#x} does not match the provided address {:#x}. Please ensure that the provided salt, class hash, and account type are correct.",
            computed_address,
            import.address
        );
    }

    let legacy = check_if_legacy_contract(Some(class_hash), import.address, provider).await?;

    let account_json = prepare_account_json(
        private_key,
        import.address,
        deployed,
        legacy,
        &import.account_type,
        Some(class_hash),
        import.salt,
    );

    write_account_to_accounts_file(&account_name, accounts_file, chain_id, account_json.clone())?;

    if import.add_profile.is_some() {
        if let Some(url) = &import.rpc.url {
            let config = CastConfig {
                url: url.clone(),
                account: account_name.clone(),
                accounts_file: accounts_file.into(),
                ..Default::default()
            };
            add_created_profile_to_configuration(import.add_profile.as_deref(), &config, None)?;
        } else {
            unreachable!("Conflicting arguments should be handled in clap");
        }
    }

    Ok(AccountImportResponse {
        add_profile: import.add_profile.as_ref().map_or_else(
            || "--add-profile flag was not set. No profile added to snfoundry.toml".to_string(),
            |profile_name| format!("Profile {profile_name} successfully added to snfoundry.toml"),
        ),
        account_name: account.map_or_else(|| Some(account_name), |_| None),
    })
}

fn get_private_key_from_file(file_path: &Utf8PathBuf) -> Result<Felt> {
    let private_key_string = std::fs::read_to_string(file_path.clone())?;
    Ok(private_key_string.parse()?)
}

fn parse_input_to_felt(input: &String) -> Result<Felt> {
    // Original is from spec https://github.com/starkware-libs/starknet-specs/blob/6d88b7399f56260ece3821c71f9ce53ec55f830b/api/starknet_api_openrpc.json#L1303
    // doesn't allow for padded felts, hence we use adjusted one
    let felt_re = Regex::new(r"^0x[0-9a-fA-F]{1,64}$").unwrap();
    if input.starts_with("0x") && !felt_re.is_match(input) {
        bail!(
            "Failed to parse value {} to felt. Invalid hex value was passed",
            input
        );
    } else if let Ok(felt_from_hex) = Felt::try_from_hex_str(input) {
        return Ok(felt_from_hex);
    } else if let Ok(felt_from_dec) = Felt::try_from_dec_str(input) {
        return Ok(felt_from_dec);
    }
    bail!("Failed to parse value {} to felt", input);
}

fn get_private_key_from_input() -> Result<Felt> {
    let input = rpassword::prompt_password("Type in your private key and press enter: ")
        .expect("Failed to read private key from input");
    parse_input_to_felt(&input)
}

#[cfg(test)]
mod tests {
    use crate::starknet_commands::account::import::parse_input_to_felt;
    use conversions::string::TryFromHexStr;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_parse_hex_str() {
        let hex_str = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_input_to_felt(&hex_str.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Felt::try_from_hex_str("0x1").unwrap());
    }

    #[test]
    fn test_parse_hex_str_padded() {
        let hex_str = "0x1a2b3c";
        let result = parse_input_to_felt(&hex_str.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Felt::try_from_hex_str("0x1a2b3c").unwrap());
    }

    #[test]
    fn test_parse_hex_str_invalid() {
        let hex_str = "0xz";
        let result = parse_input_to_felt(&hex_str.to_string());

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!(
            "Failed to parse value 0xz to felt. Invalid hex value was passed",
            error_message
        );
    }

    #[test]
    fn test_parse_dec_str() {
        let dec_str = "123";
        let result = parse_input_to_felt(&dec_str.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Felt::from(123));
    }

    #[test]
    fn test_parse_dec_str_negative() {
        let dec_str = "-123";
        let result = parse_input_to_felt(&dec_str.to_string());

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!("Failed to parse value -123 to felt", error_message);
    }

    #[test]
    fn test_parse_invalid_str() {
        let invalid_str = "invalid";
        let result = parse_input_to_felt(&invalid_str.to_string());

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!("Failed to parse value invalid to felt", error_message);
    }
}
