mod cairo_deserialize;
mod cairo_serialize;

#[proc_macro_derive(CairoDeserialize)]
pub fn derive_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cairo_deserialize::derive_deserialize(item)
}

#[proc_macro_derive(CairoSerialize)]
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cairo_serialize::derive_serialize(item)
}
