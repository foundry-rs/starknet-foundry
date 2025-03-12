use crate::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
    CheatArguments, Operation,
};
use crate::state::CheatSpan;
use crate::{CHEAT_MAGIC, CheatnetState};
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

impl CheatnetState {
    pub fn cheat_block_hash(&mut self, block_number: u64, operation: Operation<Felt>) {
        match operation {
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

                if let Some(_entry) = self.global_block_hash.get(&block_number) {
                    self.block_hash_contracts.insert(
                        (contract_address, block_number),
                        (CheatSpan::TargetCalls(CHEAT_MAGIC), Felt::from(CHEAT_MAGIC)),
                    );
                }
            }
            Operation::StartGlobal(block_hash) => {
                self.global_block_hash.insert(block_number, block_hash);

                self.block_hash_contracts
                    .retain(|(_, bn), _| *bn != block_number);
            }
            Operation::StopGlobal => {
                self.global_block_hash.remove(&block_number);

                self.block_hash_contracts
                    .retain(|(_, bn), _| *bn != block_number);
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
        self.cheat_block_hash(
            block_number,
            Operation::Start(CheatArguments {
                value: block_hash,
                span: CheatSpan::Indefinite,
                target: contract_address,
            }),
        );
    }

    pub fn stop_cheat_block_hash(&mut self, contract_address: ContractAddress, block_number: u64) {
        self.cheat_block_hash(block_number, Operation::Stop(contract_address));
    }

    pub fn start_cheat_block_hash_global(&mut self, block_number: u64, block_hash: Felt) {
        self.cheat_block_hash(block_number, Operation::StartGlobal(block_hash));
    }

    pub fn stop_cheat_block_hash_global(&mut self, block_number: u64) {
        self.cheat_block_hash(block_number, Operation::StopGlobal);
    }
}
