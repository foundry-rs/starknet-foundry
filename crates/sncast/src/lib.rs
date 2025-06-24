use crate::helpers::constants::{DEFAULT_STATE_FILE_SUFFIX, WAIT_RETRY_INTERVAL, WAIT_TIMEOUT};
use crate::response::errors::SNCastProviderError;
use anyhow::{Context, Error, Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::ValueEnum;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::UI;
use helpers::braavos::check_braavos_account_compatibility;
use helpers::constants::{KEYSTORE_PASSWORD_ENV_VAR, UDC_ADDRESS};
use rand::RngCore;
use rand::rngs::OsRng;
use response::errors::SNCastStarknetError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
use shared::rpc::create_rpc_client;
use starknet::accounts::{AccountFactory, AccountFactoryError};
use starknet::core::types::{
    BlockId, BlockTag,
    BlockTag::{Latest, Pending},
    ContractClass, ContractErrorData,
    StarknetError::{ClassHashNotFound, ContractNotFound, TransactionHashNotFound},
};
use starknet::core::types::{ContractExecutionError, ExecutionResult};
use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet::core::utils::{UdcUniqueSettings, UdcUniqueness};
use starknet::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    providers::{
        Provider, ProviderError,
        ProviderError::StarknetError,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
    signers::{LocalWallet, SigningKey},
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use thiserror::Error;

pub mod helpers;
pub mod response;
pub mod state;

use conversions::byte_array::ByteArray;

pub type NestedMap<T> = HashMap<String, HashMap<String, T>>;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    #[serde(rename = "open_zeppelin")]
    OpenZeppelin,
    Argent,
    Braavos,
}

impl FromStr for AccountType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open_zeppelin" | "open-zeppelin" | "oz" => Ok(AccountType::OpenZeppelin),
            "argent" => Ok(AccountType::Argent),
            "braavos" => Ok(AccountType::Braavos),
            account_type => Err(anyhow!("Invalid account type = {account_type}")),
        }
    }
}

impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub const MAINNET: Felt =
    Felt::from_hex_unchecked(const_hex::const_encode::<7, true>(b"SN_MAIN").as_str());

pub const SEPOLIA: Felt =
    Felt::from_hex_unchecked(const_hex::const_encode::<10, true>(b"SN_SEPOLIA").as_str());

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Network {
    Mainnet,
    Sepolia,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Sepolia => write!(f, "sepolia"),
        }
    }
}

impl TryFrom<Felt> for Network {
    type Error = anyhow::Error;

    fn try_from(value: Felt) -> std::result::Result<Self, Self::Error> {
        if value == MAINNET {
            Ok(Network::Mainnet)
        } else if value == SEPOLIA {
            Ok(Network::Sepolia)
        } else {
            bail!("Given network is neither Mainnet nor Sepolia")
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccountData {
    pub private_key: Felt,
    pub public_key: Felt,
    pub address: Option<Felt>,
    pub salt: Option<Felt>,
    pub deployed: Option<bool>,
    pub class_hash: Option<Felt>,
    pub legacy: Option<bool>,

    #[serde(default, rename(serialize = "type", deserialize = "type"))]
    pub account_type: Option<AccountType>,
}

#[derive(Clone, Copy)]
pub struct WaitForTx {
    pub wait: bool,
    pub wait_params: ValidatedWaitParams,
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy, PartialEq)]
pub struct ValidatedWaitParams {
    #[serde(default)]
    timeout: u16,

    #[serde(
        default,
        rename(serialize = "retry-interval", deserialize = "retry-interval")
    )]
    retry_interval: u8,
}

impl ValidatedWaitParams {
    #[must_use]
    pub fn new(retry_interval: u8, timeout: u16) -> Self {
        assert!(
            !(retry_interval == 0 || timeout == 0 || u16::from(retry_interval) > timeout),
            "Invalid values for retry_interval and/or timeout!"
        );

        Self {
            timeout,
            retry_interval,
        }
    }

