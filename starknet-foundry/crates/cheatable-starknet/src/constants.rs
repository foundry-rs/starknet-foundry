use std::collections::HashMap;

use blockifier::{block_context::BlockContext, transaction::objects::AccountTransactionContext, execution::execution_utils::felt_to_stark_felt, state::cached_state::CachedState};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ChainId, ContractAddress, Nonce, ClassHash, PatriciaKey}, block::{BlockNumber, BlockTimestamp}, patricia_key, transaction::{TransactionHash, Fee, TransactionVersion, TransactionSignature, DeclareTransactionV0V1, Calldata, InvokeTransactionV1}, hash::{StarkFelt, StarkHash}};

use crate::state::DictStateReader;

pub const TEST_SEQUENCER_ADDRESS: &str = "0x1000";
pub const TEST_ERC20_CONTRACT_ADDRESS: &str = "0x1001";
pub const TEST_ACCOUNT_CONTRACT_ADDRESS: &str = "0x101";
pub const MAX_FEE: u128 = 1000000 * 100000000000; // 1000000 * min_gas_price.

pub fn create_block_context_for_testing() -> BlockContext {
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
        max_fee: Fee(1000000000000000000000000000),
        version: TransactionVersion(StarkFelt::from(1_u8)),
        signature: TransactionSignature::default(),
        nonce: Nonce(felt_to_stark_felt(nonce)),
        sender_address: ContractAddress::default(),
    }
}

pub fn build_declare_transaction (nonce: Nonce, class_hash: ClassHash, sender_address: ContractAddress) -> DeclareTransactionV0V1 {    
    DeclareTransactionV0V1 {
        nonce: nonce,
        max_fee: Fee(1000000000000000000000000000),
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


// pub fn create_state_with_trivial_validation_account() -> CachedState<DictStateReader> {
//     let account_balance = BALANCE;
//     create_account_tx_test_state(
//         ContractClassV0::from_file(ACCOUNT_CONTRACT_CAIRO0_PATH).into(),
//         TEST_ACCOUNT_CONTRACT_CLASS_HASH,
//         TEST_ACCOUNT_CONTRACT_ADDRESS,
//         test_erc20_account_balance_key(),
//         account_balance,
//     )
// }

pub fn build_testing_state() -> CachedState<DictStateReader> {
    CachedState::new(DictStateReader {
        ..Default::default()
    })
}
