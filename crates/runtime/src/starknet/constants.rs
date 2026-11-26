use blockifier::blockifier_versioned_constants::VersionedConstants;
use starknet_api::block::StarknetVersion;
use starknet_api::versioned_constants_logic::VersionedConstantsTrait;

pub const TEST_ENTRY_POINT_SELECTOR: &str = "TEST_CONTRACT_SELECTOR";
pub const TEST_CONTRACT_CLASS_HASH: &str = "0x117";
// snforge_std/src/cheatcodes.cairo::test_address
pub const TEST_ADDRESS: &str = "0x01724987234973219347210837402";

/// Starknet version used by snforge, should be identical to the live Starknet version.
pub const STARKNET_VERSION: StarknetVersion = StarknetVersion::V0_14_4;

/// Returns versioned constants for [`STARKNET_VERSION`].
#[must_use]
pub fn versioned_constants() -> &'static VersionedConstants {
    VersionedConstants::get(&STARKNET_VERSION)
        .expect("Versioned constants should exist for the hardcoded Starknet version")
}
