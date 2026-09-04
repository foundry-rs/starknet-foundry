use std::num::{NonZeroU8, NonZeroU16};

use crate::helpers::account::{check_account_exists, is_devnet_account};
use crate::helpers::configuration::CastConfig;
use crate::helpers::constants::{DEFAULT_ACCOUNTS_FILE, WAIT_RETRY_INTERVAL, WAIT_TIMEOUT};
use crate::helpers::rpc::RpcArgs;
use crate::response::errors::SNCastProviderError;
use anyhow::{Context, Error, Result, anyhow, bail, ensure};
use clap::ValueEnum;
use configuration::Override;
use helpers::constants::UDC_ADDRESS;
use rand::RngCore;
use rand::rngs::OsRng;
use response::errors::SNCastStarknetError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::rpc::create_rpc_client;
use starknet_rust::accounts::{AccountFactory, AccountFactoryError};
use starknet_rust::core::types::{
    BlockId, BlockTag,
    BlockTag::{Latest, PreConfirmed},
    ContractClass, ContractErrorData,
    StarknetError::{ClassHashNotFound, ContractNotFound, TransactionHashNotFound},
};
use starknet_rust::core::types::{ContractExecutionError, ExecutionResult};
use starknet_rust::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet_rust::core::utils::{UdcUniqueSettings, UdcUniqueness};
use starknet_rust::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    providers::{
        Provider, ProviderError,
        ProviderError::StarknetError,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::fmt::Display;
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use url::Url;

pub mod accounts;
pub mod compat;
pub mod helpers;
pub mod response;
pub mod signers;

use crate::response::ui::UI;
pub use accounts::AccountType;
use conversions::byte_array::ByteArray;
use foundry_ui::components::warning::WarningMessage;
pub use helpers::signer::SignerSource;
use signers::RuntimeSigner;
pub type RuntimeAccount<'a> = SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, RuntimeSigner>;

pub const MAINNET: Felt =
    Felt::from_hex_unchecked(const_hex::const_encode::<7, true>(b"SN_MAIN").as_str());

pub const SEPOLIA: Felt =
    Felt::from_hex_unchecked(const_hex::const_encode::<10, true>(b"SN_SEPOLIA").as_str());

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Sepolia,
    Devnet,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Sepolia => write!(f, "sepolia"),
            Network::Devnet => write!(f, "devnet"),
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

#[derive(Clone, Copy)]
pub struct WaitForTx {
    pub wait: bool,
    pub wait_params: ValidatedWaitParams,
    pub show_ui_outputs: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct PartialWaitParams {
    pub timeout: Option<NonZeroU16>,
    #[serde(
        default,
        rename(serialize = "retry-interval", deserialize = "retry-interval")
    )]
    pub retry_interval: Option<NonZeroU8>,

    /// Additional data not captured by deserializer.
    #[doc(hidden)]
    #[serde(flatten, default, skip_serializing)]
    pub unknown_fields: HashMap<String, Value>,
}

impl Override for PartialWaitParams {
    fn override_with(&self, other: PartialWaitParams) -> PartialWaitParams {
        PartialWaitParams {
            timeout: other.timeout.or(self.timeout),
            retry_interval: other.retry_interval.or(self.retry_interval),
            unknown_fields: HashMap::default(),
        }
    }
}

impl TryFrom<PartialWaitParams> for ValidatedWaitParams {
    type Error = anyhow::Error;

    fn try_from(p: PartialWaitParams) -> anyhow::Result<Self> {
        let d = ValidatedWaitParams::default();
        Self::new(
            p.retry_interval.unwrap_or(d.retry_interval),
            p.timeout.unwrap_or(d.timeout),
        )
    }
}

impl PartialWaitParams {
    /// Rejects invalid params, allows not fully specified params
    pub fn validate(&self) -> anyhow::Result<()> {
        if let (Some(retry_interval), Some(timeout)) = (self.retry_interval, self.timeout) {
            ValidatedWaitParams::new(retry_interval, timeout)?;
        }
        Ok(())
    }
}

