use super::data_representation::{
    AllowedCalldataArgument, CalldataEnum, CalldataStruct, CalldataStructField, CalldataTuple,
};
use anyhow::{bail, ensure, Context, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{
    Expr, ExprFunctionCall, ExprListParenthesized, ExprPath, ExprStructCtorCall,
    OptionStructArgExpr, PathSegment, StructArg,
};
use cairo_lang_syntax::node::{Terminal, Token, TypedSyntaxNode};
use itertools::Itertools;
use starknet::core::types::contract::{AbiEntry, AbiEnum, AbiNamedMember, AbiStruct};
use std::collections::HashSet;

use super::parsing::{parse_argument_list, parse_expression};
use super::{build_representation, SupportedCalldataKind};

pub trait EnumOrStruct {
    const VARIANT: &'static str;
    const VARIANT_CAPITALIZED: &'static str;
    fn name(&self) -> String;
}

impl EnumOrStruct for AbiStruct {
    const VARIANT: &'static str = "struct";
    const VARIANT_CAPITALIZED: &'static str = "Struct";

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl EnumOrStruct for AbiEnum {
    const VARIANT: &'static str = "enum";
    const VARIANT_CAPITALIZED: &'static str = "Enum";

    fn name(&self) -> String {
        self.name.clone()
    }
}

fn validate_path_argument(
    param_type: &str,
    path_argument: &[String],
    path_argument_joined: &String,
) -> Result<()> {
    if *path_argument.last().unwrap() != param_type.split("::").last().unwrap()
        && path_argument_joined != param_type
    {
        bail!(
            r#"Invalid argument type, expected "{}", got "{}""#,
            param_type,
            path_argument_joined
        )
    }
    Ok(())
}

fn split(path: &ExprPath, db: &SimpleParserDatabase) -> Result<Vec<String>> {
    path.elements(db)
        .iter()
        .map(|p| match p {
            PathSegment::Simple(segment) => Ok(segment.ident(db).token(db).text(db).to_string()),
            PathSegment::WithGenericArgs(_) => {
                bail!("Cannot use generic args when specifying struct/enum path")
            }
        })
        .collect::<Result<_>>()
}

fn find_all_structs(abi: &[AbiEntry]) -> Vec<&AbiStruct> {
    abi.iter()
        .filter_map(|entry| match entry {
            AbiEntry::Struct(r#struct) => Some(r#struct),
            _ => None,
        })
        .collect()
}

fn find_enum_variant_position<'a>(
    variant: &String,
    path: &[String],
    abi: &'a [AbiEntry],
) -> Result<(usize, &'a AbiNamedMember)> {
    let enums_from_abi = abi
        .iter()
        .filter_map(|abi_entry| {
            if let AbiEntry::Enum(abi_enum) = abi_entry {
                Some(abi_enum)
            } else {
                None
            }
        })
        .collect::<Vec<&AbiEnum>>();

    let enum_abi_definition = find_item_with_path(enums_from_abi, path)?;

    let position_and_enum_variant = enum_abi_definition
        .variants
        .iter()
        .find_position(|item| item.name == *variant)
        .with_context(|| {
            format!(
                r#"Couldn't find variant "{}" in enum "{}""#,
                variant,
                path.join("::")
            )
        })?;

    Ok(position_and_enum_variant)
}

/// Structs and enums in ABI can be searched in the same way. 'item' here refers either to an enum or a struct
fn find_item_with_path<'item, T: EnumOrStruct>(
    items_from_abi: Vec<&'item T>,
    path: &[String],
) -> Result<&'item T> {
    // Argument is a module path to an item (module_name::StructName {})
    if path.len() > 1 {
        let full_path_item = items_from_abi
            .into_iter()
            .find(|x| x.name() == path.join("::"));

        ensure!(
            full_path_item.is_some(),
            r#"{} "{}" not found in ABI"#,
            T::VARIANT_CAPITALIZED,
            path.join("::")
        );

        return Ok(full_path_item.unwrap());
    }

    // Argument is just the name of the item (Struct {})
    let mut matching_items_from_abi: Vec<&T> = items_from_abi
        .into_iter()
        .filter(|x| x.name().split("::").last() == path.last().map(String::as_str))
        .collect();

    ensure!(
        !matching_items_from_abi.is_empty(),
        r#"{} "{}" not found in ABI"#,
        T::VARIANT_CAPITALIZED,
        path.join("::")
    );

    ensure!(
        matching_items_from_abi.len() == 1,
        r#"Found more than one {} "{}" in ABI, please specify a full path to the item"#,
        T::VARIANT,
        path.join("::")
    );

    Ok(matching_items_from_abi.pop().unwrap())
}

fn get_struct_arguments_with_values(
    arguments: &[StructArg],
    db: &SimpleParserDatabase,
) -> Result<Vec<(String, Expr)>> {
    arguments
        .iter()
        .map(|elem| {
            match elem {
                // Holds info about parameter and argument in struct creation, e.g.:
                // in case of "Struct { a: 1, b: 2 }", two separate StructArgSingle hold info
                // about "a: 1" and "b: 2" respectively.
                StructArg::StructArgSingle(whole_arg) => {
                    match whole_arg.arg_expr(db) {
                        // It's probably a case of constructor invocation `Struct {a, b}` catching variables `a` and `b` from context
                        OptionStructArgExpr::Empty(_) => {
                            bail!(
                                "Shorthand arguments are not allowed - used \"{ident}\", expected \"{ident}: value\"",
                                ident = whole_arg.identifier(db).text(db)
                            )
                        }
                        // Holds info about the argument, e.g.: in case of "a: 1" holds info
                        // about ": 1"
                        OptionStructArgExpr::StructArgExpr(arg_value_with_colon) => Ok((
                            whole_arg.identifier(db).text(db).to_string(),
                            arg_value_with_colon.expr(db),
                        )),
                    }
                }
                StructArg::StructArgTail(_) => {
                    bail!("Struct initialization with \"..\" operator is not allowed")
                }
            }
        })
        .collect()
}

