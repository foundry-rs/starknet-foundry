use super::cheat_execution_info::{
    BlockInfoMockOperations, CheatArguments, ExecutionInfoMockOperations, Operation,
};
use crate::CheatnetState;
use crate::state::CheatSpan;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn cheat_block_timestamp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: u64,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_timestamp: Operation::Start(CheatArguments {
                    value: timestamp,
                    span,
                    target: contract_address,
                }),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_block_timestamp_global(&mut self, timestamp: u64) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_timestamp: Operation::StartGlobal(timestamp),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_block_timestamp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: u64,
    ) {
        self.cheat_block_timestamp(contract_address, timestamp, CheatSpan::Indefinite);
    }

    pub fn stop_cheat_block_timestamp(&mut self, contract_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_timestamp: Operation::Stop(contract_address),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn stop_cheat_block_timestamp_global(&mut self) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                block_timestamp: Operation::StopGlobal,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}
