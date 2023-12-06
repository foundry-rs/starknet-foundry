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

fn declare(
    contract_name: felt252, max_fee: Option<felt252>, nonce: Option<felt252>
) -> DeclareResult {
    let mut inputs = array![contract_name];

    match max_fee {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    match nonce {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    let buf = cheatcode::<'declare'>(inputs.span());

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
    class_hash: ClassHash,
    constructor_calldata: Array::<felt252>,
    salt: Option<felt252>,
    unique: bool,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
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
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    inputs.append(unique.into());

    match max_fee {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    match nonce {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    let buf = cheatcode::<'deploy'>(inputs.span());

    let contract_address: ContractAddress = (*buf[0])
        .try_into()
        .expect('Invalid contract address value');
    let transaction_hash = *buf[1];

    DeployResult { contract_address, transaction_hash }
}

#[derive(Drop, Clone)]
struct InvokeResult {
    transaction_hash: felt252,
}

fn invoke(
    contract_address: ContractAddress,
    entry_point_selector: felt252,
    calldata: Array::<felt252>,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
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
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    match nonce {
        Option::Some(val) => {
            inputs.append(0);
            inputs.append(val);
        },
        Option::None => inputs.append(1),
    };

    let buf = cheatcode::<'invoke'>(inputs.span());

    let transaction_hash = *buf[0];

    InvokeResult { transaction_hash }
}
