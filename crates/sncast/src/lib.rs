use anyhow::{anyhow, bail, Context, Error, Result};
use camino::Utf8PathBuf;
use helpers::constants::{KEYSTORE_PASSWORD_ENV_VAR, UDC_ADDRESS};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
use starknet::core::types::{
    BlockId, BlockTag,
    BlockTag::{Latest, Pending},
    ContractClass, ContractErrorData, FieldElement,
    StarknetError::{ClassHashNotFound, ContractNotFound, TransactionHashNotFound},
};
use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet::core::utils::{UdcUniqueSettings, UdcUniqueness};
use starknet::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, ProviderError,
        ProviderError::StarknetError,
    },
    signers::{LocalWallet, SigningKey},
};

use crate::helpers::constants::{DEFAULT_STATE_FILE_SUFFIX, WAIT_RETRY_INTERVAL, WAIT_TIMEOUT};
use crate::response::errors::SNCastProviderError;
use conversions::serde::serialize::CairoSerialize;
use serde::de::DeserializeOwned;
use shared::rpc::create_rpc_client;
use starknet::accounts::{AccountFactory, AccountFactoryError};
use std::collections::HashMap;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use thiserror::Error;

pub mod helpers;
pub mod response;
pub mod state;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    #[serde(rename = "open_zeppelin")]
    Oz,
    Argent,
    Braavos,
}

