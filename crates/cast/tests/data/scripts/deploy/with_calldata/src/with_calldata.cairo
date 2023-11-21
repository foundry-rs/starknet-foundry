use sncast_std::{deploy, DeployResult};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;
    let class_hash: ClassHash = 0x6e10d493d7c807e0fbaad4f0c31792f24d64747fa328830a68cb5d2313f9a55
        .try_into()
        .expect('Invalid class hash value');

    let deploy_result = deploy(
        class_hash, array![0x2, 0x2, 0x0], Option::Some(salt), true, Option::Some(max_fee)
    );

    assert(
        deploy_result
            .transaction_hash == 0x219016861d63cab224142aaaccbd335887c7c2f45e5e487be6d7c2eaf777797,
        deploy_result.transaction_hash
    );
}
