use cairo_felt::Felt252;
use itertools::Itertools;
use sha3::Digest;
use sha3::Sha3_256;

// if we change API this might have collisions with old API hashes
pub fn generate_id(selector: &str, inputs: &[Felt252]) -> String {
    let inputs_bytes: Vec<u8> = inputs.iter().flat_map(Felt252::to_bytes_be).collect_vec();
    let hash = Sha3_256::new()
        .chain_update(selector)
        .chain_update(inputs_bytes)
        .finalize();
    base16ct::lower::encode_string(&hash)
}

#[cfg(test)]
mod tests {
    use crate::state::hashing::generate_id;
    use cairo_felt::Felt252;
    use num_traits::Num;

    #[test]
    fn basic_case() {
        let hash = generate_id("aaa", &[Felt252::from(b'a')]);
        assert_eq!(
            hash,
            "28913c89fa628136fffce7ded99d65a4e3f5c211f82639fed4adca30d53b8dff"
        );
    }

    #[test]
    fn declare() {
        let inputs = [
            Felt252::from_str_radix("332347236658", 10).unwrap(),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = generate_id("declare", inputs.as_ref());
        assert_eq!(
            hash,
            "e759b4df4e28627248db61c7aaed0104a428b783e15f094ec41abede07e26af5"
        );
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
        assert_eq!(
            hash,
            "baa5f2c5e61ece9fdc7fa54bd287d33a30175a375d18a3243fdd61ca113ad6ae"
        );
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
        assert_eq!(
            hash,
            "45d549ba1db7bf0a5bfcdfe5dde0fce2c93d44b15f4d7f1c18d5fc2b7dd98fc3"
        );
    }
}
