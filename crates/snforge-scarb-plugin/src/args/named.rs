use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::ast::Expr;
use smol_str::SmolStr;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Default)]
pub struct NamedArgs(HashMap<SmolStr, Vec<Expr>>);

impl Deref for NamedArgs {
    type Target = HashMap<SmolStr, Vec<Expr>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NamedArgs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl NamedArgs {
    pub fn as_once(&self, arg: &str) -> Result<&Expr, Diagnostic> {
        let exprs = self
            .0
            .get(arg)
            .ok_or_else(|| Diagnostic::error(format!("<{arg}> argument is missing")))?;

        Self::once(exprs, arg)
    }

    pub fn as_once_optional(&self, arg: &str) -> Result<Option<&Expr>, Diagnostic> {
        let exprs = self.0.get(arg);

        match exprs {
            None => Ok(None),
            Some(exprs) => Self::once(exprs, arg).map(Some),
        }
    }

    fn once<'a>(exprs: &'a [Expr], arg: &str) -> Result<&'a Expr, Diagnostic> {
        if exprs.len() == 1 {
            Ok(exprs.last().unwrap())
        } else {
            Err(Diagnostic::error(format!(
                "<{arg}> argument was specified {} times, expected to be used only once",
                exprs.len()
            )))
        }
    }

    pub fn one_of_once<T: Display + Copy>(&self, args: &[T]) -> Result<(T, &Expr), Diagnostic> {
        let (field, values) = self.one_of(args)?;

        let value = Self::once(values, &field.to_string())?;

        Ok((field, value))
    }

    pub fn one_of<T: Display + Copy>(&self, args: &[T]) -> Result<(T, &Vec<Expr>), Diagnostic> {
        fn message<T: Iterator<Item = Item>, Item: Display>(fields: T) -> String {
            fields
                .map(|field| format!("<{field}>"))
                .collect::<Vec<_>>()
                .join(" | ")
        }

        let mut existing = args
            .iter()
            .filter(|arg| self.0.contains_key(arg.to_string().as_str()))
            .peekable();

        let first = existing.next();

        if existing.peek().is_some() {
            Err(format!(
                "only one of {} should be specified",
                message(std::iter::once(*first.unwrap()).chain(existing.copied()))
            ))
        } else {
            match first {
                None => Err(format!(
                    "one of {} must be specified",
                    message(args.iter().copied())
                )),
                Some(field) => Ok((*field, self.0.get(field.to_string().as_str()).unwrap())),
            }
        }
        .map_err(Diagnostic::error)
    }
}
