use sncast_std::{invoke, InvokeResult, ScriptCommandError, ProviderError, StarknetError};
use starknet::{ContractAddress, Felt252TryIntoContractAddress};
use traits::Into;

fn main() {
    let map_contract_address = 0x07537a17e169c96cf2b0392508b3a66cbc50c9a811a8a7896529004c5e93fdf6
        .try_into()
        .expect('Invalid contract address value');

    let invoke_result = invoke(
        map_contract_address, selector!("mariusz"), array![0x10, 0x1], Option::None, Option::None
    )
        .unwrap_err();
    println!("{:?}", invoke_result);
}

