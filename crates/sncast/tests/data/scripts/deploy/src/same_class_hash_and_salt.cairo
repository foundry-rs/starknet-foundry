use sncast_std::{get_nonce, deploy, DeployResult, ScriptCommandError, ProviderError, StarknetError};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x34542;

    let class_hash: ClassHash = 0x6d5e0eea81df9a6b03b9be2319a096d5322bd78ff1d2e6e315a91e9a4ac02ed
        .try_into()
        .expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::Some(deploy_nonce)
    ).expect('1st deploy failed');

    let class_hash: ClassHash = 0x6d5e0eea81df9a6b03b9be2319a096d5322bd78ff1d2e6e315a91e9a4ac02ed
        .try_into()
        .expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::Some(deploy_nonce)
    ).unwrap_err();

    println!("test");
    println!("{:?}", deploy_result);
}
