use starknet::{ContractAddress, EthAddress};

#[starknet::interface]
trait IMessageToL1Checker<TContractState> {
    fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
}

#[starknet::contract]
mod MessageToL1Checker {
    use starknet::{ContractAddress, EthAddress, send_message_to_l1_syscall};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IMessageToL1Checker of super::IMessageToL1Checker<ContractState> {
        fn send_message(ref self: ContractState, some_data: Array<felt252>, to_address: EthAddress) {
            send_message_to_l1_syscall(to_address.into(), some_data.span()).unwrap()
        }
    }
}
