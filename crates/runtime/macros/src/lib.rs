use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(FromReader)]
pub fn derive_from_reader(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let span = item.clone().into();
    let mut input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;
    let generics = &mut input.generics;
    let data = &input.data;

    let mod_name = syn::Ident::new(
        &format!("{name}__internal__from_reader_impl___"),
        Span::call_site(),
    );

    add_trait_bounds(generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = create_func_body(data, &span);

    quote! {
        mod #mod_name {
            use runtime::utils::from_reader::{FromReader, BufferReader, BufferReadResult};
            use super::*;

            impl #impl_generics FromReader for #name #ty_generics #where_clause {
                fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                    #body
                }
            }
        }
    }
    .into()
}

fn add_trait_bounds(generics: &mut Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(FromReader));
        }
    }
}

fn from_field(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let recurse = fields.named.iter().map(|f| {
                let name = &f.ident;

                quote_spanned! {f.span() =>
                    #name: FromReader::from_reader(reader)?,
                }
            });

            quote! {
                {#(#recurse)*}
            }
        }
        Fields::Unnamed(fields) => {
            let recurse = fields.unnamed.iter().map(|f| {
                quote_spanned! {f.span()=>
                    FromReader::from_reader(reader)?
                }
            });

            quote! {
                (#(#recurse),*)
            }
        }
        Fields::Unit => TokenStream::new(),
    }
}

fn create_func_body(data: &Data, span: &TokenStream) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) | Fields::Unnamed(_) => {
                let fields = from_field(&data.fields);

                quote! {
                    Result::Ok(Self
                        #fields
                    )
                }
            }
            Fields::Unit => {
                quote!(Result::Ok(Self))
            }
        },
        Data::Enum(data) => {
            let arms = data.variants.iter().enumerate().map(|(i, variant)| {
                let name = &variant.ident;
                let fields = from_field(&variant.fields);
                let lit = syn::parse_str::<syn::LitInt>(&i.to_string()).unwrap();

                quote! {
                    #lit => Self::#name #fields
                }
            });

            quote! {
                let variant = reader.read_felt()?;
                let variant = num_traits::cast::ToPrimitive::to_usize(&variant).unwrap();

                let this = match variant {
                    #(#arms,)*
                    _ => Result::Err(BufferReadError::ParseFailed)?,
                };

                Result::Ok(this)
            }
        }
        Data::Union(_) => {
            syn::Error::new_spanned(span, "FromReader can be derived only on structs and enums")
                .into_compile_error()
        }
    }
}
