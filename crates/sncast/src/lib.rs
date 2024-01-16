use anyhow::{anyhow, bail, Context, Error, Result};
use camino::Utf8PathBuf;
use helpers::constants::{KEYSTORE_PASSWORD_ENV_VAR, UDC_ADDRESS};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet::core::types::{
    BlockId,
    BlockTag::{Latest, Pending},
    FieldElement,
    StarknetError::{
        BlockNotFound, ClassAlreadyDeclared, ClassHashNotFound, CompilationFailed,
        CompiledClassHashMismatch, ContractClassSizeIsTooLarge, ContractError, ContractNotFound,
        DuplicateTx, FailedToReceiveTransaction, InsufficientAccountBalance, InsufficientMaxFee,
        InvalidTransactionIndex, InvalidTransactionNonce, NonAccount, TransactionExecutionError,
        TransactionHashNotFound, UnsupportedContractClassVersion, UnsupportedTxVersion,
        ValidationFailure,
    },
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

use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use url::Url;

pub mod helpers;
pub mod response;

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
    pub timeout: u16,
    pub retry_interval: u8,
}

pub fn get_provider(url: &str) -> Result<JsonRpcClient<HttpTransport>> {
    raise_if_empty(url, "RPC url")?;
    let parsed_url = Url::parse(url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(parsed_url));
    Ok(provider)
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
    let account = if let Some(keystore) = keystore {
        get_account_from_keystore(provider, chain_id, &keystore, account)?
    } else {
        get_account_from_accounts_file(account, accounts_file, provider, chain_id)?
    };
    Ok(account)
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

pub async fn wait_for_tx(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: FieldElement,
    timeout: u16,
    retry_interval: u8,
) -> Result<&str> {
    println!("Transaction hash = {tx_hash:#x}");

    if retry_interval == 0 || timeout == 0 || u16::from(retry_interval) > timeout {
        return Err(anyhow!("Invalid values for retry_interval and/or timeout!"));
    }
    let retries = timeout / u16::from(retry_interval);
    for i in (1..retries).rev() {
        match provider.get_transaction_status(tx_hash).await {
            Ok(starknet::core::types::TransactionStatus::Rejected) => {
                return Err(anyhow!("Transaction has been rejected"));
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
                let remaining_time = i * u16::from(retry_interval);
                println!("Waiting for transaction to be accepted ({i} retries / {remaining_time}s left until timeout)");
            }
            Err(err) => return Err(err.into()),
        };

        sleep(Duration::from_secs(retry_interval.into()));
    }

    Err(anyhow!(
        "Failed to get transaction with hash = {tx_hash:#x}; Transaction rejected, not received or sncast timed out"
    ))
}

async fn get_revert_reason(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: FieldElement,
) -> Result<&str> {
    let receipt = provider.get_transaction_receipt(tx_hash).await?;

    if let starknet::core::types::ExecutionResult::Reverted { reason } = receipt.execution_result()
    {
        Err(anyhow!("Transaction has been reverted = {reason}"))
    } else {
        unreachable!();
    }
}

pub fn handle_rpc_error<T>(error: ProviderError) -> std::result::Result<T, Error> {
    match error {
        StarknetError(FailedToReceiveTransaction) => {
            Err(anyhow!("Node failed to receive transaction"))
        }
        StarknetError(ContractNotFound) => {
            Err(anyhow!("There is no contract at the specified address"))
        }
        StarknetError(BlockNotFound) => Err(anyhow!("Block was not found")),
        StarknetError(TransactionHashNotFound) => Err(anyhow!(
            "Transaction with provided hash was not found (does not exist)"
        )),
        StarknetError(InvalidTransactionIndex) => {
            Err(anyhow!("There is no transaction with such an index"))
        }
        StarknetError(ClassHashNotFound) => Err(anyhow!("Provided class hash does not exist")),
        StarknetError(ContractError(err)) => Err(anyhow!(
            "An error occurred in the called contract = {err:?}"
        )),
        StarknetError(InvalidTransactionNonce) => Err(anyhow!("Invalid transaction nonce")),
        StarknetError(InsufficientMaxFee) => Err(anyhow!(
            "Max fee is smaller than the minimal transaction cost"
        )),
        StarknetError(InsufficientAccountBalance) => Err(anyhow!(
            "Account balance is too small to cover transaction fee"
        )),
        StarknetError(ClassAlreadyDeclared) => Err(anyhow!(
            "Contract with the same class hash is already declared"
        )),
        StarknetError(TransactionExecutionError(err)) => {
            Err(anyhow!("Transaction execution error = {err:?}"))
        }
        StarknetError(ValidationFailure(err)) => {
            Err(anyhow!("Contract failed the validation = {err}"))
        }
        StarknetError(CompilationFailed) => Err(anyhow!("Contract failed to compile in starknet")),
        StarknetError(ContractClassSizeIsTooLarge) => {
            Err(anyhow!("Contract class size is too large"))
        }
        StarknetError(NonAccount) => Err(anyhow!("No account")),
        StarknetError(DuplicateTx) => Err(anyhow!("Transaction already exists")),
        StarknetError(CompiledClassHashMismatch) => Err(anyhow!("Compiled class hash mismatch")),
        StarknetError(UnsupportedTxVersion) => Err(anyhow!("Unsupported transaction version")),
        StarknetError(UnsupportedContractClassVersion) => {
            Err(anyhow!("Unsupported contract class version"))
        }
        _ => Err(anyhow!("Unknown RPC error")),
    }
}

pub async fn handle_wait_for_tx<T>(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
    return_value: T,
    wait_config: WaitForTx,
) -> Result<T> {
    if wait_config.wait {
        return match wait_for_tx(
            provider,
            transaction_hash,
            wait_config.timeout,
            wait_config.retry_interval,
        )
        .await
        {
            Ok(_) => Ok(return_value),
            Err(message) => Err(anyhow!(message)),
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
        bail!("{value_name} not passed nor found in Scarb.toml")
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
    use starknet::core::utils::UdcUniqueSettings;
    use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
    use starknet::{
        core::types::{
            BlockId,
            BlockTag::{Latest, Pending},
            FieldElement,
        },
        providers::{jsonrpc::HttpTransport, JsonRpcClient},
    };
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
