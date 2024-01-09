use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce,
};
use debug::PrintTrait;
use starknet::{ContractAddress};
fn main() {
    let max_fee = 99999999999999999;

    let declare_map = declare('Map', Option::Some(max_fee), Option::None);
    'Map'.print();
    'declare'.print();

    let declare_with_params = declare(
        'ConstructorWithParams', Option::Some(max_fee), Option::None
    );
    'ConstructorWithParams'.print();
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

    let invoke_nonce = get_nonce('pending');
    invoke_nonce.print();
    let salt = 0x4;

    let deploy_result = deploy(
        declare_with_params.class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::None
    );

    let invoke_result = invoke(
        deploy_result.contract_address, 'put', array![0x1, 0x2], Option::Some(max_fee), Option::None
    );

}
