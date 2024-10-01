use anyhow::{bail, Context};
use conversions::{
    byte_array::ByteArray,
    serde::serialize::{BufferWriter, CairoSerialize},
    u256::CairoU256,
    u512::CairoU512,
};
use starknet::core::types::Felt;
use std::{any::type_name, str::FromStr};

#[derive(Debug)]
pub(super) struct CalldataStructField(AllowedCalldataArguments);

impl CalldataStructField {
    pub fn new(value: AllowedCalldataArguments) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub(super) struct CalldataStruct(Vec<CalldataStructField>);

impl CalldataStruct {
    pub fn new(arguments: Vec<CalldataStructField>) -> Self {
        Self(arguments)
    }
}

#[derive(Debug)]
pub(super) struct CalldataArrayMacro(Vec<AllowedCalldataArguments>);

impl CalldataArrayMacro {
    pub fn new(arguments: Vec<AllowedCalldataArguments>) -> Self {
        Self(arguments)
    }
}

#[derive(Debug)]
pub(super) struct CalldataEnum {
    position: usize,
    argument: Option<Box<AllowedCalldataArguments>>,
}

impl CalldataEnum {
    pub fn new(position: usize, argument: Option<Box<AllowedCalldataArguments>>) -> Self {
        Self { position, argument }
    }
}

#[derive(Debug)]
pub(super) enum CalldataSingleArgument {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(CairoU256),
    U512(CairoU512),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Felt(Felt),
    ByteArray(ByteArray),
}

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

impl CalldataSingleArgument {
    pub(super) fn try_new(type_with_path: &str, value: &str) -> anyhow::Result<Self> {
        // TODO add all corelib types
        let type_str = type_with_path
            .split("::")
            .last()
            .context("Couldn't parse parameter type from ABI")?;

        match type_str {
            "u8" => Ok(Self::U8(parse_with_type(value)?)),
            "u16" => Ok(Self::U16(parse_with_type(value)?)),
            "u32" => Ok(Self::U32(parse_with_type(value)?)),
            "u64" => Ok(Self::U64(parse_with_type(value)?)),
            "u128" => Ok(Self::U128(parse_with_type(value)?)),
            "u256" => Ok(Self::U256(parse_with_type(value)?)),
            "u512" => Ok(Self::U512(parse_with_type(value)?)),
            "i8" => Ok(Self::I8(parse_with_type(value)?)),
            "i16" => Ok(Self::I16(parse_with_type(value)?)),
            "i32" => Ok(Self::I32(parse_with_type(value)?)),
            "i64" => Ok(Self::I64(parse_with_type(value)?)),
            "i128" => Ok(Self::I128(parse_with_type(value)?)),
            // TODO check if bytes31 is actually a felt
            // (e.g. alexandria_data_structures::bit_array::BitArray uses that)
            // https://github.com/starkware-libs/cairo/blob/bf48e658b9946c2d5446eeb0c4f84868e0b193b5/corelib/src/bytes_31.cairo#L14
            // There is `bytes31_try_from_felt252`, which means it isn't always a valid felt?
            "felt252" | "felt" | "ContractAddress" | "ClassHash" | "bytes31" => {
                let felt = Felt::from_dec_str(value)
                    .with_context(|| neat_parsing_error_message(value, type_with_path, None))?;
                Ok(Self::Felt(felt))
            }
            "bool" => Ok(Self::Bool(parse_with_type(value)?)),
            "ByteArray" => Ok(Self::ByteArray(ByteArray::from(value))),
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

#[derive(Debug)]
pub(super) struct CalldataTuple(Vec<AllowedCalldataArguments>);

impl CalldataTuple {
    pub fn new(arguments: Vec<AllowedCalldataArguments>) -> Self {
        Self(arguments)
    }
}

#[derive(Debug)]
pub(super) enum AllowedCalldataArguments {
    Struct(CalldataStruct),
    ArrayMacro(CalldataArrayMacro),
    Enum(CalldataEnum),
    // TODO rename to BasicType or smth
    SingleArgument(CalldataSingleArgument),
    Tuple(CalldataTuple),
}

impl CairoSerialize for CalldataSingleArgument {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            CalldataSingleArgument::Bool(value) => value.serialize(output),
            CalldataSingleArgument::U8(value) => value.serialize(output),
            CalldataSingleArgument::U16(value) => value.serialize(output),
            CalldataSingleArgument::U32(value) => value.serialize(output),
            CalldataSingleArgument::U64(value) => value.serialize(output),
            CalldataSingleArgument::U128(value) => value.serialize(output),
            CalldataSingleArgument::U256(value) => value.serialize(output),
            CalldataSingleArgument::U512(value) => value.serialize(output),
            CalldataSingleArgument::I8(value) => value.serialize(output),
            CalldataSingleArgument::I16(value) => value.serialize(output),
            CalldataSingleArgument::I32(value) => value.serialize(output),
            CalldataSingleArgument::I64(value) => value.serialize(output),
            CalldataSingleArgument::I128(value) => value.serialize(output),
            CalldataSingleArgument::Felt(value) => value.serialize(output),
            CalldataSingleArgument::ByteArray(value) => value.serialize(output),
        };
    }
}

impl CairoSerialize for CalldataStructField {
    // Every argument serialized in order of occurrence
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.serialize(output);
    }
}

impl CairoSerialize for CalldataStruct {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_structs
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

impl CairoSerialize for CalldataTuple {
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

impl CairoSerialize for CalldataArrayMacro {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_arrays
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.len().serialize(output);
        self.0.iter().for_each(|field| field.serialize(output));
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
impl CairoSerialize for AllowedCalldataArguments {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            AllowedCalldataArguments::Struct(value) => value.serialize(output),
            AllowedCalldataArguments::ArrayMacro(value) => value.serialize(output),
            AllowedCalldataArguments::Enum(value) => value.serialize(output),
            AllowedCalldataArguments::SingleArgument(value) => value.serialize(output),
            AllowedCalldataArguments::Tuple(value) => value.serialize(output),
        }
    }
}
