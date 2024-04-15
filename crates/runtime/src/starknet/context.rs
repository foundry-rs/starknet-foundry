use std::collections::BTreeMap;
use std::num::NonZeroU128;
use std::sync::Arc;

use blockifier::blockifier::block::{BlockInfo, GasPrices};
use blockifier::context::{BlockContext, ChainInfo, FeeTokenAddresses, TransactionContext};
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::EntryPointExecutionContext;
use blockifier::transaction::objects::{
    CommonAccountFields, CurrentTransactionInfo, TransactionInfo,
};
use blockifier::versioned_constants::VersionedConstants;

use cairo_vm::vm::runners::cairo_runner::RunResources;
use serde::{Deserialize, Serialize};
use starknet_api::data_availability::DataAvailabilityMode;

use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::transaction::{Resource, ResourceBounds, ResourceBoundsMapping};
use starknet_api::{
    contract_address,
    core::{ChainId, ContractAddress, Nonce, PatriciaKey},
    hash::{StarkFelt, StarkHash},
    patricia_key,
    transaction::{TransactionHash, TransactionSignature, TransactionVersion},
};

pub const DEFAULT_BLOCK_NUMBER: u64 = 2000;
pub const SEQUENCER_ADDRESS: &str = "0x1000";
pub const ERC20_CONTRACT_ADDRESS: &str = "0x1001";

#[must_use]
pub fn build_block_context(block_info: &BlockInfo) -> BlockContext {
    BlockContext::new_unchecked(
        block_info,
        &ChainInfo {
            chain_id: ChainId("SN_SEPOLIA".to_string()),
            fee_token_addresses: FeeTokenAddresses {
                strk_fee_token_address: contract_address!(ERC20_CONTRACT_ADDRESS),
                eth_fee_token_address: contract_address!(ERC20_CONTRACT_ADDRESS),
            },
        },
        VersionedConstants::latest_constants(), // 0.13.1
    )
}

fn build_tx_info() -> TransactionInfo {
    TransactionInfo::Current(CurrentTransactionInfo {
        common_fields: CommonAccountFields {
            transaction_hash: TransactionHash::default(),
            version: TransactionVersion::THREE,
            signature: TransactionSignature::default(),
            nonce: Nonce(StarkFelt::from(0_u8)),
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
        tip: Default::default(),
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
    })
}

#[must_use]
pub fn build_transaction_context(block_info: &BlockInfo) -> TransactionContext {
    TransactionContext {
        block_context: build_block_context(block_info),
        tx_info: build_tx_info(),
    }
}

#[must_use]
pub fn build_context(block_info: &BlockInfo) -> EntryPointExecutionContext {
    let transaction_context = Arc::new(build_transaction_context(block_info));

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
            sequencer_address: contract_address!(SEQUENCER_ADDRESS),
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
