use starknet::ClassHash;
use sncast_std::{deploy, FeeSettings};

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
    let salt = 0x1;
    let nonce = 0x1;

    let class_hash: ClassHash = 0x03a8b191831033ba48ee176d5dde7088e71c853002b02a1cfa5a760aa98be046
        .try_into()
        .expect('Invalid class hash value');

    let result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, fee_settings, Option::Some(nonce)
    )
        .expect('deploy failed');

    println!("deploy result: {}", result);
    println!("debug deploy result: {:?}", result);
}
