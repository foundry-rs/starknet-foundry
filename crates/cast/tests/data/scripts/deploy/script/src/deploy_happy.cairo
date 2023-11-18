use array::ArrayTrait;
use sncast_std::{deploy, DeployResult};
use starknet::ClassHash;

fn main() {
    let max_fee = 99999999999999999;
    let class_hash: ClassHash = 0x3a8b191831033ba48ee176d5dde7088e71c853002b02a1cfa5a760aa98be046
        .try_into()
        .expect('Invalid class hash value');
    let salt = 0x3;
    let deploy_result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, Option::Some(max_fee)
    );
}
