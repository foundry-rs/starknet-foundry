use std::str::FromStr;

use crate::starknet_commands::account::{
    compute_account_address, generate_add_profile_message, prepare_account_json,
    write_account_to_accounts_file,
};
use crate::starknet_commands::utils::felt_or_id::{ClassHash, ContractAddress};
use anyhow::{Context, Result, bail, ensure};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::{TryFromDecStr, TryFromHexStr};
use sncast::check_if_legacy_contract;
use sncast::helpers::account::generate_account_name;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
use sncast::helpers::keystore_locator::KeystoreImport;
use sncast::helpers::ledger;
use sncast::helpers::ledger::LedgerKeyLocatorAccount;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::import::AccountImportResponse;
use sncast::response::ui::UI;
use sncast::{
    AccountType, SignerType, check_class_hash_exists, get_chain_id, get_keystore_password,
    handle_rpc_error, parse_starkli_account_json, read_and_parse_json_file,
};
use starknet_rust::core::types::{BlockId, BlockTag, StarknetError};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::providers::{Provider, ProviderError};
use starknet_rust::signers::{DerivationPath, SigningKey};
use starknet_types_core::felt::Felt;

struct ResolvedImportContext {
    signer_type: SignerType,
    public_key: Felt,
    address: Felt,
    account_type: AccountType,
    salt: Option<Felt>,
    keystore_class_hash: Option<Felt>,
}

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Import {
    /// Name of the account to be imported
    #[arg(short, long)]
    pub name: Option<String>,

    /// Address of the account (hex, decimal, or @alias from snfoundry.toml)
    #[arg(
        short,
        long,
        required_unless_present = "keystore_account",
        conflicts_with = "keystore_account"
    )]
    pub address: Option<ContractAddress>,

    /// Type of the account
    #[arg(
        short = 't',
        long = "type",
        value_parser = AccountType::from_str,
        required_unless_present = "keystore_account",
        conflicts_with = "keystore_account"
    )]
    pub account_type: Option<AccountType>,

    /// Class hash of the account (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short, long, conflicts_with = "keystore_account")]
    pub class_hash: Option<ClassHash>,

    /// Account private key
    #[arg(
        long,
        group = "private_key_input",
        conflicts_with_all = ["ledger_key_locator_account", "keystore"]
    )]
    pub private_key: Option<Felt>,

    /// Path to the file holding account private key
    #[arg(
        long = "private-key-file",
        group = "private_key_input",
        conflicts_with_all = ["ledger_key_locator_account", "keystore"]
    )]
    pub private_key_file_path: Option<Utf8PathBuf>,

    /// Salt for the address
    #[arg(short, long, conflicts_with = "keystore_account")]
    pub salt: Option<Felt>,

    /// If passed, a profile with the provided name and corresponding data will be created in snfoundry.toml
    #[arg(long)]
    pub add_profile: Option<String>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[arg(long)]
    pub silent: bool,

    #[command(flatten)]
    pub keystore_import: KeystoreImport,

    #[command(flatten)]
    pub ledger_key_locator: LedgerKeyLocatorAccount,
}

impl Import {
    pub fn resolved_address(&self, config: &CastConfig) -> Result<Option<Felt>> {
        self.address
            .as_ref()
            .map(|address| address.resolve(config))
            .transpose()
    }

    pub fn resolved_class_hash(&self, config: &CastConfig) -> Result<Option<Felt>> {
        ClassHash::resolve_optional(self.class_hash.as_ref(), config)
    }
}

