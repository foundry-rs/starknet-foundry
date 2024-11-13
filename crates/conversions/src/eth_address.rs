use crate::FromConv;
use starknet_api::core::EthAddress;
use starknet_types_core::felt::Felt;

impl FromConv<Felt> for EthAddress {
    fn from_(value: Felt) -> EthAddress {
        EthAddress::try_from(value).expect("Conversion of felt to EthAddress failed")
    }
}

impl FromConv<EthAddress> for Felt {
    fn from_(value: EthAddress) -> Felt {
        value.into()
    }
}
