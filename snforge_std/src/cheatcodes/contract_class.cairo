use starknet::{ContractAddress, ClassHash, SyscallResult};
use super::super::byte_array::byte_array_as_felt_array;
use super::super::_cheatcode::execute_cheatcode_and_deserialize;

#[derive(Drop, Serde, Copy)]
pub struct ContractClass {
    pub class_hash: ClassHash,
}

#[derive(Drop, Serde, Clone)]
pub enum DeclareResult {
    Success: ContractClass,
    AlreadyDeclared: ContractClass,
}

pub trait ContractClassTrait {
    /// Calculates an address of a contract in advance that would be returned when calling `deploy`
    /// The precalculated address is only correct for the very next deployment
    /// The `constructor_calldata` has a direct impact on the resulting contract address
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// and unpacking `DeclareResult`
    /// `constructor_calldata` - serialized calldata for the deploy constructor
    /// Returns the precalculated `ContractAddress`
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>,
    ) -> ContractAddress;

    /// Deploys a contract
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// and unpacking `DeclareResult`
    /// `constructor_calldata` - calldata for the constructor, serialized with `Serde`
    /// Returns the address the contract was deployed at and serialized constructor return data, or
    /// panic data if it failed
    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>,
    ) -> SyscallResult<(ContractAddress, Span<felt252>)>;

    /// Deploys a contract at a given address
    /// `self` - an instance of the struct `ContractClass` which is obtained by calling `declare`
    /// and unpacking `DeclareResult`
    /// `constructor_calldata` - serialized calldata for the constructor
    /// `contract_address` - address the contract should be deployed at
    /// Returns the address the contract was deployed at and serialized constructor return data, or
    /// panic data if it failed
    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress,
    ) -> SyscallResult<(ContractAddress, Span<felt252>)>;

    /// Utility method for creating a new `ContractClass` instance
    /// `class_hash` - a numeric value that can be converted into the class hash of `ContractClass`
    /// Returns the created `ContractClass`
    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass;
}

impl ContractClassImpl of ContractClassTrait {
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>,
    ) -> ContractAddress {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);

        execute_cheatcode_and_deserialize::<'precalculate_address'>(inputs.span())
    }

    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>,
    ) -> SyscallResult<(ContractAddress, Span<felt252>)> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);

        execute_cheatcode_and_deserialize::<'deploy'>(inputs.span())
    }

    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress,
    ) -> SyscallResult<(ContractAddress, Span<felt252>)> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);
        inputs.append(contract_address.into());

        execute_cheatcode_and_deserialize::<'deploy_at'>(inputs.span())
    }

    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass {
        ContractClass { class_hash: class_hash.into() }
    }
}

pub trait DeclareResultTrait {
    /// Gets inner `ContractClass`
    /// `self` - an instance of the struct `DeclareResult` which is obtained by calling `declare`
    // Returns the `@ContractClass`
    fn contract_class(self: @DeclareResult) -> @ContractClass;
}

impl DeclareResultImpl of DeclareResultTrait {
    fn contract_class(self: @DeclareResult) -> @ContractClass {
        match self {
            DeclareResult::Success(contract_class) => contract_class,
            DeclareResult::AlreadyDeclared(contract_class) => contract_class,
        }
    }
}

/// Declares a contract
/// `contract` - name of a contract as Cairo string. It is a name of the contract (part after mod
/// keyword) e.g. "HelloStarknet"
/// Returns the `DeclareResult` that encapsulated possible outcomes in the enum:
/// - `Success`: Contains the successfully declared `ContractClass`.
/// - `AlreadyDeclared`: Contains `ContractClass` and signals that the contract has already been
/// declared.
pub fn declare(contract: ByteArray) -> Result<DeclareResult, Array<felt252>> {
    execute_cheatcode_and_deserialize::<'declare'>(byte_array_as_felt_array(@contract).span())
}

/// Retrieves a class hash of a contract deployed under the given address
/// `contract_address` - target contract address
/// Returns the `ClassHash` under given address
pub fn get_class_hash(contract_address: ContractAddress) -> ClassHash {
    execute_cheatcode_and_deserialize::<'get_class_hash'>(array![contract_address.into()].span())
}

fn _prepare_calldata(
    class_hash: @ClassHash, constructor_calldata: @Array::<felt252>,
) -> Array::<felt252> {
    let class_hash: felt252 = class_hash.clone().into();
    let mut inputs: Array::<felt252> = array![class_hash];
    constructor_calldata.serialize(ref inputs);
    inputs
}
