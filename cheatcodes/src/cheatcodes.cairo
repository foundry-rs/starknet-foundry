mod structs;
use structs::PreparedContract;
use structs::RevertedTransaction;
use structs::RevertedTransactionTrait;

use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use integer::Into;
use integer::TryInto;
use option::OptionTrait;
use starknet::testing::cheatcode;

fn declare(contract: felt252) -> Result::<felt252, felt252> {
    let span = cheatcode::<'declare'>(array![contract].span());

    let exit_code = *span[0];
    let result = *span[1];

    if exit_code == 0 {
        Result::<felt252, felt252>::Ok(result)
    } else {
        Result::<felt252, felt252>::Err(result)
    }
}

fn deploy(prepared_contract: PreparedContract) -> Result::<felt252, RevertedTransaction> {
    let PreparedContract{contract_address, class_hash, constructor_calldata } = prepared_contract;
    let mut inputs = array![contract_address, class_hash];

    let calldata_len_felt = constructor_calldata.len().into();
    inputs.append(calldata_len_felt);

    let calldata_len = constructor_calldata.len();
    let mut i = 0;
    loop {
        if calldata_len == i {
            break ();
        }
        inputs.append(*constructor_calldata[i]);
        i += 1;
    };

    let outputs = cheatcode::<'deploy'>(inputs.span());
    let exit_code = *outputs[0];

    if exit_code == 0 {
        let result = *outputs[1];
        Result::<felt252, RevertedTransaction>::Ok(result)
    } else {
        // TODO: feel free to change depending on the cheatcode::<'deploy'> low level implementation of error handling
        let panic_data_len_felt = *outputs[1];
        let panic_data_len = panic_data_len_felt.try_into().unwrap();
        let mut panic_data = array![];

        let offset = 2;
        let mut i = offset;
        loop {
            if panic_data_len + offset == i {
                break ();
            }
            panic_data.append(*outputs[i]);
            i += 1;
        };

        Result::<felt252, RevertedTransaction>::Err(RevertedTransaction { panic_data })
    }
}
