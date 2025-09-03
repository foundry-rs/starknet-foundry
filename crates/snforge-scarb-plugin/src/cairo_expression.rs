use cairo_lang_macro::{TokenStream, quote};

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
        let items = self.iter().fold(TokenStream::empty(), |mut acc, val| {
            let val = val.as_cairo_expression();
            acc.extend(quote! { #val, });
            acc
        });
        quote! {
            array![#items]
        }
    }
}
