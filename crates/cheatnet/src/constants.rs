use std::collections::HashMap;
use std::sync::Arc;

use blockifier::execution::contract_class::{ContractClassV1, ContractClassV1Inner};

use blockifier::execution::contract_class::ContractClass;
use cairo_vm::types::program::Program;

use starknet_api::deprecated_contract_class::EntryPointType;

use runtime::context::{DictStateReader, ERC20_CONTRACT_ADDRESS};
use starknet_api::{
    core::{ClassHash, ContractAddress, Nonce, PatriciaKey},
    hash::{StarkFelt, StarkHash},
    patricia_key, stark_felt,
    transaction::{Calldata, DeclareTransactionV2, InvokeTransactionV1},
};

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
// snforge_std/src/cheatcodes.cairo::test_address
pub const TEST_ADDRESS: &str = "0x01724987234973219347210837402";

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
pub fn build_testing_state() -> DictStateReader {
    let test_erc20_class_hash = ClassHash(stark_felt!(TEST_ERC20_CONTRACT_CLASS_HASH));
    let test_contract_class_hash = ClassHash(stark_felt!(TEST_CONTRACT_CLASS_HASH));

    let class_hash_to_class = HashMap::from([
        // This is dummy put here only to satisfy blockifier
        // this class is not used and the test contract cannot be called
        (test_contract_class_hash, contract_class_no_entrypoints()),
    ]);

    let test_erc20_address = ContractAddress(patricia_key!(ERC20_CONTRACT_ADDRESS));
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
