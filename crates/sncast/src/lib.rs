use anyhow::{anyhow, bail, Context, Error, Result};
use camino::Utf8PathBuf;
use helpers::constants::{KEYSTORE_PASSWORD_ENV_VAR, UDC_ADDRESS};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::BlockTag::{Latest, Pending};
use starknet::core::types::StarknetError::{
    ClassHashNotFound, ContractNotFound, TransactionHashNotFound,
};
use starknet::core::types::{BlockId, BlockTag, ContractErrorData, FieldElement};
use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet::core::utils::{UdcUniqueSettings, UdcUniqueness};
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::providers::ProviderError::StarknetError;
use starknet::providers::{Provider, ProviderError};
use starknet::signers::{LocalWallet, SigningKey};

use crate::helpers::constants::{WAIT_RETRY_INTERVAL, WAIT_TIMEOUT};
use crate::response::errors::SNCastProviderError;
use cairo_felt::Felt252;
use conversions::felt252::SerializeAsFelt252Vec;
use shared::rpc::create_rpc_client;
use starknet::accounts::{AccountFactoryError, ConnectedAccount};
use starknet::signers::local_wallet::SignError;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use thiserror::Error;

pub mod helpers;
pub mod response;
pub mod state;

#[derive(Deserialize, Serialize, Clone)]
struct Account {
    private_key: String,
    public_key: String,
    address: String,
    salt: Option<String>,
    deployed: Option<bool>,
    class_hash: Option<String>,
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

pub struct WaitForTx {
    pub wait: bool,
    pub wait_params: ValidatedWaitParams,
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy, PartialEq)]
pub struct ValidatedWaitParams {
    timeout: u16,
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

fn get_account_info(name: &str, chain_id: FieldElement, path: &Utf8PathBuf) -> Result<Account> {
    raise_if_empty(name, "Account name")?;
    let file_content =
        fs::read_to_string(path).with_context(|| format!("Failed to read a file = {path}"))?;
    let accounts: HashMap<String, HashMap<String, Account>> =
        serde_json::from_str(&file_content)
            .with_context(|| format!("Failed to parse file = {path} to JSON"))?;
    let network_name = chain_id_to_network_name(chain_id);
    let account = accounts
        .get(&network_name)
        .and_then(|accounts_map| accounts_map.get(name))
        .cloned();

    account.ok_or_else(|| {
        anyhow!(
            "Account = {} not found under network = {}",
            name,
            network_name
        )
    })
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
        "SN_GOERLI" => "alpha-goerli".into(),
        "SN_GOERLI2" => "alpha-goerli2".into(),
        "SN_MAIN" => "alpha-mainnet".into(),
        "SN_SEPOLIA" => "alpha-sepolia".into(),
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
    Ok(provider
        .get_nonce(
            get_block_id(block_id).expect("Failed to obtain block id"),
            address,
        )
        .await
        .expect("Failed to get a nonce"))
}

pub async fn get_account<'a>(
    account: &str,
    accounts_file: &Utf8PathBuf,
    provider: &'a JsonRpcClient<HttpTransport>,
    keystore: Option<Utf8PathBuf>,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    let chain_id = get_chain_id(provider).await?;
    let mut account = if let Some(keystore) = keystore {
        get_account_from_keystore(provider, chain_id, &keystore, account)?
    } else {
        get_account_from_accounts_file(account, accounts_file, provider, chain_id)?
    };

    account.set_block_id(get_block_id("pending")?);
    verify_account_address(account.clone()).await?;

    Ok(account)
}

async fn verify_account_address(account: impl ConnectedAccount + std::marker::Sync) -> Result<()> {
    match account.get_nonce().await {
        Ok(_) => Ok(()),
        Err(error) => {
            if let StarknetError(ContractNotFound) = error {
                Err(anyhow!("Invalid account address"))
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

fn get_account_from_keystore<'a>(
    provider: &'a JsonRpcClient<HttpTransport>,
    chain_id: FieldElement,
    keystore_path: &Utf8PathBuf,
    account: &str,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    if !keystore_path.exists() {
        bail!("Failed to find keystore file");
    }
    if account.is_empty() {
        bail!("Passed empty path for `--account`");
    }
    let path_to_account = Utf8PathBuf::from(account);
    if !path_to_account.exists() {
        bail!("File containing the account does not exist: When using `--keystore` argument, the `--account` argument should be a path to the starkli JSON account file");
    }

    let signer = LocalWallet::from(SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?.as_str(),
    )?);

    let file_content = fs::read_to_string(path_to_account.clone())
        .with_context(|| format!("Failed to read a file = {}", &path_to_account))?;
    let account_info: Value = serde_json::from_str(&file_content)
        .with_context(|| format!("Failed to parse file = {} to JSON", &path_to_account))?;
    let address = FieldElement::from_hex_be(
        account_info
            .get("deployment")
            .and_then(|deployment| deployment.get("address"))
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Failed to get address from account JSON file - make sure the account is deployed"))?
    )?;

    Ok(SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id,
        ExecutionEncoding::Legacy,
    ))
}

fn get_account_from_accounts_file<'a>(
    name: &str,
    accounts_file_path: &Utf8PathBuf,
    provider: &'a JsonRpcClient<HttpTransport>,
    chain_id: FieldElement,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    account_file_exists(accounts_file_path)?;
    let account_info = get_account_info(name, chain_id, accounts_file_path)?;
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(&account_info.private_key).with_context(|| {
            format!(
                "Failed to convert private key = {} to FieldElement",
                &account_info.private_key
            )
        })?,
    ));
    let address = FieldElement::from_hex_be(&account_info.address).with_context(|| {
        format!(
            "Failed to convert account address = {} to FieldElement",
            &account_info.address
        )
    })?;
    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id,
        ExecutionEncoding::Legacy,
    );

    Ok(account)
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

