use crate::common::felt_selector_from_name;
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::calculate_variable_address;

mod declare;
mod deploy;
mod elect;
mod get_class_hash;
mod load;
mod mock_call;
mod prank;
mod precalculate_address;
mod roll;
mod spoof;
mod spy_events;
mod store;
mod warp;

pub fn map_entry_address(var_name: &str, key: &[Felt252]) -> Felt252 {
    calculate_variable_address(&felt_selector_from_name(var_name), Some(key))
}

pub fn variable_address(var_name: &str) -> Felt252 {
    calculate_variable_address(&felt_selector_from_name(var_name), None)
}
