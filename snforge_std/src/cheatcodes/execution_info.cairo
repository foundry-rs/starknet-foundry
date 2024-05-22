use starknet::{ContractAddress, testing::cheatcode, contract_address_const};
use starknet::info::v2::ResourceBounds;
use snforge_std::cheatcodes::CheatSpan;
use super::super::_cheatcode::handle_cheatcode;

mod caller_address;
mod block_number;
mod block_timestamp;
mod sequencer_address;
mod version;
mod max_fee;
mod signature;
mod transaction_hash;
mod chain_id;
mod nonce;
mod resource_bounds;
mod tip;
mod paymaster_data;
mod nonce_data_availability_mode;
mod fee_data_availability_mode;
mod account_deployment_data;
mod account_contract_address;


#[derive(Serde, Drop, Copy)]
struct CheatArguments<T> {
    value: T,
    span: CheatSpan,
    target: ContractAddress,
}

#[derive(Serde, Drop, Copy)]
enum Operation<T> {
    StartGlobal: T,
    Start: CheatArguments<T>,
    Stop: ContractAddress,
    StopGlobal,
    Retain,
}

/// A structure used for setting individual fields in `TxInfo`
/// All fields are wrapped into `Operation`, meaning that the field will be:
/// - `Retain` - unchanged
/// - `Start` - changed for given contract and span
/// - `Stop` - reset to the initial value for given contract and span
/// - `StartGlobal` - changed for all contracts until overridden or stopped
/// - `StopGlobal` - reset to the initial value for all contracts
#[derive(Copy, Drop, Serde)]
struct TxInfoMock {
    version: Operation<felt252>,
    account_contract_address: Operation<ContractAddress>,
    max_fee: Operation<u128>,
    signature: Operation<Span<felt252>>,
    transaction_hash: Operation<felt252>,
    chain_id: Operation<felt252>,
    nonce: Operation<felt252>,
    // starknet::info::v2::TxInfo fields
    resource_bounds: Operation<Span<ResourceBounds>>,
    tip: Operation<u128>,
    paymaster_data: Operation<Span<felt252>>,
    nonce_data_availability_mode: Operation<u32>,
    fee_data_availability_mode: Operation<u32>,
    account_deployment_data: Operation<Span<felt252>>,
}

impl TxInfoMockImpl of Default<TxInfoMock> {
    /// Returns a default object initialized with Operation::Retain for each field
    /// Useful for setting only a few of fields instead of all of them
    fn default() -> TxInfoMock {
        TxInfoMock {
            version: Operation::Retain,
            account_contract_address: Operation::Retain,
            max_fee: Operation::Retain,
            signature: Operation::Retain,
            transaction_hash: Operation::Retain,
            chain_id: Operation::Retain,
            nonce: Operation::Retain,
            resource_bounds: Operation::Retain,
            tip: Operation::Retain,
            paymaster_data: Operation::Retain,
            nonce_data_availability_mode: Operation::Retain,
            fee_data_availability_mode: Operation::Retain,
            account_deployment_data: Operation::Retain,
        }
    }
}

/// A structure used for setting individual fields in `BlockInfo`
/// All fields are wrapped into `Operation`, meaning that the field will be:
/// - `Retain` - unchanged
/// - `Start` - changed for given contract and span
/// - `Stop` - reset to the initial value for given contract and span
/// - `StartGlobal` - changed for all contracts until overridden or stopped
/// - `StopGlobal` - reset to the initial value for all contracts
#[derive(Copy, Drop, Serde)]
struct BlockInfoMock {
    block_number: Operation<u64>,
    block_timestamp: Operation<u64>,
    sequencer_address: Operation<ContractAddress>,
}

impl BlockInfoMockImpl of Default<BlockInfoMock> {
    /// Returns a default object initialized with Operation::Retain for each field
    /// Useful for setting only a few of fields instead of all of them
    fn default() -> BlockInfoMock {
        BlockInfoMock {
            block_number: Operation::Retain,
            block_timestamp: Operation::Retain,
            sequencer_address: Operation::Retain,
        }
    }
}

/// A structure used for setting individual fields in `ExecutionInfo`
/// All fields are wrapped into `Operation`, meaning that the field will be:
/// - `Retain` - unchanged
/// - `Start` - changed for given contract and span
/// - `Stop` - reset to the initial value for given contract and span
/// - `StartGlobal` - changed for all contracts until overridden or stopped
/// - `StopGlobal` - reset to the initial value for all contracts
#[derive(Copy, Drop, Serde)]
struct ExecutionInfoMock {
    block_info: BlockInfoMock,
    tx_info: TxInfoMock,
    caller_address: Operation<ContractAddress>,
}

impl ExecutionInfoMockImpl of Default<ExecutionInfoMock> {
    /// Returns a default object initialized with Operation::Retain for each field
    /// Useful for setting only a few of fields instead of all of them
    fn default() -> ExecutionInfoMock {
        ExecutionInfoMock {
            block_info: Default::default(),
            tx_info: Default::default(),
            caller_address: Operation::Retain,
        }
    }
}

/// Changes `ExecutionInfo` returned by `get_execution_info()`
/// - `execution_info_mock` - a struct with same structure as `ExecutionInfo` (returned by
/// `get_execution_info()`)
fn cheat_execution_info(execution_info_mock: ExecutionInfoMock) {
    let mut inputs = array![];

    execution_info_mock.serialize(ref inputs);

    handle_cheatcode(cheatcode::<'cheat_execution_info'>(inputs.span()));
}
