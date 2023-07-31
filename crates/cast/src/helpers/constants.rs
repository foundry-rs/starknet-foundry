pub static DEFAULT_MULTICALL_CONTENTS: &str = r#"[[call]]
call_type = ""
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = ""
contract_address = ""
function = ""
inputs = []
"#;

pub const UDC_ADDRESS: &str = "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf";
pub const OZ_CLASS_HASH: &str =
    "0x058d97f7d76e78f44905cc30cb65b91ea49a4b908a76703c54197bca90f81773";
// used in wait_for_tx. Txs are fetched every 5 seconds
pub const DEFAULT_RETRIES: u8 = 100;
