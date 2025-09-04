use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
        CheatArguments, ExecutionInfoMockOperations, Operation, TxInfoMockOperations,
    },
    state::CheatnetState,
};
use starknet_types_core::felt::Felt;

impl CheatnetState {
    pub fn cheat_account_contract_address(&mut self, operation: Operation<Felt>) {
        match operation {
            Operation::Start(args) => {
                self.cheat_execution_info(ExecutionInfoMockOperations {
                    tx_info: TxInfoMockOperations {
                        account_contract_address: Operation::Start(CheatArguments {
                            value: args.value,
                            span: args.span,
                            target: args.target,
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            },
            Operation::Stop(contract_address) => todo!(),
            Operation::StopGlobal => todo!(),
            Operation::StartGlobal(_) => todo!(),
            Operation::Retain => todo!(),
        }
    }
}
