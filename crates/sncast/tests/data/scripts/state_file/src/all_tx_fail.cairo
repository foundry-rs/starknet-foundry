use sncast_std::{FeeSettingsTrait, declare, deploy, get_nonce, invoke};
use starknet::{ClassHash, ContractAddress};

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let salt = 0x3;

    let nonexistent_class_hash: ClassHash = 0x10101.try_into().expect('Invalid class hash value');

    let map_contract_address: ContractAddress = 0x2020202
        .try_into()
        .expect('Invalid contract address value');

    let declare_nonce = get_nonce('latest');
    declare("Not_this_time", fee_settings, Option::Some(declare_nonce))
        .expect_err('error expected declare');

    let deploy_nonce = get_nonce('pending');
    deploy(
        nonexistent_class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        fee_settings,
        Option::Some(deploy_nonce),
    )
        .expect_err('error expected deploy');

    let invoke_nonce = get_nonce('pending');
    invoke(
        map_contract_address,
        selector!("put"),
        array![0x1, 0x2],
        fee_settings,
        Option::Some(invoke_nonce),
    )
        .expect_err('error expected invoke');
}
