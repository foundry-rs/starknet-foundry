use starknet::info::{get_execution_info, TxInfo};
use result::ResultTrait;
use box::BoxTrait;
use serde::Serde;
use starknet::{ContractAddress, get_block_hash_syscall};
use array::SpanTrait;
use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address};

#[starknet::interface]
trait ICheatTxInfoChecker<TContractState> {
    fn get_tx_hash(ref self: TContractState) -> felt252;
    fn get_nonce(ref self: TContractState) -> felt252;
    fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
    fn get_signature(ref self: TContractState) -> Span<felt252>;
    fn get_version(ref self: TContractState) -> felt252;
    fn get_max_fee(ref self: TContractState) -> u128;
    fn get_chain_id(ref self: TContractState) -> felt252;
}
#[starknet::interface]
trait ICheatBlockNumberChecker<TContractState> {
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait ICheatBlockTimestampChecker<TContractState> {
    fn get_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait ICheatSequencerAddressChecker<TContractState> {
    fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
}

#[test]
fn simple_syscalls() {
    let exec_info = get_execution_info().unbox();
    assert(exec_info.caller_address.into() == 0, 'Incorrect caller address');
    assert(exec_info.contract_address == test_address(), exec_info.contract_address.into());
    // Hash of TEST_CASE_SELECTOR
    assert(
        exec_info
            .entry_point_selector
            .into() == 655947323460646800722791151288222075903983590237721746322261907338444055163,
        'Incorrect entry point selector'
    );

    let block_info = exec_info.block_info.unbox();

    let contract_cheat_block_number = declare("CheatBlockNumberChecker")
        .unwrap()
        .success_contract_class();
    let (contract_address_cheat_block_number, _) = contract_cheat_block_number
        .deploy(@ArrayTrait::new())
        .unwrap();
    let dispatcher_cheat_block_number = ICheatBlockNumberCheckerDispatcher {
        contract_address: contract_address_cheat_block_number
    };

    let contract_cheat_block_timestamp = declare("CheatBlockTimestampChecker")
        .unwrap()
        .success_contract_class();
    let (contract_address_cheat_block_timestamp, _) = contract_cheat_block_timestamp
        .deploy(@ArrayTrait::new())
        .unwrap();
    let dispatcher_cheat_block_timestamp = ICheatBlockTimestampCheckerDispatcher {
        contract_address: contract_address_cheat_block_timestamp
    };

    let contract_cheat_sequencer_address = declare("CheatSequencerAddressChecker")
        .unwrap()
        .success_contract_class();
    let (contract_address_cheat_sequencer_address, _) = contract_cheat_sequencer_address
        .deploy(@ArrayTrait::new())
        .unwrap();
    let dispatcher_cheat_sequencer_address = ICheatSequencerAddressCheckerDispatcher {
        contract_address: contract_address_cheat_sequencer_address
    };

    assert(
        dispatcher_cheat_block_number.get_block_number() == block_info.block_number,
        'Invalid block number'
    );
    assert(
        dispatcher_cheat_block_timestamp.get_block_timestamp() == block_info.block_timestamp,
        'Invalid block timestamp'
    );
    assert(
        dispatcher_cheat_sequencer_address.get_sequencer_address() == block_info.sequencer_address,
        'Invalid sequencer address'
    );

    let contract = declare("CheatTxInfoChecker").unwrap().success_contract_class();
    let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
    let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

    let tx_info = exec_info.tx_info.unbox();
    assert(tx_info.version == dispatcher.get_version(), 'Incorrect version');
    assert(
        tx_info.account_contract_address == dispatcher.get_account_contract_address(),
        'Incorrect acc_address'
    );
    assert(tx_info.max_fee == dispatcher.get_max_fee(), 'Incorrect max fee');
    assert(tx_info.signature == dispatcher.get_signature(), 'Incorrect signature');
    assert(tx_info.transaction_hash == dispatcher.get_tx_hash(), 'Incorrect transaction_hash');
    assert(tx_info.chain_id == dispatcher.get_chain_id(), 'Incorrect chain_id');
    assert(tx_info.nonce == dispatcher.get_nonce(), 'Incorrect nonce');
}
