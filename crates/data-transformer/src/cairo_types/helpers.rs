use num_bigint::BigUint;
use thiserror;

#[derive(Clone, Debug)]
pub enum RadixInput {
    Decimal(Box<[u8]>),
    Hexadecimal(Box<[u8]>),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseRadixError {
    #[error("Input contains invalid digit")]
    InvalidString,
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl<'input> TryFrom<&'input [u8]> for RadixInput {
    type Error = ParseRadixError;

    fn try_from(bytes: &'input [u8]) -> Result<Self, Self::Error> {
        let mut is_hex = false;

        if bytes.len() > 2 {
            is_hex = bytes[1] == b'x';
        }

        let result = bytes
            .iter()
            .skip_while(|&&byte| byte == b'0' || byte == b'x')
            .filter(|&&byte| byte != b'_')
            .map(|&byte| {
                if byte.is_ascii_digit() {
                    Ok(byte - b'0')
                } else if (b'a'..b'g').contains(&byte) {
                    is_hex = true;
                    Ok(byte - b'a' + 10)
                } else if (b'A'..b'G').contains(&byte) {
                    is_hex = true;
                    Ok(byte - b'A' + 10)
                } else {
                    Err(ParseRadixError::InvalidString)
                }
            })
            .collect::<Result<Box<[u8]>, ParseRadixError>>()?;

        Ok(if is_hex {
            Self::Hexadecimal(result)
        } else {
            Self::Decimal(result)
        })
    }
}

impl TryFrom<RadixInput> for BigUint {
    type Error = ParseRadixError;

    fn try_from(value: RadixInput) -> Result<Self, Self::Error> {
        match value {
            RadixInput::Decimal(digits) => {
                BigUint::from_radix_be(&digits, 10).ok_or(ParseRadixError::InvalidString)
            }
            RadixInput::Hexadecimal(digits) => {
                BigUint::from_radix_be(&digits, 16).ok_or(ParseRadixError::InvalidString)
            }
        }
    }
}
