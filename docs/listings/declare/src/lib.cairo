use sncast_std::{declare, FeeSettings};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::Some(9999999),
        l1_gas: Option::None,
        l1_gas_price: Option::None,
        l2_gas: Option::None,
        l2_gas_price: Option::None,
        l1_data_gas: Option::None,
        l2_data_gas_price: Option::None,
    };

    let result = declare("HelloStarknet", fee_settings, Option::None).expect('declare failed');

    println!("declare result: {}", result);
    println!("debug declare result: {:?}", result);
}
