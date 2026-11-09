use super::helpers::{ParseRadixError, RadixInput};
use conversions::serde::deserialize::{
    BufferReadError, BufferReadResult, BufferReader, CairoDeserialize,
};
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use num_bigint::BigUint;
use starknet_types_core::felt::Felt;
use std::fmt;
use std::fmt::Display;
use std::mem::size_of;
use std::str::FromStr;

#[expect(clippy::struct_field_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CairoU384 {
    limb_0: u128,
    limb_1: u128,
    limb_2: u128,
    limb_3: u128,
}

impl CairoU384 {
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 48]) -> Self {
        fn to_u128(slice: &[u8]) -> u128 {
            let mut padded = [0u8; 16];
            padded[4..].copy_from_slice(slice);
            u128::from_be_bytes(padded)
        }

        Self {
            limb_0: to_u128(&bytes[36..48]),
            limb_1: to_u128(&bytes[24..36]),
            limb_2: to_u128(&bytes[12..24]),
            limb_3: to_u128(&bytes[0..12]),
        }
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 48] {
        let mut result = [0; 48];

        result[36..48].copy_from_slice(&self.limb_0.to_be_bytes());
        result[24..36].copy_from_slice(&self.limb_1.to_be_bytes());
        result[12..24].copy_from_slice(&self.limb_2.to_be_bytes());
        result[0..12].copy_from_slice(&self.limb_3.to_be_bytes());

        result
    }
}

// Cairo's `Serde<u384>` (`core::circuit::U384Serde`) writes two felts, not four limbs:
// `felt0 = limb0 + limb1 * 2^96` and `felt1 = limb2 + limb3 * 2^96`.
// A felt of 192 bits or more is not a valid limb pair.
const U96_BYTE_LEN: usize = 12;
const FELT_BYTE_LEN: usize = 32;
const PACKED_PREFIX_LEN: usize = FELT_BYTE_LEN - 2 * U96_BYTE_LEN;

fn pack_u96_limbs(low: u128, high: u128) -> Felt {
    debug_assert!(low < (1_u128 << 96));
    debug_assert!(high < (1_u128 << 96));

    // A limb is 96 bits stored in a `u128` (16 bytes). The packed felt's low 24 bytes
    // are the high limb followed by the low limb.
    let limb_pad = size_of::<u128>() - U96_BYTE_LEN;
    let mut bytes = [0u8; FELT_BYTE_LEN];
    bytes[PACKED_PREFIX_LEN..PACKED_PREFIX_LEN + U96_BYTE_LEN]
        .copy_from_slice(&high.to_be_bytes()[limb_pad..]);
    bytes[PACKED_PREFIX_LEN + U96_BYTE_LEN..].copy_from_slice(&low.to_be_bytes()[limb_pad..]);
    Felt::from_bytes_be(&bytes)
}

fn unpack_u96_limbs(felt: Felt) -> BufferReadResult<(u128, u128)> {
    let bytes = felt.to_bytes_be();
    if bytes[..PACKED_PREFIX_LEN] != [0; PACKED_PREFIX_LEN] {
        return Err(BufferReadError::ParseFailed);
    }

    let limb_pad = size_of::<u128>() - U96_BYTE_LEN;
    let mut low = [0u8; size_of::<u128>()];
    let mut high = [0u8; size_of::<u128>()];
    high[limb_pad..].copy_from_slice(&bytes[PACKED_PREFIX_LEN..PACKED_PREFIX_LEN + U96_BYTE_LEN]);
    low[limb_pad..].copy_from_slice(&bytes[PACKED_PREFIX_LEN + U96_BYTE_LEN..]);
    Ok((u128::from_be_bytes(low), u128::from_be_bytes(high)))
}

impl CairoSerialize for CairoU384 {
    fn serialize(&self, output: &mut BufferWriter) {
        output.write_felt(pack_u96_limbs(self.limb_0, self.limb_1));
        output.write_felt(pack_u96_limbs(self.limb_2, self.limb_3));
    }
}

impl CairoDeserialize for CairoU384 {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let (limb_0, limb_1) = unpack_u96_limbs(reader.read_felt()?)?;
        let (limb_2, limb_3) = unpack_u96_limbs(reader.read_felt()?)?;
        Ok(Self {
            limb_0,
            limb_1,
            limb_2,
            limb_3,
        })
    }
}

impl Display for CairoU384 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = BigUint::from_bytes_be(&self.to_be_bytes());
        write!(f, "{number}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU384Error {
    #[error(transparent)]
    InvalidString(#[from] ParseRadixError),
    #[error("Number is too large to fit in 48 bytes")]
    Overflow,
}

impl FromStr for CairoU384 {
    type Err = ParseCairoU384Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let number: BigUint = RadixInput::try_from(input.as_bytes())?.try_into()?;
        let bytes = number.to_bytes_be();

        if bytes.len() > 48 {
            return Err(ParseCairoU384Error::Overflow);
        }

        let mut result = [0u8; 48];
        let start = 48 - bytes.len();
        result[start..].copy_from_slice(&bytes);

        Ok(CairoU384::from_bytes(&result))
    }
}
