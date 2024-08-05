use sncast_std::{deploy, DeployResult, FeeSettings, StrkFeeSettings};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let salt = 0x3;
    let class_hash: ClassHash = 0x059426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded
        .try_into()
        .expect('Invalid class hash value');

    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        FeeSettings::Strk(
            StrkFeeSettings {
                max_gas: Option::Some(999),
                max_gas_unit_price: Option::Some(999999999999),
                max_fee: Option::None
            }
        ),
        Option::None
    )
        .expect('deploy failed');

    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);
}
