use super::{BufferReadError, BufferReadResult, BufferReader, CairoDeserialize};
use crate::{IntoConv, byte_array::ByteArray};
use num_traits::cast::ToPrimitive;
use starknet::{core::types::U256, providers::Url};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::num::NonZero;

impl CairoDeserialize for Url {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let url: String = reader.read::<ByteArray>()?.to_string();
        Url::parse(&url).map_err(|_| BufferReadError::ParseFailed)
    }
}

impl CairoDeserialize for Felt {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        reader.read_felt()
    }
}

impl<T> CairoDeserialize for Vec<T>
where
    T: CairoDeserialize,
{
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let length: Felt = reader.read()?;
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
        let variant: Felt = reader.read()?;
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
        let num: usize = reader.read()?;

        match num {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BufferReadError::ParseFailed),
        }
    }
}

impl CairoDeserialize for NonZeroFelt {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let felt = reader.read::<Felt>()?;
        NonZeroFelt::try_from(felt).map_err(|_| BufferReadError::ParseFailed)
    }
}

macro_rules! impl_deserialize_for_nonzero_num_type {
    ($type:ty) => {
        impl CairoDeserialize for NonZero<$type> {
            fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                let val = <$type>::deserialize(reader)?;
                NonZero::new(val).ok_or(BufferReadError::ParseFailed)
            }
        }
    };
}

macro_rules! impl_deserialize_for_felt_type {
    ($type:ty) => {
        impl CairoDeserialize for $type {
            fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                Felt::deserialize(reader).map(IntoConv::into_)
            }
        }
    };
}

macro_rules! impl_deserialize_for_num_type {
    ($type:ty) => {
        impl CairoDeserialize for $type {
            fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                let felt = Felt::deserialize(reader)?;
                felt.try_into().map_err(|_| BufferReadError::ParseFailed)
            }
        }
    };
}

impl_deserialize_for_felt_type!(ClassHash);
impl_deserialize_for_felt_type!(ContractAddress);
impl_deserialize_for_felt_type!(Nonce);
impl_deserialize_for_felt_type!(EntryPointSelector);

impl_deserialize_for_nonzero_num_type!(u32);
impl_deserialize_for_nonzero_num_type!(u64);
impl_deserialize_for_nonzero_num_type!(u128);
impl_deserialize_for_nonzero_num_type!(usize);

impl_deserialize_for_num_type!(u8);
impl_deserialize_for_num_type!(u16);
impl_deserialize_for_num_type!(u32);
impl_deserialize_for_num_type!(u64);
impl_deserialize_for_num_type!(u128);
impl_deserialize_for_num_type!(U256);
impl_deserialize_for_num_type!(usize);

impl_deserialize_for_num_type!(i8);
impl_deserialize_for_num_type!(i16);
impl_deserialize_for_num_type!(i32);
impl_deserialize_for_num_type!(i64);
impl_deserialize_for_num_type!(i128);
