#[derive(Clone, Debug, PartialEq, Drop, Serde, starknet::Store)]
pub struct Pokemon {
    pub name: ByteArray,
    pub element: Element,
    pub likes: felt252,
    pub owner: starknet::ContractAddress
}

#[allow(starknet::store_no_default_variant)]
#[derive(Copy, Debug, PartialEq, Drop, Serde, starknet::Store)]
pub enum Element {
    Fire,
    Water,
    Grass
}

#[starknet::interface]
pub trait IPokemonGallery<TContractState> {
    fn like(ref self: TContractState, name: ByteArray);
    fn all(self: @TContractState) -> Array<Pokemon>;
    fn pokemon(self: @TContractState, name: ByteArray) -> Option<Pokemon>;
    fn liked(self: @TContractState) -> Array<Pokemon>;
}
