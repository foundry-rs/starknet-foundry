use conversions::string::{IntoDecStr, TryFromDecStr};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::StarkHash;
use starknet_types_core::felt::Felt;

#[test]
fn test_short_strings_conversions_happy_case() {
    let short_string = "1";

    assert_eq!(
        short_string,
        (ClassHash::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (ContractAddress::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Felt::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Felt::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Nonce::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (EntryPointSelector::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (StarkHash::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
}

#[test]
fn test_short_strings_conversions_zero() {
    let short_string = "0";

    assert_eq!(
        short_string,
        (ClassHash::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (ContractAddress::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Felt::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Felt::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Nonce::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (EntryPointSelector::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (StarkHash::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
}

#[test]
fn test_short_string_conversions_limit() {
    // 31 characters.
    let short_string = "1234567890123456789012345678901";

    assert_eq!(
        short_string,
        (ClassHash::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Felt::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Felt::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (Nonce::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (EntryPointSelector::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (StarkHash::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
    assert_eq!(
        short_string,
        (ContractAddress::try_from_dec_str(short_string).unwrap()).into_dec_string()
    );
}
