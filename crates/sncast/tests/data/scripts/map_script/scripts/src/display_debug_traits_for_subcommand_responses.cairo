use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeclareResultTrait, DeployResult, InvokeResult,
    CallResult, get_nonce, FeeSettings
};

fn main() {
    println!("test");
    let salt = 0x3;

    let declare_nonce = get_nonce('latest');
    println!("declare_nonce: {}", declare_nonce);
    println!("debug declare_nonce: {:?}", declare_nonce);

    let declare_result = declare(
        "Mapa",
        FeeSettings {
            max_fee: Option::None,
            max_gas: Option::Some(999999),
            max_gas_unit_price: Option::Some(100000000000)
        },
        Option::Some(declare_nonce)
    )
        .expect('declare failed');
    println!("declare_result: {}", declare_result);
    println!("debug declare_result: {:?}", declare_result);

    let class_hash = declare_result.class_hash();
    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        *class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        FeeSettings {
            max_fee: Option::None,
            max_gas: Option::Some(999999),
            max_gas_unit_price: Option::Some(100000000000)
        },
        Option::Some(deploy_nonce)
    )
        .expect('deploy failed');
    println!("deploy_result: {}", deploy_result);
    println!("debug deploy_result: {:?}", deploy_result);

    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x2],
        FeeSettings {
            max_fee: Option::None,
            max_gas: Option::Some(999999),
            max_gas_unit_price: Option::Some(100000000000)
        },
        Option::Some(invoke_nonce)
    )
        .expect('invoke failed');
    println!("invoke_result: {}", invoke_result);
    println!("debug invoke_result: {:?}", invoke_result);

    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('call failed');
    println!("call_result: {}", call_result);
    println!("debug call_result: {:?}", call_result);

    assert(call_result.data == array![0x2], *call_result.data.at(0));
}
