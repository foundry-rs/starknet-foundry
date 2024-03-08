use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce
};

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let map_contract_address = 0x07537a17e169c96cf2b0392508b3a66cbc50c9a811a8a7896529004c5e93fdf6
        .try_into()
        .expect('Invalid contract address value');

    let invoke_result = invoke(
        map_contract_address, selector!("put"), array![0x10, 0x1], Option::None, Option::None
    )
        .unwrap();
}
