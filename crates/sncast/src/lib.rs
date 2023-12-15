use anyhow::{anyhow, bail, Context, Error, Result};
use camino::Utf8PathBuf;
use helpers::constants::{KEYSTORE_PASSWORD_ENV_VAR, UDC_ADDRESS};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet::core::utils::{UdcUniqueSettings, UdcUniqueness};
use starknet::providers::jsonrpc::RpcError::{Code, Unknown};
use starknet::providers::ProviderError::Other;
use starknet::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, ProviderError,
    },
    signers::{LocalWallet, SigningKey},
};
use starknet::{
    core::types::{
        BlockId,
        BlockTag::{Latest, Pending},
        ExecutionResult, FieldElement, StarknetError,
    },
    providers::{MaybeUnknownErrorCode, StarknetErrorWithMessage},
};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use url::Url;

pub mod helpers;

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
pub enum ValueFormat {
    // Addresses as hex, fees as int
    Default,
    // everything as int
    Int,
    // everything as hex
    Hex,
}

impl ValueFormat {
    #[must_use]
    pub fn format_u64(&self, input: u64) -> String {
        match self {
            ValueFormat::Default | ValueFormat::Int => format!("{input}"),
            ValueFormat::Hex => format!("{input:#x}"),
        }
    }

    #[must_use]
    pub fn format_str(&self, input: &str) -> String {
        if let Ok(field) = FieldElement::from_str(input) {
            return match self {
                ValueFormat::Int => format!("{field:#}"),
                ValueFormat::Hex | ValueFormat::Default => format!("{field:#x}"),
            };
        }

        input.to_string()
    }

    #[must_use]
    pub fn format_json_value(&self, value: &Value) -> Option<String> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_u64()
                    .unwrap_or_else(|| panic!("failed to convert {n} to u64"));
                Some(self.format_u64(n))
            }
            Value::String(s) => Some(self.format_str(s)),
            Value::Array(arr) => {
                let arr_as_string = arr
                    .iter()
                    .filter_map(|item| self.format_json_value(item))
                    .collect::<Vec<String>>()
                    .join(", ");
                Some(format!("[{arr_as_string}]"))
            }
            _ => None,
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
    provider.chain_id().await.context("Couldn't fetch chain_id")
}

fn get_account_info(name: &str, chain_id: FieldElement, path: &Utf8PathBuf) -> Result<Account> {
    raise_if_empty(name, "Account name")?;
    let file_content =
        fs::read_to_string(path).with_context(|| format!("Cannot read a file {path}"))?;
    let accounts: HashMap<String, HashMap<String, Account>> =
        serde_json::from_str(&file_content)
            .with_context(|| format!("Cannot parse file {path} to JSON"))?;
    let network_name = chain_id_to_network_name(chain_id);
    let account = accounts
        .get(&network_name)
        .and_then(|accounts_map| accounts_map.get(name))
        .cloned();

    account.ok_or_else(|| anyhow!("Account {} not found under network {}", name, network_name))
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
            get_block_id(block_id).expect("Could not obtain block id"),
            address,
        )
        .await
        .expect("Could not get nonce"))
}

