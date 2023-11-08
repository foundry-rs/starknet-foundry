use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_integer::Integer;
use num_traits::{One, Zero};
use rand::prelude::StdRng;
use std::ops::{Add, Shl, Shr, Sub};

#[derive(Debug, Clone)]
pub enum CairoType {
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Felt252,
}

pub trait Argument {
    fn low(&self) -> BigUint;
    fn high(&self) -> BigUint;
    fn gen(&self, rng: &mut StdRng) -> Vec<Felt252>;
    fn min(&self) -> Vec<Felt252>;
    fn max(&self) -> Vec<Felt252>;
}

impl Argument for CairoType {
    fn low(&self) -> BigUint {
        BigUint::zero()
    }

    fn high(&self) -> BigUint {
        match self {
            CairoType::U8 => BigUint::from(u8::MAX).add(BigUint::one()),
            CairoType::U16 => BigUint::from(u16::MAX).add(BigUint::one()),
            CairoType::U32 => BigUint::from(u32::MAX).add(BigUint::one()),
            CairoType::U64 => BigUint::from(u64::MAX).add(BigUint::one()),
            CairoType::U128 => BigUint::from(u128::MAX).add(BigUint::one()),
            CairoType::U256 => {
                let max = BigUint::from(1_u32);
                max.shl(256)
            }
            CairoType::Felt252 => Felt252::prime(),
        }
    }

    fn gen(&self, rng: &mut StdRng) -> Vec<Felt252> {
        match self {
            CairoType::U8
            | CairoType::U16
            | CairoType::U32
            | CairoType::U64
            | CairoType::U128
            | CairoType::Felt252 => {
                vec![Felt252::from(
                    rng.gen_biguint_range(&self.low(), &self.high()),
                )]
            }
            CairoType::U256 => {
                let val = rng.gen_biguint_range(&self.low(), &self.high());
                u256_to_felt252(val)
            }
        }
    }

    fn min(&self) -> Vec<Felt252> {
        match self {
            CairoType::U8
            | CairoType::U16
            | CairoType::U32
            | CairoType::U64
            | CairoType::U128
            | CairoType::Felt252 => vec![Felt252::from(self.low())],
            CairoType::U256 => vec![Felt252::from(self.low()), Felt252::from(self.low())],
        }
    }

    fn max(&self) -> Vec<Felt252> {
        match self {
            CairoType::U8
            | CairoType::U16
            | CairoType::U32
            | CairoType::U64
            | CairoType::U128
            | CairoType::Felt252 => vec![Felt252::from(self.high().sub(BigUint::one()))],
            CairoType::U256 => u256_to_felt252(self.high().sub(BigUint::one())),
        }
    }
}

fn u256_to_felt252(val: BigUint) -> Vec<Felt252> {
    let low = val.mod_floor(&BigUint::from(2_u32).pow(128));
    let high = val.shr(128);
    vec![Felt252::from(low), Felt252::from(high)]
}

impl CairoType {
    pub(crate) fn from_name(name: &str) -> Result<Self> {
        match name {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            "u256" | "core::integer::u256" => Ok(Self::U256),
            "felt252" => Ok(Self::Felt252),
            _ => Err(anyhow!(
                "Tried to use incorrect type for fuzzing. Type = {name} is not supported"
            )),
        }
    }
}
