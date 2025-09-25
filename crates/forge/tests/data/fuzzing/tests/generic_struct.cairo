use snforge_std::fuzzable::Fuzzable;

#[derive(Debug, Drop)]
struct Price<T> {
    amount: T,
}

impl FuzzablePriceU64 of Fuzzable<Price<u64>> {
    fn blank() -> Price<u64> {
        Price { amount: 0 }
    }

    fn generate() -> Price<u64> {
        Price { amount: Fuzzable::generate() }
    }
}

#[fuzzer]
#[test]
fn test_generic(price: Price<u64>) {
    assert(price.amount > 0, 'Wrong price');
}
