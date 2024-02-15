use starknet::ContractAddress;

#[starknet::interface]
trait ICairo1Contract<TContractState> {
    fn start(
        ref self: TContractState, cairo0_address: ContractAddress, pranked_address: ContractAddress
    );
    fn end(self: @TContractState);
}

//https://testnet.starkscan.co/contract/0x034dad9a1512fcb0d33032c65f4605a073bdc42f70e61524510e5760c2b4f544
#[starknet::interface]
trait ICairo0Contract<TContractState> {
    fn callback(ref self: TContractState, contract_address: felt252);
}

#[starknet::contract]
mod Cairo1Contract_v1 {
    use starknet::{get_contract_address, get_caller_address, ContractAddress};
    use super::ICairo0ContractDispatcherTrait;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICairo1ContractImpl of super::ICairo1Contract<ContractState> {
        fn start(
            ref self: ContractState,
            cairo0_address: ContractAddress,
            pranked_address: ContractAddress
        ) {
            let contract_address = get_contract_address();

            let cairo0_contract = super::ICairo0ContractDispatcher {
                contract_address: cairo0_address
            };

            assert(pranked_address == get_caller_address(), 'address should be pranked');

            cairo0_contract.callback(contract_address.into());

            assert(pranked_address == get_caller_address(), 'address should be pranked');
        }

        fn end(self: @ContractState) {
            assert(123.try_into().unwrap() == get_caller_address(), 'should be same');
        }
    }
}
