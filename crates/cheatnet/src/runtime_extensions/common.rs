use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use conversions::IntoConv;
use starknet_api::core::EntryPointSelector;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;

pub fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    Calldata(calldata.into())
}

#[must_use]
pub fn create_entry_point_selector(entry_point_selector: &Felt252) -> EntryPointSelector {
    EntryPointSelector((*entry_point_selector).clone().into_())
}
