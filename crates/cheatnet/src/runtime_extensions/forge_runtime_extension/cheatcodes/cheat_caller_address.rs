use super::cheat_execution_info::{CheatArguments, ExecutionInfoMockOperations, Operation};
use crate::state::CheatSpan;
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn cheat_caller_address(
        &mut self,
        target: ContractAddress,
        caller_address: ContractAddress,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            caller_address: Operation::Start(CheatArguments {
                value: caller_address,
                span,
                target,
            }),
            ..Default::default()
        });
    }

    pub fn cheat_caller_address_global(&mut self, caller_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            caller_address: Operation::StartGlobal(caller_address),
            ..Default::default()
        });
    }

    pub fn start_cheat_caller_address(
        &mut self,
        target: ContractAddress,
        caller_address: ContractAddress,
    ) {
        self.cheat_caller_address(target, caller_address, CheatSpan::Indefinite);
    }

    pub fn stop_cheat_caller_address(&mut self, target: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            caller_address: Operation::Stop(target),
            ..Default::default()
        });
    }

    pub fn stop_cheat_caller_address_global(&mut self) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            caller_address: Operation::StopGlobal,
            ..Default::default()
        });
    }
}