#[allow(clippy::too_many_lines)]
pub async fn import(
    account: Option<String>,
    accounts_file: &Utf8PathBuf,
    provider: &JsonRpcClient<HttpTransport>,
    import: &Import,
    config: &CastConfig,
    ui: &UI,
) -> Result<AccountImportResponse> {
    let class_hash = import.resolved_class_hash(config)?;

    let resolved = if let Some(keystore_path) = &import.keystore_import.keystore {
        resolve_keystore_import(import, keystore_path, config)?
    } else if let Some(ledger_path) = import.ledger_key_locator.resolve(ui) {
        resolve_ledger_import(import, ledger_path, config, ui).await?
    } else {
        resolve_private_key_import(import, config)?
    };

    let ResolvedImportContext {
        signer_type,
        public_key,
        address,
        account_type,
        salt,
        keystore_class_hash,
    } = resolved;
    let class_hash = class_hash.or(keystore_class_hash);

    let account_name = account
        .clone()
        .unwrap_or_else(|| generate_account_name(accounts_file).unwrap());

    let fetched_class_hash = match provider
        .get_class_hash_at(BlockId::Tag(BlockTag::PreConfirmed), address)
        .await
    {
        Ok(class_hash) => Ok(Some(class_hash)),
        Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => Ok(None),
        Err(err) => Err(handle_rpc_error(err)),
    }?;

    let deployed: bool = fetched_class_hash.is_some();
    let class_hash = if let (Some(from_provider), Some(from_user)) =
        (fetched_class_hash, class_hash)
    {
        ensure!(
            from_provider == from_user,
            "Incorrect class hash {from_user:#x} for account address {address:#x} was provided"
        );
        from_provider
    } else if let Some(from_user) = class_hash {
        check_class_hash_exists(provider, from_user).await?;
        from_user
    } else if let Some(from_provider) = fetched_class_hash {
        from_provider
    } else {
        bail!(
            "Class hash for the account address {address:#x} could not be found. Please provide the class hash"
        );
    };

    let chain_id = get_chain_id(provider).await?;

    if let Some(salt) = salt {
        let computed_address = compute_account_address(
            salt,
            class_hash,
            account_type,
            chain_id,
            &signer_type,
            provider,
            ui,
        )
        .await?;
        ensure!(
            computed_address == address,
            "Computed address {computed_address:#x} does not match the provided address {address:#x}. Please ensure that the provided salt, class hash, and account type are correct."
        );
    }

    let legacy = check_if_legacy_contract(Some(class_hash), address, provider).await?;

    let account_json = prepare_account_json(
        &signer_type,
        public_key,
        address,
        deployed,
        legacy,
        account_type,
        Some(class_hash),
        salt,
    );

    write_account_to_accounts_file(&account_name, accounts_file, chain_id, account_json.clone())?;

    let add_profile_message = generate_add_profile_message(
        import.add_profile.as_ref(),
        &import.rpc,
        &account_name,
        accounts_file,
        None,
        config,
    )?;

    Ok(AccountImportResponse {
        add_profile: add_profile_message,
        account_name,
    })
}

fn resolve_keystore_import(
    import: &Import,
    keystore_path: &Utf8PathBuf,
    config: &CastConfig,
) -> Result<ResolvedImportContext> {
    ensure!(
        keystore_path.exists(),
        "Failed to find keystore file at {keystore_path}"
    );

    let password = get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?;
    let signing_key = SigningKey::from_keystore(keystore_path, password.as_str())
        .with_context(|| format!("Failed to decrypt keystore at {keystore_path}"))?;
    let derived_public_key = signing_key.verifying_key().scalar();

    let signer_type = SignerType::Keystore {
        keystore_path: keystore_path.clone(),
    };

    if let Some(account_json_path) = &import.keystore_import.keystore_account {
        let account_info: serde_json::Value = read_and_parse_json_file(account_json_path)?;
        let identity = parse_starkli_account_json(&account_info)?;
        ensure!(
            identity.public_key == derived_public_key,
            "Public key from keystore does not match account JSON"
        );

        let address = identity.address.context(
            "Account JSON has no deployment address; provide a deployed account file or use --address",
        )?;

        Ok(ResolvedImportContext {
            signer_type,
            public_key: identity.public_key,
            address,
            account_type: identity.account_type,
            salt: identity.salt,
            keystore_class_hash: identity.class_hash,
        })
    } else {
        let address = import
            .resolved_address(config)?
            .expect("clap requires --address when --keystore-account is absent");
        let account_type = import
            .account_type
            .expect("clap requires --type when --keystore-account is absent");
        Ok(ResolvedImportContext {
            signer_type,
            public_key: derived_public_key,
            address,
            account_type,
            salt: import.salt,
            keystore_class_hash: None,
        })
    }
}

async fn resolve_ledger_import(
    import: &Import,
    ledger_path: DerivationPath,
    config: &CastConfig,
    ui: &UI,
) -> Result<ResolvedImportContext> {
    let public_key = ledger::get_ledger_public_key(&ledger_path, ui).await?;
    let address = import
        .resolved_address(config)?
        .expect("clap requires --address when --keystore-account is absent");
    let account_type = import
        .account_type
        .expect("clap requires --type when --keystore-account is absent");
    Ok(ResolvedImportContext {
        signer_type: SignerType::Ledger { ledger_path },
        public_key,
        address,
        account_type,
        salt: import.salt,
        keystore_class_hash: None,
    })
}

fn resolve_private_key_import(
    import: &Import,
    config: &CastConfig,
) -> Result<ResolvedImportContext> {
    let key_felt = match (&import.private_key, &import.private_key_file_path) {
        (Some(key), _) => *key,
        (None, Some(path)) => get_private_key_from_file(path)
            .with_context(|| format!("Failed to obtain private key from the file {path}"))?,
        (None, None) => get_private_key_from_input()?,
    };

    let public_key = SigningKey::from_secret_scalar(key_felt)
        .verifying_key()
        .scalar();
    let address = import
        .resolved_address(config)?
        .expect("clap requires --address when --keystore-account is absent");
    let account_type = import
        .account_type
        .expect("clap requires --type when --keystore-account is absent");
    Ok(ResolvedImportContext {
        signer_type: SignerType::PrivateKey {
            private_key: key_felt,
        },
        public_key,
        address,
        account_type,
        salt: import.salt,
        keystore_class_hash: None,
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
