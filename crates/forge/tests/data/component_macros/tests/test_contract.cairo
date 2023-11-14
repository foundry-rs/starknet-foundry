use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;

use snforge_std::{declare, ContractClass, ContractClassTrait, start_prank, CheatTarget};

use component_macros::example::{IMyContractDispatcherTrait, IMyContractDispatcher};


#[test]
fn test_mint() {
    let contract = declare('MyContract');
    let address = contract.deploy(@array!['minter']).unwrap();
    let minter: ContractAddress = 'minter'.try_into().unwrap();

    let dispatcher = IMyContractDispatcher { contract_address: address };
    start_prank(CheatTarget::One(address), minter);
    dispatcher.mint();
}
