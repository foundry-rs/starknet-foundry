use crate::helpers::data_transformer::calldata_representation::{
    AllowedCalldataArguments, CalldataArrayMacro, CalldataEnum, CalldataSingleArgument,
    CalldataStruct, CalldataStructField, CalldataTuple,
};
use anyhow::{bail, ensure, Context, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::PathSegment::Simple;
use cairo_lang_syntax::node::ast::{
    ArgClause, ArgList, Expr, ExprFunctionCall, ExprInlineMacro, ExprListParenthesized, ExprPath,
    ExprStructCtorCall, ExprUnary, OptionStructArgExpr, PathSegment, StructArg, TerminalFalse,
    TerminalLiteralNumber, TerminalShortString, TerminalString, TerminalTrue, UnaryOperator,
    WrappedArgList,
};
use cairo_lang_syntax::node::{Terminal, Token};
use itertools::Itertools;
use regex::Regex;
use starknet::core::types::contract::{AbiEntry, AbiEnum, AbiNamedMember, AbiStruct};
use std::collections::HashSet;
use std::ops::Neg;

pub(super) fn build_representation(
    expression: Expr,
    expected_type: &str,
    abi: &[AbiEntry],
    db: &SimpleParserDatabase,
) -> Result<AllowedCalldataArguments> {
    match expression {
        Expr::StructCtorCall(item) => item.transform(expected_type, abi, db),
        Expr::Literal(item) => item.transform(expected_type, abi, db),
        Expr::Unary(item) => item.transform(expected_type, abi, db),
        Expr::ShortString(item) => item.transform(expected_type, abi, db),
        Expr::String(item) => item.transform(expected_type, abi, db),
        Expr::False(item) => item.transform(expected_type, abi, db),
        Expr::True(item) => item.transform(expected_type, abi, db),
        Expr::Path(item) => item.transform(expected_type, abi, db),
        Expr::FunctionCall(item) => item.transform(expected_type, abi, db),
        Expr::InlineMacro(item) => item.transform(expected_type, abi, db),
        Expr::Tuple(item) => item.transform(expected_type, abi, db),
        _ => {
            bail!(
                r#"Invalid argument type: unsupported expression for type "{}""#,
                expected_type
            )
        }
    }
}

trait SupportedCalldataKind {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments>;
}

impl SupportedCalldataKind for ExprStructCtorCall {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        let struct_path: Vec<String> = split(&self.path(db), db)?;
        let struct_path_joined = struct_path.clone().join("::");

        validate_path_argument(expected_type, &struct_path, &struct_path_joined)?;

        let structs_from_abi = find_all_structs(abi);
        let struct_abi_definition = find_valid_enum_or_struct(structs_from_abi, &struct_path)?;

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
            // TODO add message which arguments are invalid
            bail!(
                r#"Arguments in constructor invocation for struct {} do not match struct arguments in ABI"#,
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

        Ok(AllowedCalldataArguments::Struct(CalldataStruct::new(
            fields,
        )))
    }
}

impl SupportedCalldataKind for TerminalLiteralNumber {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        let (value, suffix) = self
            .numeric_value_and_suffix(db)
            .with_context(|| format!("Couldn't parse value: {}", self.text(db)))?;

        let proper_param_type = match suffix {
            None => expected_type,
            Some(ref suffix) => suffix.as_str(),
        };

        Ok(AllowedCalldataArguments::SingleArgument(
            CalldataSingleArgument::try_new(proper_param_type, &value.to_string())?,
        ))
    }
}

impl SupportedCalldataKind for ExprUnary {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        let (value, suffix) = match self.expr(db) {
            Expr::Literal(literal_number) => literal_number
                .numeric_value_and_suffix(db)
                .with_context(|| format!("Couldn't parse value: {}", literal_number.text(db))),
            _ => bail!("Invalid expression with unary operator, only numbers allowed"),
        }?;

        let proper_param_type = match suffix {
            None => expected_type,
            Some(ref suffix) => suffix.as_str(),
        };

        match self.op(db) {
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
            CalldataSingleArgument::try_new(proper_param_type, &value.neg().to_string())?,
        ))
    }
}

impl SupportedCalldataKind for TerminalShortString {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        let value = self
            .string_value(db)
            .context("Invalid shortstring passed as an argument")?;

