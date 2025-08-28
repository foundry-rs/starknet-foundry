use self::{named::NamedArgs, unnamed::UnnamedArgs};
use crate::attributes::{AttributeInfo, ErrorExt};
use cairo_lang_macro::Diagnostic;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{
    Terminal,
    ast::{ArgClause, Expr, OptionArgListParenthesized},
};
use smol_str::SmolStr;
use std::collections::HashMap;

pub mod named;
pub mod unnamed;

#[derive(Debug, Default)]
pub struct Arguments {
    named: NamedArgs,
    unnamed: HashMap<usize, Expr>,
    shorthand: HashMap<usize, SmolStr>,
}

impl Arguments {
    pub fn new<T: AttributeInfo>(
        db: &SimpleParserDatabase,
        args: OptionArgListParenthesized,
        warns: &mut Vec<Diagnostic>,
    ) -> Self {
        let args = match args {
            OptionArgListParenthesized::Empty(_) => vec![],
            OptionArgListParenthesized::ArgListParenthesized(args) => {
                let args = args.arguments(db).elements(db);

                if args.len() == 0 {
                    warns.push(T::warn(
                        "used with empty argument list. Either remove () or specify some arguments",
                    ));
                }

                args.collect::<Vec<_>>()
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
            }
        }

        this
    }

    pub fn is_empty(&self) -> bool {
        self.named.is_empty() && self.unnamed.is_empty() && self.shorthand.is_empty()
    }

    #[inline]
    pub fn named_only<T: AttributeInfo>(&self) -> Result<&NamedArgs, Diagnostic> {
        if self.shorthand.is_empty() && self.unnamed.is_empty() {
            Ok(&self.named)
        } else {
            Err(T::error("can be used with named arguments only"))
        }
    }

    #[inline]
    pub fn unnamed_only<T: AttributeInfo>(&self) -> Result<UnnamedArgs<'_>, Diagnostic> {
        if self.shorthand.is_empty() && self.named.is_empty() {
            Ok(UnnamedArgs::new(&self.unnamed))
        } else {
            Err(T::error("can be used with unnamed arguments only"))
        }
    }

    #[inline]
    pub fn unnamed(&self) -> UnnamedArgs<'_> {
        UnnamedArgs::new(&self.unnamed)
    }

    #[inline]
    pub fn named(&self) -> NamedArgs {
        NamedArgs::new(&self.named)
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
