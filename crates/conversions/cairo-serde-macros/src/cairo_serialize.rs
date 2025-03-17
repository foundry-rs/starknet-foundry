use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, GenericParam, Generics, parse_macro_input, parse_quote};

// works by calling `CairoSerialize::serialize(writer)` on all fields of struct
// for enums by writing 1 felt that is number of variant, then variant members
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let span = item.clone().into();
    let mut input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;
    let generics = &mut input.generics;
    let data = &input.data;

    add_trait_bounds(generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = create_func_body(data, &span);

    quote! {
        impl #impl_generics conversions::serde::serialize::CairoSerialize for #name #ty_generics #where_clause {
            fn serialize(&self, writer: &mut conversions::serde::serialize::BufferWriter) {
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
                .push(parse_quote!(conversions::serde::serialize::CairoSerialize));
        }
    }
}

#[derive(Copy, Clone)]
enum Item {
    Struct,
    Enum,
}

impl Item {
    fn get_prefix(self) -> TokenStream {
        match self {
            Self::Struct => quote! (self.),
            Self::Enum => quote!(),
        }
    }
}

// generate code for struct/enum fields (named and tuple)
fn call_trait_on_field(fields: &Fields, item: Item) -> TokenStream {
    let prefix = item.get_prefix();
    match fields {
        Fields::Named(fields) => {
            let recurse = fields.named.iter().map(|f| {
                let name = &f.ident;

                quote_spanned! {f.span() =>
                    conversions::serde::serialize::CairoSerialize::serialize(& #prefix #name, writer);
                }
            });

            quote! {
                {#(#recurse)*}
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let recurse = unnamed_fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name = match item {
                    Item::Struct => {
                        let prop = syn::parse_str::<syn::LitInt>(&i.to_string()).unwrap();

                        quote! {
                            #prefix #prop
                        }
                    }
                    Item::Enum => syn::parse_str::<syn::Ident>(&format!("field_{i}"))
                        .unwrap()
                        .to_token_stream(),
                };

                quote_spanned! {f.span()=>
                    conversions::serde::serialize::CairoSerialize::serialize(& #name, writer);
                }
            });

            quote! {
                #(#recurse),*
            }
        }
        Fields::Unit => TokenStream::new(),
    }
}

fn destruct_fields(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let recurse = fields.named.iter().map(|f| {
                let name = &f.ident;

                quote_spanned! {f.span() =>
                    #name
                }
            });

            quote! {
                {#(#recurse),*}
            }
        }
        Fields::Unnamed(fields) => {
            let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name = syn::parse_str::<syn::Ident>(&format!("field_{i}")).unwrap();

                quote_spanned! {f.span() =>
                    #name
                }
            });

            quote! {
                (#(#recurse),*)
            }
        }
        Fields::Unit => TokenStream::new(),
    }
}

// creates code for `CairoSerialize::serialize` body
fn create_func_body(data: &Data, span: &TokenStream) -> TokenStream {
    match data {
        Data::Struct(data) => {
            call_trait_on_field(&data.fields, Item::Struct)
        },
        Data::Enum(data) => {
            // generate match arms by matching on next integer literals (discriminator)
            // then generate trait calls for variants fields
            let arms = data.variants.iter().enumerate().map(|(i,variant)| {
                let name = &variant.ident;
                let calls = call_trait_on_field(&variant.fields, Item::Enum);
                let destructurization = destruct_fields(&variant.fields);
                let lit = syn::parse_str::<syn::LitInt>(&format!("{i}_u32")).unwrap();

                quote! {
                    Self::#name #destructurization => {
                        conversions::serde::serialize::CairoSerialize::serialize(&#lit, writer);
                        #calls
                    }
                }
            });

            // empty match does not work with references
            // and `self` is behind reference
            if data.variants.is_empty() {
                quote! {}
            }else{
                quote! {
                    match self {
                        #(#arms,)*
                    };
                }
            }

        }
        // can not determine which variant should be used
        // use enum instead
        Data::Union(_) => syn::Error::new_spanned(
            span,
            "conversions::serde::serialize::CairoSerialize can be derived only on structs and enums",
        )
        .into_compile_error(),
    }
}
