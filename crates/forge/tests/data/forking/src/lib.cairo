#[cfg(test)]
mod tests {
    
    
    #[starknet::interface]
    trait IHelloStarknet<TContractState> {
        fn increase_balance(ref self: TContractState, amount: felt252);
        fn get_balance(self: @TContractState) -> felt252;
    }

  
    
    

  

    #[test]
    
    fn print_block_number_when_latest() {
        assert(1 == 1, '');
    }
}
