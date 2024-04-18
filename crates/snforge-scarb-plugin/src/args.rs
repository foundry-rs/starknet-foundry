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
    ) -> (Self, Option<String>) {
        let mut warn = None;

        let args = match args {
            OptionArgListParenthesized::Empty(_) => vec![],
            OptionArgListParenthesized::ArgListParenthesized(args) => {
                let args = args.arguments(db).elements(db);

                if args.is_empty() {
                    warn = Some(format!(
                        "#[{}] used with empty argument list. Either remove () or specify some arguments",
                        T::ATTR_NAME
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

        (this, warn)
    }

    #[inline]
    fn is_both_empty<K2, K3, V2, V3>(a: &HashMap<K2, V2>, b: &HashMap<K3, V3>) -> bool {
        a.is_empty() && b.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.named.is_empty() && self.unnamed.is_empty() && self.shorthand.is_empty()
    }

    #[inline]
    pub fn named_only<T: AttributeInfo>(&self) -> Result<&NamedArgs, Diagnostic> {
        if Self::is_both_empty(&self.shorthand, &self.unnamed) {
            Ok(&self.named)
        } else {
            Err(T::error("can be used with named attributes only"))
        }
    }

    #[inline]
    pub fn unnamed_only<T: AttributeInfo>(&self) -> Result<UnnamedArgs, Diagnostic> {
        if Self::is_both_empty(&self.shorthand, &self.named) {
            Ok(UnnamedArgs::new(&self.unnamed))
        } else {
            Err(T::error("can be used with unnamed attributes only"))
        }
    }
}
