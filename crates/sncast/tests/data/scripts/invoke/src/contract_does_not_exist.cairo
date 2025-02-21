use sncast_std::{
    invoke, FeeSettings,
};

fn main() {
    let map_contract_address = 0x123.try_into().expect('Invalid contract address value');

    let invoke_result = invoke(
        map_contract_address,
        selector!("put"),
        array![0x10, 0x1],
        FeeSettings {
            max_fee: Option::None, max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::None
    )
        .unwrap_err();
    println!("{:?}", invoke_result);
}

