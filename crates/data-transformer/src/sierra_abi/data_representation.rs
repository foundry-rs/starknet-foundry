use crate::cairo_types::{CairoBytes31, CairoU256, CairoU384, CairoU512, CairoU96};
use anyhow::{bail, Context};
use conversions::felt::FromShortString;
use conversions::{
    byte_array::ByteArray,
    serde::serialize::{BufferWriter, CairoSerialize},
};
use starknet_types_core::felt::Felt;
use std::{any::type_name, str::FromStr};

fn neat_parsing_error_message(value: &str, parsing_type: &str, reason: Option<&str>) -> String {
    if let Some(message) = reason {
        format!(r#"Failed to parse value "{value}" into type "{parsing_type}": {message}"#)
    } else {
        format!(r#"Failed to parse value "{value}" into type "{parsing_type}""#)
    }
}

fn parse_with_type<T: FromStr>(value: &str) -> anyhow::Result<T>
where
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    value
        .parse::<T>()
        .context(neat_parsing_error_message(value, type_name::<T>(), None))
}

/// A fundamental struct for representing expression types supported by the transformer
#[derive(Debug)]
pub enum AllowedCalldataArgument {
    Struct(CalldataStruct),
    ArrayMacro(CalldataArrayMacro),
    Enum(CalldataEnum),
    Primitive(CalldataPrimitive),
    Tuple(CalldataTuple),
}

impl CairoSerialize for AllowedCalldataArgument {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            AllowedCalldataArgument::Struct(value) => value.serialize(output),
            AllowedCalldataArgument::ArrayMacro(value) => value.serialize(output),
            AllowedCalldataArgument::Enum(value) => value.serialize(output),
            AllowedCalldataArgument::Primitive(value) => value.serialize(output),
            AllowedCalldataArgument::Tuple(value) => value.serialize(output),
        }
    }
}

#[derive(Debug)]
pub enum CalldataPrimitive {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U96(CairoU96),
    U128(u128),
    U256(CairoU256),
    U384(CairoU384),
    U512(CairoU512),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Felt(Felt),
    ByteArray(ByteArray),
}

impl CalldataPrimitive {
    pub(super) fn try_from_str_with_type(
        value: &str,
        type_with_path: &str,
    ) -> anyhow::Result<Self> {
        let type_str = type_with_path
            .split("::")
            .last()
            .context("Couldn't parse parameter type from ABI")?;

        // TODO add all corelib types (Issue #2550)
        match type_str {
            "bool" => Ok(Self::Bool(parse_with_type(value)?)),
            "u8" => Ok(Self::U8(parse_with_type(value)?)),
            "u16" => Ok(Self::U16(parse_with_type(value)?)),
            "u32" => Ok(Self::U32(parse_with_type(value)?)),
            "u64" => Ok(Self::U64(parse_with_type(value)?)),
            "u96" => Ok(Self::U96(parse_with_type(value)?)),
            "u128" => Ok(Self::U128(parse_with_type(value)?)),
            "u256" => Ok(Self::U256(parse_with_type(value)?)),
            "u384" => Ok(Self::U384(parse_with_type(value)?)),
            "u512" => Ok(Self::U512(parse_with_type(value)?)),
            "i8" => Ok(Self::I8(parse_with_type(value)?)),
            "i16" => Ok(Self::I16(parse_with_type(value)?)),
            "i32" => Ok(Self::I32(parse_with_type(value)?)),
            "i64" => Ok(Self::I64(parse_with_type(value)?)),
            "i128" => Ok(Self::I128(parse_with_type(value)?)),
            "ByteArray" => Ok(Self::ByteArray(ByteArray::from(value))),
            // bytes31 is a helper type defined in Cairo corelib;
            // (e.g. alexandria_data_structures::bit_array::BitArray uses that)
            // https://github.com/starkware-libs/cairo/blob/bf48e658b9946c2d5446eeb0c4f84868e0b193b5/corelib/src/bytes_31.cairo#L14
            // It's actually felt under the hood. Although conversion from felt252 to bytes31 returns Result, it never fails.
            "bytes31" => Ok(Self::Felt(parse_with_type::<CairoBytes31>(value)?.into())),
            "shortstring" => {
                let felt = Felt::from_short_string(value)?;
                Ok(Self::Felt(felt))
            }
            "felt252" | "felt" | "ContractAddress" | "ClassHash" | "StorageAddress"
            | "EthAddress" => {
                let felt = Felt::from_dec_str(value)
                    .with_context(|| neat_parsing_error_message(value, type_with_path, None))?;
                Ok(Self::Felt(felt))
            }
            _ => {
                bail!(neat_parsing_error_message(
                    value,
                    type_with_path,
                    Some(&format!("unsupported type {type_with_path}"))
                ))
            }
        }
    }
}

impl CairoSerialize for CalldataPrimitive {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            CalldataPrimitive::Bool(value) => value.serialize(output),
            CalldataPrimitive::U8(value) => value.serialize(output),
            CalldataPrimitive::U16(value) => value.serialize(output),
            CalldataPrimitive::U32(value) => value.serialize(output),
            CalldataPrimitive::U64(value) => value.serialize(output),
            CalldataPrimitive::U96(value) => value.serialize(output),
            CalldataPrimitive::U128(value) => value.serialize(output),
            CalldataPrimitive::U256(value) => value.serialize(output),
            CalldataPrimitive::U384(value) => value.serialize(output),
            CalldataPrimitive::U512(value) => value.serialize(output),
            CalldataPrimitive::I8(value) => value.serialize(output),
            CalldataPrimitive::I16(value) => value.serialize(output),
            CalldataPrimitive::I32(value) => value.serialize(output),
            CalldataPrimitive::I64(value) => value.serialize(output),
            CalldataPrimitive::I128(value) => value.serialize(output),
            CalldataPrimitive::Felt(value) => value.serialize(output),
            CalldataPrimitive::ByteArray(value) => value.serialize(output),
        };
    }
}

#[derive(Debug)]
pub struct CalldataTuple(Vec<AllowedCalldataArgument>);

impl CalldataTuple {
    pub fn new(arguments: Vec<AllowedCalldataArgument>) -> Self {
        Self(arguments)
    }
}

impl CairoSerialize for CalldataTuple {
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

#[derive(Debug, CairoSerialize)]
pub struct CalldataStructField(AllowedCalldataArgument);

impl CalldataStructField {
    pub fn new(value: AllowedCalldataArgument) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct CalldataStruct(Vec<CalldataStructField>);

impl CalldataStruct {
    pub fn new(arguments: Vec<CalldataStructField>) -> Self {
        Self(arguments)
    }
}

impl CairoSerialize for CalldataStruct {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_structs
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

#[derive(Debug)]
pub struct CalldataEnum {
    position: usize,
    argument: Option<Box<AllowedCalldataArgument>>,
}

impl CalldataEnum {
    pub fn new(position: usize, argument: Option<Box<AllowedCalldataArgument>>) -> Self {
        Self { position, argument }
    }
}

impl CairoSerialize for CalldataEnum {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_enums
    fn serialize(&self, output: &mut BufferWriter) {
        self.position.serialize(output);
        if self.argument.is_some() {
            self.argument.as_ref().unwrap().serialize(output);
        }
    }
}

#[derive(Debug, CairoSerialize)]
pub struct CalldataArrayMacro(Vec<AllowedCalldataArgument>);

impl CalldataArrayMacro {
    pub fn new(arguments: Vec<AllowedCalldataArgument>) -> Self {
        Self(arguments)
    }
}
