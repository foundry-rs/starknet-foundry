use crate::FromConv;
use starknet_api::core::EthAddress;
use starknet_types_core::felt::Felt as Felt252;

impl FromConv<Felt252> for EthAddress {
    fn from_(value: Felt252) -> EthAddress {
        EthAddress::try_from(value).expect("Conversion of felt252 to EthAddress failed")
    }
}

impl FromConv<EthAddress> for Felt252 {
    fn from_(value: EthAddress) -> Felt252 {
        value.into()
    }
}
