use super::data_representation::{
    AllowedCalldataArgument, CalldataEnum, CalldataStruct, CalldataStructField, CalldataTuple,
};
use super::parsing::parse_argument_list;
use super::{SupportedCalldataKind, build_representation};
use crate::shared;
use crate::shared::parsing::parse_expression;
use crate::shared::path::SplitResult;
use anyhow::{Context, Result, bail, ensure};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{
    Expr, ExprFunctionCall, ExprListParenthesized, ExprPath, ExprStructCtorCall,
    OptionStructArgExpr, StructArg,
};
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use itertools::Itertools;
use starknet_rust::core::types::contract::{AbiEntry, AbiEnum, AbiNamedMember, AbiStruct};
use std::collections::HashSet;

pub trait EnumOrStruct {
    const VARIANT: &'static str;
    const VARIANT_CAPITALIZED: &'static str;
    fn name(&self) -> String;
}

/// A resolved enum variant with its serialization position and optional inner type.
struct ResolvedEnumVariant<'a> {
    /// 0-based position used for serialization
    position: usize,
    /// Inner type, or `None` for unit variants
    inner_type: Option<&'a str>,
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

const UNIT_TYPE: &str = "()";

fn strip_generic_suffix(type_str: &str) -> &str {
    if let Some(idx) = type_str.find("::<") {
        &type_str[..idx]
    } else {
        type_str
    }
}

fn base_type_name(type_str: &str) -> &str {
    strip_generic_suffix(type_str)
        .split("::")
        .last()
        .unwrap_or(type_str)
}

fn validate_path_argument(
    param_type: &str,
    path_argument: &[String],
    path_argument_joined: &str,
) -> Result<()> {
        .last()
        .expect("path_argument must be non-empty: caller ensures split_last() succeeded");
    if *last != base_type_name(param_type) && path_argument_joined != param_type {
    let last = path_argument
        bail!(r#"Invalid argument type, expected "{param_type}", got "{path_argument_joined}""#)
    }
    Ok(())
}

fn is_valid_corelib_enum_path(type_name: &str, enum_path: &[String]) -> bool {
    let module = type_name.to_lowercase();
    matches!(enum_path, [only] if only == type_name)
        || matches!(enum_path, [core, m, last] if core == "core" && m == &module && last == type_name)
}

/// Validates the user-supplied enum path against the expected corelib type, then
/// looks up `variant_name` in `variants` (a slice of `(name, position, inner_type)`).
fn resolve_corelib_enum_variant_with<'a>(
    expected_type: &'a str,
    type_name: &str,
    enum_path: &[String],
    variant_name: &str,
    variants: &[(&str, usize, Option<&'a str>)],
) -> Result<ResolvedEnumVariant<'a>> {
    if !is_valid_corelib_enum_path(type_name, enum_path) {
        return Err(anyhow::anyhow!(
            r#"Invalid argument type, expected "{expected_type}", got "{}""#,
            enum_path.join("::")
        ));
    }
    variants
        .iter()
        .find(|(name, _, _)| *name == variant_name)
        .map(|&(_, position, inner_type)| ResolvedEnumVariant {
            position,
            inner_type,
        })
        .ok_or_else(|| {
            anyhow::anyhow!(r#"Invalid variant "{variant_name}" for type "{expected_type}""#)
        })
}

fn resolve_corelib_enum_variant<'a>(
    expected_type: &'a str,
    variant_name: &str,
    enum_path: &[String],
) -> Option<Result<ResolvedEnumVariant<'a>>> {
    // core::option::Option::<T>  ->  Some(T) at 0, None at 1
    if let Some(inner) = expected_type
        .strip_prefix("core::option::Option::<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return Some(resolve_corelib_enum_variant_with(
            expected_type,
            "Option",
            enum_path,
            variant_name,
            &[("Some", 0, Some(inner)), ("None", 1, None)],
        ));
    }

    // core::result::Result::<T, E>  ->  Ok(T) at 0, Err(E) at 1
    if let Some(inner) = expected_type
        .strip_prefix("core::result::Result::<")
        .and_then(|s| s.strip_suffix('>'))
    {
        // A well-formed Result type always contains a top-level comma separating T and E.
        // If absent the ABI entry is malformed; fall through to ABI lookup.
        let split_pos = top_level_comma_pos(inner)?;
        let ok_type = inner[..split_pos].trim();
        let err_type = inner[split_pos + 1..].trim();
        return Some(resolve_corelib_enum_variant_with(
            expected_type,
            "Result",
            enum_path,
            variant_name,
            &[("Ok", 0, Some(ok_type)), ("Err", 1, Some(err_type))],
        ));
    }

    None
}

fn top_level_comma_pos(s: &str) -> Option<usize> {
    let mut angle_depth = 0;
    let mut paren_depth = 0;
    s.char_indices().find_map(|(i, c)| {
        match c {
            '<' => angle_depth += 1,
            '>' => angle_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            ',' if angle_depth == 0 && paren_depth == 0 => return Some(i),
            _ => {}
        }
        None
    })
}

fn split_variant_from_path(path: &[String]) -> Result<(&str, &[String])> {
    let (variant, rest) = path
        .split_last()
        .ok_or_else(|| anyhow::anyhow!("Expected an enum variant path, got an empty path"))?;
    Ok((variant, rest))
}

