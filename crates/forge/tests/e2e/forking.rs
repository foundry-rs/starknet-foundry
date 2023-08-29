use crate::integration::common::running_tests::{run_fork_test_case};
use crate::{assert_passed, test_case};
use indoc::indoc;

#[test]
fn simple_call_and_invoke() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, ContractClassTrait };
        use starknet::contract_address_const;

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn do_a_panic(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
        }

        #[test]
        fn call_and_invoke() {
            let dispatcher = IHelloStarknetDispatcher {
                contract_address: contract_address_const::<3216637956526895219277698311134811322769343974163380838558193911733621219342>()
            };

            let balance = dispatcher.get_balance();
            assert(balance == 2, 'balance == 2');

            dispatcher.increase_balance(100);

            let balance = dispatcher.get_balance();
            assert(balance == 102, 'balance == 102');
        }
    "#
        ),
    );

    let result = run_fork_test_case(&test);

    assert_passed!(result);
}