#[derive(Debug)]
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

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction has been rejected")]
    Rejected,
    #[error("Transaction has been reverted = {}", .0.data)]
    Reverted(ErrorData),
}

#[derive(Error, Debug)]
pub enum WaitForTransactionError {
    #[error(transparent)]
    TransactionError(TransactionError),
    #[error("sncast timed out while waiting for transaction to succeed")]
    TimedOut,
    #[error(transparent)]
    ProviderError(#[from] SNCastProviderError),
}

impl SerializeAsFelt252Vec for WaitForTransactionError {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        match self {
            WaitForTransactionError::TransactionError(err) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(err.serialize_as_felt252_vec());
                res
            }
            WaitForTransactionError::TimedOut => vec![Felt252::from(1)],
            WaitForTransactionError::ProviderError(err) => {
                let mut res = vec![Felt252::from(2)];
                res.extend(err.serialize_as_felt252_vec());
                res
            }
        }
    }
}

impl SerializeAsFelt252Vec for TransactionError {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        match self {
            TransactionError::Rejected => vec![Felt252::from(0)],
            TransactionError::Reverted(err) => {
                let mut res = vec![Felt252::from(1)];
                res.extend(err.data.as_str().serialize_as_felt252_vec());
                res
            }
        }
    }
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
pub fn handle_account_factory_error(err: AccountFactoryError<SignError>) -> anyhow::Error {
    match err {
        AccountFactoryError::Provider(error) => handle_rpc_error(error),
        error => anyhow!(error),
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

pub fn parse_number(number_as_str: &str) -> Result<FieldElement> {
    let contract_address = match &number_as_str[..2] {
        "0x" => FieldElement::from_hex_be(number_as_str)?,
        _ => FieldElement::from_dec_str(number_as_str)?,
    };
    Ok(contract_address)
}

pub fn raise_if_empty(value: &str, value_name: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{value_name} not passed nor found in snfoundry.toml")
    }
    Ok(())
}

pub fn account_file_exists(accounts_file_path: &Utf8PathBuf) -> Result<()> {
    if !accounts_file_path.exists() {
        bail! {"Accounts file = {} does not exist! If you do not have an account create one with `account create` command \
        or if you're using a custom accounts file, make sure to supply correct path to it with `--accounts-file` argument.", accounts_file_path}
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
            udc_contract_address: parse_number(UDC_ADDRESS).expect("Failed to parse UDC address"),
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

#[cfg(test)]
mod tests {
    use crate::{
        chain_id_to_network_name, extract_or_generate_salt, get_account_from_accounts_file,
        get_block_id, udc_uniqueness,
    };
    use camino::Utf8PathBuf;
    use starknet::core::types::BlockTag::{Latest, Pending};
    use starknet::core::types::{BlockId, FieldElement};
    use starknet::core::utils::UdcUniqueSettings;
    use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
    use starknet::providers::jsonrpc::HttpTransport;
    use starknet::providers::JsonRpcClient;
    use url::Url;

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
        let network_name_goerli = chain_id_to_network_name(
            FieldElement::from_byte_slice_be("SN_GOERLI".as_bytes()).unwrap(),
        );
        let network_name_katana = chain_id_to_network_name(
            FieldElement::from_byte_slice_be("KATANA".as_bytes()).unwrap(),
        );
        let network_name_sepolia = chain_id_to_network_name(
            FieldElement::from_byte_slice_be("SN_SEPOLIA".as_bytes()).unwrap(),
        );
        assert_eq!(network_name_goerli, "alpha-goerli");
        assert_eq!(network_name_katana, "KATANA");
        assert_eq!(network_name_sepolia, "alpha-sepolia");
    }

    #[test]
    fn test_get_account_wrong_chain_id() {
        let mock_url = Url::parse("https://example.net").unwrap();
        let mock_provider = JsonRpcClient::new(HttpTransport::new(mock_url));
        let account = get_account_from_accounts_file(
            "user1",
            &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
            &mock_provider,
            FieldElement::from_hex_be("0x435553544f4d5f434841494e5f4944")
                .expect("Failed to convert chain id from hex"),
        );
        let err = account.unwrap_err();
        assert!(err
            .to_string()
            .contains("Account = user1 not found under network = CUSTOM_CHAIN_ID"));
    }
}
