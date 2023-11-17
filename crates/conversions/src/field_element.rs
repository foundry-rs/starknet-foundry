use crate::FromConv;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;

impl FromConv<Felt252> for FieldElement {
    fn from_(value: Felt252) -> FieldElement {
        FieldElement::from_bytes_be(&value.to_be_bytes()).unwrap()
    }
}