impl FromStr for AccountType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open_zeppelin" | "oz" => Ok(AccountType::Oz),
            "argent" => Ok(AccountType::Argent),
            "braavos" => Ok(AccountType::Braavos),
            account_type => Err(anyhow!("Invalid account type = {account_type}")),
        }
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccountData {
    pub private_key: FieldElement,
    pub public_key: FieldElement,
    pub address: Option<FieldElement>,
    pub salt: Option<FieldElement>,
    pub deployed: Option<bool>,
    pub class_hash: Option<FieldElement>,
    pub legacy: Option<bool>,

    #[serde(default, rename(serialize = "type", deserialize = "type"))]
    pub account_type: Option<AccountType>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NumbersFormat {
    Default,
    Decimal,
    Hex,
}

impl NumbersFormat {
    #[must_use]
    pub fn from_flags(hex_format: bool, dec_format: bool) -> Self {
        assert!(
            !(hex_format && dec_format),
            "Exclusivity should be validated by clap"
        );
        if hex_format {
            NumbersFormat::Hex
        } else if dec_format {
            NumbersFormat::Decimal
        } else {
            NumbersFormat::Default
        }
    }
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

pub async fn get_chain_id(provider: &JsonRpcClient<HttpTransport>) -> Result<FieldElement> {
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
pub fn chain_id_to_network_name(chain_id: FieldElement) -> String {
    let decoded = decode_chain_id(chain_id);

    match &decoded[..] {
        "SN_MAIN" => "alpha-mainnet".into(),
        "SN_SEPOLIA" => "alpha-sepolia".into(),
        "SN_INTEGRATION_SEPOLIA" => "alpha-integration-sepolia".into(),
        _ => decoded,
    }
}

#[must_use]
pub fn decode_chain_id(chain_id: FieldElement) -> String {
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
    address: FieldElement,
) -> Result<FieldElement> {
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

    let account = build_account(account_data, chain_id, provider).await?;

    Ok(account)
}

async fn build_account(
    account_data: AccountData,
    chain_id: FieldElement,
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
    address: FieldElement,
    chain_id: FieldElement,
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
    class_hash: FieldElement,
) -> Result<()> {
    match provider.get_class(BlockId::Tag(BlockTag::Latest), class_hash).await {
        Ok(_) => Ok(()),
        Err(err) => match err {
            StarknetError(ClassHashNotFound) => Err(anyhow!("Class with hash {class_hash:#x} is not declared, try using --class-hash with a hash of the declared class")),
            _ => Err(handle_rpc_error(err))
        }
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

    let parse_to_felt = |pointer: &str| -> Option<FieldElement> {
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

    let public_key = match account_type.clone().context("Failed to get type key")? {
        AccountType::Argent => parse_to_felt("/variant/owner"),
        AccountType::Oz => parse_to_felt("/variant/public_key"),
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
fn get_braavos_account_public_key(account_info: &Value) -> Result<Option<FieldElement>> {
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
    chain_id: FieldElement,
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

fn read_and_parse_json_file<T: DeserializeOwned>(path: &Utf8PathBuf) -> Result<T> {
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
    class_hash: Option<FieldElement>,
    address: FieldElement,
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
    class_hash: Option<FieldElement>,
    address: FieldElement,
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
    address: FieldElement,
) -> Result<Option<FieldElement>> {
    match provider
        .get_class_hash_at(BlockId::Tag(Pending), address)
        .await
    {
        Ok(class_hash) => Ok(Some(class_hash)),
        Err(StarknetError(ContractNotFound)) => Ok(None),
        Err(err) => Err(handle_rpc_error(err)),
    }
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
        _ if value.starts_with("0x") => Ok(BlockId::Hash(FieldElement::from_hex_be(value)?)),
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
    pub data: String,
}

impl ErrorData {
    #[must_use]
    pub fn new(data: String) -> Self {
        ErrorData { data }
    }
}

impl From<ContractErrorData> for ErrorData {
    fn from(value: ContractErrorData) -> Self {
        ErrorData {
            data: value.revert_error,
        }
    }
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
    tx_hash: FieldElement,
    wait_params: ValidatedWaitParams,
) -> Result<&str, WaitForTransactionError> {
    println!("Transaction hash = {tx_hash:#x}");

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
            ) => match execution_status {
                starknet::core::types::TransactionExecutionStatus::Succeeded => {
                    return Ok("Transaction accepted")
                }
                starknet::core::types::TransactionExecutionStatus::Reverted => {
                    return get_revert_reason(provider, tx_hash).await
                }
            },
            Ok(starknet::core::types::TransactionStatus::Received)
            | Err(StarknetError(TransactionHashNotFound)) => {
                let remaining_time = wait_params.remaining_time(i);
                println!("Waiting for transaction to be accepted ({i} retries / {remaining_time}s left until timeout)");
            }
            Err(ProviderError::RateLimited) => {
                println!("Request rate limited while waiting for transaction to be accepted");
                sleep(Duration::from_secs(wait_params.get_retry_interval().into()));
            }
            Err(err) => return Err(WaitForTransactionError::ProviderError(err.into())),
        };

        sleep(Duration::from_secs(wait_params.get_retry_interval().into()));
    }

    Err(WaitForTransactionError::TimedOut)
}

async fn get_revert_reason(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: FieldElement,
) -> Result<&str, WaitForTransactionError> {
    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(SNCastProviderError::from)?;

    if let starknet::core::types::ExecutionResult::Reverted { reason } = receipt.execution_result()
    {
        Err(WaitForTransactionError::TransactionError(
            TransactionError::Reverted(ErrorData {
                data: reason.clone(),
            }),
        ))
    } else {
        unreachable!();
    }
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
    transaction_hash: FieldElement,
    return_value: T,
    wait_config: WaitForTx,
) -> Result<T, WaitForTransactionError> {
    if wait_config.wait {
        return match wait_for_tx(provider, transaction_hash, wait_config.wait_params).await {
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
        bail!("File containing the account does not exist: When using `--keystore` argument, the `--account` argument should be a path to the starkli JSON account file");
    }
    Ok(())
}

#[must_use]
pub fn extract_or_generate_salt(salt: Option<FieldElement>) -> FieldElement {
    salt.unwrap_or(FieldElement::from(OsRng.next_u64()))
}

#[must_use]
pub fn udc_uniqueness(unique: bool, account_address: FieldElement) -> UdcUniqueness {
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

#[must_use]
pub fn get_default_state_file_name(script_name: &str, chain_id: &str) -> String {
    format!("{script_name}_{chain_id}_{DEFAULT_STATE_FILE_SUFFIX}")
}

#[cfg(test)]
mod tests {
    use crate::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
    use crate::{
        chain_id_to_network_name, extract_or_generate_salt, get_account_data_from_accounts_file,
        get_account_data_from_keystore, get_block_id, udc_uniqueness, AccountType,
    };
    use camino::Utf8PathBuf;
    use conversions::string::IntoHexStr;
    use starknet::core::types::{
        BlockId,
        BlockTag::{Latest, Pending},
        FieldElement,
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
                FieldElement::from_hex_be(
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

        assert!(salt >= FieldElement::ZERO);
    }

    #[test]
    fn test_extract_salt() {
        let salt = extract_or_generate_salt(Some(FieldElement::THREE));

        assert_eq!(salt, FieldElement::THREE);
    }

    #[test]
    fn test_udc_uniqueness_unique() {
        let uniqueness = udc_uniqueness(true, FieldElement::ONE);

        assert!(matches!(uniqueness, Unique(UdcUniqueSettings { .. })));
    }

    #[test]
    fn test_udc_uniqueness_not_unique() {
        let uniqueness = udc_uniqueness(false, FieldElement::ONE);

        assert!(matches!(uniqueness, NotUnique));
    }

    #[test]
    fn test_chain_id_to_network_name() {
        let network_name_katana = chain_id_to_network_name(
            FieldElement::from_byte_slice_be("KATANA".as_bytes()).unwrap(),
        );
        let network_name_sepolia = chain_id_to_network_name(
            FieldElement::from_byte_slice_be("SN_SEPOLIA".as_bytes()).unwrap(),
        );
        assert_eq!(network_name_katana, "KATANA");
        assert_eq!(network_name_sepolia, "alpha-sepolia");
    }

    #[test]
    fn test_get_account_data_from_accounts_file() {
        let account = get_account_data_from_accounts_file(
            "user1",
            FieldElement::from_byte_slice_be("SN_SEPOLIA".as_bytes()).unwrap(),
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
        assert_eq!(account.account_type, Some(AccountType::Oz));
    }

    #[test]
    fn test_get_account_data_from_keystore() {
        env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
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
        assert_eq!(account.account_type, Some(AccountType::Oz));
    }

    #[test]
    fn test_get_braavos_account_from_keystore_with_multisig_on() {
        env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
        let err = get_account_data_from_keystore(
            "tests/data/keystore/my_account_braavos_invalid_multisig.json",
            &Utf8PathBuf::from("tests/data/keystore/my_key.json"),
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("Braavos accounts cannot be deployed with multisig on"));
    }

    #[test]
    fn test_get_braavos_account_from_keystore_multiple_signers() {
        env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
        let err = get_account_data_from_keystore(
            "tests/data/keystore/my_account_braavos_multiple_signers.json",
            &Utf8PathBuf::from("tests/data/keystore/my_key.json"),
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("Braavos accounts can only be deployed with one seed signer"));
    }

    #[test]
    fn test_get_account_data_wrong_chain_id() {
        let account = get_account_data_from_accounts_file(
            "user1",
            FieldElement::from_hex_be("0x435553544f4d5f434841494e5f4944")
                .expect("Failed to convert chain id from hex"),
            &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        );
        let err = account.unwrap_err();
        assert!(err
            .to_string()
            .contains("Account = user1 not found under network = CUSTOM_CHAIN_ID"));
    }
}
