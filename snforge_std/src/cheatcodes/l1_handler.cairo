use core::array::SpanTrait;
use core::serde::Serde;
use starknet::{ContractAddress, testing::cheatcode, SyscallResult};
use super::super::_cheatcode::handle_cheatcode;

#[derive(Drop, Clone)]
struct L1Handler {
    target: ContractAddress,
    selector: felt252,
}

trait L1HandlerTrait {
    fn new(target: ContractAddress, selector: felt252) -> L1Handler;
    fn execute(self: L1Handler, from_address: felt252, payload: Span::<felt252>) -> SyscallResult<()>;
}

impl L1HandlerImpl of L1HandlerTrait {
    /// `target` - The target starknet contract address
    /// `selector` - Selector of a `#[l1_handler]` function
    /// Returns a structure referring to a L1 handler function
    fn new(target: ContractAddress, selector: felt252) -> L1Handler {
        L1Handler {
            target, selector,
        }
    }

    /// `self` - `L1Handler` structure refering to a L1 handler function
    /// `from_address` - Ethereum address of the contract that you want to a emulate message from
    /// `payload` - The message payload that may contain any Cairo data structure that can be serialized with
    /// Mocks a L1 -> L2 message from Ethereum handled by the given L1 handler function.
    fn execute(self: L1Handler, from_address: felt252, payload: Span::<felt252>) -> SyscallResult<()> {
        let mut inputs: Array::<felt252> = array![
            self.target.into(), self.selector, from_address.into(),
        ];
        payload.serialize(ref inputs);

        let mut outputs = handle_cheatcode(cheatcode::<'l1_handler_execute'>(inputs.span()));
        let exit_code = *outputs.pop_front().unwrap();

        if exit_code == 0 {
            SyscallResult::Ok(())
        } else {
            let panic_data = Serde::<Array<felt252>>::deserialize(ref outputs).unwrap();
            SyscallResult::Err(panic_data)
        }
    }
}