    #[must_use]
    pub fn get_retries(&self) -> u16 {
        self.timeout / u16::from(self.retry_interval)
    }

    #[must_use]
    pub fn remaining_time(&self, steps_done: u16) -> u16 {
        steps_done * u16::from(self.retry_interval)
    }

    #[must_use]
    pub fn get_retry_interval(&self) -> u8 {
        self.retry_interval
    }

    #[must_use]
    pub fn get_timeout(&self) -> u16 {
        self.timeout
    }
}

impl Default for ValidatedWaitParams {
    fn default() -> Self {
        Self::new(WAIT_RETRY_INTERVAL, WAIT_TIMEOUT)
    }
}

pub fn get_provider(url: &str) -> Result<JsonRpcClient<HttpTransport>> {
    raise_if_empty(url, "RPC url")?;
    create_rpc_client(url)
}

pub async fn get_chain_id(provider: &JsonRpcClient<HttpTransport>) -> Result<Felt> {
    provider
        .chain_id()
        .await
        .context("Failed to fetch chain_id")
}

pub fn get_keystore_password(env_var: &str) -> std::io::Result<String> {
    match env::var(env_var) {
        Ok(password) => Ok(password),
        _ => rpassword::prompt_password("Enter password: "),
    }
}

#[must_use]
pub fn chain_id_to_network_name(chain_id: Felt) -> String {
    let decoded = decode_chain_id(chain_id);

    match &decoded[..] {
        "SN_MAIN" => "alpha-mainnet".into(),
        "SN_SEPOLIA" => "alpha-sepolia".into(),
        "SN_INTEGRATION_SEPOLIA" => "alpha-integration-sepolia".into(),
        _ => decoded,
    }
}

#[must_use]
pub fn decode_chain_id(chain_id: Felt) -> String {
    let non_zero_bytes: Vec<u8> = chain_id
        .to_bytes_be()
        .iter()
        .copied()
        .filter(|&byte| byte != 0)
        .collect();

    String::from_utf8(non_zero_bytes).unwrap_or_default()
}

pub async fn get_nonce(
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &str,
    address: Felt,
) -> Result<Felt> {
    provider
        .get_nonce(
            get_block_id(block_id).context("Failed to obtain block id")?,
            address,
        )
        .await
        .context("Failed to get a nonce")
}

pub async fn get_account<'a>(
    account: &str,
    accounts_file: &Utf8PathBuf,
    provider: &'a JsonRpcClient<HttpTransport>,
    keystore: Option<Utf8PathBuf>,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    let chain_id = get_chain_id(provider).await?;
    let account_data = if let Some(keystore) = keystore {
        get_account_data_from_keystore(account, &keystore)?
    } else {
        get_account_data_from_accounts_file(account, chain_id, accounts_file)?
    };

    // Braavos accounts before v1.2.0 are not compatible with starknet >= 0.13.4
    // For more, read https://community.starknet.io/t/starknet-devtools-for-0-13-5/115495#p-2359168-braavos-compatibility-issues-3
    if let Some(class_hash) = account_data.class_hash {
        check_braavos_account_compatibility(class_hash)?;
    }

    let account = build_account(account_data, chain_id, provider).await?;

    Ok(account)
}

pub async fn get_contract_class(
    class_hash: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<ContractClass> {
    let result = provider
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await;

    if let Err(ProviderError::StarknetError(ClassHashNotFound)) = result {
        // Imitate error thrown on chain to achieve particular error message (Issue #2554)
        let artificial_transaction_revert_error = SNCastProviderError::StarknetError(
            SNCastStarknetError::ContractError(ContractErrorData {
                revert_error: ContractExecutionError::Message(format!(
                    "Class with hash {class_hash:#x} is not declared"
                )),
            }),
        );

        return Err(handle_rpc_error(artificial_transaction_revert_error));
    }

    result.map_err(handle_rpc_error)
}

async fn build_account(
    account_data: AccountData,
    chain_id: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(account_data.private_key));

    let address = account_data
        .address
        .context("Failed to get address - make sure the account is deployed")?;
    verify_account_address(address, chain_id, provider).await?;

    let class_hash = account_data.class_hash;

    let account_encoding =
        get_account_encoding(account_data.legacy, class_hash, address, provider).await?;

    let mut account =
        SingleOwnerAccount::new(provider, signer, address, chain_id, account_encoding);

    account.set_block_id(BlockId::Tag(Pending));

    Ok(account)
}

