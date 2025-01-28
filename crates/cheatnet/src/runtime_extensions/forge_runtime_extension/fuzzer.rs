use anyhow::ensure;
use num_bigint::RandBigInt;
use rand::prelude::StdRng;
use starknet_types_core::felt::Felt;
use std::sync::{Arc, Mutex};

pub(crate) fn generate_arg(
    fuzzer_rng: Option<Arc<Mutex<StdRng>>>,
    min_value: Felt,
    max_value: Felt,
) -> anyhow::Result<Felt> {
    let min_big_int = if min_value > (Felt::MAX + Felt::from(i128::MIN)) && min_value > max_value {
        // Negative value x is serialized as P + x, where P is the STARK prime number
        // hence to deserialize and get the actual x we need to subtract P (== Felt::MAX + 1)
        min_value.to_bigint() - Felt::MAX.to_bigint() - 1
    } else {
        min_value.to_bigint()
    };

    let max_big_int = max_value.to_bigint();

    ensure!(
        min_big_int <= max_big_int,
        format!("`generate_arg` cheatcode: `min_value` must be <= `max_value`, provided values after deserialization: {min_big_int} and {max_big_int}")
    );

    let value = if let Some(fuzzer_rng) = fuzzer_rng {
        fuzzer_rng
            .lock()
            .unwrap()
            .gen_bigint_range(&min_big_int, &(max_big_int + 1))
    } else {
        // `generate_arg` cheatcode can be also used outside the fuzzer context
        rand::thread_rng().gen_bigint_range(&min_big_int, &(max_big_int + 1))
    };

    Ok(Felt::from(value))
}
