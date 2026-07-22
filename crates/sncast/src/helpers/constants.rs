use std::num::{NonZeroU8, NonZeroU16};

use starknet_rust::macros::felt;
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
    felt!("0x02ceed65a4bd731034c01113685c831b01c15d7d432f71afb1cf1634b53a2125");
pub const OZ_CLASS_HASH: Felt =
    felt!("0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564"); // v1.0.0
pub const READY_CLASS_HASH: Felt =
    felt!("0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f"); // v0.4.0

pub const BRAAVOS_CLASS_HASH: Felt =
    felt!("0x03957f9f5a1cbfe918cedc2015c85200ca51a5f7506ecb6de98a5207b759bf8a"); // v1.2.0

pub const BRAAVOS_BASE_ACCOUNT_CLASS_HASH: Felt =
    felt!("0x03d16c7a9a60b0593bd202f660a28c5d76e0403601d9ccc7e4fa253b6a70c201"); // v1.2.0

pub const MAP_CONTRACT_ADDRESS_SEPOLIA: &str =
    "0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008";

pub const MAP_CONTRACT_CLASS_HASH_SEPOLIA: &str =
    "0x2a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321";

pub const MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA: &str =
    "0x4f644d3ea723b9c28781f2bea76e9c2cd8cc667b2861faf66b4e45402ea221c";

// used in wait_for_tx. Txs will be fetched every 5s with timeout of 300s - so 60 attempts
pub const WAIT_TIMEOUT: NonZeroU16 = NonZeroU16::new(300).unwrap();
pub const WAIT_RETRY_INTERVAL: NonZeroU8 = NonZeroU8::new(5).unwrap();

pub const DEFAULT_ACCOUNTS_FILE: &str = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json";

pub const KEYSTORE_PASSWORD_ENV_VAR: &str = "KEYSTORE_PASSWORD";
pub const CREATE_KEYSTORE_PASSWORD_ENV_VAR: &str = "CREATE_KEYSTORE_PASSWORD";
