use crate::{FromConv, IntoConv, from_thru_felt};
use conversions::padded_felt::PaddedFelt;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt;

impl FromConv<Felt> for ClassHash {
    fn from_(value: Felt) -> ClassHash {
        ClassHash(value.into_())
    }
}

from_thru_felt!(ContractAddress, ClassHash);
from_thru_felt!(Nonce, ClassHash);
from_thru_felt!(EntryPointSelector, ClassHash);
from_thru_felt!(PaddedFelt, ClassHash);