/// Effective wait params used at runtime.
/// Note: Built from [`PartialWaitParams`], not (de)serialized.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct ValidatedWaitParams {
    timeout: NonZeroU16,
    retry_interval: NonZeroU8,
}

impl ValidatedWaitParams {
    pub fn new(retry_interval: NonZeroU8, timeout: NonZeroU16) -> Result<Self> {
        let res = Self {
            timeout,
            retry_interval,
        };
        res.validate()?;
        Ok(res)
    }

    #[must_use]
    pub fn get_retries(&self) -> u16 {
        self.timeout.get() / u16::from(self.retry_interval.get())
    }

    /// Remaining time (in seconds) until timeout, given the number of retries still left to run.
    #[must_use]
    pub fn remaining_time(&self, retries_left: u16) -> u16 {
        retries_left * u16::from(self.retry_interval.get())
    }

    #[must_use]
    pub fn get_retry_interval(&self) -> NonZeroU8 {
        self.retry_interval
    }

    #[must_use]
    pub fn get_timeout(&self) -> NonZeroU16 {
        self.timeout
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            u16::from(self.retry_interval.get()) <= self.timeout.get(),
            "retry_interval cannot be greater than timeout"
        );
        Ok(())
    }
}

impl Default for ValidatedWaitParams {
    fn default() -> Self {
        Self::new(WAIT_RETRY_INTERVAL, WAIT_TIMEOUT).unwrap()
    }
}

pub fn get_provider(url: &Url) -> Result<JsonRpcClient<HttpTransport>> {
    create_rpc_client(url)
}

pub async fn get_chain_id(provider: &JsonRpcClient<HttpTransport>) -> Result<Felt> {
    provider
        .chain_id()
        .await
        .context("Failed to fetch chain_id")
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
    config: &'a CastConfig,
    provider: &'a JsonRpcClient<HttpTransport>,
    rpc_args: &RpcArgs,
    ui: &UI,
) -> Result<RuntimeAccount<'a>> {
    let chain_id = get_chain_id(provider).await?;

    let network_name = chain_id_to_network_name(chain_id);
    let account = &config.account;
    if account.is_empty() {
        bail!("Account name not passed nor found in snfoundry.toml");
    }
    let is_devnet_account = is_devnet_account(account);

    if is_devnet_account
        && let Some(network) = rpc_args.network
        && (network == Network::Mainnet || network == Network::Sepolia)
    {
        bail!(format!(
            "Devnet accounts cannot be used with `--network {network}`"
        ));
    }

    let repository = accounts::AccountRepository::new(config.accounts_file.clone())?;
    let accounts_file = repository.path();
    // Devnet accounts don't require an accounts file.
    // When the default accounts file is used, we don't enforce its existence.
    // When accounts file is set explicitly, it is still required to exist then.
    let uses_default_accounts_file =
        accounts_file.as_str() == shellexpand::tilde(DEFAULT_ACCOUNTS_FILE).as_ref();
    let accounts_file_required = !(is_devnet_account && uses_default_accounts_file);
    let exists_in_accounts_file =
        check_account_exists(account, &network_name, &repository, accounts_file_required)?;

    let (selector, devnet_url) = match (is_devnet_account, exists_in_accounts_file) {
        (true, true) => {
            ui.print_warning(WarningMessage::new(format!(
                "Using account {account} from accounts file {accounts_file}. \
                To use an inbuilt devnet account, please rename your existing account or use an account with a different number."
            )));
            ui.print_blank_line();
            (accounts::AccountSelector::named(account.clone())?, None)
        }
        (true, false) => {
            let url = rpc_args
                .get_url(config)
                .await
                .context("Failed to get url")?;
            (accounts::AccountSelector::devnet(account)?, Some(url))
        }
        _ => (accounts::AccountSelector::named(account.clone())?, None),
    };

    accounts::AccountService::new(repository)
        .connected_account(&selector, provider, devnet_url.as_ref(), ui)
        .await
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

