use super::{BufferReadError, BufferReadResult, BufferReader, CairoDeserialize};
use crate::IntoConv;
use cairo_felt::Felt252;
use num_traits::cast::ToPrimitive;
use starknet::core::types::FieldElement;
use starknet_api::{
    core::{ClassHash, ContractAddress, EntryPointSelector, Nonce},
    hash::StarkFelt,
};

impl CairoDeserialize for Felt252 {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        reader.read_felt()
    }
}

impl<T> CairoDeserialize for Vec<T>
where
    T: CairoDeserialize,
{
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let length: Felt252 = reader.read()?;
        let length = length.to_usize().ok_or(BufferReadError::ParseFailed)?;

        let mut result = Vec::with_capacity(length);

        for _ in 0..length {
            result.push(reader.read()?);
        }

        Ok(result)
    }
}

impl<T> CairoDeserialize for Option<T>
where
    T: CairoDeserialize,
{
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let variant: Felt252 = reader.read()?;
        let variant: usize = variant.to_usize().ok_or(BufferReadError::ParseFailed)?;

        match variant {
            0 => Ok(Some(reader.read()?)),
            1 => Ok(None),
            _ => Err(BufferReadError::ParseFailed),
        }
    }
}

impl CairoDeserialize for bool {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        reader.read::<Felt252>().map(|felt| felt == 1.into())
    }
}

macro_rules! impl_deserialize_for_felt_type {
    ($type:ty) => {
        impl CairoDeserialize for $type {
            fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                Felt252::deserialize(reader).map(IntoConv::into_)
            }
        }
    };
}
macro_rules! impl_deserialize_for_num_type {
    ($type:ty) => {
        impl CairoDeserialize for $type {
            fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                let felt = Felt252::deserialize(reader)?;

                felt.to_bigint()
                    .try_into()
                    .map_err(|_| BufferReadError::ParseFailed)
            }
        }
    };
}

impl_deserialize_for_felt_type!(FieldElement);
impl_deserialize_for_felt_type!(ClassHash);
impl_deserialize_for_felt_type!(StarkFelt);
impl_deserialize_for_felt_type!(ContractAddress);
impl_deserialize_for_felt_type!(Nonce);
impl_deserialize_for_felt_type!(EntryPointSelector);

impl_deserialize_for_num_type!(u8);
impl_deserialize_for_num_type!(u16);
impl_deserialize_for_num_type!(u32);
impl_deserialize_for_num_type!(u64);
impl_deserialize_for_num_type!(u128);
impl_deserialize_for_num_type!(usize);
