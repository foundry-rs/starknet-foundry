use sncast_std::{declare, deploy, invoke, call, DeclareResultTrait, get_nonce, FeeSettingsTrait};

fn second_contract() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );

    let declare_result = declare("Mapa2", fee_settings, Option::None)
        .expect('mapa2 declare failed');

    let deploy_result = deploy(
        *declare_result.class_hash(),
        ArrayTrait::new(),
        Option::None,
        false,
        fee_settings,
        Option::None
    )
        .expect('mapa deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x3],
        fee_settings,
        Option::None
    )
        .expect('mapa2 invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('mapa2 call failed');
    assert(call_result.data == array![0x3], *call_result.data.at(0));
}

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let salt = 0x3;

    let declare_nonce = get_nonce('latest');
    println!("DDD");
    let declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce))
        .expect('mapa declare failed');

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
        .expect('mapa deploy failed');
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address,
        selector!("put"),
        array![0x1, 0x2],
        fee_settings,
        Option::Some(invoke_nonce)
    )
        .expect('mapa invoke failed');
    assert(invoke_result.transaction_hash != 0, invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1])
        .expect('mapa call failed');
    assert(call_result.data == array![0x2], *call_result.data.at(0));

    second_contract();
}
