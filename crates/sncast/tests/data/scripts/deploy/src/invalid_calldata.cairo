use sncast_std::{
    get_nonce, deploy, DeployResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettingsTrait,
};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let salt = 0x3;

    let class_hash: ClassHash = 0x059426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded
        .try_into()
        .expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        class_hash, array![0x2], Option::Some(salt), true, fee_settings, Option::Some(deploy_nonce),
    )
        .unwrap_err();

    println!("{:?}", deploy_result);
}
