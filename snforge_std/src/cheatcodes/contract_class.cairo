use starknet::{ContractAddress, ClassHash, testing::cheatcode};

#[derive(Drop, Clone)]
struct RevertedTransaction {
    panic_data: Array::<felt252>,
}

trait RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252;
}

impl RevertedTransactionImpl of RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252 {
        *self.panic_data.at(0)
    }
}

#[derive(Drop, Clone, Copy)]
struct ContractClass {
    class_hash: ClassHash,
}

trait ContractClassTrait {
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> ContractAddress;

    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> Result<ContractAddress, RevertedTransaction>;

    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> Result<ContractAddress, RevertedTransaction>;
}

impl ContractClassImpl of ContractClassTrait {
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> ContractAddress {
        let mut inputs: Array::<felt252> = _prepare_calldata(self.class_hash, constructor_calldata);

        let outputs = cheatcode::<'precalculate_address'>(inputs.span());
        (*outputs[0]).try_into().unwrap()
    }

    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> Result<ContractAddress, RevertedTransaction> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);

        let outputs = cheatcode::<'deploy'>(inputs.span());
        let exit_code = *outputs[0];

        if exit_code == 0 {
            let result = *outputs[1];
            Result::<ContractAddress, RevertedTransaction>::Ok(result.try_into().unwrap())
        } else {
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

            Result::<ContractAddress, RevertedTransaction>::Err(RevertedTransaction { panic_data })
        }
    }

    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> Result<ContractAddress, RevertedTransaction> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);
        inputs.append(contract_address.into());

        let outputs = cheatcode::<'deploy_at'>(inputs.span());
        let exit_code = *outputs[0];

        if exit_code == 0 {
            let result = *outputs[1];
            Result::<ContractAddress, RevertedTransaction>::Ok(result.try_into().unwrap())
        } else {
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

            Result::<ContractAddress, RevertedTransaction>::Err(RevertedTransaction { panic_data })
        }
    }
}

fn declare(contract: felt252) -> ContractClass {
    let span = cheatcode::<'declare'>(array![contract].span());

    let exit_code = *span[0];
    let result = *span[1];
    assert(exit_code == 0, 'declare should never fail');
    let class_hash = result.try_into().unwrap();

    ContractClass { class_hash }
}

fn get_class_hash(contract_address: ContractAddress) -> ClassHash {
    let contract_address_felt: felt252 = contract_address.into();

    // Expecting a buffer with one felt252, being the class hash.
    let buf = cheatcode::<'get_class_hash'>(array![contract_address_felt].span());
    (*buf[0]).try_into().expect('Invalid class hash value')
}

fn _prepare_calldata(
    class_hash: @ClassHash, constructor_calldata: @Array::<felt252>
) -> Array::<felt252> {
    let class_hash: felt252 = class_hash.clone().into();
    let mut inputs: Array::<felt252> = array![class_hash];
    let calldata_len_felt = constructor_calldata.len().into();
    inputs.append(calldata_len_felt);

    let calldata_len = constructor_calldata.len();
    let mut i = 0;

    loop {
        if i == calldata_len {
            break ();
        }
        inputs.append(*constructor_calldata[i]);
        i += 1;
    };

    inputs
}