fn split(path: &ExprPath, db: &SimpleParserDatabase) -> Result<Vec<String>> {
    match shared::path::split(path, db)? {
        SplitResult::Simple { splits } => Ok(splits),
        SplitResult::WithGenericArgs { .. } => {
            bail!("Cannot use generic args when specifying struct/enum path")
        }
    }
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
    variant: &str,
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
        let path_joined = path.join("::");
        let full_path_item = items_from_abi.into_iter().find(|x| {
            let name = x.name();
            // Also match monomorphized generics, e.g. ABI name `foo::Bar::<u32>` against path `foo::Bar`
            name == path_joined || strip_generic_suffix(&name) == path_joined
        });

        return full_path_item.ok_or_else(|| {
            anyhow::anyhow!(
                r#"{} "{}" not found in ABI"#,
                T::VARIANT_CAPITALIZED,
                path.join("::")
            )
        });
    }

    // Argument is just the name of the item (Struct {})
    let mut matching_items_from_abi: Vec<&T> = items_from_abi
        .into_iter()
        .filter(|x| base_type_name(&x.name()) == path.last().map_or("", String::as_str))
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

fn get_struct_arguments_with_values<'a>(
    arguments: &'a [StructArg<'a>],
    db: &'a SimpleParserDatabase,
) -> Result<Vec<(String, Expr<'a>)>> {
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
                                ident = whole_arg.identifier(db).text(db).to_string(db)
                            )
                        }
                        // Holds info about the argument, e.g.: in case of "a: 1" holds info
                        // about ": 1"
                        OptionStructArgExpr::StructArgExpr(arg_value_with_colon) => Ok((
                            whole_arg.identifier(db).text(db).to_string(db),
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
impl SupportedCalldataKind for ExprStructCtorCall<'_> {
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

        let struct_args = self
            .arguments(db)
            .arguments(db)
            .elements(db)
            .collect::<Vec<_>>();

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
                r"Arguments in constructor invocation for struct {expected_type} do not match struct arguments in ABI",
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

fn resolve_enum_variant<'a>(
    variant_name: &str,
    enum_path: &[String],
    expected_type: &'a str,
    abi: &'a [AbiEntry],
) -> Result<ResolvedEnumVariant<'a>> {
    let enum_path_joined = enum_path.join("::");
    validate_path_argument(expected_type, enum_path, &enum_path_joined)?;

    if let Some(result) = resolve_corelib_enum_variant(expected_type, variant_name, enum_path) {
        result
    } else {
        let (position, variant) = find_enum_variant_position(variant_name, enum_path, abi)?;
        Ok(ResolvedEnumVariant {
            position,
            inner_type: Some(variant.r#type.as_str()).filter(|t| *t != UNIT_TYPE),
        })
    }
}

// Unit enum variants
impl SupportedCalldataKind for ExprPath<'_> {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let path = split(self, db)?;
        let (enum_variant_name, enum_path) = split_variant_from_path(&path)?;

        let resolved = resolve_enum_variant(enum_variant_name, enum_path, expected_type, abi)?;

        ensure!(
            resolved.inner_type.is_none(),
            r#"Variant "{enum_variant_name}" of "{expected_type}" requires a value, use "{enum_variant_name}(<value>)""#
        );

        if enum_variant.r#type != "()" {
            bail!(r#"Couldn't find variant "{enum_variant_name}" in enum "{enum_path_joined}""#)
        }

        Ok(AllowedCalldataArgument::Enum(CalldataEnum::new(
            resolved.position,
            None,
        )))
    }
}

// Tuple-like enum variants
impl SupportedCalldataKind for ExprFunctionCall<'_> {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let path = split(&self.path(db), db)?;
        let (enum_variant_name, enum_path) = split_variant_from_path(&path)?;

        let resolved = resolve_enum_variant(enum_variant_name, enum_path, expected_type, abi)?;

        let inner_type = resolved.inner_type.with_context(|| {
            format!(r#"Variant "{enum_variant_name}" of "{expected_type}" takes no value"#)
        })?;

        let arguments = self.arguments(db).arguments(db);

        let mut args_list = parse_argument_list(&arguments, db)?;
        ensure!(
            args_list.len() == 1,
            r#"Variant "{enum_variant_name}" of "{expected_type}" expects exactly 1 argument, got {}"#,
            args_list.len()
        );
        let expr = args_list.pop().unwrap();

        let parsed_expr = build_representation(expr, inner_type, abi, db)?;

        Ok(AllowedCalldataArgument::Enum(CalldataEnum::new(
            resolved.position,
            Some(Box::new(parsed_expr)),
        )))
    }
}

// Tuples
impl SupportedCalldataKind for ExprListParenthesized<'_> {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let Expr::Tuple(tuple) = parse_expression(expected_type, db)? else {
            bail!(r#"Invalid argument type, expected "{expected_type}", got tuple"#);
        };

        let tuple_types = tuple
            .expressions(db)
            .elements(db)
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
            .zip(tuple_types)
            .map(|(expr, single_param)| build_representation(expr, single_param, abi, db))
            .collect::<Result<Vec<_>>>()?;

        Ok(AllowedCalldataArgument::Tuple(CalldataTuple::new(
            parsed_exprs,
        )))
    }
}
