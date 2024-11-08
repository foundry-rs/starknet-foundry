use super::cheat_execution_info::{
    BlockInfoMockOperations, CheatArguments, ExecutionInfoMockOperations, Operation,
};
use crate::state::CheatSpan;
use crate::CheatnetState;
use cairo_vm::Felt252;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn cheat_block_hash(
        &mut self,
        contract_address: ContractAddress,
        block_hash: Felt252,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_hash: Operation::Start(CheatArguments {
                    value: block_hash,
                    span,
                    target: contract_address,
                }),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_block_hash_global(&mut self, block_hash: Felt252) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_hash: Operation::StartGlobal(block_hash),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_block_hash(
        &mut self,
        contract_address: ContractAddress,
        block_hash: Felt252,
    ) {
        self.cheat_block_hash(contract_address, block_hash, CheatSpan::Indefinite);
    }

    pub fn stop_cheat_block_hash(&mut self, contract_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_hash: Operation::Stop(contract_address),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn stop_cheat_block_hash_global(&mut self) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_hash: Operation::StopGlobal,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}