// Structs
impl SupportedCalldataKind for ExprStructCtorCall {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let struct_path: Vec<String> = split(&self.path(db), db)?;
        let struct_path_joined = struct_path.clone().join("::");

        validate_path_argument(expected_type, &struct_path, &struct_path_joined)?;

        let structs_from_abi = find_all_structs(abi);
        let struct_abi_definition = find_item_with_path(structs_from_abi, &struct_path)?;

        let struct_args = self.arguments(db).arguments(db).elements(db);

        let struct_args_with_values = get_struct_arguments_with_values(&struct_args, db)
            .context("Found invalid expression in struct argument")?;

        if struct_args_with_values.len() != struct_abi_definition.members.len() {
            bail!(
                r#"Invalid number of struct arguments in struct "{}", expected {} arguments, found {}"#,
                struct_path_joined,
                struct_abi_definition.members.len(),
                struct_args.len()
            )
        }

        // validate if all arguments' names have corresponding names in abi
        if struct_args_with_values
            .iter()
            .map(|(arg_name, _)| arg_name.clone())
            .collect::<HashSet<String>>()
            != struct_abi_definition
                .members
                .iter()
                .map(|x| x.name.clone())
                .collect::<HashSet<String>>()
        {
            // TODO add message which arguments are invalid (Issue #2549)
            bail!(
                r"Arguments in constructor invocation for struct {} do not match struct arguments in ABI",
                expected_type
            )
        }

        let fields = struct_args_with_values
            .into_iter()
            .map(|(arg_name, expr)| {
                let abi_entry = struct_abi_definition
                    .members
                    .iter()
                    .find(|&abi_member| abi_member.name == arg_name)
                    .expect("Arg name should be in ABI - it is checked before with HashSets");
                Ok(CalldataStructField::new(build_representation(
                    expr,
                    &abi_entry.r#type,
                    abi,
                    db,
                )?))
            })
            .collect::<Result<Vec<CalldataStructField>>>()?;

        Ok(AllowedCalldataArgument::Struct(CalldataStruct::new(fields)))
    }
}

// Unit enum variants
impl SupportedCalldataKind for ExprPath {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        // Enums with no value - Enum::Variant
        let enum_path_with_variant = split(self, db)?;
        let (enum_variant_name, enum_path) = enum_path_with_variant.split_last().unwrap();
        let enum_path_joined = enum_path.join("::");

        validate_path_argument(expected_type, enum_path, &enum_path_joined)?;

        let (enum_position, enum_variant) =
            find_enum_variant_position(enum_variant_name, enum_path, abi)?;

        if enum_variant.r#type != "()" {
            bail!(
                r#"Couldn't find variant "{}" in enum "{}""#,
                enum_variant_name,
                enum_path_joined
            )
        }

        Ok(AllowedCalldataArgument::Enum(CalldataEnum::new(
            enum_position,
            None,
        )))
    }
}

// Tuple-like enum variants
impl SupportedCalldataKind for ExprFunctionCall {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        // Enums with value - Enum::Variant(10)
        let enum_path_with_variant = split(&self.path(db), db)?;
        let (enum_variant_name, enum_path) = enum_path_with_variant.split_last().unwrap();
        let enum_path_joined = enum_path.join("::");

        validate_path_argument(expected_type, enum_path, &enum_path_joined)?;

        let (enum_position, enum_variant) =
            find_enum_variant_position(enum_variant_name, enum_path, abi)?;

        // Enum variant constructor invocation has one argument - an ArgList.
        // We parse it to a vector of expressions and pop + unwrap safely.
        let expr = parse_argument_list(&self.arguments(db).arguments(db), db)?
            .pop()
            .unwrap();

        let parsed_expr = build_representation(expr, &enum_variant.r#type, abi, db)?;

        Ok(AllowedCalldataArgument::Enum(CalldataEnum::new(
            enum_position,
            Some(Box::new(parsed_expr)),
        )))
    }
}

// Tuples
impl SupportedCalldataKind for ExprListParenthesized {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let Expr::Tuple(tuple) = parse_expression(expected_type, db)? else {
            bail!(
                r#"Invalid argument type, expected "{}", got tuple"#,
                expected_type
            );
        };

        let tuple_types = tuple
            .expressions(db)
            .elements(db)
            .into_iter()
            .map(|element| match element {
                Expr::Path(path) => Ok(path.as_syntax_node().get_text(db)),
                other => bail!(
                    "Unexpected expression found in ABI: {}. Contract ABI may be invalid",
                    other.as_syntax_node().get_text(db)
                ),
            })
            .collect::<Result<Vec<_>>>()?;

        let parsed_exprs = self
            .expressions(db)
            .elements(db)
            .into_iter()
            .zip(tuple_types)
            .map(|(expr, ref single_param)| build_representation(expr, single_param, abi, db))
            .collect::<Result<Vec<_>>>()?;

        Ok(AllowedCalldataArgument::Tuple(CalldataTuple::new(
            parsed_exprs,
        )))
    }
}
