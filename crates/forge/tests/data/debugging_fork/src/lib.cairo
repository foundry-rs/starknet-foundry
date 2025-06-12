use starknet::ContractAddress;

#[derive(Drop, Serde, Clone)]
struct RecursiveCall {
    contract_address: ContractAddress,
    payload: Array<RecursiveCall>,
}

// `RecursiveCaller` is implemented in the `debugging` package
#[starknet::interface]
trait RecursiveCaller<T> {
    fn execute_calls(self: @T, calls: Array<RecursiveCall>) -> Array<RecursiveCall>;
}

// `Failing` is implemented in the `debugging` package
#[starknet::interface]
trait Failing<TContractState> {
    fn fail(self: @TContractState, data: Array<felt252>);
}