pub(crate) async fn verify_account_address(
    address: Felt,
    chain_id: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<()> {
    match provider
        .get_nonce(BlockId::Tag(PreConfirmed), address)
        .await
    {
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

pub fn get_account_record_from_repository(
    name: &str,
    chain_id: Felt,
    repository: &accounts::AccountRepository,
) -> Result<accounts::AccountRecord> {
    if name.is_empty() {
        bail!("Account name not passed nor found in snfoundry.toml")
    }
    check_account_file_exists(repository)?;
    let network_name = chain_id_to_network_name(chain_id);
    repository
        .find(&network_name, name)
        .map_err(|error| match error {
            accounts::AccountsError::AccountNotFound { .. } => {
                anyhow!("Account = {name} not found under network = {network_name}")
            }
            error => anyhow!(error),
        })
}

pub fn check_account_file_exists(repository: &accounts::AccountRepository) -> Result<()> {
    if !repository.exists() {
        let path = repository.path();
        bail!(
            "Accounts file = {path} does not exist! If you do not have an account create one with `account create` command \
             or if you're using a custom accounts file, make sure to supply correct path to it with `--accounts-file` argument."
        )
    }
    Ok(())
}

pub(crate) async fn get_account_encoding(
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
        Some(class_hash) => {
            provider
                .get_class(BlockId::Tag(PreConfirmed), class_hash)
                .await
        }
        None => {
            provider
                .get_class_at(BlockId::Tag(PreConfirmed), address)
                .await
        }
    }
    .map_err(handle_rpc_error)?;

    Ok(is_legacy_contract(&contract_class))
}

pub async fn get_class_hash_by_address(
    provider: &JsonRpcClient<HttpTransport>,
    address: Felt,
) -> Result<Felt> {
    let result = provider
        .get_class_hash_at(BlockId::Tag(PreConfirmed), address)
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
        "pre_confirmed" => Ok(BlockId::Tag(PreConfirmed)),
        "latest" => Ok(BlockId::Tag(Latest)),
        _ if value.starts_with("0x") => Ok(BlockId::Hash(Felt::from_hex(value)?)),
        _ => match value.parse::<u64>() {
            Ok(value) => Ok(BlockId::Number(value)),
            Err(_) => Err(anyhow::anyhow!(
                "Incorrect value passed for block_id = {value}. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)"
            )),
        },
    }
}

#[derive(Debug)]
pub struct ErrorData {
    pub data: ByteArray,
}

#[derive(Error, Debug)]
pub enum TransactionError {
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

pub async fn wait_for_tx(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: Felt,
    wait_params: ValidatedWaitParams,
    ui: Option<&UI>,
) -> Result<String, WaitForTransactionError> {
    ui.inspect(|ui| ui.print_notification(format!("Transaction hash: {tx_hash:#x}")));

    let retries = wait_params.get_retries();
    for i in (1..retries).rev() {
        match provider.get_transaction_status(tx_hash).await {
            Ok(starknet_rust::core::types::TransactionStatus::PreConfirmed(
                ExecutionResult::Reverted { reason },
            )) => {
                return Err(WaitForTransactionError::TransactionError(
                    TransactionError::Reverted(ErrorData {
                        data: ByteArray::from(reason.as_str()),
                    }),
                ));
            }
            Ok(
                starknet_rust::core::types::TransactionStatus::AcceptedOnL2(execution_status)
                | starknet_rust::core::types::TransactionStatus::AcceptedOnL1(execution_status),
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
            Ok(starknet_rust::core::types::TransactionStatus::PreConfirmed(
                ExecutionResult::Succeeded,
            )) => {
                ui.inspect(|ui| {
                    let remaining_time = wait_params.remaining_time(i);
                    ui.print_notification("Transaction status: PRE_CONFIRMED".to_string());
                    ui.print_notification(format!(
                        "Waiting for transaction to be accepted ({i} retries / {remaining_time}s left until timeout)"
                    ));
                });
            }
            Ok(
                starknet_rust::core::types::TransactionStatus::Received
                | starknet_rust::core::types::TransactionStatus::Candidate,
            )
            | Err(StarknetError(TransactionHashNotFound)) => {
                ui.inspect(|ui| {
                        let remaining_time = wait_params.remaining_time(i);
                        ui.print_notification(format!(
                            "Waiting for transaction to be accepted ({i} retries / {remaining_time}s left until timeout)"
                        ));
                    });
            }
            Err(ProviderError::RateLimited) => {
                ui.inspect(|ui| {
                    ui.print_notification(
                        "Request rate limited while waiting for transaction to be accepted"
                            .to_string(),
                    );
                });
                sleep(Duration::from_secs(
                    wait_params.get_retry_interval().get().into(),
                ));
            }
            Err(err) => return Err(WaitForTransactionError::ProviderError(err.into())),
        }

        sleep(Duration::from_secs(
            wait_params.get_retry_interval().get().into(),
        ));
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
        return match wait_for_tx(
            provider,
            transaction_hash,
            wait_config.wait_params,
            wait_config.show_ui_outputs.then_some(ui),
        )
        .await
        {
            Ok(_) => Ok(return_value),
            Err(error) => Err(error),
        };
    }

    Ok(return_value)
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
                value = $crate::apply_optional(value, $option, $setter);
            )*
            value
        }
    };
}