pub async fn get_account<'a>(
    account: &str,
    accounts_file: &Utf8PathBuf,
    provider: &'a JsonRpcClient<HttpTransport>,
    keystore: &Utf8PathBuf,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
    let chain_id = get_chain_id(provider).await?;
    let account = if keystore == &Utf8PathBuf::default() {
        get_account_from_accounts_file(account, accounts_file, provider, chain_id)?
    } else {
        get_account_from_keystore(provider, chain_id, keystore, account)?
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
        bail!("keystore file does not exist");
    }
    if account.is_empty() {
        bail!("Path passed with --account cannot be empty!");
    }
    let path_to_account = Utf8PathBuf::from(account);
    if !path_to_account.exists() {
        bail!("account file does not exist; when using --keystore, --account argument should be a path to the starkli JSON account file");
    }

    let signer = LocalWallet::from(SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?.as_str(),
    )?);

    let file_content = fs::read_to_string(path_to_account.clone())
        .with_context(|| format!("Cannot read a file {}", &path_to_account))?;
    let account_info: serde_json::Value = serde_json::from_str(&file_content)
        .with_context(|| format!("Cannot parse file {} to JSON", &path_to_account))?;
    let address = FieldElement::from_hex_be(
        account_info
            .get("deployment")
            .and_then(|deployment| deployment.get("address"))
            .and_then(serde_json::Value::as_str)
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
                "Failed to convert private key: {} to FieldElement",
                &account_info.private_key
            )
        })?,
    ));
    let address = FieldElement::from_hex_be(&account_info.address).with_context(|| {
        format!(
            "Failed to convert account address: {} to FieldElement",
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
                "No such block id {}! Possible values are pending, latest, block hash (hex) and block number (u64).",
                value
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
    println!("Transaction hash: {tx_hash:#x}");

    if retry_interval == 0 || timeout == 0 || u16::from(retry_interval) > timeout {
        return Err(anyhow!("Invalid values for retry_interval and/or timeout!"));
    }
    let retries = timeout / u16::from(retry_interval);
    for i in (1..retries).rev() {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(receipt) => match receipt.execution_result() {
                ExecutionResult::Succeeded => {
                    return Ok("Transaction accepted");
                }
                ExecutionResult::Reverted { reason } => {
                    return Err(anyhow!("Transaction has been reverted: {}", reason));
                }
            },
            Err(ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::TransactionHashNotFound),
                message: _,
            })) => {
                let remaining_time = i * u16::from(retry_interval);
                println!("Waiting for transaction to be accepted ({i} retries / {remaining_time}s left until timeout)");
            }
            Err(err) => return Err(err.into()),
        };

        sleep(Duration::from_secs(retry_interval.into()));
    }

    Err(anyhow!(
        "Could not get transaction with hash: {tx_hash:#x}. Transaction rejected, not received or sncast timed out."
    ))
}

#[must_use]
pub fn get_rpc_error_message(error: &StarknetError) -> &'static str {
    match error {
        StarknetError::FailedToReceiveTransaction => "Node failed to receive transaction",
        StarknetError::ContractNotFound => "There is no contract at the specified address",
        StarknetError::BlockNotFound => "Block was not found",
        StarknetError::TransactionHashNotFound => {
            "Transaction with provided hash was not found (does not exist)"
        }
        StarknetError::InvalidTransactionIndex => "There is no transaction with such an index",
        StarknetError::ClassHashNotFound => "Provided class hash does not exist",
        StarknetError::ContractError => "An error occurred in the called contract",
        StarknetError::InvalidTransactionNonce => "Invalid transaction nonce",
        StarknetError::InsufficientMaxFee => "Max fee is smaller then the minimal transaction cost",
        StarknetError::InsufficientAccountBalance => {
            "Account balance is too small to cover transaction fee"
        }
        StarknetError::ClassAlreadyDeclared => {
            "Contract with the same class hash is already declared"
        }
        _ => "Unknown RPC error",
    }
}

