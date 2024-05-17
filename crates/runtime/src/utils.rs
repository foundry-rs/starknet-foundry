use crate::EnhancedHintError;
use conversions::serde::deserialize::BufferReadError;
use indoc::indoc;

impl From<BufferReadError> for EnhancedHintError {
    fn from(value: BufferReadError) -> Self {
        EnhancedHintError::Anyhow(
            anyhow::Error::from(value)
                .context(
                    indoc!(r"
                        Reading from buffer failed, this can be caused by calling starknet::testing::cheatcode with invalid arguments.
                        Probably `snforge_std`/`sncast_std` version is incompatible, check above for incompatibility warning.
                    ")
                )
        )
    }
}
