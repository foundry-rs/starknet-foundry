use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

// works by calling `FromReader::from_reader(reader)` on all fields of struct
// for enums by reading 1 felt that is then matched on to determine which variant should be used
#[proc_macro_derive(FromReader)]
pub fn derive_from_reader(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let span = item.clone().into();
    let mut input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;
    let generics = &mut input.generics;
    let data = &input.data;

    add_trait_bounds(generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = create_func_body(data, &span);

    quote! {
        impl #impl_generics runtime::utils::from_reader::FromReader for #name #ty_generics #where_clause {
            fn from_reader(reader: &mut runtime::utils::buffer_reader::BufferReader<'_>) -> runtime::utils::buffer_reader::BufferReadResult<Self> {
                #body
            }
        }
    }
    .into()
}

fn add_trait_bounds(generics: &mut Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(parse_quote!(runtime::utils::from_reader::FromReader));
        }
    }
}

// generate code for struct/enum fields (named and tuple)
fn call_trait_on_field(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let recurse = fields.named.iter().map(|f| {
                let name = &f.ident;

                quote_spanned! {f.span() =>
                    #name: runtime::utils::from_reader::FromReader::from_reader(reader)?,
                }
            });

            quote! {
                {#(#recurse)*}
            }
        }
        Fields::Unnamed(fields) => {
            let recurse = fields.unnamed.iter().map(|f| {
                quote_spanned! {f.span()=>
                    runtime::utils::from_reader::FromReader::from_reader(reader)?
                }
            });

            quote! {
                (#(#recurse),*)
            }
        }
        Fields::Unit => TokenStream::new(),
    }
}

// creates code for `FromReader::from_reader` body
fn create_func_body(data: &Data, span: &TokenStream) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) | Fields::Unnamed(_) => {
                let fields = call_trait_on_field(&data.fields);

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
            // generate match arms by matching on next integer literals (discriminator)
            // then generate trait calls for variants fields
            let arms = data.variants.iter().enumerate().map(|(i, variant)| {
                let name = &variant.ident;
                let fields = call_trait_on_field(&variant.fields);
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
                    _ => Result::Err(runtime::utils::buffer_reader::BufferReadError::ParseFailed)?,
                };

                Result::Ok(this)
            }
        }
        // can not determine which variant should be used
        // use enum instead
        Data::Union(_) => syn::Error::new_spanned(
            span,
            "runtime::utils::from_reader::FromReader can be derived only on structs and enums",
        )
        .into_compile_error(),
    }
}
