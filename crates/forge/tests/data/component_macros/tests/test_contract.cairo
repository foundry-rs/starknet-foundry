use component_macros::example::{IMyContractDispatcher, IMyContractDispatcherTrait};
use core::option::OptionTrait;
use core::result::ResultTrait;
use core::traits::TryInto;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare, start_cheat_caller_address};
use starknet::ContractAddress;


#[test]
fn test_mint() {
    let contract = declare("MyContract").unwrap().contract_class();
    let (address, _) = contract.deploy(@array!['minter']).unwrap();
    let minter: ContractAddress = 'minter'.try_into().unwrap();

    let dispatcher = IMyContractDispatcher { contract_address: address };
    start_cheat_caller_address(address, minter);
    dispatcher.mint();
}
