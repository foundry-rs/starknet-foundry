use std::collections::HashMap;

use blockifier::{block_context::BlockContext, transaction::objects::AccountTransactionContext, execution::execution_utils::felt_to_stark_felt};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ChainId, ContractAddress, Nonce, ClassHash, PatriciaKey}, block::{BlockNumber, BlockTimestamp}, patricia_key, transaction::{TransactionHash, Fee, TransactionVersion, TransactionSignature, DeclareTransactionV0V1}, hash::{StarkFelt, StarkHash}, stark_felt};

pub const TEST_SEQUENCER_ADDRESS: &str = "0x1000";
pub const TEST_ERC20_CONTRACT_ADDRESS: &str = "0x1001";

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

pub fn get_transaction_context() -> AccountTransactionContext {
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

pub fn build_declare_transaction (nonce: Nonce, class_hash: &str, sender_address: ContractAddress) -> DeclareTransactionV0V1 {    
    DeclareTransactionV0V1 {
        max_fee: Fee(1000000000000000000000000000),
        class_hash: ClassHash(stark_felt!(class_hash)),
        sender_address: sender_address,
        signature: TransactionSignature::default(),
        ..Default::default()
    }
}