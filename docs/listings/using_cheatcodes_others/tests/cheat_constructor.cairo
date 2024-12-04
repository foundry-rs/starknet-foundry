use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_number,
    start_cheat_block_timestamp
};

use using_cheatcodes_others::{ICheatcodeCheckerDispatcher, ICheatcodeCheckerDispatcherTrait};

#[test]
fn call_and_invoke() {
    let contract = declare("CheatcodeChecker").unwrap().contract_class();

    // Precalculate the address to obtain the contract address before the constructor call (deploy)
    // itself
    let contract_address = contract.precalculate_address(@array![]);

    // Change the block number and timestamp before the call to contract.deploy
    start_cheat_block_number(contract_address, 0x420_u64);
    start_cheat_block_timestamp(contract_address, 0x2137_u64);

    // Deploy as normally
    contract.deploy(@array![]).unwrap();

    // Construct a dispatcher with the precalculated address
    let dispatcher = ICheatcodeCheckerDispatcher { contract_address };

    let block_number = dispatcher.get_block_number_at_construction();
    let block_timestamp = dispatcher.get_block_timestamp_at_construction();

    assert_eq!(block_number, 0x420_u64);
    assert_eq!(block_timestamp, 0x2137_u64);
}
