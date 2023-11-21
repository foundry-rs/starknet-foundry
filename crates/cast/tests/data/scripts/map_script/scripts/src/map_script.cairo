use sncast_std::{declare, deploy, DeclareResult, DeployResult};

fn second_contract() {
    let declare_result = declare('Mapa2', Option::None);
    let deploy_result = deploy(
        declare_result.class_hash, ArrayTrait::new(), Option::None, false, Option::None
    );
}

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let declare_result = declare('Mapa', Option::Some(max_fee));

    let class_hash = declare_result.class_hash;
    let deploy_result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, Option::Some(max_fee)
    );
    assert(deploy_result.transaction_hash != 0, deploy_result.transaction_hash);

    second_contract();
}
