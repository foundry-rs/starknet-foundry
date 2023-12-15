use sncast_std::{deploy, DeployResult};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;
    let class_hash: ClassHash = 0x5cadc0ecbc6e2a502a7a7e8e5b55400a1a92afccb68ed5dc5efad86c1fc4edb
        .try_into()
        .expect('Invalid class hash value');

    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::None
    );

    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);
}

