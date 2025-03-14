use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeclareResultTrait, DeployResult, InvokeResult,
    CallResult, get_nonce, FeeSettings
};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::Some(1000000),
        l1_gas_price: Option::Some(10000000000000),
        l2_gas: Option::Some(1000000000),
        l2_gas_price: Option::Some(100000000000000000),
        l1_data_gas: Option::Some(1000000),
        l2_data_gas_price: Option::Some(10000000000000),
    };
    let salt = 0x5;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare("State", fee_settings, Option::Some(declare_nonce))
        .expect('state declare failed');

    let class_hash = declare_result.class_hash();
    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        *class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        fee_settings,
        Option::Some(deploy_nonce)
    )
        .expect('state deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x2],
        fee_settings,
        Option::Some(invoke_nonce)
    )
        .expect('state invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('state call failed');
    assert(call_result.data == array![0x2], *call_result.data.at(0));
}
