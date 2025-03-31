use sncast_std::{
    invoke, InvokeResult, ScriptCommandError, ProviderError, StarknetError, FeeSettingsTrait,
};
use starknet::{ContractAddress};

fn main() {
    let map_contract_address = 0x123.try_into().expect('Invalid contract address value');
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let invoke_result = invoke(
        map_contract_address, selector!("put"), array![0x10, 0x1], fee_settings, Option::None,
    )
        .unwrap_err();
    println!("{:?}", invoke_result);
}

