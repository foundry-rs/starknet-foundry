use core::array::SpanTrait;
use core::serde::Serde;
use starknet::{ContractAddress, testing::cheatcode, SyscallResult};
use super::super::_cheatcode::handle_cheatcode;

#[derive(Drop, Clone)]
struct L1Handler {
    contract_address: ContractAddress,
    function_selector: felt252,
    from_address: felt252,
    payload: Span::<felt252>,
}

trait L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_selector: felt252) -> L1Handler;
    fn execute(self: L1Handler) -> SyscallResult<()>;
}

impl L1HandlerImpl of L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_selector: felt252) -> L1Handler {
        L1Handler {
            contract_address, function_selector, from_address: 0, payload: array![].span(),
        }
    }

    fn execute(self: L1Handler) -> SyscallResult<()> {
        let mut inputs: Array::<felt252> = array![
            self.contract_address.into(), self.function_selector, self.from_address,
        ];
        self.payload.serialize(ref inputs);

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
