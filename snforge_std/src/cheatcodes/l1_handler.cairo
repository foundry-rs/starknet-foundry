use core::array::SpanTrait;
use core::serde::Serde;
use starknet::{ContractAddress, testing::cheatcode, SyscallResult};
use super::super::_cheatcode::handle_cheatcode;

#[derive(Drop, Clone)]
pub struct L1Handler {
    pub target: ContractAddress,
    pub selector: felt252,
}

pub trait L1HandlerTrait {
    fn new(target: ContractAddress, selector: felt252) -> L1Handler;
    fn execute(
        self: L1Handler, from_address: felt252, payload: Span::<felt252>
    ) -> SyscallResult<()>;
}

impl L1HandlerImpl of L1HandlerTrait {
    /// `target` - The target starknet contract address
    /// `selector` - Selector of a `#[l1_handler]` function. Can be acquired with
    /// `selector!("function_handler_name")` macro Returns a structure referring to a L1 handler
    /// function
    fn new(target: ContractAddress, selector: felt252) -> L1Handler {
        L1Handler { target, selector, }
    }

    /// Mocks L1 -> L2 message from Ethereum handled by the given L1 handler function
    /// `self` - `L1Handler` structure referring to a L1 handler function
    /// `from_address` - Ethereum address of the contract that you want to be the message sender
    /// `payload` - The handlers' function arguments serialized with `Serde`
    /// Returns () or panic data if it failed
    fn execute(
        self: L1Handler, from_address: felt252, payload: Span::<felt252>
    ) -> SyscallResult<()> {
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
