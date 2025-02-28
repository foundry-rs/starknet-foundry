use crate::{from_thru_felt, FromConv, IntoConv};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt;

impl FromConv<Felt> for Nonce {
    fn from_(value: Felt) -> Nonce {
        Nonce(value.into_())
    }
}

from_thru_felt!(ClassHash, Nonce);
from_thru_felt!(ContractAddress, Nonce);
from_thru_felt!(EntryPointSelector, Nonce);
