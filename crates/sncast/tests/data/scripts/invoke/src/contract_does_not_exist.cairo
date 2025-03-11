use sncast_std::{
    invoke, InvokeResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings,
};
use starknet::{ContractAddress, Felt252TryIntoContractAddress};
use traits::Into;

fn main() {
    let map_contract_address = 0x123.try_into().expect('Invalid contract address value');
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::None,
        l1_gas_price: Option::None,
        l2_gas: Option::None,
        l2_gas_price: Option::None,
        l1_data_gas: Option::None,
        l2_data_gas_price: Option::None,
    };
    let invoke_result = invoke(
        map_contract_address, selector!("put"), array![0x10, 0x1], fee_settings, Option::None
    )
        .unwrap_err();
    println!("{:?}", invoke_result);
}

