use self::{named::NamedArgs, unnamed::UnnamedArgs};
use crate::attributes::{AttributeInfo, ErrorExt};
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
    pub fn named_only<T: AttributeInfo>(&self) -> Result<&NamedArgs, Diagnostic> {
        if self.shorthand.is_empty() && self.unnamed.is_empty() {
            Ok(&self.named)
        } else {
            Err(T::error("can be used with named attributes only"))
        }
    }

    #[inline]
    pub fn unnamed_only<T: AttributeInfo>(&self) -> Result<UnnamedArgs, Diagnostic> {
        if self.shorthand.is_empty() && self.named.is_empty() {
            Ok(UnnamedArgs::new(&self.unnamed))
        } else {
            Err(T::error("can be used with unnamed attributes only"))
        }
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
