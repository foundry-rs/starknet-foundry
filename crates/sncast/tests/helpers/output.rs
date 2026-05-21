use serde::Deserialize;
use starknet_types_core::felt::Felt;

pub fn get_declared_class_hash_from_json_output(output: &[u8]) -> Felt {
    #[derive(Deserialize)]
    struct DeclareClassHashJsonOutput {
        class_hash: Felt,
    }

    output
        .split(|byte| *byte == b'\n')
        .find_map(|line| serde_json::from_slice::<DeclareClassHashJsonOutput>(line).ok())
        .map(|output| output.class_hash)
        .expect("Failed to deserialize declared class hash from stdout JSON")
}
