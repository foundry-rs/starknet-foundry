#[starknet::interface]
trait IResponseWith2Felts<TContractState> {
    fn get(self: @TContractState) -> Response;
}

#[derive(Drop, Serde)]
struct Response {
    a: felt252,
    b: felt252,
}

#[starknet::contract]
mod ResponseWith2Felts {
    use core::array::ArrayTrait;
    use super::Response;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IResponseWith2FeltsImpl of super::IResponseWith2Felts<ContractState> {
        // Increases the balance by the given amount
        fn get(self: @ContractState) -> Response {
            Response { a: 8, b: 43 }
        }
    }
}
