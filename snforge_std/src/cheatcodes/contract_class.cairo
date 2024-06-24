use core::serde::Serde;
use core::traits::TryInto;
use starknet::{ContractAddress, ClassHash, testing::cheatcode, SyscallResult};
use super::super::byte_array::byte_array_as_felt_array;
use super::super::_cheatcode::handle_cheatcode;
use core::traits::Into;

#[derive(Drop, Clone, Copy)]
struct ContractClass {
    class_hash: ClassHash,
}

trait ContractClassTrait {
    /// Calculates an address of a contract in advance that would be returned when calling `deploy`
    /// The precalculated address is only correct for the very next deployment
    /// The `constructor_calldata` has a direct impact on the resulting contract address
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// `constructor_calldata` - serialized calldata for the deploy constructor
    /// Returns the precalculated `ContractAddress`
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> ContractAddress;

    /// Deploys a contract
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// `constructor_calldata` - serialized calldata for the constructor
    /// Returns the address the contract was deployed at and serialized constructor return data, or panic data if it failed
    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> SyscallResult<(ContractAddress, Span<felt252>)>;

    /// Deploys a contract at a given address
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// `constructor_calldata` - serialized calldata for the constructor
    /// `contract_address` - address the contract should be deployed at
    /// Returns the address the contract was deployed at and serialized constructor return data, or panic data if it failed
    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> SyscallResult<(ContractAddress, Span<felt252>)>;

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

        let outputs = handle_cheatcode(cheatcode::<'precalculate_address'>(inputs.span()));
        (*outputs[0]).try_into().unwrap()
    }

    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> SyscallResult<(ContractAddress, Span<felt252>)> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);

        let mut outputs = handle_cheatcode(cheatcode::<'deploy'>(inputs.span()));
        let exit_code = *outputs.pop_front().unwrap();

        if exit_code == 0 {
            let contract_address_felt = *outputs.pop_front().unwrap();
            let contract_address = contract_address_felt.try_into().unwrap();
            let retdata = Serde::<Span<felt252>>::deserialize(ref outputs).unwrap();
            SyscallResult::Ok((contract_address, retdata))
        } else {
            let panic_data = Serde::<Array<felt252>>::deserialize(ref outputs).unwrap();
            SyscallResult::Err(panic_data)
        }
    }

    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> SyscallResult<(ContractAddress, Span<felt252>)> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);
        inputs.append(contract_address.into());

        let mut outputs = handle_cheatcode(cheatcode::<'deploy_at'>(inputs.span()));
        let exit_code = *outputs.pop_front().unwrap();

        if exit_code == 0 {
            let contract_address_felt = *outputs.pop_front().unwrap();
            let contract_address = contract_address_felt.try_into().unwrap();
            let retdata = Serde::<Span<felt252>>::deserialize(ref outputs).unwrap();
            SyscallResult::Ok((contract_address, retdata))
        } else {
            let panic_data = Serde::<Array<felt252>>::deserialize(ref outputs).unwrap();
            SyscallResult::Err(panic_data)
        }
    }

    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass {
        ContractClass { class_hash: class_hash.into() }
    }
}

/// Declares a contract
/// `contract` - name of a contract as Cairo string. It is a name of the contract (part after mod keyword) e.g. "HelloStarknet"
/// Returns the `ContractClass` which was declared or panic data if declaration failed
fn declare(contract: ByteArray) -> Result<ContractClass, Array<felt252>> {
    let mut span = handle_cheatcode(
        cheatcode::<'declare'>(byte_array_as_felt_array(@contract).span())
    );

    let exit_code = *span.pop_front().unwrap();

    if exit_code == 0 {
        let result = *span.pop_front().unwrap();
        let class_hash = result.try_into().unwrap();
        let contract_class = ContractClass { class_hash };
        Result::Ok(contract_class)
    } else {
        let panic_data = Serde::<Array<felt252>>::deserialize(ref span).unwrap();
        Result::Err(panic_data)
    }
}

/// Retrieves a class hash of a contract deployed under the given address
/// `contract_address` - target contract address
/// Returns the `ClassHash` under given address
fn get_class_hash(contract_address: ContractAddress) -> ClassHash {
    let contract_address_felt: felt252 = contract_address.into();

    // Expecting a buffer with one felt252, being the class hash.
    let buf = handle_cheatcode(cheatcode::<'get_class_hash'>(array![contract_address_felt].span()));
    match (*buf[0]).try_into() {
        Option::Some(hash) => hash,
        Option::None => panic!("Invalid class hash value")
    }
}

fn _prepare_calldata(
    class_hash: @ClassHash, constructor_calldata: @Array::<felt252>
) -> Array::<felt252> {
    let class_hash: felt252 = class_hash.clone().into();
    let mut inputs: Array::<felt252> = array![class_hash];
    constructor_calldata.serialize(ref inputs);
    inputs
}
