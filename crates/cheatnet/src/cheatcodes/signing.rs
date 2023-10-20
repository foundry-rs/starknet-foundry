use cairo_felt::Felt252;
use conversions::StarknetConversions;
use starknet::signers::SigningKey;

#[must_use]
pub fn generate_ecdsa_keys() -> (Felt252, Felt252) {
    let key_pair = SigningKey::from_random();

    (
        key_pair.secret_scalar().to_felt252(),
        key_pair.verifying_key().scalar().to_felt252(),
    )
}

pub fn ecdsa_sign_message(
    private_key: &Felt252,
    message_hash: &Felt252,
) -> Result<(Felt252, Felt252), String> {
    let key_pair = SigningKey::from_secret_scalar(private_key.to_field_element());

    match key_pair.sign(&message_hash.to_field_element()) {
        Ok(signature) => Ok((signature.r.to_felt252(), signature.s.to_felt252())),
        Err(_) => Err("message_hash out of range".to_string()),
    }
}
