use starknet::ContractAddress;

#[starknet::interface]
trait ICairo1Contract<TContractState> {
    fn start(
        ref self: TContractState,
        cairo0_address: ContractAddress,
        expected_caller_address: ContractAddress,
    );
    fn end(ref self: TContractState);
}

//https://testnet.starkscan.co/contract/0x034dad9a1512fcb0d33032c65f4605a073bdc42f70e61524510e5760c2b4f544
#[starknet::interface]
trait ICairo0Contract<TContractState> {
    fn callback(ref self: TContractState, contract_address: felt252);
}

#[starknet::contract]
mod Cairo1Contract_v1 {
    use core::traits::Into;
    use starknet::{get_contract_address, get_caller_address, ContractAddress};
    use super::ICairo0ContractDispatcherTrait;

    #[storage]
    struct Storage {
        expected_caller_address: ContractAddress,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        End: EndCalled
    }

    #[derive(Drop, starknet::Event)]
    struct EndCalled {
        expected_caller_address: felt252,
    }

    #[abi(embed_v0)]
    impl ICairo1ContractImpl of super::ICairo1Contract<ContractState> {
        fn start(
            ref self: ContractState,
            cairo0_address: ContractAddress,
            expected_caller_address: ContractAddress
        ) {
            let contract_address = get_contract_address();

            let cairo0_contract = super::ICairo0ContractDispatcher {
                contract_address: cairo0_address
            };

            self.expected_caller_address.write(expected_caller_address);

            assert(expected_caller_address == get_caller_address(), 'address should be pranked');

            cairo0_contract.callback(contract_address.into());

            assert(expected_caller_address == get_caller_address(), 'address should be pranked');
        }

        fn end(ref self: ContractState) {
            let expected_caller_address = self.expected_caller_address.read();

            assert(expected_caller_address == get_caller_address(), 'should be same');

            self
                .emit(
                    Event::End(
                        EndCalled { expected_caller_address: expected_caller_address.into() }
                    )
                );
        }
    }
}
