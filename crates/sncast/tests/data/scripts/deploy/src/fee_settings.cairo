use sncast_std::{deploy, DeployResult, FeeSettings};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::Some(100000),
        l1_gas_price: Option::Some(10000000000000),
        l2_gas: Option::Some(1000000000),
        l2_gas_price: Option::Some(100000000000000000000),
        l1_data_gas: Option::Some(100000),
        l2_data_gas_price: Option::Some(10000000000000),
    };
    let salt = 0x3;
    let class_hash: ClassHash = 0x059426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded
        .try_into()
        .expect('Invalid class hash value');

    let deploy_result = deploy(
        class_hash, array![0x2, 0x2, 0x0], Option::Some(salt), true, fee_settings, Option::None
    )
        .expect('deploy failed');

    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);
}
