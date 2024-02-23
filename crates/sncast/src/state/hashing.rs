use cairo_felt::Felt252;
use itertools::Itertools;

// if we change API this might have collisions with old API hashes
pub fn generate_id(selector: &str, inputs: &[Felt252]) -> String {
    let mut res = selector.as_bytes().to_owned();
    let mut inputs_bytes: Vec<u8> = inputs.iter().flat_map(Felt252::to_bytes_be).collect_vec();
    res.append(&mut inputs_bytes);
    res.iter().map(|item| format!("{item:x?}")).join("").clone()
}

#[cfg(test)]
mod tests {
    use crate::state::hashing::generate_id;
    use cairo_felt::Felt252;
    use num_traits::Num;

    const DECLARE_HEX: &str = "6465636c617265";
    const DEPLOY_HEX: &str = "6465706c6f79";
    const INVOKE_HEX: &str = "696e766f6b65";

    #[test]
    fn basic_case() {
        let hash = generate_id("aaa", &[Felt252::from(b'a')]);
        assert_eq!(hash, "61616161");
    }

    #[test]
    fn declare() {
        let inputs = [
            Felt252::from_str_radix("332347236658", 10).unwrap(),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = generate_id("declare", inputs.as_ref());
        assert_eq!(hash, DECLARE_HEX.to_owned() + "4d6170613211");
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
            DEPLOY_HEX.to_owned()
                + "774bf6a8340629c8989ddfd523b7c6524886b1aaaf0bd93887cb65eb7a59be601011"
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
            INVOKE_HEX.to_owned()
                + "d6bb24d851d5cb774faa732c58df4420c0f96844b3b63becec202d3aa79c70757421311"
        );
    }
}
