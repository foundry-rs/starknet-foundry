use super::cheat_execution_info::{
    BlockInfoMockOperations, CheatArguments, ExecutionInfoMockOperations, Operation,
};
use crate::state::CheatSpan;
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn cheat_block_number(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_number: Operation::Start(CheatArguments {
                    value: block_number,
                    span,
                    target: contract_address,
                }),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_block_number_global(&mut self, block_number: u64) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_number: Operation::StartGlobal(block_number),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_block_number(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
    ) {
        self.cheat_block_number(contract_address, block_number, CheatSpan::Indefinite);
    }

    pub fn stop_cheat_block_number(&mut self, contract_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_number: Operation::Stop(contract_address),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn stop_cheat_block_number_global(&mut self) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_number: Operation::StopGlobal,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}
