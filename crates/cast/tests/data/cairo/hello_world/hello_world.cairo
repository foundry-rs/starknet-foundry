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
    result_data_len.print();
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

fn main() {
    'Hello, World!'.print();

    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let addr = 0x0089496091c660345BaA480dF76c1A900e57cf34759A899eFd1EADb362b20DB5;
    let call_result = call(eth.try_into().unwrap(), 'allowance', array![addr, addr]);
    call_result.data.print();

    let invoke_result = invoke(eth.try_into().unwrap(), 'increaseAllowance', array![addr, 1, 0], Option::None);
    invoke_result.transaction_hash.print();

    'Bye, World!'.print();
}
