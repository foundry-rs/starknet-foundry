use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, DeclareResultTrait, InvokeResult,
    CallResult, get_nonce, FeeSettings, EthFeeSettings
};

fn second_contract() {
    let declare_result = declare(
        "Mapa2", FeeSettings::Eth(EthFeeSettings { max_fee: Option::None }), Option::None
    )
        .expect('mapa2 declare failed');

    let deploy_result = deploy(
        *declare_result.class_hash(),
        ArrayTrait::new(),
        Option::None,
        false,
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::None }),
        Option::None
    )
        .expect('mapa deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x3],
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::None }),
        Option::None
    )
        .expect('mapa2 invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('mapa2 call failed');
    assert(call_result.data == array![0x3], *call_result.data.at(0));
}

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare(
        "Mapa",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(declare_nonce)
    )
        .expect('mapa declare failed');

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
        .expect('mapa deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x2],
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(invoke_nonce)
    )
        .expect('mapa invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('mapa call failed');
    assert(call_result.data == array![0x2], *call_result.data.at(0));

    second_contract();
}
