use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use core::result::ResultTrait;
use core::option::OptionTrait;
use core::traits::TryInto;
use starknet::ContractAddress;


use snforge_std::{declare, ContractClassTrait, start_cheat_caller_address};

use component_macros::example::{IMyContractDispatcherTrait, IMyContractDispatcher};


#[test]
fn test_mint() {
    let contract = declare("MyContract").unwrap().contract_class();
    let (address, _) = contract.deploy(@array!['minter']).unwrap();
    let minter: ContractAddress = 'minter'.try_into().unwrap();

    let dispatcher = IMyContractDispatcher { contract_address: address };
    start_cheat_caller_address(address, minter);
    dispatcher.mint();
}
