#[contract]
mod ConstructorWithParams {
    struct Storage {
        value1: felt252,
        value2: u256,
    }

    #[constructor]
    fn constructor(first: felt252, second: u256) {
        value1::write(first);
        value2::write(second);
    }
}
