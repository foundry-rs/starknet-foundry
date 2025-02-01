use blockifier::blockifier::block::{BlockInfo, GasPrices};
use blockifier::bouncer::BouncerConfig;
use blockifier::context::{BlockContext, ChainInfo, FeeTokenAddresses, TransactionContext};
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::EntryPointExecutionContext;
use blockifier::transaction::objects::{
    CommonAccountFields, CurrentTransactionInfo, TransactionInfo,
};
use blockifier::versioned_constants::VersionedConstants;
use cairo_vm::vm::runners::cairo_runner::RunResources;
use conversions::string::TryFromHexStr;
use serde::{Deserialize, Serialize};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::transaction::Tip;
use starknet_api::transaction::{
    AccountDeploymentData, PaymasterData, Resource, ResourceBounds, ResourceBoundsMapping,
};
use starknet_api::{
    contract_address,
    core::{ChainId, ContractAddress, Nonce, PatriciaKey},
    felt, patricia_key,
    transaction::{TransactionHash, TransactionSignature, TransactionVersion},
};
use starknet_types_core::felt::Felt;
use std::collections::BTreeMap;
use std::num::NonZeroU128;
use std::sync::Arc;

pub const DEFAULT_CHAIN_ID: &str = "SN_SEPOLIA";
pub const DEFAULT_BLOCK_NUMBER: u64 = 2000;
pub const SEQUENCER_ADDRESS: &str = "0x1000";
pub const ERC20_CONTRACT_ADDRESS: &str = "0x1001";

fn default_chain_id() -> ChainId {
    ChainId::from(String::from(DEFAULT_CHAIN_ID))
}

#[must_use]
pub fn build_block_context(block_info: &BlockInfo, chain_id: Option<ChainId>) -> BlockContext {
    BlockContext::new(
        block_info.clone(),
        ChainInfo {
            chain_id: chain_id.unwrap_or_else(default_chain_id),
            fee_token_addresses: FeeTokenAddresses {
                strk_fee_token_address: contract_address!(ERC20_CONTRACT_ADDRESS),
                eth_fee_token_address: contract_address!(ERC20_CONTRACT_ADDRESS),
            },
        },
        VersionedConstants::latest_constants().clone(), // 0.13.1
        BouncerConfig::default(),
    )
}

fn build_tx_info() -> TransactionInfo {
    TransactionInfo::Current(CurrentTransactionInfo {
        common_fields: CommonAccountFields {
            transaction_hash: TransactionHash::default(),
            version: TransactionVersion::THREE,
            signature: TransactionSignature::default(),
            nonce: Nonce(Felt::from(0_u8)),
            sender_address: ContractAddress::default(),
            only_query: false,
        },
        resource_bounds: ResourceBoundsMapping(BTreeMap::from([
            (
                Resource::L1Gas,
                ResourceBounds {
                    max_amount: 0,
                    max_price_per_unit: 1,
                },
            ),
            (
                Resource::L2Gas,
                ResourceBounds {
                    max_amount: 0,
                    max_price_per_unit: 0,
                },
            ),
        ])),
        tip: Tip::default(),
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: PaymasterData::default(),
        account_deployment_data: AccountDeploymentData::default(),
    })
}

#[must_use]
pub fn build_transaction_context(
    block_info: &BlockInfo,
    chain_id: Option<ChainId>,
) -> TransactionContext {
    TransactionContext {
        block_context: build_block_context(block_info, chain_id),
        tx_info: build_tx_info(),
    }
}

#[must_use]
pub fn build_context(
    block_info: &BlockInfo,
    chain_id: Option<ChainId>,
) -> EntryPointExecutionContext {
    let transaction_context = Arc::new(build_transaction_context(block_info, chain_id));

    EntryPointExecutionContext::new(transaction_context, ExecutionMode::Execute, false).unwrap()
}

pub fn set_max_steps(entry_point_ctx: &mut EntryPointExecutionContext, max_n_steps: u32) {
    // override it to omit [`EntryPointExecutionContext::max_steps`] restrictions
    entry_point_ctx.vm_run_resources = RunResources::new(max_n_steps as usize);
}

// We need to be copying those 1:1 for serialization (caching purposes)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SerializableBlockInfo {
    pub block_number: BlockNumber,
    pub block_timestamp: BlockTimestamp,
    pub sequencer_address: ContractAddress,
    pub gas_prices: SerializableGasPrices,
    // A field which indicates if EIP-4844 blobs are used for publishing state diffs to l1
    // This has influence on the cost of publishing the data on l1
    pub use_kzg_da: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableGasPrices {
    eth_l1_gas_price: NonZeroU128,
    strk_l1_gas_price: NonZeroU128,
    eth_l1_data_gas_price: NonZeroU128,
    strk_l1_data_gas_price: NonZeroU128,
}
impl Default for SerializableGasPrices {
    fn default() -> Self {
        Self {
            eth_l1_gas_price: NonZeroU128::try_from(100 * u128::pow(10, 9)).unwrap(),
            strk_l1_gas_price: NonZeroU128::try_from(100 * u128::pow(10, 9)).unwrap(),
            eth_l1_data_gas_price: NonZeroU128::try_from(u128::pow(10, 6)).unwrap(),
            strk_l1_data_gas_price: NonZeroU128::try_from(u128::pow(10, 9)).unwrap(),
        }
    }
}

impl Default for SerializableBlockInfo {
    fn default() -> Self {
        Self {
            block_number: BlockNumber(DEFAULT_BLOCK_NUMBER),
            block_timestamp: BlockTimestamp::default(),
            sequencer_address: TryFromHexStr::try_from_hex_str(SEQUENCER_ADDRESS).unwrap(),
            gas_prices: SerializableGasPrices::default(),
            use_kzg_da: true,
        }
    }
}

impl From<SerializableBlockInfo> for BlockInfo {
    fn from(forge_block_info: SerializableBlockInfo) -> Self {
        Self {
            block_number: forge_block_info.block_number,
            block_timestamp: forge_block_info.block_timestamp,
            sequencer_address: forge_block_info.sequencer_address,
            gas_prices: forge_block_info.gas_prices.into(),
            use_kzg_da: forge_block_info.use_kzg_da,
        }
    }
}

impl From<BlockInfo> for SerializableBlockInfo {
    fn from(block_info: BlockInfo) -> Self {
        Self {
            block_number: block_info.block_number,
            block_timestamp: block_info.block_timestamp,
            sequencer_address: block_info.sequencer_address,
            gas_prices: block_info.gas_prices.into(),
            use_kzg_da: block_info.use_kzg_da,
        }
    }
}
impl From<SerializableGasPrices> for GasPrices {
    fn from(forge_gas_prices: SerializableGasPrices) -> Self {
        Self {
            eth_l1_gas_price: forge_gas_prices.eth_l1_gas_price,
            strk_l1_gas_price: forge_gas_prices.strk_l1_gas_price,
            eth_l1_data_gas_price: forge_gas_prices.eth_l1_data_gas_price,
            strk_l1_data_gas_price: forge_gas_prices.strk_l1_data_gas_price,
        }
    }
}

impl From<GasPrices> for SerializableGasPrices {
    fn from(gas_prices: GasPrices) -> Self {
        Self {
            eth_l1_gas_price: gas_prices.eth_l1_gas_price,
            strk_l1_gas_price: gas_prices.strk_l1_gas_price,
            eth_l1_data_gas_price: gas_prices.eth_l1_data_gas_price,
            strk_l1_data_gas_price: gas_prices.strk_l1_data_gas_price,
        }
    }
}
