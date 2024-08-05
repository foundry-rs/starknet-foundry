use starknet::core::types::FieldElement;
use starknet::macros::felt;

pub static DEFAULT_MULTICALL_CONTENTS: &str = r#"[[call]]
call_type = "deploy"
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = "invoke"
contract_address = ""
function = ""
inputs = []
"#;

pub const UDC_ADDRESS: FieldElement =
    felt!("0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf");
pub const OZ_CLASS_HASH: FieldElement =
    felt!("0x00e2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6");
pub const ARGENT_CLASS_HASH: FieldElement =
    felt!("0x029927c8af6bccf3f6fda035981e765a7bdbf18a2dc0d630494f8758aa908e2b");

pub const BRAAVOS_CLASS_HASH: FieldElement =
    felt!("0x00816dd0297efc55dc1e7559020a3a825e81ef734b558f03c83325d4da7e6253");

pub const BRAAVOS_BASE_ACCOUNT_CLASS_HASH: FieldElement =
    felt!("0x013bfe114fb1cf405bfc3a7f8dbe2d91db146c17521d40dcf57e16d6b59fa8e6");

// used in wait_for_tx. Txs will be fetched every 5s with timeout of 300s - so 60 attempts
#[allow(dead_code)]
pub const WAIT_TIMEOUT: u16 = 300;
#[allow(dead_code)]
pub const WAIT_RETRY_INTERVAL: u8 = 5;

#[allow(dead_code)]
pub const DEFAULT_ACCOUNTS_FILE: &str = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json";

pub const KEYSTORE_PASSWORD_ENV_VAR: &str = "KEYSTORE_PASSWORD";
pub const CREATE_KEYSTORE_PASSWORD_ENV_VAR: &str = "CREATE_KEYSTORE_PASSWORD";

pub const SCRIPT_LIB_ARTIFACT_NAME: &str = "__sncast_script_lib";

pub const STATE_FILE_VERSION: u8 = 1;

pub const INIT_SCRIPTS_DIR: &str = "scripts";

pub const DEFAULT_STATE_FILE_SUFFIX: &str = "state.json";
