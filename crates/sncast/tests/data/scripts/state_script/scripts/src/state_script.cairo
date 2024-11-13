use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeclareResultTrait, DeployResult, InvokeResult,
    CallResult, get_nonce, FeeSettings, EthFeeSettings
};

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x5;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare(
        "State",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(declare_nonce)
    )
        .expect('state declare failed');

    let class_hash = declare_result.class_hash();
    let deploy_nonce = get_nonce('pending');
    let deploy_result = deploy(
        *class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(deploy_nonce)
    )
        .expect('state deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x2],
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(invoke_nonce)
    )
        .expect('state invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('state call failed');
    assert(call_result.data == array![0x2], *call_result.data.at(0));
}
