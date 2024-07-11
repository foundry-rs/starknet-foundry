use crate::{from_thru_felt252, FromConv, IntoConv};
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::EthAddress;
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for EthAddress {
    fn from_(value: Felt252) -> EthAddress {
        let sf: StarkFelt = value.into_();
        EthAddress::try_from(sf).expect("Conversion of felt252 to EthAddress failed")
    }
}

impl FromConv<EthAddress> for Felt252 {
    fn from_(value: EthAddress) -> Felt252 {
        let sf: StarkFelt = value.into();
        sf.into_()
    }
}

from_thru_felt252!(FieldElement, EthAddress);
from_thru_felt252!(StarkFelt, EthAddress);
