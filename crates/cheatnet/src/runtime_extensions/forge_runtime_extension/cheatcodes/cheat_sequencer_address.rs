use super::cheat_execution_info::{
    BlockInfoMockOperations, CheatArguments, ExecutionInfoMockOperations, Operation,
};
use crate::CheatnetState;
use crate::state::CheatSpan;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn cheat_sequencer_address(
        &mut self,
        contract_address: ContractAddress,
        sequencer_address: ContractAddress,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                sequencer_address: Operation::Start(CheatArguments {
                    value: sequencer_address,
                    span,
                    target: contract_address,
                }),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_sequencer_address(
        &mut self,
        contract_address: ContractAddress,
        sequencer_address: ContractAddress,
    ) {
        self.cheat_sequencer_address(contract_address, sequencer_address, CheatSpan::Indefinite);
    }

    pub fn start_cheat_sequencer_address_global(&mut self, sequencer_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                sequencer_address: Operation::StartGlobal(sequencer_address),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn stop_cheat_sequencer_address(&mut self, contract_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                sequencer_address: Operation::Stop(contract_address),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn stop_cheat_sequencer_address_global(&mut self) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            block_info: BlockInfoMockOperations {
                sequencer_address: Operation::StopGlobal,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}
