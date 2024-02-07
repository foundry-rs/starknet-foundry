#[starknet::contract]
mod DictUsingContract {
    use core::num::traits::{One};

    fn unique_count(mut ary: Array<felt252>) -> u32 {
        let mut dict: Felt252Dict<felt252> = Default::default();
        let mut counter = 0;
        loop {
            match ary.pop_front() {
                Option::Some(value) => {
                    if dict.get(value).is_one() {
                        continue;
                    }
                    dict.insert(value, One::one());
                    counter += 1;
                },
                Option::None => { break; }
            }
        };
        counter
    }

    #[storage]
    struct Storage {
        unique_count: u32
    }

    #[constructor]
    fn constructor(ref self: ContractState, values: Array<felt252>) {
        self.unique_count.write(unique_count(values.clone()));  // 2 invocations for 2 dict allocations
        self.unique_count.write(unique_count(values));
    }

    #[external(v0)]
    fn get_unique(self: @ContractState) -> u32 {
        self.unique_count.read()
    }

    #[external(v0)]
    fn write_unique(ref self: ContractState, values: Array<felt252>) {
        self.unique_count.write(unique_count(values.clone()));  // 2 invocations for 2 dict allocations
        self.unique_count.write(unique_count(values));
    }
}