        Ok(AllowedCalldataArguments::SingleArgument(
            CalldataSingleArgument::try_new(expected_type, &value)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalString {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        let value = self
            .string_value(db)
            .context("Invalid string passed as an argument")?;

        Ok(AllowedCalldataArguments::SingleArgument(
            CalldataSingleArgument::try_new(expected_type, &value)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalFalse {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        // Could use terminal_false.boolean_value(db) and simplify try_new()
        let value = self.text(db).to_string();

        Ok(AllowedCalldataArguments::SingleArgument(
            CalldataSingleArgument::try_new(expected_type, &value)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalTrue {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        let value = self.text(db).to_string();

        Ok(AllowedCalldataArguments::SingleArgument(
            CalldataSingleArgument::try_new(expected_type, &value)?,
        ))
    }
}

impl SupportedCalldataKind for ExprPath {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
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

        Ok(AllowedCalldataArguments::Enum(CalldataEnum::new(
            enum_position,
            None,
        )))
    }
}

impl SupportedCalldataKind for ExprFunctionCall {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        // Enums with value - Enum::Variant(10)
        let enum_path_with_variant = split(&self.path(db), db)?;
        let (enum_variant_name, enum_path) = enum_path_with_variant.split_last().unwrap();
        let enum_path_joined = enum_path.join("::");

        validate_path_argument(expected_type, enum_path, &enum_path_joined)?;

        let (enum_position, enum_variant) =
            find_enum_variant_position(enum_variant_name, enum_path, abi)?;

        // When creating an enum with variant, there can be only one argument. Parsing the
        // argument inside ArgList (enum_expr_path_with_value.arguments(db).arguments(db)),
        // then popping from the vector and unwrapping safely.
        let expr = parse_argument_list(&self.arguments(db).arguments(db), db)?
            .pop()
            .unwrap();
        let parsed_expr = build_representation(expr, &enum_variant.r#type, abi, db)?;

        Ok(AllowedCalldataArguments::Enum(CalldataEnum::new(
            enum_position,
            Some(Box::new(parsed_expr)),
        )))
    }
}

impl SupportedCalldataKind for ExprInlineMacro {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        // array![] calls
        let parsed_exprs = parse_inline_macro(self, db)?;

        let array_element_type_pattern = Regex::new("core::array::Array::<(.*)>").unwrap();
        let abi_argument_type = array_element_type_pattern
            .captures(expected_type)
            .with_context(|| {
                format!(r#"Invalid argument type, expected "{expected_type}", got array"#,)
            })?
            .get(1)
            // TODO better message
            .with_context(|| {
                format!(
                "Couldn't parse array element type from the ABI array parameter: {expected_type}"
            )
            })?
            .as_str();

        let arguments = parsed_exprs
            .into_iter()
            .map(|arg| build_representation(arg, abi_argument_type, abi, db))
            .collect::<Result<Vec<AllowedCalldataArguments>>>()?;

        Ok(AllowedCalldataArguments::ArrayMacro(
            CalldataArrayMacro::new(arguments),
        ))
    }
}

impl SupportedCalldataKind for ExprListParenthesized {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArguments> {
        // Regex capturing types between the parentheses, e.g.: for "(core::felt252, core::u8)"
        // will capture "core::felt252, core::u8"
        let tuple_types_pattern = Regex::new(r"\(([^)]+)\)").unwrap();
        let tuple_types: Vec<&str> = tuple_types_pattern
            .captures(expected_type)
            .with_context(|| {
                format!(r#"Invalid argument type, expected "{expected_type}", got tuple"#,)
            })?
            .get(1)
            .map(|x| x.as_str().split(", ").collect())
            .unwrap();

        let parsed_exprs = self
            .expressions(db)
            .elements(db)
            .into_iter()
            .zip(tuple_types)
            .map(|(expr, single_param)| build_representation(expr, single_param, abi, db))
            .collect::<Result<Vec<_>>>()?;

        Ok(AllowedCalldataArguments::Tuple(CalldataTuple::new(
            parsed_exprs,
        )))
    }
}

fn split(path: &ExprPath, db: &SimpleParserDatabase) -> Result<Vec<String>> {
    path.elements(db)
        .iter()
        .map(|p| match p {
            Simple(segment) => Ok(segment.ident(db).token(db).text(db).to_string()),
            PathSegment::WithGenericArgs(_) => {
                bail!("Cannot use generic args when specifying struct/enum path")
            }
        })
        .collect::<Result<Vec<String>>>()
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
                StructArg::StructArgTail(_) => {
                    bail!("Struct unpack-init with \"..\" operator is not allowed")
                }
            }
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

    let enum_abi_definition = find_valid_enum_or_struct(enums_from_abi, path)?;

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

fn parse_argument_list(arguments: &ArgList, db: &SimpleParserDatabase) -> Result<Vec<Expr>> {
    let arguments = arguments.elements(db);
    if arguments
        .iter()
        .map(|arg| arg.modifiers(db).elements(db))
        .any(|mod_list| !mod_list.is_empty())
    {
        bail!("\"ref\" and \"mut\" modifiers are not allowed")
    }

    arguments
        .iter()
        .map(|arg| match arg.arg_clause(db) {
            ArgClause::Unnamed(expr) => Ok(expr.value(db)),
            ArgClause::Named(_) => {
                bail!("Named arguments are not allowed")
            }
            ArgClause::FieldInitShorthand(_) => {
                bail!("Field init shorthands are not allowed")
            }
        })
        .collect::<Result<Vec<Expr>>>()
}

fn parse_inline_macro(
    invocation: &ExprInlineMacro,
    db: &SimpleParserDatabase,
) -> Result<Vec<Expr>> {
    match invocation
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

    let macro_arg_list = match invocation.arguments(db) {
        WrappedArgList::BracketedArgList(args) => {
            // TODO arglist parsing here
            args.arguments(db)
        }
        WrappedArgList::ParenthesizedArgList(_) | WrappedArgList::BracedArgList(_) =>
            bail!("`array` macro supports only square brackets: array![]"),
        WrappedArgList::Missing(_) => unreachable!("If any type of parentheses is missing, then diagnostics have been reported and whole flow should have already been terminated.")
    };
    parse_argument_list(&macro_arg_list, db)
}

fn find_all_structs(abi: &[AbiEntry]) -> Vec<&AbiStruct> {
    abi.iter()
        .filter_map(|entry| match entry {
            AbiEntry::Struct(r#struct) => Some(r#struct),
            _ => None,
        })
        .collect()
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

trait EnumOrStruct {
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

// 'item' here means enum or struct
fn find_valid_enum_or_struct<'item, T: EnumOrStruct>(
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
