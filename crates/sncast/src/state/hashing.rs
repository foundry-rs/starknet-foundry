use sha3::Digest;
use sha3::Sha3_256;
use starknet_types_core::felt::Felt;
use std::vec;

trait SerialiseAsBytes {
    fn serialise_as_bytes(&self) -> Vec<u8>;
}

impl<T: SerialiseAsBytes> SerialiseAsBytes for Option<T> {
    fn serialise_as_bytes(&self) -> Vec<u8> {
        match self {
            None => {
                vec![0]
            }
            Some(val) => {
                let mut res = vec![1u8];
                res.extend(val.serialise_as_bytes());
                res
            }
        }
    }
}

impl<T: SerialiseAsBytes> SerialiseAsBytes for &[T] {
    fn serialise_as_bytes(&self) -> Vec<u8> {
        self.iter()
            .flat_map(SerialiseAsBytes::serialise_as_bytes)
            .collect()
    }
}

impl SerialiseAsBytes for str {
    fn serialise_as_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl SerialiseAsBytes for Felt {
    fn serialise_as_bytes(&self) -> Vec<u8> {
        self.to_bytes_be().to_vec()
    }
}

impl SerialiseAsBytes for bool {
    fn serialise_as_bytes(&self) -> Vec<u8> {
        vec![u8::from(*self)]
    }
}

// if we change API this might have collisions with old API hashes
pub(super) fn generate_id(selector: &str, inputs_bytes: Vec<u8>) -> String {
    let hash = Sha3_256::new()
        .chain_update(selector)
        .chain_update(inputs_bytes)
        .finalize();
    base16ct::lower::encode_string(&hash)
}

#[must_use]
pub fn generate_declare_tx_id(contract_name: &str) -> String {
    generate_id("declare", contract_name.serialise_as_bytes())
}

#[must_use]
pub fn generate_deploy_tx_id(
    class_hash: Felt,
    constructor_calldata: &[Felt],
    salt: Option<Felt>,
    unique: bool,
) -> String {
    let bytes = [
        class_hash.serialise_as_bytes(),
        constructor_calldata.serialise_as_bytes(),
        salt.serialise_as_bytes(),
        unique.serialise_as_bytes(),
    ]
    .concat();
    generate_id("deploy", bytes)
}

#[must_use]
pub fn generate_invoke_tx_id(
    contract_address: Felt,
    function_selector: Felt,
    calldata: &[Felt],
) -> String {
    let bytes = [
        contract_address.serialise_as_bytes(),
        function_selector.serialise_as_bytes(),
        calldata.serialise_as_bytes(),
    ]
    .concat();
    generate_id("invoke", bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::hashing::{
        generate_declare_tx_id, generate_deploy_tx_id, generate_id, generate_invoke_tx_id,
    };
    use conversions::IntoConv;

    #[test]
    fn basic_case() {
        let hash = generate_id("aaa", vec![b'a']);
        assert_eq!(
            hash,
            "28913c89fa628136fffce7ded99d65a4e3f5c211f82639fed4adca30d53b8dff"
        );
    }

    #[test]
    fn declare() {
        let contract_name = "testcontract";
        let hash = generate_declare_tx_id(contract_name);
        assert_eq!(
            hash,
            "058d80fb318b7a9aefce7c3725d062f1e449197909a654920b773d3f2c8bb7ce"
        );
    }

    #[test]
    fn deploy() {
        let class_hash: Felt = Felt::from_dec_str(
            "3372465304726137760522924034754430320558984443503992760655017624209518336998",
        )
        .unwrap()
        .into_();
        let constructor_calldata = vec![Felt::from(12u32), Felt::from(4u32)];
        let salt = Some(Felt::from(89u32));
        let unique = true;

        let hash = generate_deploy_tx_id(class_hash, &constructor_calldata, salt, unique);
        assert_eq!(
            hash,
            "c4146aa83f3d3c4e700db0bb8a2781d5b33914d899559d98918d73eb97985480"
        );
    }

    #[test]
    fn invoke() {
        let contract_address = Felt::from_dec_str(
            "379396891768624119314138643760266110764950106055405813326441497989022918556",
        )
        .unwrap()
        .into_();
        let function_selector = Felt::from(890u32);
        let calldata = vec![Felt::from(1809u32), Felt::from(14u32)];
        let hash = generate_invoke_tx_id(contract_address, function_selector, &calldata);
        assert_eq!(
            hash,
            "9b7d3fa2d93d1360a343bfd1d3d76aedef74aace5a5ad47ddbda136d9ce9b244"
        );
    }
}
