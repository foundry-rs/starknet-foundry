use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce,
};
use debug::PrintTrait;
use starknet::{ContractAddress};
fn main() {
    let max_fee = 99999999999999999;

    let declare_map = declare('Map', Option::Some(max_fee), Option::None);
    'Mapw'.print();
    'declare'.print();

    let nonce_latest = get_nonce('latest');
    nonce_latest.print();

    let invoke_nonce = get_nonce('pending');
    invoke_nonce.print();

    let declare_with_params = declare(
        'ConstructorWithParams', Option::Some(max_fee), Option::Some(invoke_nonce)
    );
    'ConstructorWithParams'.print();
    'declare'.print();
}
