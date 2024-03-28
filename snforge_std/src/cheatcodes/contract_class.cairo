use starknet::{ContractAddress, ClassHash, testing::cheatcode};
use super::super::byte_array::byte_array_as_felt_array;
use core::traits::Into;

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
    /// Calculates an address of a contract in advance that would be returned when calling `deploy`
    /// The precalculated address is only correct for the very next deployment
    /// The `constructor_calldata` has a direct impact on the resulting contract address
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// `constructor_calldata` - snapshot of calldata for the deploy constructor
    /// Returns the precalculated `ContractAddress`
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> ContractAddress;

    /// Deploys a contract and returns its address
    /// `self` - an instance of `ContractClass` struct
    /// `constructor_calldata` - snapshot of calldata for the constructor
    /// Returns the address the contract was deployed at, or a `RevertedTransaction` if it failed
    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> Result<ContractAddress, RevertedTransaction>;

    /// Deploys a contract at a given address
    /// `self` - an instance of `ContractClass` struct
    /// `constructor_calldata` - snapshot of calldata for the constructor
    /// `contract_address` - address the contract should be deployed at
    /// Returns the address the contract was deployed at, or a `RevertedTransaction` if it failed
    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> Result<ContractAddress, RevertedTransaction>;

    /// Utility method for creating a new `ContractClass` instance
    /// `class_hash` - a numeric value that can be converted into the class hash of `ContractClass`
    /// Returns the created `ContractClass`
    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass;
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
            Result::Ok(result.try_into().unwrap())
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

            Result::Err(RevertedTransaction { panic_data })
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
            Result::Ok(result.try_into().unwrap())
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

            Result::Err(RevertedTransaction { panic_data })
        }
    }

    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass {
        ContractClass { class_hash: class_hash.into() }
    }
}

/// Declares a contract
/// `contract` - name of a contract as Cairo string. It is a name of the contract (part after mod keyword) e.g. "HelloStarknet"
/// Returns the `ContractClass` which was declared
fn declare(contract: ByteArray) -> ContractClass {
    let span = cheatcode::<'declare'>(byte_array_as_felt_array(@contract).span());

    let exit_code = *span[0];
    let result = *span[1];
    assert(exit_code == 0, 'declare should never fail');
    let class_hash = result.try_into().unwrap();

    ContractClass { class_hash }
}

/// Retrieves a class hash of a contract deployed under the given address.
/// `contract_address` - target contract address
/// Returns the `ClassHash` under given address
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
