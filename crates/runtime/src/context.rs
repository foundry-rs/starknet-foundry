use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use blockifier::block_context::{FeeTokenAddresses, GasPrices};

use blockifier::state::cached_state::ContractStorageKey;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{StateReader, StateResult};
use blockifier::transaction::objects::{CommonAccountFields, CurrentAccountTransactionContext};
use blockifier::{
    abi::constants, block_context::BlockContext, execution::contract_class::ContractClass,
    transaction::objects::AccountTransactionContext,
};

use cairo_vm::vm::runners::builtin_runner::{
    BITWISE_BUILTIN_NAME, EC_OP_BUILTIN_NAME, HASH_BUILTIN_NAME, KECCAK_BUILTIN_NAME,
    OUTPUT_BUILTIN_NAME, POSEIDON_BUILTIN_NAME, RANGE_CHECK_BUILTIN_NAME,
    SEGMENT_ARENA_BUILTIN_NAME, SIGNATURE_BUILTIN_NAME,
};
use serde::{Deserialize, Serialize};
use starknet_api::data_availability::DataAvailabilityMode;

use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::core::CompiledClassHash;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Resource, ResourceBounds, ResourceBoundsMapping};
use starknet_api::{
    core::{ChainId, ClassHash, ContractAddress, Nonce, PatriciaKey},
    hash::{StarkFelt, StarkHash},
    patricia_key,
    transaction::{TransactionHash, TransactionSignature, TransactionVersion},
};

pub const SEQUENCER_ADDRESS: &str = "0x1000";
pub const ERC20_CONTRACT_ADDRESS: &str = "0x1001";
pub const STEP_RESOURCE_COST: f64 = 0.01_f64;

// HOW TO FIND:
// 1. https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#general_case
// 2. src/starkware/cairo/lang/instances.py::starknet_with_keccak_instance
#[must_use]
pub fn build_default_block_context() -> BlockContext {
    build_block_context(BlockInfo::default())
}

#[must_use]
pub fn build_block_context(block_info: BlockInfo) -> BlockContext {
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
        invoke_tx_max_n_steps: 3_000_000,
        validate_max_n_steps: 1_000_000,
        max_recursion_depth: 50,
        fee_token_addresses: FeeTokenAddresses {
            strk_fee_token_address: ContractAddress(patricia_key!(ERC20_CONTRACT_ADDRESS)),
            eth_fee_token_address: ContractAddress(patricia_key!(ERC20_CONTRACT_ADDRESS)),
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

/// A simple implementation of `StateReader` using `HashMap`s as storage.
#[derive(Debug, Default)]
pub struct DictStateReader {
    pub storage_view: HashMap<ContractStorageKey, StarkFelt>,
    pub address_to_nonce: HashMap<ContractAddress, Nonce>,
    pub address_to_class_hash: HashMap<ContractAddress, ClassHash>,
    pub class_hash_to_class: HashMap<ClassHash, ContractClass>,
    pub class_hash_to_compiled_class_hash: HashMap<ClassHash, CompiledClassHash>,
}

impl StateReader for DictStateReader {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        let contract_storage_key = (contract_address, key);
        self.storage_view
            .get(&contract_storage_key)
            .copied()
            .ok_or(StateError::StateReadError(format!(
                "Unable to get storage at address: {contract_address:?} and key: {key:?} form DictStateReader"
            )))
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.address_to_nonce
            .get(&contract_address)
            .copied()
            .ok_or(StateError::StateReadError(format!(
                "Unable to get nonce at {contract_address:?} from DictStateReader"
            )))
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.address_to_class_hash
            .get(&contract_address)
            .copied()
            .ok_or(StateError::UnavailableContractAddress(contract_address))
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClass> {
        let contract_class = self.class_hash_to_class.get(class_hash).cloned();
        match contract_class {
            Some(contract_class) => Ok(contract_class),
            _ => Err(StateError::UndeclaredClassHash(*class_hash)),
        }
    }

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        let compiled_class_hash = self
            .class_hash_to_compiled_class_hash
            .get(&class_hash)
            .copied()
            .unwrap_or_default();
        Ok(compiled_class_hash)
    }
}
