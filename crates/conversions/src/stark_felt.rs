use crate::FromConv;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for StarkFelt {
    fn from_(value: Felt252) -> StarkFelt {
        felt_to_stark_felt(&value)
    }
}
