use sncast_std::{invoke, InvokeResult};
use starknet::{ContractAddress, Felt252TryIntoContractAddress};
use traits::Into;

fn main() {
    let map_contract_address = 0x123.try_into().expect('Invalid contract address value');
    invoke(map_contract_address, selector!("put"), array![0x10, 0x1], Option::None, Option::None);
}

