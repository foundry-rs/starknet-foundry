#[starknet::interface]
trait ITraceDummy<T> {
    fn from_proxy_dummy(ref self: T);
}

#[starknet::contract]
mod TraceDummy {
    #[storage]
    struct Storage {
        balance: u8
    }

    #[abi(embed_v0)]
    impl ITraceDummyImpl of super::ITraceDummy<ContractState> {
        fn from_proxy_dummy(ref self: ContractState) {
            self.balance.write(7);
        }
    }
}
