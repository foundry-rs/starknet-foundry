use sncast_std::{declare, DeclareResult, FeeSettings, EthFeeSettings};

fn main() {
    let max_fee = 9999999;

    let result = declare(
        "HelloStarknet",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::None
    )
        .expect('declare failed');

    println!("declare result: {}", result);
    println!("debug declare result: {:?}", result);
}
