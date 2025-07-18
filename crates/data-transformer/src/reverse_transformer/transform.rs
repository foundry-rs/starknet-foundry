use crate::reverse_transformer::types::{
    Enum, Primitive, Sequence, SequenceType, Struct, StructField, Tuple, Type,
};
use crate::shared::parsing::{ParseError, parse_expression};
use crate::shared::path::{PathSplitError, SplitResult, split};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{Expr, ExprListParenthesized, ExprPath};
use conversions::serde::deserialize::{BufferReadError, BufferReader};
use starknet::core::types::contract::{AbiEntry, AbiEnum, AbiStruct};
use starknet_types_core::felt::Felt;

/// An error that can occur during the transformation process.
#[derive(Debug, thiserror::Error)]
pub enum TransformationError {
    #[error("type unsupported by reverse transformer")]
    UnsupportedType,
    #[error("reading from buffer failed with error: {0}")]
    BufferReaderError(#[from] BufferReadError),
    #[error("{enum_name} enum does not have variant at position {variant_position}")]
    NoSuchEnumVariant {
        enum_name: String,
        variant_position: usize,
    },
    #[error("abi is invalid")]
    InvalidAbi,
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    PathSplitError(#[from] PathSplitError),
}

/// An error that can occur when trying to transform a primitive type.
#[derive(Debug, thiserror::Error)]
enum PrimitiveError {
    #[error("Type {0} is not primitive")]
    NotFound(String),
    #[error(transparent)]
    BufferReaderError(#[from] BufferReadError),
}

pub struct ReverseTransformer<'a> {
    abi: &'a [AbiEntry],
    buffer_reader: BufferReader<'a>,
    db: SimpleParserDatabase,
}

impl<'a> ReverseTransformer<'a> {
    /// Creates a new instance of [`ReverseTransformer`].
    pub fn new(inputs: &'a [Felt], abi: &'a [AbiEntry]) -> Self {
        Self {
            abi,
            buffer_reader: BufferReader::new(inputs),
            db: SimpleParserDatabase::default(),
        }
    }

    /// Parses the given `&str` to an [`Expr`] and then transforms it to a [`Type`].
    pub fn parse_and_transform(&mut self, expr: &str) -> Result<Type, TransformationError> {
        self.transform_expr(&parse_expression(expr, &self.db)?)
    }

    /// Transforms the given [`Expr`] to a [`Type`].
    fn transform_expr(&mut self, expr: &Expr) -> Result<Type, TransformationError> {
        match expr {
            Expr::Tuple(expr) => self.transform_tuple(expr),
            Expr::Path(expr) => self.transform_path(expr),
            _ => Err(TransformationError::UnsupportedType),
        }
    }

    /// Transforms a tuple expression to a [`Type::Tuple`].
    fn transform_tuple(
        &mut self,
        expr: &ExprListParenthesized,
    ) -> Result<Type, TransformationError> {
        let parsed_exprs = expr
            .expressions(&self.db)
            .elements(&self.db)
            .into_iter()
            .map(|expr| self.transform_expr(&expr))
            .collect::<Result<Vec<_>, TransformationError>>()?;

        Ok(Type::Tuple(Tuple(parsed_exprs)))
    }

    /// Transforms a [`ExprPath`] to a [`Type`].
    fn transform_path(&mut self, expr: &ExprPath) -> Result<Type, TransformationError> {
        match split(expr, &self.db)? {
            SplitResult::Simple { splits } => self.transform_simple_path(&splits),
            SplitResult::WithGenericArgs {
                splits,
                generic_args,
            } => self.transform_generic_path(&splits, &generic_args),
        }
    }

    /// Transforms a generic path to a [`Type::Sequence`].
    ///
    /// It first checks if the path is a known sequence type (Array or Span).
    /// If it is, it reads the length of the sequence from the buffer.
    /// Then it recursively transforms the elements of the sequence.
    fn transform_generic_path(
        &mut self,
        splits: &[String],
        generic_args: &str,
    ) -> Result<Type, TransformationError> {
        let sequence_type = match splits.join("::").as_str() {
            "core::array::Array" => SequenceType::Array,
            "core::array::Span" => SequenceType::Span,
            _ => return Err(TransformationError::UnsupportedType),
        };

        let length = self.buffer_reader.read::<usize>()?;

        let sequence = (0..length)
            .map(|_| self.parse_and_transform(generic_args))
            .collect::<Result<Vec<_>, TransformationError>>()?;

        Ok(Type::Sequence(Sequence {
            sequence_type,
            sequence,
        }))
    }

    /// Transforms a simple path to a [`Type`].
    ///
    /// It first tries to transform it to a primitive type.
    /// If that fails, it means it is a complex type.
    /// Then it tries to find the type in the ABI and transform it to the matching representation.
    fn transform_simple_path(&mut self, parts: &[String]) -> Result<Type, TransformationError> {
        let name = parts.last().expect("path should not be empty").to_owned();

        match self.transform_primitive_type(&name) {
            Err(PrimitiveError::NotFound(_)) => (),
            Ok(primitive) => return Ok(Type::Primitive(primitive)),
            Err(PrimitiveError::BufferReaderError(error)) => {
                return Err(TransformationError::BufferReaderError(error));
            }
        }

        match find_item(self.abi, parts).ok_or(TransformationError::InvalidAbi)? {
            AbiStructOrEnum::Enum(enum_abi_definition) => {
                self.transform_enum(enum_abi_definition, name)
            }
            AbiStructOrEnum::Struct(struct_type_definition) => {
                self.transform_struct(struct_type_definition)
            }
        }
    }

    /// Transforms to a [`Type::Struct`].
    ///
    /// Recursively transforms the fields of the struct and then creates a [`Type::Struct`] instance.
    fn transform_struct(&mut self, abi_struct: &AbiStruct) -> Result<Type, TransformationError> {
        let fields = abi_struct
            .members
            .iter()
            .map(|member| {
                let value = self.parse_and_transform(&member.r#type)?;
                let name = member.name.clone();

                Ok(StructField { name, value })
            })
            .collect::<Result<Vec<_>, TransformationError>>()?;

        let name = abi_struct
            .name
            .split("::")
            .last()
            .expect("path should not be empty")
            .to_owned();

        Ok(Type::Struct(Struct { name, fields }))
    }

    /// Transforms to a [`Type::Enum`].
    ///
    /// It first reads the position of the enum variant from the buffer.
    /// Then it retrieves the variant name and type from the ABI.
    /// If the variant type is unit, it sets the argument to `None`, else it recursively transforms the variant type.
    /// Finally, it creates a [`Type::Enum`] instance.
    fn transform_enum(
        &mut self,
        abi_enum: &AbiEnum,
        name: String,
    ) -> Result<Type, TransformationError> {
        let position: usize = self.buffer_reader.read()?;

        let variant = abi_enum.variants.get(position).ok_or_else(|| {
            TransformationError::NoSuchEnumVariant {
                enum_name: name.clone(),
                variant_position: position,
            }
        })?;

        let enum_variant_type = variant.r#type.as_str();
        let variant = variant.name.clone();

        let argument = if enum_variant_type == "()" {
            None
        } else {
            Some(Box::new(self.parse_and_transform(enum_variant_type)?))
        };

        Ok(Type::Enum(Enum {
            name,
            variant,
            argument,
        }))
    }

    /// Transforms a primitive type string to a [`Primitive`].
    ///
    /// It matches the type string against known primitive types and reads the corresponding value from the buffer.
    fn transform_primitive_type(&mut self, type_str: &str) -> Result<Primitive, PrimitiveError> {
        match type_str {
            "bool" => Ok(Primitive::Bool(self.buffer_reader.read()?)),
            "u8" => Ok(Primitive::U8(self.buffer_reader.read()?)),
            "u16" => Ok(Primitive::U16(self.buffer_reader.read()?)),
            "u32" => Ok(Primitive::U32(self.buffer_reader.read()?)),
            "u64" => Ok(Primitive::U64(self.buffer_reader.read()?)),
            "u96" => Ok(Primitive::U96(self.buffer_reader.read()?)),
            "u128" => Ok(Primitive::U128(self.buffer_reader.read()?)),
            "u256" => Ok(Primitive::U256(self.buffer_reader.read()?)),
            "u384" => Ok(Primitive::U384(self.buffer_reader.read()?)),
            "u512" => Ok(Primitive::U512(self.buffer_reader.read()?)),
            "i8" => Ok(Primitive::I8(self.buffer_reader.read()?)),
            "i16" => Ok(Primitive::I16(self.buffer_reader.read()?)),
            "i32" => Ok(Primitive::I32(self.buffer_reader.read()?)),
            "i64" => Ok(Primitive::I64(self.buffer_reader.read()?)),
            "i128" => Ok(Primitive::I128(self.buffer_reader.read()?)),
            "ByteArray" => Ok(Primitive::ByteArray(self.buffer_reader.read()?)),
            "bytes31" => Ok(Primitive::CairoBytes31(self.buffer_reader.read()?)),
            "ContractAddress" => Ok(Primitive::ContractAddress(self.buffer_reader.read()?)),
            "ClassHash" => Ok(Primitive::ClassHash(self.buffer_reader.read()?)),
            "StorageAddress" => Ok(Primitive::StorageAddress(self.buffer_reader.read()?)),
            "EthAddress" => Ok(Primitive::EthAddress(self.buffer_reader.read()?)),
            "felt" | "felt252" => Ok(Primitive::Felt(self.buffer_reader.read()?)),
            _ => Err(PrimitiveError::NotFound(type_str.to_string())),
        }
    }
}

enum AbiStructOrEnum<'a> {
    Struct(&'a AbiStruct),
    Enum(&'a AbiEnum),
}

fn find_item<'a>(items_from_abi: &'a [AbiEntry], path: &[String]) -> Option<AbiStructOrEnum<'a>> {
    let path = path.join("::");
    items_from_abi.iter().find_map(|item| match item {
        AbiEntry::Struct(abi_struct) if abi_struct.name == path => {
            Some(AbiStructOrEnum::Struct(abi_struct))
        }
        AbiEntry::Enum(abi_enum) if abi_enum.name == path => Some(AbiStructOrEnum::Enum(abi_enum)),
        _ => None,
    })
}
