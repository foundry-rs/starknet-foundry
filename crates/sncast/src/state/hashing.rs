use cairo_felt::Felt252;
use itertools::Itertools;

// if we change API this might have collisions with old API hashes
#[allow(dead_code)]
pub fn hash_script_subcommand_call(selector: &str, inputs: &[Felt252]) -> String {
    let mut res = selector.as_bytes().to_owned();
    let mut inputs_bytes: Vec<u8> = inputs.iter().flat_map(Felt252::to_bytes_be).collect_vec();
    res.append(&mut inputs_bytes);
    res.iter().map(|item| format!("{item:x?}")).join("").clone()
}

#[cfg(test)]
mod tests {
    use crate::state::hashing::hash_script_subcommand_call;
    use cairo_felt::Felt252;
    use num_traits::Num;

    const CALL_HEX: &str = "63616c6c";
    const DECLARE_HEX: &str = "6465636c617265";
    const DEPLOY_HEX: &str = "6465706c6f79";
    const INVOKE_HEX: &str = "696e766f6b65";

    #[test]
    fn basic_case() {
        let hash = hash_script_subcommand_call("aaa", &[Felt252::from(b'a')]);
        assert_eq!(hash, "61616161");
    }

    #[test]
    fn call() {
        let inputs = [
            Felt252::from_str_radix(
                "559168041797100451294927166283688929222884562339612449063481797630338747795",
                10,
            )
            .unwrap(),
            Felt252::from(6_776_180),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = hash_script_subcommand_call("call", inputs.as_ref());
        assert_eq!(
            hash,
            CALL_HEX.to_owned()
                + "13c7a576625f15b424f294f6b98d3aba744c089e4b010ffc97f488aa6334d9367657411"
        );
    }

    #[test]
    fn declare() {
        let inputs = [
            Felt252::from_str_radix("332347236658", 10).unwrap(),
            Felt252::from(1),
            Felt252::from(1),
        ];
        let hash = hash_script_subcommand_call("declare", inputs.as_ref());
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
        let hash = hash_script_subcommand_call("deploy", inputs.as_ref());
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
        let hash = hash_script_subcommand_call("invoke", inputs.as_ref());
        assert_eq!(
            hash,
            INVOKE_HEX.to_owned()
                + "d6bb24d851d5cb774faa732c58df4420c0f96844b3b63becec202d3aa79c70757421311"
        );
    }
}
