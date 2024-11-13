use self::{named::NamedArgs, unnamed::UnnamedArgs};
use crate::attributes::{AttributeInfo, ErrorExt, ValidArgs, ValidArgsTypes, ValidNamedArgs};
use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::{
    ast::{ArgClause, Expr, OptionArgListParenthesized},
    db::SyntaxGroup,
    Terminal,
};
use smol_str::SmolStr;
use std::collections::HashMap;

pub mod named;
pub mod unnamed;

#[derive(Debug, Default)]
pub struct Arguments {
    pub named: NamedArgs,
    pub unnamed: HashMap<usize, Expr>,
    pub shorthand: HashMap<usize, SmolStr>,
}

impl Arguments {
    pub fn new<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        args: OptionArgListParenthesized,
        warns: &mut Vec<Diagnostic>,
    ) -> Self {
        let args = match args {
            OptionArgListParenthesized::Empty(_) => vec![],
            OptionArgListParenthesized::ArgListParenthesized(args) => {
                let args = args.arguments(db).elements(db);

                if args.is_empty() {
                    warns.push(T::warn(
                        "used with empty argument list. Either remove () or specify some arguments",
                    ));
                }

                args
            }
        };

        let mut this = Self::default();

        for (i, arg) in args.into_iter().enumerate() {
            match arg.arg_clause(db) {
                ArgClause::Unnamed(value) => {
                    this.unnamed.insert(i, value.value(db));
                }
                ArgClause::FieldInitShorthand(value) => {
                    this.shorthand.insert(i, value.name(db).name(db).text(db));
                }
                ArgClause::Named(value) => {
                    this.named
                        .entry(value.name(db).text(db))
                        .or_default()
                        .push(value.value(db));
                }
            };
        }

        this
    }

    pub fn is_empty(&self) -> bool {
        self.named.is_empty() && self.unnamed.is_empty() && self.shorthand.is_empty()
    }

    #[inline]
    pub fn named_only<T: AttributeInfo + ValidArgs>(&self) -> Result<&NamedArgs, Diagnostic> {
        if self.shorthand.is_empty() && self.unnamed.is_empty() {
            match T::VALID_ARGS {
                ValidArgsTypes::Named(valid_named_args)
                | ValidArgsTypes::Both { valid_named_args } => match valid_named_args {
                    ValidNamedArgs::All => {}
                    ValidNamedArgs::Restricted(valid_named_args) => {
                        if let Some(arg) = self
                            .named
                            .iter()
                            .map(|(arg, _)| arg)
                            .find(|arg| !valid_named_args.contains(&arg.as_str()))
                        {
                            return Err(T::error(format!(
                                "unsupported named argument \"{arg}\" provided",
                            )));
                        }
                    }
                },
                ValidArgsTypes::Unnamed => panic!(
                    "`named_only` arguments requested where only `Unnamed` arguments are valid"
                ),
                ValidArgsTypes::None => {
                    panic!("`named_only` arguments requested where no arguments are valid")
                }
            }

            Ok(&self.named)
        } else {
            Err(T::error("can be used with named attributes only"))
        }
    }

    #[inline]
    pub fn unnamed_only<T: AttributeInfo + ValidArgs>(&self) -> Result<UnnamedArgs, Diagnostic> {
        if self.shorthand.is_empty() && self.named.is_empty() {
            match T::VALID_ARGS {
                ValidArgsTypes::Named(_) => panic!("`unnamed_arguments` arguments requested where only `Named` arguments are valid"),
                ValidArgsTypes::Unnamed | ValidArgsTypes::Both { .. } => {},
                ValidArgsTypes::None => panic!("`named_only` arguments requested where no arguments are valid")
            }

            Ok(UnnamedArgs::new(&self.unnamed))
        } else {
            Err(T::error("can be used with unnamed attributes only"))
        }
    }

    #[inline]
    pub fn unnamed(&self) -> UnnamedArgs {
        UnnamedArgs::new(&self.unnamed)
    }

    #[inline]
    pub fn assert_is_empty<T: AttributeInfo>(&self) -> Result<(), Diagnostic> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(T::error("does not accept any arguments"))?
        }
    }
}
