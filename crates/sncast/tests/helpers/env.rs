use sncast::helpers::constants::{CREATE_KEYSTORE_PASSWORD_ENV_VAR, KEYSTORE_PASSWORD_ENV_VAR};
use std::env;

pub fn set_keystore_password_env() {
    // SAFETY: Tests run in parallel and share the same environment variables.
    // However, we only set this variable once to a fixed value and never modify or unset it.
    // The only potential issue would be if a test explicitly required this variable to be unset,
    // but to the best of our knowledge, no such test exists.
    unsafe {
        env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
    };
}

pub fn set_create_keystore_password_env() {
    // SAFETY: Tests run in parallel and share the same environment variables.
    // However, we only set this variable once to a fixed value and never modify or unset it.
    // The only potential issue would be if a test explicitly required this variable to be unset,
    // but to the best of our knowledge, no such test exists.
    unsafe {
        env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");
    };
}
