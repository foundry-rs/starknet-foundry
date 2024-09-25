use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;

use snforge_std::{declare, ContractClass, ContractClassTrait, start_cheat_caller_address};

use component_macros::example::{IMyContractDispatcherTrait, IMyContractDispatcher};


#[test]
fn test_mint() {
    let contract = declare("MyContract").unwrap().contract_class();
    let (address, _) = contract.deploy(['minter'].span()).unwrap();
    let minter: ContractAddress = 'minter'.try_into().unwrap();

    let dispatcher = IMyContractDispatcher { contract_address: address };
    start_cheat_caller_address(address, minter);
    dispatcher.mint();
}
