use crate::cairo_types::{CairoBytes31, CairoU96, CairoU256, CairoU384, CairoU512};
use conversions::byte_array::ByteArray;
use starknet_types_core::felt::Felt;
use std::fmt;
use std::fmt::Display;

/// Types that are supported by the reverse transformer
#[derive(Debug)]
pub enum Type {
    Struct(Struct),
    Sequence(Sequence),
    Enum(Enum),
    Primitive(Primitive),
    Tuple(Tuple),
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Struct(value) => write!(f, "{value}"),
            Type::Sequence(value) => write!(f, "{value}"),
            Type::Enum(value) => write!(f, "{value}"),
            Type::Primitive(value) => write!(f, "{value}"),
            Type::Tuple(value) => write!(f, "{value}"),
        }
    }
}

/// Primitive types supported by the reverse transformer
#[derive(Debug)]
pub enum Primitive {
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
    CairoBytes31(CairoBytes31),
    ByteArray(ByteArray),
    ContractAddress(Felt),
    ClassHash(Felt),
    StorageAddress(Felt),
    EthAddress(Felt),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Primitive::Bool(value) => write!(f, "{value}"),
            Primitive::U8(value) => write!(f, "{value}_u8"),
            Primitive::U16(value) => write!(f, "{value}_u16"),
            Primitive::U32(value) => write!(f, "{value}_u32"),
            Primitive::U64(value) => write!(f, "{value}_u64"),
            Primitive::U96(value) => write!(f, "{value}_96"),
            Primitive::U128(value) => write!(f, "{value}_u128"),
            Primitive::U256(value) => write!(f, "{value}_u256"),
            Primitive::U384(value) => write!(f, "{value}_u384"),
            Primitive::U512(value) => write!(f, "{value}_u512"),
            Primitive::I8(value) => write!(f, "{value}_i8"),
            Primitive::I16(value) => write!(f, "{value}_i16"),
            Primitive::I32(value) => write!(f, "{value}_i32"),
            Primitive::I64(value) => write!(f, "{value}_i64"),
            Primitive::I128(value) => write!(f, "{value}_i128"),
            Primitive::Felt(value) => write!(f, "{value}_felt252"),
            Primitive::ByteArray(value) => write!(f, "\"{value}\""),
            Primitive::ContractAddress(value) => write!(f, "ContractAddress({value:#x})"),
            Primitive::ClassHash(value) => write!(f, "ClassHash({value:#x})"),
            Primitive::StorageAddress(value) => write!(f, "StorageAddress({value:#x})"),
            Primitive::EthAddress(value) => write!(f, "EthAddress({value:#x})"),
            Primitive::CairoBytes31(value) => {
                let felt = Felt::from(*value);
                write!(f, "CairoBytes31({felt:#x})")
            }
        }
    }
}

#[derive(Debug)]
pub struct Tuple(pub Vec<Type>);

impl Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_tuple("");
        for item in &self.0 {
            dbg.field(&format_args!("{item}"));
        }
        dbg.finish()
    }
}

#[derive(Debug)]
pub struct StructField {
    pub name: String,
    pub value: Type,
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
}

impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct(&self.name);
        for field in &self.fields {
            dbg.field(&field.name, &format_args!("{}", field.value));
        }
        dbg.finish()
    }
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variant: String,
    pub argument: Option<Box<Type>>,
}

impl Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_name = format!("{}::{}", self.name, self.variant);

        if let Some(arg) = &self.argument {
            let mut dbg = f.debug_tuple(&variant_name);
            dbg.field(&format_args!("{arg}"));
            dbg.finish()
        } else {
            write!(f, "{variant_name}")
        }
    }
}

#[derive(Debug)]
pub enum SequenceType {
    Array,
    Span,
}

#[derive(Debug)]
pub struct Sequence {
    pub sequence_type: SequenceType,
    pub sequence: Vec<Type>,
}

impl Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "array!")?;

        let mut list = f.debug_list();
        for item in &self.sequence {
            list.entry(&format_args!("{item}"));
        }
        list.finish()?;

        if let SequenceType::Span = &self.sequence_type {
            write!(f, ".span()")?;
        }

        Ok(())
    }
}
