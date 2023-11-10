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
fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let addr = 0x0089496091c660345BaA480dF76c1A900e57cf34759A899eFd1EADb362b20DB5;
    let call_result = call(eth.try_into().unwrap(), 'allowance', array![addr, addr]);
    let call_result = *call_result.data[0];
    assert(call_result == 0, call_result);

    let call_result = call(eth.try_into().unwrap(), 'decimals', array![]);
    let call_result = *call_result.data[0];
    assert(call_result == 18, call_result);
}
