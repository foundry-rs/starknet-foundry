use std::fmt::Display;

use itertools::Itertools;
use starknet_rust::core::types::contract::AbiNamedMember;

/// Formats ABI members as a name: type list, e.g. a: `core::felt252`, b: `core::integer::u32`.                                             
pub fn format_abi_members(members: &[AbiNamedMember]) -> String {
    members
        .iter()
        .map(|member| format!("{}: {}", member.name, member.r#type))
        .join(", ")
}

/// Appends an indented passed: / expected: comparison to an error headline.                                                            
pub fn format_passed_vs_expected(
    headline: impl Display,
    passed: impl Display,
    expected: impl Display,
) -> String {
    format!("{headline}\n  passed: {passed}\n  expected: {expected}")
}
