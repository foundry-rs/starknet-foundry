use std::str::FromStr;

use super::deploy::compute_account_address;
use crate::starknet_commands::account::{
    generate_add_profile_message, prepare_account_json, write_account_to_accounts_file,
};
use anyhow::{Context, Result, bail, ensure};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::{TryFromDecStr, TryFromHexStr};
use sncast::check_if_legacy_contract;
use sncast::helpers::account::generate_account_name;
use sncast::helpers::braavos::check_braavos_account_compatibility;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::import::AccountImportResponse;
use sncast::{AccountType, check_class_hash_exists, get_chain_id, handle_rpc_error};
use starknet::core::types::{BlockId, BlockTag, StarknetError};
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::providers::{Provider, ProviderError};
use starknet::signers::SigningKey;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Import {
    /// Name of the account to be imported
    #[arg(short, long)]
    pub name: Option<String>,

    /// Address of the account
    #[arg(short, long)]
    pub address: Felt,

    /// Type of the account
    #[arg(short = 't', long = "type", value_parser = AccountType::from_str)]
    pub account_type: AccountType,

    /// Class hash of the account
    #[arg(short, long)]
    pub class_hash: Option<Felt>,

    /// Account private key
    #[arg(long, group = "private_key_input")]
    pub private_key: Option<Felt>,

    /// Path to the file holding account private key
    #[arg(long = "private-key-file", group = "private_key_input")]
    pub private_key_file_path: Option<Utf8PathBuf>,

    /// Salt for the address
    #[arg(short, long)]
    pub salt: Option<Felt>,

    /// If passed, a profile with the provided name and corresponding data will be created in snfoundry.toml
    #[arg(long, conflicts_with = "network")]
    pub add_profile: Option<String>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[arg(long)]
    pub silent: bool,
}

pub async fn import(
    account: Option<String>,
    accounts_file: &Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    import: &Import,
) -> Result<AccountImportResponse> {
    // Braavos accounts before v1.2.0 are not compatible with Starknet >= 0.13.4
    // For more, read https://community.starknet.io/t/starknet-devtools-for-0-13-5/115495#p-2359168-braavos-compatibility-issues-3
    if let Some(class_hash) = import.class_hash {
        check_braavos_account_compatibility(class_hash)?;
    }

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
        let sncast_account_type = import.account_type;
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
        import.account_type,
        Some(class_hash),
        import.salt,
    );

    write_account_to_accounts_file(&account_name, accounts_file, chain_id, account_json.clone())?;

    let add_profile_message = generate_add_profile_message(
        import.add_profile.as_ref(),
        &import.rpc,
        &account_name,
        accounts_file,
        None,
    )?;

    Ok(AccountImportResponse {
        add_profile: add_profile_message,
        account_name,
    })
}

fn get_private_key_from_file(file_path: &Utf8PathBuf) -> Result<Felt> {
    let private_key_string = std::fs::read_to_string(file_path.clone())?;
    Ok(private_key_string.parse()?)
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

#[cfg(test)]
mod tests {
    use crate::starknet_commands::account::import::parse_input_to_felt;
    use conversions::string::TryFromHexStr;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_parse_hex_str() {
        let hex_str = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_input_to_felt(hex_str);

        assert_eq!(result.unwrap(), Felt::try_from_hex_str("0x1").unwrap());
    }

    #[test]
    fn test_parse_hex_str_padded() {
        let hex_str = "0x1a2b3c";
        let result = parse_input_to_felt(hex_str);

        assert_eq!(result.unwrap(), Felt::try_from_hex_str("0x1a2b3c").unwrap());
    }

    #[test]
    fn test_parse_hex_str_invalid() {
        let hex_str = "0xz";
        let result = parse_input_to_felt(hex_str);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!("Failed to parse the value 0xz as a felt", error_message);
    }

    #[test]
    fn test_parse_dec_str() {
        let dec_str = "123";
        let result = parse_input_to_felt(dec_str);

        assert_eq!(result.unwrap(), Felt::from(123));
    }

    #[test]
    fn test_parse_dec_str_negative() {
        let dec_str = "-123";
        let result = parse_input_to_felt(dec_str);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!("Failed to parse the value -123 as a felt", error_message);
    }

    #[test]
    fn test_parse_invalid_str() {
        let invalid_str = "invalid";
        let result = parse_input_to_felt(invalid_str);

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!("Failed to parse the value invalid as a felt", error_message);
    }
}
