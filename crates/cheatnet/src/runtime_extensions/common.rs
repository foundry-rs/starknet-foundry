use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;

pub fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    Calldata(calldata.into())
}
