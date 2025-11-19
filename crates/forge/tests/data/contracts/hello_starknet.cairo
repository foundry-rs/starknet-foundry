#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn call_other_contract(
        self: @TContractState,
        other_contract_address: felt252,
        selector: felt252,
        calldata: Option<Array<felt252>>,
    ) -> Span<felt252>;
    fn do_a_panic(self: @TContractState);
    fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
    fn do_a_panic_with_bytearray(self: @TContractState);
}

#[starknet::contract]
mod HelloStarknet {
    use array::ArrayTrait;
    use starknet::{SyscallResultTrait, syscalls};

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Returns the current balance
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }

        fn call_other_contract(
            self: @ContractState,
            other_contract_address: felt252,
            selector: felt252,
            calldata: Option<Array<felt252>>,
        ) -> Span<felt252> {
            syscalls::call_contract_syscall(
                other_contract_address.try_into().unwrap(),
                selector,
                match calldata {
                    Some(data) => data.span(),
                    None => array![].span(),
                },
            )
                .unwrap_syscall()
        }

        // Panics
        fn do_a_panic(self: @ContractState) {
            let mut arr = ArrayTrait::new();
            arr.append('PANIC');
            arr.append('DAYTAH');
            panic(arr);
        }

        // Panics with given array data
        fn do_a_panic_with(self: @ContractState, panic_data: Array<felt252>) {
            panic(panic_data);
        }

        // Panics with a bytearray
        fn do_a_panic_with_bytearray(self: @ContractState) {
            assert!(
                false,
                "This is a very long\n and multiline message that is certain to fill the buffer",
            );
        }
    }
}
