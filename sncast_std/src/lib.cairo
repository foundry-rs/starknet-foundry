use starknet::{testing::cheatcode, ContractAddress, ClassHash};

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

#[derive(Drop, Clone)]
struct DeclareResult {
    class_hash: ClassHash,
    transaction_hash: felt252,
}

fn declare(contract_name: felt252, max_fee: Option<felt252>) -> DeclareResult {
    let mut inputs = array![contract_name];

    match max_fee {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(max_fee.unwrap());
        },
        Option::None => inputs.append(1),
    };

    let buf = cheatcode::<'declare'>(inputs.span());

    let class_hash: ClassHash = (*buf[0]).try_into().expect('Invalid class hash value');
    let transaction_hash = *buf[1];

    DeclareResult { class_hash, transaction_hash }
}
