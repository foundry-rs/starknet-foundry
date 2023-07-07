use std::{collections::HashMap, path::PathBuf, fs};

use blockifier::{block_context::BlockContext, transaction::objects::AccountTransactionContext, execution::{execution_utils::felt_to_stark_felt, contract_class::{ContractClassV1, ContractClassV0, ContractClass}}, state::cached_state::CachedState, abi::abi_utils::get_storage_var_address};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ChainId, ContractAddress, Nonce, ClassHash, PatriciaKey}, block::{BlockNumber, BlockTimestamp}, patricia_key, transaction::{TransactionHash, Fee, TransactionVersion, TransactionSignature, Calldata, InvokeTransactionV1, DeclareTransactionV2}, hash::{StarkFelt, StarkHash}, state::StorageKey, stark_felt};

use crate::state::DictStateReader;

pub const TEST_SEQUENCER_ADDRESS: &str = "0x1000";
pub const TEST_ERC20_CONTRACT_ADDRESS: &str = "0x1001";
pub const TEST_ACCOUNT_CONTRACT_ADDRESS: &str = "0x101";
pub const MAX_FEE: u128 = 1000000 * 100000000000; // 1000000 * min_gas_price.
pub const INITIAL_BALANCE: &str = "0x100000000000000000000000000000000000";

// Mocked class hashes, those are not checked anywhere
pub const TEST_CLASS_HASH: &str = "0x110";
pub const TEST_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x111";
pub const TEST_EMPTY_CONTRACT_CLASS_HASH: &str = "0x112";
pub const TEST_FAULTY_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x113";
pub const SECURITY_TEST_CLASS_HASH: &str = "0x114";
pub const TEST_ERC20_CONTRACT_CLASS_HASH: &str = "0x1010";


pub fn build_block_context() -> BlockContext {
    BlockContext {
        chain_id: ChainId("SN_GOERLI".to_string()),
        block_number: BlockNumber(2000),
        block_timestamp: BlockTimestamp::default(),
        sequencer_address: ContractAddress(patricia_key!(TEST_SEQUENCER_ADDRESS)),
        fee_token_address: ContractAddress(patricia_key!(TEST_ERC20_CONTRACT_ADDRESS)),
        vm_resource_fee_cost: HashMap::default(),
        gas_price: 100 * u128::pow(10, 9),
        invoke_tx_max_n_steps: 1_000_000,
        validate_max_n_steps: 1_000_000,
        max_recursion_depth: 50,
    }
    
}

pub fn build_transaction_context() -> AccountTransactionContext {
    let nonce = &Felt252::from(0);
    AccountTransactionContext {
        transaction_hash: TransactionHash::default(),
        max_fee: Fee::default(),
        version: TransactionVersion(StarkFelt::from(2_u8)),
        signature: TransactionSignature::default(),
        nonce: Nonce(felt_to_stark_felt(nonce)),
        sender_address: ContractAddress::default(),
    }
}

pub fn build_declare_transaction (nonce: Nonce, class_hash: ClassHash, sender_address: ContractAddress) -> DeclareTransactionV2 {    
    DeclareTransactionV2 {
        nonce: nonce,
        max_fee: Fee::default(),
        class_hash: class_hash,
        sender_address: sender_address,
        signature: TransactionSignature::default(),
        ..Default::default()
    }
}

pub fn build_invoke_transaction (calldata: Calldata, sender_address: ContractAddress) -> InvokeTransactionV1 {    
    InvokeTransactionV1 {
        max_fee: Fee(1000000000000000000000000000),
        sender_address: sender_address,
        calldata,
        signature: TransactionSignature::default(),
        ..Default::default()
    }
}


fn get_raw_contract_class(contract_path: &str) -> String {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), contract_path].iter().collect();
    fs::read_to_string(path).unwrap()
}

fn load_erc20_from_file(contract_path: &str) -> ContractClassV0 {
    let raw_contract_class = get_raw_contract_class(contract_path);
    ContractClassV0::try_from_json_string(&raw_contract_class).unwrap()
}

fn load_account_from_file(contract_path: &str) -> ContractClassV1 {
    let raw_contract_class = get_raw_contract_class(contract_path);
    ContractClassV1::try_from_json_string(&raw_contract_class).unwrap()
}

fn erc20_account_balance_key() -> StorageKey {
    get_storage_var_address("ERC20_balances", &[stark_felt!(TEST_ACCOUNT_CONTRACT_ADDRESS)])
        .unwrap()
}


// Creates a state with predeployed account and erc20 used to send transactions during tests
pub fn build_testing_state(
) -> CachedState<DictStateReader> {
    let account_class = load_account_from_file("./predeployed-contracts/account_contract.casm.json");
    let erc20_class = load_erc20_from_file("./predeployed-contracts/erc20_contract_without_some_syscalls_compiled.json");
    let block_context = build_block_context();
    let test_account_class_hash = ClassHash(stark_felt!(TEST_ACCOUNT_CONTRACT_CLASS_HASH));
    let test_erc20_class_hash = ClassHash(stark_felt!(TEST_ERC20_CONTRACT_CLASS_HASH));

    let class_hash_to_class = HashMap::from([
        (test_account_class_hash, ContractClass::V1(account_class)),
        (test_erc20_class_hash, ContractClass::V0(erc20_class)),
    ]);

    // A random address that is unlikely to equal the result of the calculation of a contract
    // address.
    let test_account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
    let test_erc20_address = block_context.fee_token_address;
    let address_to_class_hash = HashMap::from([
        (test_account_address, test_account_class_hash),
        (test_erc20_address, test_erc20_class_hash),
    ]);
    let minter_var_address = get_storage_var_address("permitted_minter", &[])
        .expect("Failed to get permitted_minter storage address.");
    let storage_view = HashMap::from([
        ((test_erc20_address, erc20_account_balance_key()), stark_felt!(INITIAL_BALANCE)),
        // Give the account mint permission.
        ((test_erc20_address, minter_var_address), *test_account_address.0.key()),
    ]);
    CachedState::new(DictStateReader {
        address_to_class_hash,
        class_hash_to_class,
        storage_view,
        ..Default::default()
    })
}
