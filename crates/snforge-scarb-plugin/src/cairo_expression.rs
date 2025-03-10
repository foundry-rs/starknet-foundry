use crate::utils::create_single_token;
use cairo_lang_macro::{quote, TokenStream};

pub trait CairoExpression {
    fn as_cairo_expression(&self) -> TokenStream;
}

impl<T> CairoExpression for Option<T>
where
    T: CairoExpression,
{
    fn as_cairo_expression(&self) -> TokenStream {
        if let Some(v) = self {
            let v = v.as_cairo_expression();
            quote!(Option::Some( #v ))
        } else {
            quote!(Option::None)
        }
    }
}

impl<T> CairoExpression for Vec<T>
where
    T: CairoExpression,
{
    fn as_cairo_expression(&self) -> TokenStream {
        let mut result = TokenStream::new(vec![create_single_token("array![")]);

        for e in self {
            result.extend(e.as_cairo_expression().into_iter());

            result.push_token(create_single_token(","));
        }

        result.push_token(create_single_token("]"));

        result
    }
}
