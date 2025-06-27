#[starknet::interface]
pub trait IMessageSender<TContractState> {
    fn greet_ethereum(ref self: TContractState, receiver: felt252);
}

#[starknet::contract]
pub mod MessageSender {
    #[storage]
    struct Storage {}
    use starknet::syscalls::send_message_to_l1_syscall;

    #[external(v0)]
    pub fn greet_ethereum(ref self: ContractState, receiver: felt252) {
        let payload = array!['hello'];
        send_message_to_l1_syscall(receiver, payload.span()).unwrap();
    }
}
