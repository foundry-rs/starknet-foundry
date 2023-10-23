use array::ArrayTrait;
use array::SpanTrait;
use traits::Into;
use traits::TryInto;
use serde::Serde;
use traits::Drop;

use debug::PrintTrait;

use starknet::{
    testing::cheatcode, ClassHash, ContractAddress, ClassHashIntoFelt252,
    ContractAddressIntoFelt252, Felt252TryIntoClassHash, Felt252TryIntoContractAddress,
    contract_address_const
};

#[derive(Drop, Clone)]
struct CallResult {
    data: Array::<felt252>,
}

fn call(
    contract_address: ContractAddress, function_name: felt252, calldata: Array::<felt252>
) -> CallResult {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_name];

    let calldata_len = calldata.len();
    inputs.append(calldata_len.into());

    let mut i = 0;
    loop {
        if i == calldata_len {
            break;
        }
        inputs.append(*calldata[i]);
        i += 1;
    };

    let buf = cheatcode::<'call'>(inputs.span());

    // TODO: handle Result
    // let exit_code = *buf[0];
    let result_data_len = (*buf[0]).try_into().unwrap();
    
    let mut result_data = array![];
    let mut i = 1;
    loop {
        if result_data_len + 1 == i {
            break;
        }
        result_data.append(*buf[i]);
        i += 1;
    };

    CallResult { data: result_data }
}

#[derive(Drop, Clone)]
struct InvokeResult {
    transaction_hash: felt252,
}

fn invoke(
    contract_address: ContractAddress, entry_point_selector: felt252, calldata: Array::<felt252>, max_fee: Option<felt252>
) -> InvokeResult {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, entry_point_selector];

    let calldata_len = calldata.len();
    inputs.append(calldata_len.into());

    let mut i = 0;
    loop {
        if i == calldata_len {
            break;
        }
        inputs.append(*calldata[i]);
        i += 1;
    };

    match max_fee {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(max_fee.unwrap());
        },
        Option::None => inputs.append(1),
    };

    let buf = cheatcode::<'invoke'>(inputs.span());

    // TODO: handle Result
    // let exit_code = *buf[0];
    let transaction_hash = *buf[0];

    InvokeResult { transaction_hash }
}

#[derive(Drop, Clone)]
struct DeclareResult {
    class_hash: ClassHash,
    transaction_hash: felt252,
}

fn declare(
    contract_name: felt252, max_fee: Option<felt252>
) -> DeclareResult {
    let mut inputs = array![contract_name];

    match max_fee {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(max_fee.unwrap());
        },
        Option::None => inputs.append(1),
    };

    let buf = cheatcode::<'declare'>(inputs.span());

    // TODO: handle Result
    // let exit_code = *buf[0];
    let class_hash: ClassHash = (*buf[0]).try_into().expect('Invalid class hash value');
    let transaction_hash = *buf[1];

    DeclareResult { class_hash, transaction_hash }
}

#[derive(Drop, Clone)]
struct DeployResult {
    contract_address: ContractAddress,
    transaction_hash: felt252,
}

fn deploy(
    class_hash: ClassHash, constructor_calldata: Array::<felt252>, salt: Option<felt252>, unique: bool, max_fee: Option<felt252>
) -> DeployResult {
    let class_hash_felt: felt252 = class_hash.into();
    let mut inputs = array![class_hash_felt];

    let calldata_len = constructor_calldata.len();
    inputs.append(calldata_len.into());

    let mut i = 0;
    loop {
        if i == calldata_len {
            break;
        }
        inputs.append(*constructor_calldata[i]);
        i += 1;
    };

    match salt {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(salt.unwrap());
        },
        Option::None => inputs.append(1),
    };

    inputs.append(unique.into());

    match max_fee {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(max_fee.unwrap());
        },
        Option::None => inputs.append(1),
    };


    let buf = cheatcode::<'deploy'>(inputs.span());

    // TODO: handle Result
    // let exit_code = *buf[0];
    let contract_address: ContractAddress = (*buf[0]).try_into().expect('Invalid contract address value');
    let transaction_hash = *buf[1];

    DeployResult { contract_address, transaction_hash }
}

fn main() {
    // 'Hello, World!'.print();

    // starknet-devnet --seed 0
    // Account #0
    let addr = 0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a;

    // 'Declare'.print();
    let declare_result = declare('Map', Option::None);
    // let class_hash_felt: felt252 = declare_result.class_hash.into();
    // class_hash_felt.print();
    // declare_result.transaction_hash.print();

    // 'Deploy'.print();
    let deploy_result = deploy(declare_result.class_hash, array![], Option::None, true, Option::None);
    // let contract_address_felt: felt252 = deploy_result.contract_address.into();
    // contract_address_felt.print();
    // deploy_result.transaction_hash.print();

    // 'Invoke'.print();
    let invoke_result = invoke(deploy_result.contract_address, 'put', array![111, 222], Option::None);
    // invoke_result.transaction_hash.print();

    // 'Call'.print();
    let call_result = call(deploy_result.contract_address, 'get', array![111]);
    assert(call_result.data == array![222], 'value should be saved');
    // call_result.data.print();

    // 'Bye, World!'.print();
}
