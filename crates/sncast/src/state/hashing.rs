use base64ct::{Base64, Encoding};
use cairo_felt::Felt252;
use itertools::Itertools;
use sha3::Digest;
use sha3::Sha3_256;

// if we change API this might have collisions with old API hashes
pub fn generate_id(selector: &str, inputs: &[Felt252]) -> String {
    let mut res = selector.as_bytes().to_owned();
    let mut inputs_bytes: Vec<u8> = inputs.iter().flat_map(Felt252::to_bytes_be).collect_vec();
    res.append(&mut inputs_bytes);
    let res = res.iter().map(|item| format!("{item:x?}")).join("").clone();

    let hash = Sha3_256::new().chain_update(res).finalize();
    Base64::encode_string(&hash)
}

#[cfg(test)]
mod tests {
    use crate::state::hashing::generate_id;
    use cairo_felt::Felt252;
    use num_traits::Num;

    #[test]
    fn basic_case() {
        let hash = generate_id("aaa", &[Felt252::from(b'a')]);
        assert_eq!(hash, "oiA+mIeajh+kzDjne963zIMgLxyGYR0bZeZDQLjhb1A=");
    }

    #[test]
    fn declare() {
        let inputs = [
            Felt252::from_str_radix("332347236658", 10).unwrap(),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = generate_id("declare", inputs.as_ref());
        assert_eq!(hash, "WdIwShcOtrdOVwAjg8x3/xhQKhccIGemBmLENbkqfHk=");
    }

    #[test]
    fn deploy() {
        let inputs = [
            Felt252::from_str_radix(
                "3372465304726137760522924034754430320558984443503992760655017624209518336998",
                10,
            )
            .unwrap(),
            Felt252::from(0),
            Felt252::from(1),
            Felt252::from(0),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = generate_id("deploy", inputs.as_ref());
        assert_eq!(hash, "aGZ4kfVg3DM9u4MGzyB5m0uHPVcWs992qQPYklLg+Ac=");
    }

    #[test]
    fn invoke() {
        let inputs = [
            Felt252::from_str_radix(
                "379396891768624119314138643760266110764950106055405813326441497989022918556",
                10,
            )
            .unwrap(),
            Felt252::from(7_370_100),
            Felt252::from(2),
            Felt252::from(1),
            Felt252::from(3),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = generate_id("invoke", inputs.as_ref());
        assert_eq!(hash, "AnAKZVPOmmWCLxBEfCY06laZMDaJfkW5pkRLuf4k51w=");
    }
}
