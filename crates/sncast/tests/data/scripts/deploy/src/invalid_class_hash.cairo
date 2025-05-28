use sncast_std::{
    DeployResult, FeeSettingsTrait, ProviderError, ScriptCommandError, StarknetError, deploy,
    get_nonce,
};
use starknet::ClassHash;

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let salt = 0x3;

    let class_hash: ClassHash = 0xdddd.try_into().expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        fee_settings,
        Option::Some(deploy_nonce),
    )
        .unwrap_err();

    println!("{:?}", deploy_result);
}
