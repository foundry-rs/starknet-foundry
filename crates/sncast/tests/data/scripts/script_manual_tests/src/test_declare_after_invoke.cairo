use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce,
};
use debug::PrintTrait;
use starknet::{ContractAddress};
fn main() {
    let max_fee = 99999999999999999;

    let contract_address: ContractAddress =
        0x18abd0648d901555975edde1330cffcf37c1da1edbf20b889ba1101e70a6c22
        .try_into()
        .expect('Invalid class hash value');

    let invoke_result = invoke(
        contract_address, 'put', array![0x11, 0x2], Option::Some(max_fee), Option::None
    );

    let invoke_nonce = get_nonce('pending');
    invoke_nonce.print();

    let declare_map = declare('Map', Option::Some(max_fee), Option::None);
    'Map'.print();
    'declare'.print();

    let invoke_nonce = get_nonce('pending');
    invoke_nonce.print();
    let salt = 0x3;
    let deploy_result = deploy(
        declare_map.class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::None
    );

    let nonce_latest = get_nonce('latest');
    nonce_latest.print();
    let invoke_nonce = get_nonce('pending');
    invoke_nonce.print();

    let declare_with_params = declare('ConstructorWithParams', Option::Some(max_fee), Option::None);
    'ConstructorWithParams'.print();
    'declare'.print();
}