async fn verify_account_address(
    address: Felt,
    chain_id: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<()> {
    match provider.get_nonce(BlockId::Tag(Pending), address).await {
        Ok(_) => Ok(()),
        Err(error) => {
            if let StarknetError(ContractNotFound) = error {
                let decoded_chain_id = decode_chain_id(chain_id);
                Err(anyhow!(
                    "Account with address {address:#x} not found on network {decoded_chain_id}"
                ))
            } else {
                Err(handle_rpc_error(error))
            }
        }
    }
}

pub async fn check_class_hash_exists(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: Felt,
) -> Result<()> {
    match provider
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => match err {
            StarknetError(ClassHashNotFound) => Err(anyhow!(
                "Class with hash {class_hash:#x} is not declared, try using --class-hash with a hash of the declared class"
            )),
            _ => Err(handle_rpc_error(err)),
        },
    }
}

pub fn get_account_data_from_keystore(
    account: &str,
    keystore_path: &Utf8PathBuf,
) -> Result<AccountData> {
    check_keystore_and_account_files_exist(keystore_path, account)?;
    let path_to_account = Utf8PathBuf::from(account);

    let private_key = SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?.as_str(),
    )?
    .secret_scalar();

    let account_info: Value = read_and_parse_json_file(&path_to_account)?;

    let parse_to_felt = |pointer: &str| -> Option<Felt> {
        get_string_value_from_json(&account_info, pointer).and_then(|value| value.parse().ok())
    };

    let address = parse_to_felt("/deployment/address");
    let class_hash = parse_to_felt("/deployment/class_hash");
    let salt = parse_to_felt("/deployment/salt");
    let deployed = get_string_value_from_json(&account_info, "/deployment/status")
        .map(|status| status == "deployed");
    let legacy = account_info
        .pointer("/variant/legacy")
        .and_then(Value::as_bool);
    let account_type = get_string_value_from_json(&account_info, "/variant/type")
        .and_then(|account_type| account_type.parse().ok());

    let public_key = match account_type.context("Failed to get type key")? {
        AccountType::Argent => parse_to_felt("/variant/owner"),
        AccountType::OpenZeppelin => parse_to_felt("/variant/public_key"),
        AccountType::Braavos => get_braavos_account_public_key(&account_info)?,
    }
    .context("Failed to get public key from account JSON file")?;

    Ok(AccountData {
        private_key,
        public_key,
        address,
        salt,
        deployed,
        class_hash,
        legacy,
        account_type,
    })
}
fn get_braavos_account_public_key(account_info: &Value) -> Result<Option<Felt>> {
    get_string_value_from_json(account_info, "/variant/multisig/status")
        .filter(|status| status == "off")
        .context("Braavos accounts cannot be deployed with multisig on")?;

    account_info
        .pointer("/variant/signers")
        .and_then(Value::as_array)
        .filter(|signers| signers.len() == 1)
        .context("Braavos accounts can only be deployed with one seed signer")?;

    Ok(
        get_string_value_from_json(account_info, "/variant/signers/0/public_key")
            .and_then(|value| value.parse().ok()),
    )
}
fn get_string_value_from_json(json: &Value, pointer: &str) -> Option<String> {
    json.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
}
pub fn get_account_data_from_accounts_file(
    name: &str,
    chain_id: Felt,
    path: &Utf8PathBuf,
) -> Result<AccountData> {
    raise_if_empty(name, "Account name")?;
    check_account_file_exists(path)?;

    let accounts: HashMap<String, HashMap<String, AccountData>> = read_and_parse_json_file(path)?;
    let network_name = chain_id_to_network_name(chain_id);

    accounts
        .get(&network_name)
        .and_then(|accounts_map| accounts_map.get(name))
        .cloned()
        .ok_or_else(|| anyhow!("Account = {name} not found under network = {network_name}"))
}

