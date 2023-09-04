use starknet::{ContractAddress, testing::cheatcode};
use traits::Into;


#[derive(Drop, Clone)]
struct L1Handler {
    contract_address: ContractAddress,
    function_name: felt252,
    from_address: felt252,
    fee: u128,
    payload: Span::<felt252>,
}

trait L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_name: felt252) -> L1Handler;
    fn execute(self: L1Handler);
}

impl L1HandlerImpl of L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_name: felt252) -> L1Handler {
        L1Handler {
            contract_address, function_name, from_address: 0, fee: 1_u128, payload: array![].span(),
        }
    }

    fn execute(self: L1Handler) {
        let mut inputs: Array::<felt252> = array![
            self.contract_address.into(),
            self.function_name,
            self.from_address,
            self.fee.into(),
            self.payload.len().into(),
        ];

        let payload_len = self.payload.len();
        let mut i = 0;
        loop {
            if payload_len == i {
                break ();
            }
            inputs.append(*self.payload[i]);
            i += 1;
        };

        cheatcode::<'l1_handler_execute'>(inputs.span());
    }
}
