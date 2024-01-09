use starknet::{ContractAddress, testing::cheatcode};
use snforge_std::cheatcodes::contract_class::RevertedTransaction;

#[derive(Drop, Clone)]
struct L1Handler {
    contract_address: ContractAddress,
    function_name: felt252,
    from_address: felt252,
    payload: Span::<felt252>,
}

trait L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_name: felt252) -> L1Handler;
    fn execute(self: L1Handler) -> Result::<(), RevertedTransaction>;
}

impl L1HandlerImpl of L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_name: felt252) -> L1Handler {
        L1Handler { contract_address, function_name, from_address: 0, payload: array![].span(), }
    }

    fn execute(self: L1Handler) -> Result::<(), RevertedTransaction> {
        let mut inputs: Array::<felt252> = array![
            self.contract_address.into(),
            self.function_name,
            self.from_address,
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

        let outputs = cheatcode::<'l1_handler_execute'>(inputs.span());
        let exit_code = *outputs[0];

        if exit_code == 0 {
            Result::<(), RevertedTransaction>::Ok(())
        } else {
            let panic_data_len_felt = *outputs[1];
            let panic_data_len = panic_data_len_felt.try_into().unwrap();
            let mut panic_data = array![];

            let offset = 2;
            let mut i = offset;
            loop {
                if panic_data_len + offset == i {
                    break ();
                }
                panic_data.append(*outputs[i]);
                i += 1;
            };

            Result::<(), RevertedTransaction>::Err(RevertedTransaction { panic_data })
        }
    }
}
