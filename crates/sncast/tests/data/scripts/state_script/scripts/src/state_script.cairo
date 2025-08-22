use sncast_std::{DeclareResultTrait, FeeSettingsTrait, call, declare, deploy, get_nonce, invoke};

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let salt = 0x5;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare("State", fee_settings, Option::Some(declare_nonce))
        .expect('state declare failed');

    let class_hash = declare_result.class_hash();
    let deploy_nonce = get_nonce('pre_confirmed');
    let deploy_result = deploy(
        *class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        fee_settings,
        Option::Some(deploy_nonce),
    )
        .expect('state deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('pre_confirmed');
    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x2],
        fee_settings,
        Option::Some(invoke_nonce),
    )
        .expect('state invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('state call failed');
    assert(call_result.data == array![0x2], *call_result.data.at(0));
}
