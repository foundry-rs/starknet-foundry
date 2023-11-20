use sncast_std::{declare, deploy, DeclareResult, DeployResult};

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let declare_result = declare('Mapa', Option::Some(max_fee));
    let declare_result_mapa2 = declare('Mapa2', Option::None);

    let class_hash = declare_result.class_hash;
    let deploy_result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, Option::Some(max_fee)
    );
}
