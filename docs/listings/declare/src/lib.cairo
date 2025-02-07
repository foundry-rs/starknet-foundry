use sncast_std::{declare, DeclareResult, FeeSettings};

fn main() {
    let max_fee = 9999999;

    let result = declare(
        "HelloStarknet",
        FeeSettings {
            max_fee: Option::Some(max_fee), max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::None
    )
        .expect('declare failed');

    println!("declare result: {}", result);
    println!("debug declare result: {:?}", result);
}
