use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use blockifier::block_context::{FeeTokenAddresses, GasPrices};

use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::EntryPointExecutionContext;
use blockifier::transaction::objects::{CommonAccountFields, CurrentAccountTransactionContext};
use blockifier::{
    abi::constants, block_context::BlockContext, transaction::objects::AccountTransactionContext,
};

use cairo_vm::vm::runners::builtin_runner::{
    BITWISE_BUILTIN_NAME, EC_OP_BUILTIN_NAME, HASH_BUILTIN_NAME, KECCAK_BUILTIN_NAME,
    OUTPUT_BUILTIN_NAME, POSEIDON_BUILTIN_NAME, RANGE_CHECK_BUILTIN_NAME,
    SEGMENT_ARENA_BUILTIN_NAME, SIGNATURE_BUILTIN_NAME,
};
use cairo_vm::vm::runners::cairo_runner::RunResources;
use serde::{Deserialize, Serialize};
use starknet_api::contract_address;
use starknet_api::data_availability::DataAvailabilityMode;

use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::transaction::{Resource, ResourceBounds, ResourceBoundsMapping};
use starknet_api::{
    core::{ChainId, ContractAddress, Nonce, PatriciaKey},
    hash::{StarkFelt, StarkHash},
    patricia_key,
    transaction::{TransactionHash, TransactionSignature, TransactionVersion},
};

pub const SEQUENCER_ADDRESS: &str = "0x1000";
pub const ERC20_CONTRACT_ADDRESS: &str = "0x1001";
pub const STEP_RESOURCE_COST: f64 = 0.005_f64;
pub const DEFAULT_MAX_N_STEPS: u32 = 3_000_000;

// HOW TO FIND:
// 1. https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#calculation_of_computation_costs
#[must_use]
fn build_block_context(block_info: BlockInfo) -> BlockContext {
    // blockifier::test_utils::create_for_account_testing
    let vm_resource_fee_cost = Arc::new(HashMap::from([
        (constants::N_STEPS_RESOURCE.to_string(), STEP_RESOURCE_COST),
        (HASH_BUILTIN_NAME.to_string(), 32_f64 * STEP_RESOURCE_COST),
        (
            RANGE_CHECK_BUILTIN_NAME.to_string(),
            16_f64 * STEP_RESOURCE_COST,
        ),
        (
            SIGNATURE_BUILTIN_NAME.to_string(),
            2048_f64 * STEP_RESOURCE_COST,
        ), // ECDSA
        (
            BITWISE_BUILTIN_NAME.to_string(),
            64_f64 * STEP_RESOURCE_COST,
        ),
        (
            POSEIDON_BUILTIN_NAME.to_string(),
            32_f64 * STEP_RESOURCE_COST,
        ),
        (OUTPUT_BUILTIN_NAME.to_string(), 0_f64 * STEP_RESOURCE_COST),
        (
            EC_OP_BUILTIN_NAME.to_string(),
            1024_f64 * STEP_RESOURCE_COST,
        ),
        (
            KECCAK_BUILTIN_NAME.to_string(),
            2048_f64 * STEP_RESOURCE_COST, // 2**11
        ),
        (
            SEGMENT_ARENA_BUILTIN_NAME.to_string(),
            0_f64 * STEP_RESOURCE_COST,
        ),
    ]));

    BlockContext {
        chain_id: ChainId("SN_GOERLI".to_string()),
        block_number: block_info.block_number,
        block_timestamp: block_info.timestamp,
        sequencer_address: block_info.sequencer_address,
        vm_resource_fee_cost,
        invoke_tx_max_n_steps: DEFAULT_MAX_N_STEPS,
        validate_max_n_steps: 1_000_000,
        max_recursion_depth: 50,
        fee_token_addresses: FeeTokenAddresses {
            strk_fee_token_address: contract_address!(ERC20_CONTRACT_ADDRESS),
            eth_fee_token_address: contract_address!(ERC20_CONTRACT_ADDRESS),
        },
        gas_prices: GasPrices {
            eth_l1_gas_price: 100 * u128::pow(10, 9),
            strk_l1_gas_price: 100 * u128::pow(10, 9),
        },
    }
}

#[must_use]
pub fn build_transaction_context() -> AccountTransactionContext {
    AccountTransactionContext::Current(CurrentAccountTransactionContext {
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
pub fn build_context(block_info: BlockInfo) -> EntryPointExecutionContext {
    let block_context = build_block_context(block_info);
    let account_context = build_transaction_context();

    EntryPointExecutionContext::new(
        &block_context,
        &account_context,
        ExecutionMode::Execute,
        false,
    )
    .unwrap()
}

pub fn set_max_steps(entry_point_ctx: &mut EntryPointExecutionContext, max_n_steps: u32) {
    entry_point_ctx.block_context.invoke_tx_max_n_steps = max_n_steps;

    // override it to omit [`EntryPointExecutionContext::max_steps`] restrictions
    entry_point_ctx.vm_run_resources = RunResources::new(max_n_steps as usize);
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct BlockInfo {
    pub block_number: BlockNumber,
    pub timestamp: BlockTimestamp,
    pub sequencer_address: ContractAddress,
}

impl Default for BlockInfo {
    fn default() -> Self {
        Self {
            block_number: BlockNumber(2000),
            timestamp: BlockTimestamp::default(),
            sequencer_address: ContractAddress(patricia_key!(SEQUENCER_ADDRESS)),
        }
    }
}