pub fn read_and_parse_json_file<T: DeserializeOwned>(path: &Utf8PathBuf) -> Result<T> {
    let file_content =
        fs::read_to_string(path).with_context(|| format!("Failed to read a file = {path}"))?;
    let deserializer = &mut Deserializer::from_str(&file_content);
    serde_path_to_error::deserialize(deserializer).map_err(|err| {
        let path_to_field = err.path().to_string();
        anyhow!(
            "Failed to parse field `{path_to_field}` in file '{path}': {}",
            err.into_inner()
        )
    })
}

async fn get_account_encoding(
    legacy: Option<bool>,
    class_hash: Option<Felt>,
    address: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<ExecutionEncoding> {
    if let Some(legacy) = legacy {
        Ok(map_encoding(legacy))
    } else {
        let legacy = check_if_legacy_contract(class_hash, address, provider).await?;
        Ok(map_encoding(legacy))
    }
}

pub async fn check_if_legacy_contract(
    class_hash: Option<Felt>,
    address: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<bool> {
    let contract_class = match class_hash {
        Some(class_hash) => provider.get_class(BlockId::Tag(Pending), class_hash).await,
        None => provider.get_class_at(BlockId::Tag(Pending), address).await,
    }
    .map_err(handle_rpc_error)?;

    Ok(is_legacy_contract(&contract_class))
}

pub async fn get_class_hash_by_address(
    provider: &JsonRpcClient<HttpTransport>,
    address: Felt,
) -> Result<Felt> {
    let result = provider
        .get_class_hash_at(BlockId::Tag(Pending), address)
        .await;

    if let Err(ProviderError::StarknetError(ContractNotFound)) = result {
        // Imitate error thrown on chain to achieve particular error message (Issue #2554)
        let artificial_transaction_revert_error = SNCastProviderError::StarknetError(
            SNCastStarknetError::ContractError(ContractErrorData {
                revert_error: ContractExecutionError::Message(format!(
                    "Requested contract address {address:#x} is not deployed",
                )),
            }),
        );

        return Err(handle_rpc_error(artificial_transaction_revert_error));
    }

    result.map_err(handle_rpc_error).with_context(|| {
        format!("Couldn't retrieve class hash of a contract with address {address:#x}")
    })
}

#[must_use]
pub fn is_legacy_contract(contract_class: &ContractClass) -> bool {
    match contract_class {
        ContractClass::Legacy(_) => true,
        ContractClass::Sierra(_) => false,
    }
}

fn map_encoding(legacy: bool) -> ExecutionEncoding {
    if legacy {
        ExecutionEncoding::Legacy
    } else {
        ExecutionEncoding::New
    }
}

pub fn get_block_id(value: &str) -> Result<BlockId> {
    match value {
        "pending" => Ok(BlockId::Tag(Pending)),
        "latest" => Ok(BlockId::Tag(Latest)),
        _ if value.starts_with("0x") => Ok(BlockId::Hash(Felt::from_hex(value)?)),
        _ => match value.parse::<u64>() {
            Ok(value) => Ok(BlockId::Number(value)),
            Err(_) => Err(anyhow::anyhow!(
                "Incorrect value passed for block_id = {value}. Possible values are pending, latest, block hash (hex) and block number (u64)"
            )),
        },
    }
}

#[derive(Debug, CairoSerialize)]
pub struct ErrorData {
    pub data: ByteArray,
}

#[derive(Error, Debug, CairoSerialize)]
pub enum TransactionError {
    #[error("Transaction has been rejected")]
    Rejected,
    #[error("Transaction has been reverted = {}", .0.data)]
    Reverted(ErrorData),
}

#[derive(Error, Debug, CairoSerialize)]
pub enum WaitForTransactionError {
    #[error(transparent)]
    TransactionError(TransactionError),
    #[error("sncast timed out while waiting for transaction to succeed")]
    TimedOut,
    #[error(transparent)]
    ProviderError(#[from] SNCastProviderError),
}

pub async fn wait_for_tx(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: Felt,
    wait_params: ValidatedWaitParams,
    ui: &UI,
) -> Result<String, WaitForTransactionError> {
    ui.println(&format!("Transaction hash: {tx_hash:#x}"));

    let retries = wait_params.get_retries();
    for i in (1..retries).rev() {
        match provider.get_transaction_status(tx_hash).await {
            Ok(starknet::core::types::TransactionStatus::Rejected) => {
                return Err(WaitForTransactionError::TransactionError(
                    TransactionError::Rejected,
                ));
            }
            Ok(
                starknet::core::types::TransactionStatus::AcceptedOnL2(execution_status)
                | starknet::core::types::TransactionStatus::AcceptedOnL1(execution_status),
            ) => {
                return match execution_status {
                    ExecutionResult::Succeeded => Ok("Transaction accepted".to_string()),
                    ExecutionResult::Reverted { reason } => {
                        Err(WaitForTransactionError::TransactionError(
                            TransactionError::Reverted(ErrorData {
                                data: ByteArray::from(reason.as_str()),
                            }),
                        ))
                    }
                };
            }
            Ok(starknet::core::types::TransactionStatus::Received)
            | Err(StarknetError(TransactionHashNotFound)) => {
                let remaining_time = wait_params.remaining_time(i);
                ui.println(&format!(
                    "Waiting for transaction to be accepted ({i} retries / {remaining_time}s left until timeout)"
                ));
            }
            Err(ProviderError::RateLimited) => {
                ui.println(&"Request rate limited while waiting for transaction to be accepted");
                sleep(Duration::from_secs(wait_params.get_retry_interval().into()));
            }
            Err(err) => return Err(WaitForTransactionError::ProviderError(err.into())),
        }

        sleep(Duration::from_secs(wait_params.get_retry_interval().into()));
    }

    Err(WaitForTransactionError::TimedOut)
}

#[must_use]
pub fn handle_rpc_error(error: impl Into<SNCastProviderError>) -> Error {
    let err: SNCastProviderError = error.into();
    err.into()
}

#[must_use]
pub fn handle_account_factory_error<T>(err: AccountFactoryError<T::SignError>) -> anyhow::Error
where
    T: AccountFactory + Sync,
{
    match err {
        AccountFactoryError::Provider(error) => handle_rpc_error(error),
        error => anyhow!(error.to_string()),
    }
}

pub async fn handle_wait_for_tx<T>(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: Felt,
    return_value: T,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<T, WaitForTransactionError> {
    if wait_config.wait {
        return match wait_for_tx(provider, transaction_hash, wait_config.wait_params, ui).await {
            Ok(_) => Ok(return_value),
            Err(error) => Err(error),
        };
    }

    Ok(return_value)
}

pub fn raise_if_empty(value: &str, value_name: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{value_name} not passed nor found in snfoundry.toml")
    }
    Ok(())
}

pub fn check_account_file_exists(accounts_file_path: &Utf8PathBuf) -> Result<()> {
    if !accounts_file_path.exists() {
        bail! {"Accounts file = {} does not exist! If you do not have an account create one with `account create` command \
        or if you're using a custom accounts file, make sure to supply correct path to it with `--accounts-file` argument.", accounts_file_path}
    }
    Ok(())
}

pub fn check_keystore_and_account_files_exist(
    keystore_path: &Utf8PathBuf,
    account: &str,
) -> Result<()> {
    if !keystore_path.exists() {
        bail!("Failed to find keystore file");
    }
    if account.is_empty() {
        bail!("Argument `--account` must be passed and be a path when using `--keystore`");
    }
    let path_to_account = Utf8PathBuf::from(account);
    if !path_to_account.exists() {
        bail!(
            "File containing the account does not exist: When using `--keystore` argument, the `--account` argument should be a path to the starkli JSON account file"
        );
    }
    Ok(())
}

#[must_use]
pub fn extract_or_generate_salt(salt: Option<Felt>) -> Felt {
    salt.unwrap_or(Felt::from(OsRng.next_u64()))
}

#[must_use]
pub fn udc_uniqueness(unique: bool, account_address: Felt) -> UdcUniqueness {
    if unique {
        Unique(UdcUniqueSettings {
            deployer_address: account_address,
            udc_contract_address: UDC_ADDRESS,
        })
    } else {
        NotUnique
    }
}

pub fn apply_optional<T, R, F: FnOnce(T, R) -> T>(initial: T, option: Option<R>, function: F) -> T {
    match option {
        Some(value) => function(initial, value),
        None => initial,
    }
}

#[macro_export]
macro_rules! apply_optional_fields {
    ($initial:expr, $( $option:expr => $setter:expr ),* ) => {
        {
            let mut value = $initial;
            $(
                value = ::sncast::apply_optional(value, $option, $setter);
            )*
            value
        }
    };
}

#[must_use]
pub fn get_default_state_file_name(script_name: &str, chain_id: &str) -> String {
    format!("{script_name}_{chain_id}_{DEFAULT_STATE_FILE_SUFFIX}")
}

#[cfg(test)]
mod tests {
    use crate::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
    use crate::{
        AccountType, chain_id_to_network_name, extract_or_generate_salt,
        get_account_data_from_accounts_file, get_account_data_from_keystore, get_block_id,
        udc_uniqueness,
    };
    use camino::Utf8PathBuf;
    use conversions::string::IntoHexStr;
    use starknet::core::types::{
        BlockId,
        BlockTag::{Latest, Pending},
        Felt,
    };
    use starknet::core::utils::UdcUniqueSettings;
    use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
    use std::env;

    #[test]
    fn test_get_block_id() {
        let pending_block = get_block_id("pending").unwrap();
        let latest_block = get_block_id("latest").unwrap();

        assert_eq!(pending_block, BlockId::Tag(Pending));
        assert_eq!(latest_block, BlockId::Tag(Latest));
    }

    #[test]
    fn test_get_block_id_hex() {
        let block = get_block_id("0x0").unwrap();

        assert_eq!(
            block,
            BlockId::Hash(
                Felt::from_hex(
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn test_get_block_id_num() {
        let block = get_block_id("0").unwrap();

        assert_eq!(block, BlockId::Number(0));
    }

    #[test]
    fn test_get_block_id_invalid() {
        let block = get_block_id("mariusz").unwrap_err();
        assert!(block
            .to_string()
            .contains("Incorrect value passed for block_id = mariusz. Possible values are pending, latest, block hash (hex) and block number (u64)"));
    }

    #[test]
    fn test_generate_salt() {
        let salt = extract_or_generate_salt(None);

        assert!(salt >= Felt::ZERO);
    }

    #[test]
    fn test_extract_salt() {
        let salt = extract_or_generate_salt(Some(Felt::THREE));

        assert_eq!(salt, Felt::THREE);
    }

    #[test]
    fn test_udc_uniqueness_unique() {
        let uniqueness = udc_uniqueness(true, Felt::ONE);

        assert!(matches!(uniqueness, Unique(UdcUniqueSettings { .. })));
    }

    #[test]
    fn test_udc_uniqueness_not_unique() {
        let uniqueness = udc_uniqueness(false, Felt::ONE);

        assert!(matches!(uniqueness, NotUnique));
    }

    #[test]
    fn test_chain_id_to_network_name() {
        let network_name_katana =
            chain_id_to_network_name(Felt::from_bytes_be_slice("KATANA".as_bytes()));
        let network_name_sepolia =
            chain_id_to_network_name(Felt::from_bytes_be_slice("SN_SEPOLIA".as_bytes()));
        assert_eq!(network_name_katana, "KATANA");
        assert_eq!(network_name_sepolia, "alpha-sepolia");
    }

    #[test]
    fn test_get_account_data_from_accounts_file() {
        let account = get_account_data_from_accounts_file(
            "user1",
            Felt::from_bytes_be_slice("SN_SEPOLIA".as_bytes()),
            &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        )
        .unwrap();
        assert_eq!(
            account.private_key.into_hex_string(),
            "0xffd33878eed7767e7c546ce3fc026295"
        );
        assert_eq!(
            account.public_key.into_hex_string(),
            "0x17b62d16ee2b9b5ccd3320e2c0b234dfbdd1d01d09d0aa29ce164827cddf46a"
        );
        assert_eq!(
            account.address.map(IntoHexStr::into_hex_string),
            Some("0xf6ecd22832b7c3713cfa7826ee309ce96a2769833f093795fafa1b8f20c48b".to_string())
        );
        assert_eq!(
            account.salt.map(IntoHexStr::into_hex_string),
            Some("0x14b6b215424909f34f417ddd7cbaca48de2d505d03c92467367d275e847d252".to_string())
        );
        assert_eq!(account.deployed, Some(true));
        assert_eq!(account.class_hash, None);
        assert_eq!(account.legacy, None);
        assert_eq!(account.account_type, Some(AccountType::OpenZeppelin));
    }

    #[test]
    fn test_get_account_data_from_keystore() {
        set_keystore_password_env();
        let account = get_account_data_from_keystore(
            "tests/data/keystore/my_account.json",
            &Utf8PathBuf::from("tests/data/keystore/my_key.json"),
        )
        .unwrap();
        assert_eq!(
            account.private_key.into_hex_string(),
            "0x55ae34c86281fbd19292c7e3bfdfceb4"
        );
        assert_eq!(
            account.public_key.into_hex_string(),
            "0xe2d3d7080bfc665e0060a06e8e95c3db3ff78a1fec4cc81ddc87e49a12e0a"
        );
        assert_eq!(
            account.address.map(IntoHexStr::into_hex_string),
            Some("0xcce3217e4aea0ab738b55446b1b378750edfca617db549fda1ede28435206c".to_string())
        );
        assert_eq!(account.salt, None);
        assert_eq!(account.deployed, Some(true));
        assert_eq!(account.legacy, Some(true));
        assert_eq!(account.account_type, Some(AccountType::OpenZeppelin));
    }

    #[test]
    fn test_get_braavos_account_from_keystore_with_multisig_on() {
        set_keystore_password_env();
        let err = get_account_data_from_keystore(
            "tests/data/keystore/my_account_braavos_invalid_multisig.json",
            &Utf8PathBuf::from("tests/data/keystore/my_key.json"),
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("Braavos accounts cannot be deployed with multisig on")
        );
    }

    #[test]
    fn test_get_braavos_account_from_keystore_multiple_signers() {
        set_keystore_password_env();
        let err = get_account_data_from_keystore(
            "tests/data/keystore/my_account_braavos_multiple_signers.json",
            &Utf8PathBuf::from("tests/data/keystore/my_key.json"),
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("Braavos accounts can only be deployed with one seed signer")
        );
    }

    #[test]
    fn test_get_account_data_wrong_chain_id() {
        let account = get_account_data_from_accounts_file(
            "user1",
            Felt::from_hex("0x435553544f4d5f434841494e5f4944")
                .expect("Failed to convert chain id from hex"),
            &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        );
        let err = account.unwrap_err();
        assert!(
            err.to_string()
                .contains("Account = user1 not found under network = CUSTOM_CHAIN_ID")
        );
    }

    fn set_keystore_password_env() {
        // SAFETY: Tests run in parallel and share the same environment variables.
        // However, we only set this variable once to a fixed value and never modify or unset it.
        // The only potential issue would be if a test explicitly required this variable to be unset,
        // but to the best of our knowledge, no such test exists.
        unsafe {
            env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
        };
    }
}
