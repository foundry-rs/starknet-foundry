use std::collections::BTreeMap;
use std::sync::Arc;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::state::{CheatnetBlockInfo, DictStateReader};
use blockifier::block_context::{FeeTokenAddresses, GasPrices};
use blockifier::execution::contract_class::{ContractClassV1, ContractClassV1Inner};
use blockifier::transaction::objects::{CommonAccountFields, CurrentAccountTransactionContext};
use blockifier::{
    abi::constants,
    block_context::BlockContext,
    execution::contract_class::{ContractClass, ContractClassV0},
    transaction::objects::AccountTransactionContext,
};
use cairo_vm::types::program::Program;
use cairo_vm::vm::runners::builtin_runner::{
    BITWISE_BUILTIN_NAME, EC_OP_BUILTIN_NAME, HASH_BUILTIN_NAME, KECCAK_BUILTIN_NAME,
    OUTPUT_BUILTIN_NAME, POSEIDON_BUILTIN_NAME, RANGE_CHECK_BUILTIN_NAME,
    SEGMENT_ARENA_BUILTIN_NAME, SIGNATURE_BUILTIN_NAME,
};
use camino::Utf8PathBuf;
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::transaction::{Resource, ResourceBounds, ResourceBoundsMapping};
use starknet_api::{
    core::{ChainId, ClassHash, ContractAddress, Nonce, PatriciaKey},
    hash::{StarkFelt, StarkHash},
    patricia_key, stark_felt,
    transaction::{
        Calldata, DeclareTransactionV2, InvokeTransactionV1, TransactionHash, TransactionSignature,
        TransactionVersion,
    },
};

pub const TEST_SEQUENCER_ADDRESS: &str = "0x1000";
pub const TEST_ERC20_CONTRACT_ADDRESS: &str = "0x1001";
pub const MAX_FEE: u128 = 1_000_000 * 100_000_000_000; // 1000000 * min_gas_price.
pub const INITIAL_BALANCE: u128 = 10 * MAX_FEE;

// Mocked class hashes, those are not checked anywhere
pub const TEST_CLASS_HASH: &str = "0x110";
pub const TEST_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x111";
pub const TEST_EMPTY_CONTRACT_CLASS_HASH: &str = "0x112";
pub const TEST_FAULTY_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x113";
pub const SECURITY_TEST_CLASS_HASH: &str = "0x114";
pub const TEST_ERC20_CONTRACT_CLASS_HASH: &str = "0x1010";

pub const TEST_CONTRACT_CLASS_HASH: &str = "0x117";
pub const STEP_RESOURCE_COST: f64 = 0.01_f64;
// snforge_std/src/cheatcodes.cairo::test_address
pub const TEST_ADDRESS: &str = "0x01724987234973219347210837402";

// HOW TO FIND:
// 1. https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#general_case
// 2. src/starkware/cairo/lang/instances.py::starknet_with_keccak_instance
#[must_use]
pub fn build_block_context(block_info: CheatnetBlockInfo) -> BlockContext {
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
        invoke_tx_max_n_steps: 1_000_000,
        validate_max_n_steps: 1_000_000,
        max_recursion_depth: 50,
        fee_token_addresses: FeeTokenAddresses {
            strk_fee_token_address: ContractAddress(patricia_key!(TEST_ERC20_CONTRACT_ADDRESS)),
            eth_fee_token_address: ContractAddress(patricia_key!(TEST_ERC20_CONTRACT_ADDRESS)),
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
pub fn build_declare_transaction(
    nonce: Nonce,
    class_hash: ClassHash,
    sender_address: ContractAddress,
) -> DeclareTransactionV2 {
    DeclareTransactionV2 {
        nonce,
        class_hash,
        sender_address,
        ..Default::default()
    }
}

#[must_use]
pub fn build_invoke_transaction(
    calldata: Calldata,
    sender_address: ContractAddress,
) -> InvokeTransactionV1 {
    InvokeTransactionV1 {
        sender_address,
        calldata,
        ..Default::default()
    }
}

fn read_predeployed_contract_file(
    predeployed_contracts: &Utf8PathBuf,
    contract_path: &str,
) -> String {
    let full_contract_path: PathBuf = predeployed_contracts.join(contract_path).into();
    fs::read_to_string(full_contract_path).expect("Failed to read predeployed contracts")
}

fn load_v0_contract_class(
    predeployed_contracts: &Utf8PathBuf,
    contract_path: &str,
) -> ContractClassV0 {
    let raw_contract_class = read_predeployed_contract_file(predeployed_contracts, contract_path);
    ContractClassV0::try_from_json_string(&raw_contract_class).unwrap()
}

fn contract_class_no_entrypoints() -> ContractClass {
    let inner = ContractClassV1Inner {
        program: Program::default(),
        entry_points_by_type: HashMap::from([
            (EntryPointType::External, vec![]),
            (EntryPointType::Constructor, vec![]),
            (EntryPointType::L1Handler, vec![]),
        ]),

        hints: HashMap::new(),
    };
    ContractClass::V1(ContractClassV1(Arc::new(inner)))
}

// Creates a state with predeployed account and erc20 used to send transactions during tests.
// Deployed contracts are cairo 0 contracts
// Account does not include validations
#[must_use]
pub fn build_testing_state(predeployed_contracts: &Utf8PathBuf) -> DictStateReader {
    let erc20_class = load_v0_contract_class(
        predeployed_contracts,
        "erc20_contract_without_some_syscalls_compiled.json",
    );

    let test_erc20_class_hash = ClassHash(stark_felt!(TEST_ERC20_CONTRACT_CLASS_HASH));
    let test_contract_class_hash = ClassHash(stark_felt!(TEST_CONTRACT_CLASS_HASH));

    let class_hash_to_class = HashMap::from([
        // This is dummy put here only to satisfy blockifier
        // this class is not used and the test contract cannot be called
        (test_contract_class_hash, contract_class_no_entrypoints()),
        (test_erc20_class_hash, ContractClass::V0(erc20_class)),
    ]);

    let test_erc20_address = ContractAddress(patricia_key!(TEST_ERC20_CONTRACT_ADDRESS));
    let test_address = ContractAddress(patricia_key!(TEST_ADDRESS));
    let address_to_class_hash = HashMap::from([
        (test_erc20_address, test_erc20_class_hash),
        (test_address, test_contract_class_hash),
    ]);

    DictStateReader {
        address_to_class_hash,
        class_hash_to_class,
        ..Default::default()
    }
}