#[cfg(test)]
mod tests {
    use std::num::{NonZeroU8, NonZeroU16};

    use crate::{
        PartialWaitParams, chain_id_to_network_name, extract_or_generate_salt, get_block_id,
        udc_uniqueness,
    };
    use configuration::{Override, override_optional};
    use starknet_rust::core::types::{
        BlockId,
        BlockTag::{Latest, PreConfirmed},
        Felt,
    };
    use starknet_rust::core::utils::UdcUniqueSettings;
    use starknet_rust::core::utils::UdcUniqueness::{NotUnique, Unique};

    #[test]
    fn test_get_block_id() {
        let pending_block = get_block_id("pre_confirmed").unwrap();
        let latest_block = get_block_id("latest").unwrap();

        assert_eq!(pending_block, BlockId::Tag(PreConfirmed));
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
            .contains("Incorrect value passed for block_id = mariusz. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)"));
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
    fn test_partial_wait_params_override_with() {
        let base = PartialWaitParams {
            timeout: NonZeroU16::new(200),
            retry_interval: NonZeroU8::new(5),
            ..Default::default()
        };
        let other = PartialWaitParams {
            timeout: NonZeroU16::new(300),
            retry_interval: None,
            ..Default::default()
        };
        let overridden = base.override_with(other);
        assert_eq!(overridden.timeout, NonZeroU16::new(300));
        assert_eq!(overridden.retry_interval, NonZeroU8::new(5));

        let base2 = PartialWaitParams {
            timeout: None,
            retry_interval: NonZeroU8::new(5),
            ..Default::default()
        };
        let other2 = PartialWaitParams {
            timeout: NonZeroU16::new(200),
            retry_interval: None,
            ..Default::default()
        };
        let overridden2 = base2.override_with(other2);
        assert_eq!(overridden2.timeout, NonZeroU16::new(200));
        assert_eq!(overridden2.retry_interval, NonZeroU8::new(5));
    }

    #[test]
    fn test_wait_params_override_optional() {
        let base = PartialWaitParams {
            timeout: NonZeroU16::new(200),
            retry_interval: NonZeroU8::new(5),
            ..Default::default()
        };
        let other = PartialWaitParams {
            timeout: None,
            retry_interval: NonZeroU8::new(5),
            ..Default::default()
        };
        assert_eq!(
            override_optional(Some(base.clone()), Some(other.clone())),
            Some(PartialWaitParams {
                timeout: NonZeroU16::new(200),
                retry_interval: NonZeroU8::new(5),
                ..Default::default()
            })
        );
        assert_eq!(
            override_optional::<PartialWaitParams>(None, Some(other.clone())),
            Some(other)
        );
        assert_eq!(override_optional(Some(base.clone()), None), Some(base));
        assert_eq!(override_optional::<PartialWaitParams>(None, None), None);
    }
}
