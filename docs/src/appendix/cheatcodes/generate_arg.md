# `generate_arg`

> `fn generate_arg<T, +Serde<T>, +Drop<T>, +Into<T, felt252>>(min_value: T, max_value: T) -> T`

Returns a random number from a range `[min_value, max_value]`.
It is used in the context of fuzz testing to implement [`Fuzzable`](../snforge-library/fuzzable.md) trait.
