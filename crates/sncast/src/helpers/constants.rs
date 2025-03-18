use starknet::macros::felt;
use starknet_types_core::felt::Felt;

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

pub const UDC_ADDRESS: Felt =
    felt!("0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf");
pub const OZ_CLASS_HASH: Felt =
    felt!("0x00e2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6");
pub const ARGENT_CLASS_HASH: Felt =
    felt!("0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f");

pub const BRAAVOS_CLASS_HASH: Felt =
    felt!("0x02c8c7e6fbcfb3e8e15a46648e8914c6aa1fc506fc1e7fb3d1e19630716174bc");

pub const BRAAVOS_BASE_ACCOUNT_CLASS_HASH: Felt =
    felt!("0x03d16c7a9a60b0593bd202f660a28c5d76e0403601d9ccc7e4fa253b6a70c201");

// used in wait_for_tx. Txs will be fetched every 5s with timeout of 300s - so 60 attempts
pub const WAIT_TIMEOUT: u16 = 300;
pub const WAIT_RETRY_INTERVAL: u8 = 5;

pub const DEFAULT_ACCOUNTS_FILE: &str = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json";

pub const KEYSTORE_PASSWORD_ENV_VAR: &str = "KEYSTORE_PASSWORD";
pub const CREATE_KEYSTORE_PASSWORD_ENV_VAR: &str = "CREATE_KEYSTORE_PASSWORD";

pub const SCRIPT_LIB_ARTIFACT_NAME: &str = "__sncast_script_lib";

pub const STATE_FILE_VERSION: u8 = 1;

pub const INIT_SCRIPTS_DIR: &str = "scripts";

pub const DEFAULT_STATE_FILE_SUFFIX: &str = "state.json";
