use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
        CheatArguments, ExecutionInfoMockOperations, Operation, TxInfoMockOperations,
    },
    state::{CheatSpan, CheatnetState},
};
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            tx_info: TxInfoMockOperations {
                account_contract_address: Operation::Start(CheatArguments {
                    value: account_contract_address.into(),
                    span,
                    target,
                }),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
    ) {
        self.cheat_account_contract_address(
            target,
            account_contract_address,
            CheatSpan::Indefinite,
        );
    }

    pub fn stop_cheat_account_contract_address(&mut self, target: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            tx_info: TxInfoMockOperations {
                account_contract_address: Operation::Stop(target),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn start_cheat_account_contract_address_global(
        &mut self,
        account_contract_address: ContractAddress,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            tx_info: TxInfoMockOperations {
                account_contract_address: Operation::StartGlobal(account_contract_address.into()),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    pub fn stop_cheat_account_contract_address_global(
        &mut self,
        account_contract_address: ContractAddress,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            tx_info: TxInfoMockOperations {
                account_contract_address: Operation::StartGlobal(account_contract_address.into()),
                ..Default::default()
            },
            ..Default::default()
        });
    }
}
