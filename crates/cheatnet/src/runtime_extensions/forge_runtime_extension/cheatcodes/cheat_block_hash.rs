use crate::CheatnetState;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
    CheatArguments, Operation,
};
use crate::state::CheatSpan;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

impl CheatnetState {
    pub fn cheat_block_hash(&mut self, block_number: u64, operation: Operation<Felt>) {
        match operation {
            Operation::StartGlobal(block_hash) => {
                self.global_block_hash.insert(block_number, block_hash);
            }
            Operation::Start(args) => {
                let CheatArguments {
                    value,
                    span,
                    target,
                } = args;
                self.block_hash_contracts
                    .insert((target, block_number), (span, value));
            }
            Operation::Stop(contract_address) => {
                self.block_hash_contracts
                    .remove(&(contract_address, block_number));
            }
            Operation::StopGlobal => {
                self.global_block_hash.remove(&block_number);
            }
            Operation::Retain => {
                unreachable!("Retain operation isn't used for this cheat")
            }
        }
    }

    pub fn start_cheat_block_hash(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
        block_hash: Felt,
    ) {
        self.block_hash_contracts.insert(
            (contract_address, block_number),
            (CheatSpan::Indefinite, block_hash),
        );
    }

    pub fn stop_cheat_block_hash(&mut self, contract_address: ContractAddress, block_number: u64) {
        self.block_hash_contracts
            .remove(&(contract_address, block_number));
    }

    pub fn start_cheat_block_hash_global(&mut self, block_number: u64, block_hash: Felt) {
        self.global_block_hash.insert(block_number, block_hash);
    }

    pub fn stop_cheat_block_hash_global(&mut self, block_number: u64) {
        self.global_block_hash.remove(&block_number);
    }
}
