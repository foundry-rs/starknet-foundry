use super::super::cheatcode::execute_cheatcode_and_deserialize;
use super::execution_info::Operation;
use super::execution_info::CheatArguments;
use super::CheatSpan;
use starknet::ContractAddress;

/// Changes the block hash for the given block number and contract address.
/// - `contract_address` - The contract address to which the cheat applies.
/// - `block_number` - Block number to be modified.
/// - `block_hash` - `felt252` representing the new block hash.
/// - `span` - instance of `CheatSpan` specifying the number of syscalls with the cheat
pub fn cheat_block_hash(
    contract_address: ContractAddress, block_number: u64, block_hash: felt252, span: CheatSpan,
) {
    _cheat_block_hash(
        block_number,
        Operation::Start(CheatArguments { value: block_hash, span, target: contract_address }),
    );
}

/// Starts a global block hash modification.
/// - `block_number` - Block number to be modified.
/// - `block_hash` - The block hash value to set globally.
pub fn start_cheat_block_hash_global(block_number: u64, block_hash: felt252) {
    _cheat_block_hash(block_number, Operation::StartGlobal(block_hash));
}

/// Cancels the `start_cheat_block_hash_global`.
/// - `block_number` - Block number for which the cheat should be stopped.
pub fn stop_cheat_block_hash_global(block_number: u64) {
    _cheat_block_hash(block_number, Operation::StopGlobal);
}

/// Starts a block hash modification for a specific contract.
/// - `contract_address` - Contract address associated with the modification.
/// - `block_number` - Block number to be modified.
/// - `block_hash` - The block hash to set.
pub fn start_cheat_block_hash(
    contract_address: ContractAddress, block_number: u64, block_hash: felt252,
) {
    _cheat_block_hash(
        block_number,
        Operation::Start(
            CheatArguments {
                value: block_hash, span: CheatSpan::Indefinite, target: contract_address,
            },
        ),
    );
}

/// Cancels the `cheat_block_hash`/`start_cheat_block_hash` for a specific contract.
/// - `block_number` - Block number for which the cheat should be stopped.
/// - `contract_address` - The contract affected by the previous cheat.
pub fn stop_cheat_block_hash(contract_address: ContractAddress, block_number: u64) {
    _cheat_block_hash(block_number, Operation::Stop(contract_address));
}

fn _cheat_block_hash(block_number: u64, operation: Operation<felt252>) {
    let mut inputs = array![block_number.into()];
    operation.serialize(ref inputs);

    execute_cheatcode_and_deserialize::<'set_block_hash', ()>(inputs.span());
}
