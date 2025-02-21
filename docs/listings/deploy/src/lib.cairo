use starknet::ClassHash;
use sncast_std::{deploy, FeeSettings};

fn main() {
    let max_fee = 9999999;
    let salt = 0x1;
    let nonce = 0x1;

    let class_hash: ClassHash = 0x03a8b191831033ba48ee176d5dde7088e71c853002b02a1cfa5a760aa98be046
        .try_into()
        .expect('Invalid class hash value');

    let result = deploy(
        class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        FeeSettings {
            max_fee: Option::Some(max_fee), max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::Some(nonce)
    )
        .expect('deploy failed');

    println!("deploy result: {}", result);
    println!("debug deploy result: {:?}", result);
}
