use crate::CheatnetState;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
    CheatArguments, Operation,
};
use crate::state::CheatSpan;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::execution::syscalls::syscall_base::SyscallResult;
use starknet_api::block::BlockHash;
use starknet_api::core::ContractAddress;
use starknet_api::hash::StarkHash;
use starknet_types_core::felt::Felt;
use std::num::NonZeroUsize;

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

                if let Some((_, except)) = self.global_block_hash.get_mut(&block_number) {
                    except.push(contract_address);
                }
            }
            Operation::StartGlobal(block_hash) => {
                self.global_block_hash
                    .insert(block_number, (block_hash, vec![]));

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

    pub fn get_cheated_block_hash_for_contract(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
    ) -> Option<BlockHash> {
        if let Some((cheat_span, block_hash)) = self
            .block_hash_contracts
            .get(&(contract_address, block_number))
            .copied()
        {
            match cheat_span {
                CheatSpan::TargetCalls(n) if n.get() == 1 => {
                    self.block_hash_contracts
                        .remove(&(contract_address, block_number));
                }
                CheatSpan::TargetCalls(num) => {
                    let calls_number = num.get() - 1;
                    self.block_hash_contracts.insert(
                        (contract_address, block_number),
                        (
                            CheatSpan::TargetCalls(
                                NonZeroUsize::new(calls_number)
                                    .expect("`NonZeroUsize` should not be zero after decrement"),
                            ),
                            block_hash,
                        ),
                    );
                }
                CheatSpan::Indefinite => {}
            }
            return Some(BlockHash(StarkHash::from(block_hash)));
        }

        if let Some((block_hash, except)) = self.global_block_hash.get(&block_number)
            && !except.contains(&contract_address)
        {
            return Some(BlockHash(StarkHash::from(*block_hash)));
        }

        None
    }

    #[expect(clippy::result_large_err)]
    pub fn get_block_hash_for_contract(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
        syscall_handler: &mut SyscallHintProcessor,
    ) -> SyscallResult<BlockHash> {
        let cheated_block_hash =
            self.get_cheated_block_hash_for_contract(contract_address, block_number);
        if let Some(cheated_block_hash) = cheated_block_hash {
            Ok(cheated_block_hash)
        } else {
            Ok(BlockHash(
                syscall_handler.base.get_block_hash(block_number)?,
            ))
        }
    }
}
