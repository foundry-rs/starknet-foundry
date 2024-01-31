use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce
};

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x4;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare('Mapa', Option::Some(max_fee), Option::Some(declare_nonce));

    let class_hash = declare_result.class_hash;
    let deploy_nonce = get_nonce('latest');
    let deploy_result = deploy(
        class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::Some(deploy_nonce)
    );
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('latest');

    // Supposed to fail - entry point does not exist
    let invoke_result = invoke(
        deploy_result.contract_address,
        'ohno',
        array![0x1, 0x2],
        Option::Some(max_fee),
        Option::Some(invoke_nonce)
    );
}
