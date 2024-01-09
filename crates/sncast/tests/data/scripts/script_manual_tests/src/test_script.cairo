use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce,
};

use starknet::{ClassHash};
use debug::PrintTrait;
use starknet::{ContractAddress};
fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3323232323124214312;
    'DUPA'.print();
    let class_hash: ClassHash = 0x032171acbf7306faa4fd35ad4a02884966ce75636a81d99f0b0f5aaf4d9b27b9
        .try_into()
        .expect('Invalid class hash value');
    let invoke_nonce = get_nonce('pending');
    invoke_nonce.print();
    'deploy start'.print();
    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::None
    );
// 'declare1'.print();

// let contract_address: ContractAddress = 0x4f4b3a0643cfea0a8fee49ec3dc9a17cd6cd4886496f9c98bfcb68e34383682.try_into()
//     .expect('Invalid class hash value');

// let invoke_result = invoke(
//     contract_address, 'put', array![0x11, 0x2], Option::Some(max_fee), Option::None
// );

// let invoke_nonce = get_nonce('latest');
// invoke_nonce.print();
// let invoke_nonce = get_nonce('pending');
// invoke_nonce.print();

// let declare_result = declare('Mapw', Option::Some(max_fee), Option::Some(invoke_nonce));
// 'Mapw'.print();
// 'declare'.print();

// let declare_result = declare('ConstructorWithParamsqqq', Option::Some(max_fee), Option::None);
// 'ConstructorWithParamsq'.print();

// let invoke_nonce = get_nonce('pending');
// invoke_nonce.print();
// let class_hash = declare_result.class_hash;

// 'Deployed'.print();
// deploy_result.contract_address.print();

// let invoke_nonce = get_nonce('pending');
// invoke_nonce.print();

// let invoke_result = invoke(
//     deploy_result.contract_address, 'put', array![0x1, 0x2], Option::Some(max_fee), Option::Some(invoke_nonce)
// );
// 'i1'.print();
// let invoke_nonce = get_nonce('pending');
// invoke_nonce.print();

// 'i2'.print();

// let invoke_result = invoke(
//     deploy_result.contract_address, 'put', array![0x3, 0x2], Option::Some(max_fee), Option::Some(invoke_nonce)
// );

// let invoke_result = invoke(
//     deploy_result.contract_address, 'put', array![0x4, 0x2], Option::Some(max_fee), Option::Some(invoke_nonce)
// );

// let invoke_result = invoke(
//     deploy_result.contract_address, 'put', array![0x5, 0x2], Option::Some(max_fee), Option::None
// );
// 'Invoke tx hash is'.print();
// invoke_result.transaction_hash.print();

// let call_result = call(deploy_result.contract_address, 'get', array![0x1]);
// assert(call_result.data == array![0x2], *call_result.data.at(0));
}
