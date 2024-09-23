use crate::helpers::data_transformer::transformer::{
    AllowedCalldataArguments, CalldataArrayMacro, CalldataEnum, CalldataSingleArgument,
    CalldataStruct, CalldataStructField, CalldataTuple,
};
use anyhow::{bail, Context, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::PathSegment::Simple;
use cairo_lang_syntax::node::ast::{
    ArgClause, ArgList, Expr, ExprInlineMacro, ExprPath, OptionStructArgExpr, PathSegment,
    StructArg, UnaryOperator, WrappedArgList,
};
use cairo_lang_syntax::node::{Terminal, Token};
use itertools::Itertools;
use regex::Regex;
use starknet::core::types::contract::{AbiEntry, AbiEnum, AbiNamedMember, AbiStruct};
use std::collections::HashSet;
use std::ops::Neg;

fn parse_expr_path_to_path_elements(
    expr_path: &ExprPath,
    db: &SimpleParserDatabase,
) -> Result<Vec<String>> {
    expr_path
        .elements(db)
        .iter()
        .map(|p| match p {
            Simple(segment) => Ok(segment.ident(db).token(db).text(db).to_string()),
            PathSegment::WithGenericArgs(_) => {
                bail!("Cannot use generic args when specifying struct/enum path")
            }
        })
        .collect::<Result<Vec<String>>>()
}

fn get_struct_args_with_values(
    struct_args: &[StructArg],
    db: &SimpleParserDatabase,
) -> Result<Vec<(String, Expr)>> {
    struct_args
        .iter()
        .map(|elem| {
            match elem {
                // Holds info about parameter and argument in struct creation, e.g.:
                // in case of "Struct { a: 1, b: 2 }", two separate StructArgSingle hold info
                // about "a: 1" and "b: 2" respectively.
                StructArg::StructArgSingle(whole_arg) => {
                    match whole_arg.arg_expr(db) {
                        // TODO add comment
                        // dunno what that is
                        // probably case when there is Struct {a, b} and there are variables a and b
                        OptionStructArgExpr::Empty(_) => {
                            bail!(
                                "Single arg, used {ident}, should be {ident}: value",
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
                // TODO add more meaningful message
                // the part with dots in Struct { ..smth }
                StructArg::StructArgTail(_) => {
                    bail!("..value is not allowed")
                }
            }
        })
        .collect()
}

//TODO rename to find or sth else
fn parse_enum_expr_path<'a>(
    enum_variant: &String,
    enum_path: &[String],
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

    let enum_abi_definition = find_valid_struct(enums_from_abi, enum_path)?;

    let position_and_enum_variant = enum_abi_definition
        .variants
        .iter()
        .find_position(|variant| variant.name == *enum_variant)
        .context(format!(
            r#"Couldn't find variant "{}" in enum "{}""#,
            enum_variant,
            enum_path.join("::")
        ))?;

    Ok(position_and_enum_variant)
}

fn arg_list_to_exprs(arg_list: &ArgList, db: &SimpleParserDatabase) -> Result<Vec<Expr>> {
    let arguments = arg_list.elements(db);
    if arguments
        .iter()
        .map(|arg| arg.modifiers(db).elements(db))
        .any(|mod_list| !mod_list.is_empty())
    {
        // TODO better message
        bail!("Cannot use ref/mut modifiers")
    }
    arguments
        .iter()
        .map(|arg| match arg.arg_clause(db) {
            ArgClause::Unnamed(unnamed_arg) => Ok(unnamed_arg.value(db)),
            ArgClause::Named(_) | ArgClause::FieldInitShorthand(_) => {
                // tODO better message
                bail!("Neither named args nor named arguments/field init shorthand are supported")
            }
        })
        .collect::<Result<Vec<Expr>>>()
}

fn parse_inline_macro_expr(
    expr_inline_macro: &ExprInlineMacro,
    db: &SimpleParserDatabase,
) -> Result<Vec<Expr>> {
    match expr_inline_macro
        .path(db)
        .elements(db)
        .iter()
        .last()
        .expect("Macro must have a name")
    {
        Simple(simple) => {
            let macro_name = simple.ident(db).text(db);
            if macro_name != "array" {
                bail!(
                    r#"Invalid macro name, expected "array![]", got "{}""#,
                    macro_name
                )
            }
        }
        PathSegment::WithGenericArgs(_) => {
            bail!("Invalid path specified: generic args in array![] macro not supported")
        }
    };

    let macro_arg_list = match expr_inline_macro.arguments(db) {
        WrappedArgList::BracketedArgList(args) => {
            // TODO arglist parsing here
            args.arguments(db)
        }
        WrappedArgList::ParenthesizedArgList(_) | WrappedArgList::BracedArgList(_) =>
            bail!("`array` macro supports only square brackets: array![]"),
        WrappedArgList::Missing(_) => unreachable!("If any type of parentheses is missing, then diagnostics have been reported and whole flow should have already been terminated.")
    };
    arg_list_to_exprs(&macro_arg_list, db)
}

fn find_new_abi_structs(abi: &[AbiEntry]) -> Vec<&AbiStruct> {
    abi.iter()
        .filter_map(|abi_entry| {
            if let AbiEntry::Struct(r#struct) = abi_entry {
                return Some(r#struct);
            }
            None
        })
        .collect()
}

fn validate_path_argument(
    param_type: &String,
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

trait EnumStructCommonTrait {
    const VARIANT: &'static str;
    const VARIANT_CAPITALIZED: &'static str;
    fn name(&self) -> String;
}

impl EnumStructCommonTrait for AbiStruct {
    const VARIANT: &'static str = "struct";
    const VARIANT_CAPITALIZED: &'static str = "Struct";
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl EnumStructCommonTrait for AbiEnum {
    const VARIANT: &'static str = "enum";
    const VARIANT_CAPITALIZED: &'static str = "Enum";
    fn name(&self) -> String {
        self.name.clone()
    }
}

fn find_valid_struct<'a, T: EnumStructCommonTrait>(
    structs_from_abi: Vec<&'a T>,
    struct_path: &[String],
) -> Result<&'a T> {
    // Argument is just the name of the struct (Struct{})
    if struct_path.len() == 1 {
        let mut matching_structs_from_abi: Vec<&T> = structs_from_abi
            .into_iter()
            .filter(|x| x.name().split("::").last() == struct_path.last().map(String::as_str))
            .collect();

        if matching_structs_from_abi.len() > 1 {
            bail!(
                r#"Found more than one {} "{}" in ABI, please specify a full path to the struct"#,
                T::VARIANT,
                struct_path.join("::")
            )
        } else if matching_structs_from_abi.is_empty() {
            bail!(
                r#"{} "{}" not found in ABI"#,
                T::VARIANT_CAPITALIZED,
                struct_path.join("::")
            )
        }

        return Ok(matching_structs_from_abi.pop().unwrap());
    }

    // Argument is a module path to a struct (a::Struct{})
    let maybe_full_path_struct = structs_from_abi
        .into_iter()
        .find(|x| x.name() == struct_path.join("::"));

    if maybe_full_path_struct.is_none() {
        bail!(
            r#"{} "{}" not found in ABI"#,
            T::VARIANT_CAPITALIZED,
            struct_path.join("::")
        )
    }

    Ok(maybe_full_path_struct.unwrap())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn parse_expr(
    expr: Expr,
    param_type: String,
    abi: &Vec<AbiEntry>,
    db: &SimpleParserDatabase,
) -> Result<AllowedCalldataArguments> {
    match expr {
        Expr::StructCtorCall(expr_struct_ctor_call) => {
            let struct_path: Vec<String> =
                parse_expr_path_to_path_elements(&expr_struct_ctor_call.path(db), db)?;
            let struct_path_joined = struct_path.clone().join("::");

            validate_path_argument(&param_type, &struct_path, &struct_path_joined)?;

            let structs_from_abi = find_new_abi_structs(abi);
            let struct_abi_definition = find_valid_struct(structs_from_abi, &struct_path)?;

            let struct_args = expr_struct_ctor_call
                .arguments(db)
                .arguments(db)
                .elements(db);

            let struct_args_with_values = get_struct_args_with_values(&struct_args, db)
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
                // TODO add message which arguments are invalid
                bail!(
                    r#"Arguments in constructor invocation for struct {} do not match struct arguments in ABI"#,
                    param_type
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
                    Ok(CalldataStructField::new(parse_expr(
                        expr,
                        abi_entry.r#type.clone(),
                        abi,
                        db,
                    )?))
                })
                .collect::<Result<Vec<CalldataStructField>>>()?;

            Ok(AllowedCalldataArguments::Struct(CalldataStruct::new(
                fields,
            )))
        }
        Expr::Literal(literal_number) => {
            let (value, suffix) = literal_number
                .numeric_value_and_suffix(db)
                .context(format!("Couldn't parse value: {}", literal_number.text(db)))?;

            let proper_param_type = match suffix {
                None => param_type,
                Some(suffix) => suffix.to_string(),
            };

            Ok(AllowedCalldataArguments::SingleArgument(
                CalldataSingleArgument::try_new(&proper_param_type, &value.to_string())?,
            ))
        }
        Expr::Unary(literal_number_with_operator) => {
            let (value, suffix) = match literal_number_with_operator.expr(db) {
                Expr::Literal(literal_number) => literal_number
                    .numeric_value_and_suffix(db)
                    .context(format!("Couldn't parse value: {}", literal_number.text(db))),
                _ => bail!("Invalid expression with unary operator, only numbers allowed"),
            }?;

            let proper_param_type = match suffix {
                None => param_type,
                Some(suffix) => suffix.to_string(),
            };

            match literal_number_with_operator.op(db) {
                UnaryOperator::Not(_) => bail!(
                    "Invalid unary operator in expression !{} , only - allowed, got !",
                    value
                ),
                UnaryOperator::Desnap(_) => bail!(
                    "Invalid unary operator in expression *{} , only - allowed, got *",
                    value
                ),
                UnaryOperator::BitNot(_) => bail!(
                    "Invalid unary operator in expression ~{} , only - allowed, got ~",
                    value
                ),
                UnaryOperator::At(_) => bail!(
                    "Invalid unary operator in expression @{} , only - allowed, got @",
                    value
                ),
                UnaryOperator::Minus(_) => {}
            }

            Ok(AllowedCalldataArguments::SingleArgument(
                CalldataSingleArgument::try_new(&proper_param_type, &value.neg().to_string())?,
            ))
        }
        Expr::ShortString(terminal_short_string) => {
            let value = terminal_short_string
                .string_value(db)
                .context("Invalid shortstring passed as an argument")?;

            Ok(AllowedCalldataArguments::SingleArgument(
                CalldataSingleArgument::try_new(&param_type, &value)?,
            ))
        }
        Expr::String(terminal_string) => {
            let value = terminal_string
                .string_value(db)
                .context("Invalid string passed as an argument")?;

            Ok(AllowedCalldataArguments::SingleArgument(
                CalldataSingleArgument::try_new(&param_type, &value)?,
            ))
        }
        Expr::False(terminal_false) => {
            // Could use terminal_false.boolean_value(db) and simplify try_new()
            let value = terminal_false.text(db).to_string();

            Ok(AllowedCalldataArguments::SingleArgument(
                CalldataSingleArgument::try_new(&param_type, &value)?,
            ))
        }
        Expr::True(terminal_true) => {
            // Could use terminal_true.boolean_value(db) and simplify try_new()
            let value = terminal_true.text(db).to_string();

            Ok(AllowedCalldataArguments::SingleArgument(
                CalldataSingleArgument::try_new(&param_type, &value)?,
            ))
        }
        Expr::Path(enum_expr_path) => {
            // Enums with no value - Enum::Variant
            let enum_path_with_variant = parse_expr_path_to_path_elements(&enum_expr_path, db)?;
            let (enum_variant_name, enum_path) = enum_path_with_variant.split_last().unwrap();
            let enum_path_joined = enum_path.join("::");

            validate_path_argument(&param_type, enum_path, &enum_path_joined)?;

            let (enum_position, enum_variant) =
                parse_enum_expr_path(enum_variant_name, enum_path, abi)?;

            if enum_variant.r#type != "()" {
                bail!(
                    r#"Couldn't find variant "{}" in enum "{}""#,
                    enum_variant_name,
                    enum_path_joined
                )
            }

            Ok(AllowedCalldataArguments::Enum(CalldataEnum::new(
                enum_position,
                None,
            )))
        }
        Expr::FunctionCall(enum_expr_path_with_value) => {
            // Enums with value - Enum::Variant(10)
            let enum_path_with_variant =
                parse_expr_path_to_path_elements(&enum_expr_path_with_value.path(db), db)?;
            let (enum_variant_name, enum_path) = enum_path_with_variant.split_last().unwrap();
            let enum_path_joined = enum_path.join("::");

            validate_path_argument(&param_type, enum_path, &enum_path_joined)?;

            let (enum_position, enum_variant) =
                parse_enum_expr_path(enum_variant_name, enum_path, abi)?;

            // When creating an enum with variant, there can be only one argument. Parsing the
            // argument inside ArgList (enum_expr_path_with_value.arguments(db).arguments(db)),
            // then popping from the vector and unwrapping safely.
            let expr =
                arg_list_to_exprs(&enum_expr_path_with_value.arguments(db).arguments(db), db)?
                    .pop()
                    .unwrap();
            let parsed_expr = parse_expr(expr, enum_variant.r#type.clone(), abi, db)?;

            Ok(AllowedCalldataArguments::Enum(CalldataEnum::new(
                enum_position,
                Some(Box::new(parsed_expr)),
            )))
        }
        Expr::InlineMacro(expr_inline_macro) => {
            // array![] calls
            let parsed_exprs = parse_inline_macro_expr(&expr_inline_macro, db)?;

            let array_element_type_pattern = Regex::new("core::array::Array::<(.*)>").unwrap();
            let abi_argument_type = array_element_type_pattern
                .captures(param_type.as_str())
                .context(format!(
                    r#"Invalid argument type, expected "{param_type}", got array"#,
                ))?
                .get(1)
                // TODO better message
                .context(format!(
                    "Couldn't parse array element type from the ABI array parameter: {param_type}"
                ))?
                .as_str();

            let arguments = parsed_exprs
                .into_iter()
                .map(|arg| parse_expr(arg, abi_argument_type.to_string(), abi, db))
                .collect::<Result<Vec<AllowedCalldataArguments>>>()?;

            Ok(AllowedCalldataArguments::ArrayMacro(
                CalldataArrayMacro::new(arguments),
            ))
        }
        Expr::Tuple(expr_list_parenthesized) => {
            // Regex capturing types between the parentheses, e.g.: for "(core::felt252, core::u8)"
            // will capture "core::felt252, core::u8"
            let tuple_types_pattern = Regex::new(r"\(([^)]+)\)").unwrap();
            let tuple_types: Vec<&str> = tuple_types_pattern
                .captures(param_type.as_str())
                .context(format!(
                    r#"Invalid argument type, expected "{param_type}", got tuple"#,
                ))?
                .get(1)
                .map(|x| x.as_str().split(", ").collect())
                .unwrap();

            let parsed_exprs = expr_list_parenthesized
                .expressions(db)
                .elements(db)
                .into_iter()
                .zip(tuple_types)
                .map(|(expr, single_param)| parse_expr(expr, single_param.to_string(), abi, db))
                .collect::<Result<Vec<_>>>()?;

            Ok(AllowedCalldataArguments::Tuple(CalldataTuple::new(
                parsed_exprs,
            )))
        }
        _ => {
            bail!(
                r#"Invalid argument type: unsupported expression for type "{}""#,
                param_type
            )
        } // TODO remove that comment
          // other possibilities are:
          // Expr::Binary(_) => {} - generics
          // Expr::Parenthesized(_) => {} - single value tuples, e.g. "(1)"
          //  Expr::Block(_) => {}
          //  Expr::Match(_) => {}
          //  Expr::If(_) => {}
          //  Expr::Loop(_) => {}
          //  Expr::While(_) => {}
          //  Expr::ErrorPropagate(_) => {}
          //  Expr::FieldInitShorthand(_) => {}
          //  Expr::Indexed(_) => {}
          //  Expr::FixedSizeArray(_) => {}
          //  Expr::Missing(_) => {}
    }
}
