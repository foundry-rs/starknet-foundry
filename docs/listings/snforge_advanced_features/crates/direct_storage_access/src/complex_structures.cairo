use core::hash::LegacyHash;

// Required for lookup of complex_mapping values
// This is consistent with `map_entry_address`, which uses pedersen hashing of keys
impl StructuredKeyHash of LegacyHash<MapKey> {
    fn hash(state: felt252, value: MapKey) -> felt252 {
        let state = LegacyHash::<felt252>::hash(state, value.a);
        LegacyHash::<felt252>::hash(state, value.b)
    }
}

#[derive(Copy, Drop, Serde)]
pub struct MapKey {
    pub a: felt252,
    pub b: felt252,
}

// Serialization of keys and values with `Serde` to make usage of `map_entry_address` easier
impl MapKeyIntoSpan of Into<MapKey, Span<felt252>> {
    fn into(self: MapKey) -> Span<felt252> {
        let mut serialized_struct: Array<felt252> = array![];
        self.serialize(ref serialized_struct);
        serialized_struct.span()
    }
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct MapValue {
    pub a: felt252,
    pub b: felt252,
}

impl MapValueIntoSpan of Into<MapValue, Span<felt252>> {
    fn into(self: MapValue) -> Span<felt252> {
        let mut serialized_struct: Array<felt252> = array![];
        self.serialize(ref serialized_struct);
        serialized_struct.span()
    }
}

#[starknet::interface]
pub trait IComplexStorageContract<TContractState> {}

#[starknet::contract]
mod ComplexStorageContract {
    use starknet::storage::Map;
    use super::{MapKey, MapValue};

    #[storage]
    struct Storage {
        complex_mapping: Map<MapKey, MapValue>,
    }
}