pub fn handle_rpc_error<T>(error: ProviderError) -> std::result::Result<T, Error> {
    match error {
        Other(x) => {
            if let Some(err) = x
                .deref()
                .as_any()
                .downcast_ref::<starknet::providers::jsonrpc::RpcError>()
            {
                match err {
                    Code(error) => Err(anyhow!(get_rpc_error_message(error))),
                    Unknown(error) => Err(anyhow!(error.message.clone())),
                }
            } else {
                Err(anyhow!("Unknown RPC error"))
            }
        }
        ProviderError::StarknetError(error) => Err(anyhow!(error.message)),
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

pub fn print_formatted(output: Vec<(&str, String)>, json: bool, error: bool) -> Result<()> {
    if json {
        let json_output: HashMap<&str, String> = output.into_iter().collect();
        let json_value: Value = serde_json::to_value(json_output)?;

        write_to_output(serde_json::to_string(&json_value)?, error);
    } else {
        for (key, value) in &output {
            write_to_output(format!("{key}: {value}"), error);
        }
    }

    Ok(())
}

pub fn print_command_result<T: Serialize>(
    command: &str,
    result: &mut Result<T>,
    value_format: ValueFormat,
    json: bool,
) -> Result<()> {
    let mut output = vec![("command", command.to_string())];
    let json_value: Value;

    let mut error = false;
    match result {
        Ok(result) => {
            json_value = serde_json::to_value(result)
                .map_err(|_| anyhow!("Failed to convert command result to serde_json::Value"))?;

            output.extend(
                json_value
                    .as_object()
                    .expect("Invalid JSON value")
                    .iter()
                    .filter_map(|(k, v)| value_format.format_json_value(v).map(|v| (k.as_str(), v)))
                    .collect::<Vec<(&str, String)>>(),
            );
        }
        Err(message) => {
            output.push(("error", format!("{message:#}")));
            error = true;
        }
    };
    print_formatted(output, json, error)
}

fn write_to_output<T: std::fmt::Display>(value: T, error: bool) {
    if error {
        eprintln!("{value}");
    } else {
        println!("{value}");
    }
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
        bail! {"Accounts file {} does not exist! If you do not have an account create one with `account create` command \
        or if you're using a custom accounts file, make sure to supply correct path to it with --accounts-file argument.", accounts_file_path}
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

#[cfg(test)]
mod tests {
    use crate::{
        chain_id_to_network_name, extract_or_generate_salt, get_account_from_accounts_file,
        get_block_id, udc_uniqueness, ValueFormat,
    };
    use camino::Utf8PathBuf;
    use serde::Serialize;
    use serde_json::json;
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
    use test_case::test_case;
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
            .contains("No such block id mariusz! Possible values are pending, latest, block hash (hex) and block number (u64)."));
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
        assert_eq!(network_name_goerli, "alpha-goerli");
        assert_eq!(network_name_katana, "KATANA");
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
                .expect("Should convert from hex"),
        );
        let err = account.unwrap_err();
        assert!(err
            .to_string()
            .contains("Account user1 not found under network CUSTOM_CHAIN_ID"));
    }

    #[test_case(
        "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
        "when value is hex"
    )]
    #[test_case("kapusta", "kapusta" ; "when value is string")]
    #[test_case(132_435, "132435" ; "when value is uint")]
    #[test_case([132_435, 121], "[132435, 121]" ; "when value is array of ints")]
    #[test_case(
        ["0x49d3657224e46f48e99674bd3fcc84644ddd6b96f7c221b1562b82f9e004dc7", "0x49d36333d4e46f48e99674bd3fcc84333ddd6b96f7c741b1562b82f9e004dc7"],
        "[0x49d3657224e46f48e99674bd3fcc84644ddd6b96f7c221b1562b82f9e004dc7, 0x49d36333d4e46f48e99674bd3fcc84333ddd6b96f7c741b1562b82f9e004dc7]";
        "when value is array of contract addresses"
    )]
    fn test_format_json_value_not_none<T: Serialize>(value: T, expected: &str) {
        let value_format = ValueFormat::Default;
        let json_value = serde_json::to_value(value).unwrap();

        let actual = value_format.format_json_value(&json_value).unwrap();

        assert_eq!(actual, expected);
    }

    #[test_case(true ; "when value is bool")]
    #[test_case(json!({ "an": "object" }) ; "when value is an object")]
    #[test_case(json!(null) ; "when value is null")]
    fn test_format_json_value_is_none<T: Serialize>(value: T) {
        let value_format = ValueFormat::Default;
        let json_value = serde_json::to_value(value).unwrap();

        let actual = value_format.format_json_value(&json_value);

        assert_eq!(actual, None);
    }
}
