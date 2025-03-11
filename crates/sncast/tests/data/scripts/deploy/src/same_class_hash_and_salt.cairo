use sncast_std::{
    get_nonce, deploy, DeployResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings,
};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::Some(99999999999999999999),
        l1_gas: Option::None,
        l1_gas_price: Option::None,
        l2_gas: Option::None,
        l2_gas_price: Option::None,
        l1_data_gas: Option::None,
        l2_data_gas_price: Option::None,
    };
    let salt = 0x34542;

    let class_hash: ClassHash = 0x059426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded
        .try_into()
        .expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending');
    deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        fee_settings,
        Option::Some(deploy_nonce)
    )
        .expect('1st deploy failed');

    let class_hash: ClassHash = 0x059426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded
        .try_into()
        .expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        fee_settings,
        Option::Some(deploy_nonce)
    )
        .unwrap_err();

    println!("{:?}", deploy_result);
}
